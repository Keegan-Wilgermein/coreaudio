//! CoreAudio I/O procedure registration and control
//!
//! An [`IOProc`] wraps a CoreAudio `AudioDeviceIOProcID`
//! Creating one registers a low-latency audio callback with the device; calling
//! [`play`](IOProc::play) starts the I/O cycle and [`pause`](IOProc::pause)
//! stops it
//! Dropping an `IOProc` automatically destroys the proc ID

#![allow(unsafe_code)]

// ---- Imports ------------
use crate::Time;
use crate::errors::OSStatus;
use crate::{
    Scope,
    errors::{CoreAudioError, ErrorKind, OSStatusCheck},
    object::{AudioObject, Device},
};
use objc2_core_audio::{
    AudioDeviceCreateIOProcID, AudioDeviceDestroyIOProcID, AudioDeviceID, AudioDeviceIOProcID,
    AudioDeviceStart, AudioDeviceStop,
};
use objc2_core_audio_types::{AudioBufferList, AudioTimeStamp};
use std::ffi::c_void;

// ---- Structs ------------

/// Heap-allocated closure and its context, passed through the C callback boundary
///
/// Boxed and cast to `*mut c_void` when registering the proc; recovered by
/// casting back inside `io_callback`
struct ClientCallbackData<F>
where
    F: FnMut(&mut [AudioBuffer]) + Send + 'static,
{
    /// The user-supplied audio render callback
    callback: F,
    /// Which side of the device this proc serves, and so which buffer list it
    /// reads from
    scope: Scope,
    /// The buffers handed to `callback`, reused every cycle so the audio
    /// thread never allocates
    /// Its lifetime is a stand-in: it's only filled for the length of one
    /// callback and cleared before returning
    buffers: Vec<AudioBuffer<'static>>,
}

/// Buffers reserved up front
const RESERVED_BUFFERS: usize = 256;

/// A single output buffer delivered to the audio render callback
///
/// Each `AudioBuffer` covers one or more interleaved channels and is valid
/// only for the duration of the callback invocation
#[derive(Debug)]
pub struct AudioBuffer<'a> {
    /// Slice of output samples to fill
    /// The length equals `frame_count * channels` for interleaved data, or
    /// `frame_count` for non-interleaved (one buffer per channel)
    pub data: &'a mut [f32],
    /// Number of audio channels carried by this buffer
    channels: u32,
    /// Number of audio frames in this I/O cycle
    frame_count: u32,
    /// The time of this I/O cycle
    now_time: Time,
    /// The time the input data in this cycle was captured
    input_time: Time,
    /// The time the output data in this cycle will be played
    output_time: Time,
}

impl<'a> AudioBuffer<'a> {
    /// Gets the channel count
    pub fn get_channels(&self) -> u32 {
        self.channels
    }

    /// Gets the frame count
    pub fn get_frame_count(&self) -> u32 {
        self.frame_count
    }

    /// Gets the time of this I/O cycle
    pub fn get_now_time(&self) -> Time {
        self.now_time
    }

    /// Gets the time the input data was captured
    pub fn get_input_time(&self) -> Time {
        self.input_time
    }

    /// Gets the time the output data will be played
    pub fn get_output_time(&self) -> Time {
        self.output_time
    }
}

/// A registered CoreAudio I/O procedure that drives audio rendering
///
/// Obtain one via [`AudioObject::<Device>::add_io_proc`]
/// The proc is stopped (but not destroyed) on creation and must be started
/// explicitly with [`play`](IOProc::play)
/// Dropping this value automatically unregisters the proc from CoreAudio
pub struct IOProc {
    /// The `AudioDeviceID` this proc is registered with
    id: AudioDeviceID,
    /// The opaque proc handle returned by `AudioDeviceCreateIOProcID`
    proc_id: AudioDeviceIOProcID,
    /// Whether the device I/O cycle is currently running
    is_running: bool,
    /// The boxed `ClientCallbackData` handed to CoreAudio, and how to free it
    /// (its closure type is erased here)
    client_data: *mut c_void,
    free_client_data: unsafe fn(*mut c_void),
}

// The client data is only touched by the IO thread while the proc runs, and
// by `Drop` once it's stopped; the closure it holds is `Send`
unsafe impl Send for IOProc {}
// No `&self` method touches the client data
unsafe impl Sync for IOProc {}

impl Drop for IOProc {
    /// Stops and unregisters the proc, then frees its callback and everything
    /// it owns
    /// Stopping from outside the IO thread returns only once the callback is no
    /// longer running, so nothing can still be using it
    /// Never drop an `IOProc` from inside its own callback
    fn drop(&mut self) {
        unsafe {
            AudioDeviceStop(self.id, self.proc_id);
            AudioDeviceDestroyIOProcID(self.id, self.proc_id);
            (self.free_client_data)(self.client_data);
        }
    }
}

/// Frees a `ClientCallbackData<F>` boxed by `IOProc::try_new`
unsafe fn free_client_data<F>(data: *mut c_void)
where
    F: FnMut(&mut [AudioBuffer]) + Send + 'static,
{
    unsafe { drop(Box::from_raw(data as *mut ClientCallbackData<F>)) };
}

impl IOProc {
    /// Registers `callback` as an I/O proc on `device`
    ///
    /// The device is immediately stopped after creation so that `play` must be
    /// called explicitly to begin audio delivery
    ///
    /// The callbacks list of `AudioBuffer` is
    /// represented depending on whether the data
    /// is interleaved or not
    ///
    /// ## Interleaved audio
    ///
    /// If the data is interleaved then the list will have length 1
    /// and the channels will be represented as `[1, 2, 3, ... , 1, 2]`
    /// in the `AudioBuffer.data` field
    ///
    /// ## Non-Interleaved audio
    ///
    /// If the data is non interleaved then the list will have a length
    /// equal to the amount of channels and the field `AudioBuffer.data`
    /// will have length 1, containing only the samples for that channel
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
                // CoreAudio may pass null buffer lists, which the bindings' `NonNull`
                // can't express; the callback checks for null itself
                Some(std::mem::transmute::<RawIOProc, _>(io_callback::<F>)),
                data_ptr,
                (&mut proc_id).into(),
            )
            .check()
        };

        if let Err(error) = created {
            // Never registered: nothing else holds the callback
            unsafe { free_client_data::<F>(data_ptr) };
            return Err(error);
        }

        // From here the proc owns the callback, and frees it when dropped
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
    /// once per buffer period
    ///
    /// Returns [`ErrorKind::AlreadyRunning`] if the proc is already active
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

    /// Stops the device I/O cycle without unregistering the proc
    ///
    /// Returns [`ErrorKind::AlreadyPaused`] if the proc is already stopped
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

    /// Unregisters the I/O proc and releases all associated resources
    ///
    /// Equivalent to dropping `self`; provided for explicit, readable teardown
    pub fn remove(self) {
        drop(self);
    }
}

// ---- Functions ------------

/// [`io_callback`]'s signature with raw pointers, since the buffer lists may be null
type RawIOProc = extern "C-unwind" fn(
    AudioDeviceID,
    *const AudioTimeStamp,
    *const AudioBufferList,
    *const AudioTimeStamp,
    *mut AudioBufferList,
    *const AudioTimeStamp,
    *mut c_void,
) -> OSStatus;

/// Reads a time stamp CoreAudio passed to the callback
///
/// A null pointer gives the default [`Time`], which has no valid clocks
///
/// # Safety
/// `time` must be null or point to a valid `AudioTimeStamp`
unsafe fn read_time(time: *const AudioTimeStamp) -> Time {
    if time.is_null() {
        Time::default()
    } else {
        unsafe { time.read() }.into()
    }
}

/// CoreAudio I/O callback invoked once per buffer period
///
/// Recovers the user closure from `client_data`, wraps each CoreAudio output
/// buffer as an [`AudioBuffer`], and calls the closure
/// Returns `0` on success
extern "C-unwind" fn io_callback<F>(
    _device: AudioDeviceID,
    now: *const AudioTimeStamp,
    input: *const AudioBufferList,
    input_time: *const AudioTimeStamp,
    output: *mut AudioBufferList,
    output_time: *const AudioTimeStamp,
    client_data: *mut c_void,
) -> OSStatus
where
    F: FnMut(&mut [AudioBuffer]) + Send + 'static,
{
    unsafe {
        let client_data = &mut *(client_data as *mut ClientCallbackData<F>);

        // The list for this proc's side; a device with nothing on that side
        // passes none
        let list = match client_data.scope {
            Scope::Input => input as *mut AudioBufferList,
            Scope::Output => output,
        };

        let buffers: &mut [objc2_core_audio_types::AudioBuffer] = if list.is_null() {
            &mut []
        } else {
            std::slice::from_raw_parts_mut(
                (*list).mBuffers.as_mut_ptr(),
                (*list).mNumberBuffers as usize,
            )
        };

        // Reuses the reserved buffers rather than collecting a new Vec as
        // allocating here can block past the IO deadline
        client_data.buffers.clear();
        client_data.buffers.extend(buffers.iter_mut().map(|buf| {
            // A buffer with no memory or no channels carries no frames and
            // must not be divided by
            let empty = buf.mData.is_null() || buf.mNumberChannels == 0;

            let (data, frame_count) = if empty {
                (
                    std::slice::from_raw_parts_mut(
                        std::ptr::NonNull::<f32>::dangling().as_ptr(),
                        0,
                    ),
                    0,
                )
            } else {
                (
                    std::slice::from_raw_parts_mut(
                        buf.mData as *mut f32,
                        buf.mDataByteSize as usize / size_of::<f32>(),
                    ),
                    buf.mDataByteSize / (buf.mNumberChannels * size_of::<f32>() as u32),
                )
            };

            AudioBuffer {
                data,
                channels: buf.mNumberChannels,
                frame_count,
                now_time: read_time(now),
                input_time: read_time(input_time),
                output_time: read_time(output_time),
            }
        }));

        (client_data.callback)(&mut client_data.buffers);
        // Nothing may outlive the cycle that owns the memory
        client_data.buffers.clear();
    }

    0
}
