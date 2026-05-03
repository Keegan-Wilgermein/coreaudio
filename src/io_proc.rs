//! # IO Proc

// ---- Imports ------------
use std::{ffi::c_void};
use coreaudio_sys::{self, AudioBufferList, AudioDeviceCreateIOProcID, AudioDeviceDestroyIOProcID, AudioDeviceID, AudioDeviceIOProcID, AudioDeviceStart, AudioDeviceStop, AudioTimeStamp, OSStatus};
use crate::{errors::{CoreAudioError, ErrorKind, OSStatusCheck}, object::{AudioObject, Device}};

// ---- Structs ------------
struct ClientCallbackData<F>
where
    F: Fn(&mut [AudioBuffer]) + Send + 'static,
{
    callback: F,
}

pub struct AudioBuffer<'a> {
    pub data: &'a mut [f32],
    pub channels: u32,
    pub is_interleaved: bool,
    pub frame_count: u32,
}

pub struct IOProc {
    id: AudioDeviceID,
    proc_id: AudioDeviceIOProcID,
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
    pub(crate) fn try_new<F>(
        device: &AudioObject<Device>,
        callback: F,
    ) -> Result<Self, CoreAudioError>
    where
        F: Fn(&mut [AudioBuffer]) + Send + 'static,
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

    pub fn play(&self) -> Result<(), CoreAudioError> {
        if self.is_running {
            return Err(CoreAudioError::from_error_kind(ErrorKind::AlreadyRunning));
        }

        unsafe {
            AudioDeviceStart(self.id, self.proc_id).check()?;
        }

        Ok(())
    }

    pub fn pause(&self) -> Result<(), CoreAudioError> {
        if !self.is_running {
            return Err(CoreAudioError::from_error_kind(ErrorKind::AlreadyPaused));
        }

        unsafe {
            AudioDeviceStop(self.id, self.proc_id).check()?;
        }

        Ok(())
    }

    pub fn remove(self) {
        drop(self);
    }
}

// ---- Functions ------------
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
    F: Fn(&mut [AudioBuffer]) + Send + 'static,
{
    unsafe {
        let client_data = &*(client_data as *mut ClientCallbackData<F>);
    
        let buffers = std::slice::from_raw_parts_mut(
            (*output).mBuffers.as_mut_ptr(),
            (*output).mNumberBuffers as usize,
        );
    
        let mut audio_buffers: Vec<AudioBuffer> = buffers.iter_mut().map(|buf| {
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

        (client_data.callback)(&mut audio_buffers)
    }
    
    0
}
