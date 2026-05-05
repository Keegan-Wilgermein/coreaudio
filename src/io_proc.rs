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
use crate::{errors::{CoreAudioError, ErrorKind, OSStatusCheck}, object::{AudioObject, Device}};

// ---- Structs ------------

/// Heap-allocated closure and its context, passed through the C callback boundary.
///
/// Boxed and cast to `*mut c_void` when registering the proc; recovered by
/// casting back inside `io_callback`.
struct ClientCallbackData<F>
where
    F: Fn(&[AudioBuffer]) + Send + 'static,
{
    /// The user-supplied audio render callback.
    callback: F,
}

/// A single output buffer delivered to the audio render callback.
///
/// Each `AudioBuffer` covers one or more interleaved channels and is valid
/// only for the duration of the callback invocation.
pub struct AudioBuffer<'a> {
    /// Slice of output samples to fill. The length equals
    /// `frame_count * channels` for interleaved data, or `frame_count` for
    /// non-interleaved (one buffer per channel).
    pub data: &'a mut [f32],
    /// Number of audio channels carried by this buffer.
    pub channels: u32,
    /// `true` if all channels share this buffer (interleaved layout).
    pub is_interleaved: bool,
    /// Number of audio frames in this I/O cycle.
    pub frame_count: u32,
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
}

impl Drop for IOProc {
    fn drop(&mut self) {
        unsafe {
            AudioDeviceDestroyIOProcID(
                self.id,
                self.proc_id,
            );
        }
    }
}

impl IOProc {
    /// Registers `callback` as an I/O proc on `device`.
    ///
    /// The device is immediately stopped after creation so that `play` must be
    /// called explicitly to begin audio delivery.
    pub(crate) fn try_new<F>(
        device: &AudioObject<Device>,
        callback: F,
    ) -> Result<Self, CoreAudioError>
    where
        F: Fn(&[AudioBuffer]) + Send + 'static,
    {
        let client_data = ClientCallbackData {
            callback,
        };

        let data_ptr = Box::into_raw(Box::new(client_data)) as *mut c_void;

        let mut proc_id: AudioDeviceIOProcID = None;

        unsafe {
            AudioDeviceCreateIOProcID(
                device.id(),
                Some(io_callback::<F>),
                data_ptr,
                &mut proc_id,
            ).check()?;

            AudioDeviceStop(device.id(), proc_id).check()?;
        }

        Ok (
            Self {
                id: device.id(),
                proc_id,
                is_running: false,
            }
        )
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
    _input: *const AudioBufferList,
    _input_time: *const AudioTimeStamp,
    output: *mut AudioBufferList,
    _output_time: *const AudioTimeStamp,
    client_data: *mut c_void,
) -> OSStatus
where
    F: Fn(&[AudioBuffer]) + Send + 'static,
{
    unsafe {
        let client_data = &*(client_data as *mut ClientCallbackData<F>);

        let buffers = std::slice::from_raw_parts_mut(
            (*output).mBuffers.as_mut_ptr(),
            (*output).mNumberBuffers as usize,
        );

        let audio_buffers: Vec<AudioBuffer> = buffers.iter_mut().map(|buf| {
            AudioBuffer {
                data: std::slice::from_raw_parts_mut(
                    buf.mData as *mut f32,
                    buf.mDataByteSize as usize / size_of::<f32>(),
                ),
                channels: buf.mNumberChannels,
                is_interleaved: buf.mNumberChannels > 1,
                frame_count: buf.mDataByteSize / (buf.mNumberChannels * size_of::<f32>() as u32),
            }
        }).collect();

        (client_data.callback)(&audio_buffers)
    }

    0
}
