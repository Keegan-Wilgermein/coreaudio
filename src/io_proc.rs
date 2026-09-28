//! CoreAudio I/O procedure registration and control.
//!
//! An [`IOProc`] wraps a CoreAudio `AudioDeviceIOProcID`. Creating one
//! registers a low-latency audio callback with the device; calling
//! [`play`](IOProc::play) starts the I/O cycle and [`pause`](IOProc::pause)
//! stops it. Dropping an `IOProc` automatically destroys the proc ID.

#![allow(unsafe_code)]

// ---- Imports ------------
use std::{ffi::c_void};
use coreaudio_sys::{self, AudioBufferList, AudioDeviceCreateIOProcID, AudioDeviceDestroyIOProcID, AudioDeviceID, AudioDeviceIOProcID, AudioDeviceStart, AudioDeviceStop, AudioTimeStamp, OSStatus};
use crate::{Scope, errors::{CoreAudioError, ErrorKind, OSStatusCheck}, object::{AudioObject, Device}};

// ---- Structs ------------

/// Heap-allocated closure and its context, passed through the C callback boundary.
///
/// Boxed and cast to `*mut c_void` when registering the proc; recovered by
/// casting back inside `io_callback`.
struct ClientCallbackData<F>
where
    F: FnMut(&mut [AudioBuffer]) + Send + 'static,
{
    /// The user-supplied audio render callback.
    callback: F,
    scope: Scope,
    /// The buffers handed to `callback`, reused every cycle so the audio
    /// thread never allocates. Its lifetime is a stand-in: it's only filled
    /// for the length of one callback and cleared before returning.
    buffers: Vec<AudioBuffer<'static>>,
}

/// Buffers reserved up front: more than any device presents per cycle (one
/// per channel on the widest non-interleaved devices). A device with more
/// grows it once, on its first cycle.
const RESERVED_BUFFERS: usize = 256;

/// A single output buffer delivered to the audio render callback.
///
/// Each `AudioBuffer` covers one or more interleaved channels and is valid
/// only for the duration of the callback invocation.
#[derive(Debug)]
pub struct AudioBuffer<'a> {
    /// Slice of output samples to fill. The length equals
    /// `frame_count * channels` for interleaved data, or `frame_count` for
    /// non-interleaved (one buffer per channel).
    pub data: &'a mut [f32],
    /// Number of audio channels carried by this buffer.
    channels: u32,
    /// `true` if all channels share this buffer (interleaved layout).
    is_interleaved: bool,
    /// Number of audio frames in this I/O cycle.
    frame_count: u32,
}

impl<'a> AudioBuffer<'a> {
    pub fn get_channels(&self) -> u32 {
        self.channels
    }

    pub fn get_interleaved(&self) -> bool {
        self.is_interleaved
    }

    pub fn get_frame_count(&self) -> u32 {
        self.frame_count
    }
}

/// A registered CoreAudio I/O procedure that drives audio rendering.
///
/// Obtain one via [`AudioObject::<Device>::add_io_proc`]. The proc is stopped
/// (but not destroyed) on creation and must be started explicitly with
/// [`play`](IOProc::play). Dropping this value automatically unregisters the
/// proc from CoreAudio.
pub struct IOProc {
    /// The `AudioDeviceID` this proc is registered with.
    id: AudioDeviceID,
    /// The opaque proc handle returned by `AudioDeviceCreateIOProcID`.
    proc_id: AudioDeviceIOProcID,
    /// Whether the device I/O cycle is currently running.
    is_running: bool,
    /// The boxed `ClientCallbackData` handed to CoreAudio, and how to free it
    /// (its closure type is erased here).
    client_data: *mut c_void,
    free_client_data: unsafe fn(*mut c_void),
}

// The client data is only touched by the IO thread while the proc runs, and
// by `Drop` once it's stopped; the closure it holds is `Send`.
unsafe impl Send for IOProc {}
// No `&self` method touches the client data.
unsafe impl Sync for IOProc {}

impl Drop for IOProc {
    /// Stops and unregisters the proc, then frees its callback and everything
    /// it owns. Stopping from outside the IO thread returns only once the
    /// callback is no longer running, so nothing can still be using it.
    /// Never drop an `IOProc` from inside its own callback.
    fn drop(&mut self) {
        unsafe {
            AudioDeviceStop(self.id, self.proc_id);
            AudioDeviceDestroyIOProcID(
                self.id,
                self.proc_id,
            );
            (self.free_client_data)(self.client_data);
        }
    }
}

/// Frees a `ClientCallbackData<F>` boxed by `IOProc::try_new`.
unsafe fn free_client_data<F>(data: *mut c_void)
where
    F: FnMut(&mut [AudioBuffer]) + Send + 'static,
{
    unsafe { drop(Box::from_raw(data as *mut ClientCallbackData<F>)) };
}

impl IOProc {
    /// Registers `callback` as an I/O proc on `device`.
    ///
    /// The device is immediately stopped after creation so that `play` must be
    /// called explicitly to begin audio delivery.
    pub(crate) fn try_new<F>(
        device: &AudioObject<Device>,
        scope: Scope,
        callback: F,
    ) -> Result<Self, CoreAudioError>
    where
        F: FnMut(&mut [AudioBuffer]) + Send + 'static,
    {
        let client_data = ClientCallbackData {
            callback,
            scope,
            buffers: Vec::with_capacity(RESERVED_BUFFERS),
        };

        let data_ptr = Box::into_raw(Box::new(client_data)) as *mut c_void;

        let mut proc_id: AudioDeviceIOProcID = None;

        let created = unsafe {
            AudioDeviceCreateIOProcID(
                device.id(),
                Some(io_callback::<F>),
                data_ptr,
                &mut proc_id,
            ).check()
        };
        if let Err(error) = created {
            // Never registered: nothing else holds the callback.
            unsafe { free_client_data::<F>(data_ptr) };
            return Err(error);
        }

        // From here the proc owns the callback, and frees it when dropped.
        let proc = Self {
            id: device.id(),
            proc_id,
            is_running: false,
            client_data: data_ptr,
            free_client_data: free_client_data::<F>,
        };
        unsafe { AudioDeviceStop(device.id(), proc_id).check()? };

        Ok(proc)
    }

    /// Starts the device I/O cycle, causing the callback to be invoked
    /// once per buffer period.
    ///
    /// Returns [`ErrorKind::AlreadyRunning`] if the proc is already active.
    pub fn play(&mut self) -> Result<(), CoreAudioError> {
        if self.is_running {
            return Err(CoreAudioError::from_error_kind(ErrorKind::AlreadyRunning));
        }

        unsafe {
            AudioDeviceStart(self.id, self.proc_id).check()?;
        }

        self.is_running = true;

        Ok(())
    }

    /// Stops the device I/O cycle without unregistering the proc.
    ///
    /// Returns [`ErrorKind::AlreadyPaused`] if the proc is already stopped.
    pub fn pause(&mut self) -> Result<(), CoreAudioError> {
        if !self.is_running {
            return Err(CoreAudioError::from_error_kind(ErrorKind::AlreadyPaused));
        }

        unsafe {
            AudioDeviceStop(self.id, self.proc_id).check()?;
        }

        self.is_running = false;

        Ok(())
    }

    /// Unregisters the I/O proc and releases all associated resources.
    ///
    /// Equivalent to dropping `self`; provided for explicit, readable teardown.
    pub fn remove(self) {
        drop(self);
    }
}

// ---- Functions ------------

/// CoreAudio I/O callback invoked once per buffer period.
///
/// Recovers the user closure from `client_data`, wraps each CoreAudio output
/// buffer as an [`AudioBuffer`], and calls the closure. Returns `0` on success.
extern "C" fn io_callback<F>(
    _device: AudioDeviceID,
    _now: *const AudioTimeStamp,
    input: *const AudioBufferList,
    _input_time: *const AudioTimeStamp,
    output: *mut AudioBufferList,
    _output_time: *const AudioTimeStamp,
    client_data: *mut c_void,
) -> OSStatus
where
    F: FnMut(&mut [AudioBuffer]) + Send + 'static,
{
    unsafe {
        let client_data = &mut *(client_data as *mut ClientCallbackData<F>);

        // The list for this proc's side; a device with nothing on that side
        // passes none.
        let list = match client_data.scope {
            Scope::Input => input as *mut AudioBufferList,
            Scope::Output => output,
        };
        let buffers: &mut [coreaudio_sys::AudioBuffer] = if list.is_null() {
            &mut []
        } else {
            std::slice::from_raw_parts_mut(
                (*list).mBuffers.as_mut_ptr(),
                (*list).mNumberBuffers as usize,
            )
        };

        // Reuses the reserved buffers rather than collecting a new Vec:
        // allocating here can block past the IO deadline.
        client_data.buffers.clear();
        client_data.buffers.extend(buffers.iter_mut().map(|buf| {
            // A buffer with no memory or no channels carries no frames (and
            // must not be divided by).
            let empty = buf.mData.is_null() || buf.mNumberChannels == 0;
            AudioBuffer {
                data: if empty {
                    &mut []
                } else {
                    std::slice::from_raw_parts_mut(
                        buf.mData as *mut f32,
                        buf.mDataByteSize as usize / size_of::<f32>(),
                    )
                },
                channels: buf.mNumberChannels,
                is_interleaved: buf.mNumberChannels > 1,
                frame_count: if empty {
                    0
                } else {
                    buf.mDataByteSize / (buf.mNumberChannels * size_of::<f32>() as u32)
                },
            }
        }));

        (client_data.callback)(&mut client_data.buffers);
        // Nothing may outlive the cycle that owns the memory.
        client_data.buffers.clear();
    }

    0
}
