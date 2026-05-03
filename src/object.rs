//! # Objects

// ---- Imports ------------
use crate::{data_types::Scope, errors::{CoreAudioError, OSStatusCheck}, property::{DEVICE_INPUT_STREAMS, DEVICE_OUTPUT_STREAMS, Listenable, Property, ReadWrite, SYSTEM_DEFAULT_INPUT, SYSTEM_DEVICES}};

use std::{ffi::c_void, marker::PhantomData, ptr::null};
use coreaudio_sys::{AudioObjectGetPropertyData, AudioObjectGetPropertyDataSize, AudioObjectID, AudioObjectSetPropertyData, kAudioHardwareUnsupportedOperationError, kAudioObjectSystemObject};

// ---- Structs ------------
/// Identifier for `AudioObject` to expose system object functions
pub(crate) struct System;

/// Identifier for `AudioObject` to expose device object functions
pub(crate) struct Device;

/// Identifier for `AudioObject` to expose stream object functions
pub(crate) struct Stream;

/// Describes an object in use by CoreAudio
/// and exposes specific functions for different variants
pub struct AudioObject<T> {
    /// ID for the object
    id: AudioObjectID,
    _marker: PhantomData<T>,
}

impl Default for AudioObject<System> {
    fn default() -> Self {
        Self {
            id: kAudioObjectSystemObject,
            _marker: PhantomData,
        }
    }
}

impl From<u32> for AudioObject<Device> {
    fn from(value: u32) -> Self {
        Self {
            id: value,
            _marker: PhantomData,
        }
    }
}

impl From<u32> for AudioObject<Stream> {
    fn from(value: u32) -> Self {
        Self {
            id: value,
            _marker: PhantomData,
        }
    }
}

impl<T> AudioObject<T> {
    /// Returns the objects ID
    pub fn id(&self) -> u32 {
        self.id
    }
}

impl AudioObject<System> {
    /// Gets all avaliable devices regardless of scope
    pub fn devices(&self) ->
    Result<Vec<AudioObject<Device>>, CoreAudioError> {
        Ok(
            get_property_internal(self.id, SYSTEM_DEVICES)?
            .iter()
            .map(|id| {
                AudioObject::<Device>::from(*id)
            }).collect()
        )
    }

    /// Gets all avaliable devices with scope
    pub fn devices_with_scope(&self, scope: Scope) ->
    Result<Vec<AudioObject<Device>>, CoreAudioError> {
        let all_devices = self.devices()?;
    
        Ok(all_devices.into_iter().filter(|device| {
            let streams = match scope {
                Scope::Input => device.get_property(DEVICE_INPUT_STREAMS),
                Scope::Output => device.get_property(DEVICE_OUTPUT_STREAMS),
            };
            
            streams.map(|s: Vec<AudioObjectID>| !s.is_empty()).unwrap_or(false)
        }).collect())
    }

    /// Gets the current device with scope
    pub fn current_device(&self, scope: Scope) ->
    Result<AudioObject<Device>, CoreAudioError> {
        let id = match scope {
            Scope::Input => self.get_property(SYSTEM_DEFAULT_INPUT)?,
            Scope::Output => self.get_property(SYSTEM_DEFAULT_INPUT)?,
        };

        Ok(AudioObject::<Device>::from(id))
    }

    /// Gets the value of an objects property
    pub fn get_property<V, A, L>(
        &self,
        property: Property<V, System, A, L>,
    ) -> Result<V, CoreAudioError> {
        get_property_internal(self.id, property)
    }

    /// Sets the value of an objects property
    pub fn set_property<V, L>(
        &self,
        property: Property<V, System, ReadWrite, L>,
        value: V,
    ) -> Result<(), CoreAudioError> {
        set_property_internal(self.id, property, value)
    }

    /// Adds a listener to a property
    pub fn add_listener<V, A>(
        &self,
        property: Property<V, System, A, Listenable>,
    ) -> Result<(), CoreAudioError> {
        todo!()
    }
}

impl AudioObject<Device> {
    /// Gets all avaliable streams regardless of scope
    pub fn streams(&self) ->
    Result<Vec<AudioObject<Stream>>, CoreAudioError> {
        let mut streams = self.get_property(DEVICE_INPUT_STREAMS)?;
        let output_streams = self.get_property(DEVICE_OUTPUT_STREAMS)?;
        output_streams.iter().for_each(|stream| {
            streams.push(*stream);
        });

        Ok(
            streams.iter().map(|stream| {
                AudioObject::<Stream>::from(*stream)
            }).collect()
        )
    }

    /// Gets all avaliable streams with scope
    pub fn streams_with_scope(&self, scope: Scope) ->
    Result<Vec<AudioObject<Stream>>, CoreAudioError> {
        let streams = match scope {
            Scope::Input => self.get_property(DEVICE_INPUT_STREAMS)?,
            Scope::Output => self.get_property(DEVICE_OUTPUT_STREAMS)?,
        };

        Ok(
            streams.iter().map(|stream| {
                AudioObject::<Stream>::from(*stream)
            }).collect()
        )
    }

    /// Gets the value of an objects property
    pub fn get_property<V, A, L>(
        &self,
        property: Property<V, Device, A, L>,
    ) -> Result<V, CoreAudioError> {
        get_property_internal(self.id, property)
    }

    /// Sets the value of an objects property
    pub fn set_property<V, L>(
        &self,
        property: Property<V, Device, ReadWrite, L>,
        value: V,
    ) -> Result<(), CoreAudioError> {
        set_property_internal(self.id, property, value)
    }

    /// Adds a listener to a property
    pub fn add_listener<V, A>(
        &self,
        property: Property<V, Device, A, Listenable>,
    ) -> Result<(), CoreAudioError> {
        todo!()
    }
}

impl AudioObject<Stream> {
    /// Gets the value of an streams property
    pub fn get_property<V, A, L>(
        &self,
        property: Property<V, Stream, A, L>,
    ) -> Result<V, CoreAudioError> {
        get_property_internal(self.id, property)
    }
}

// ---- Functions -----------
fn get_property_internal<V, D, A, L>(
    id: AudioObjectID,
    property: Property<V, D, A, L>
) -> Result<V, CoreAudioError> {
    unsafe {
        let mut size = 0u32;
        AudioObjectGetPropertyDataSize(
            id,
            &property.address,
            0,
            null(),
            &mut size,
        ).check()?;

        let mut buffer = vec![0u8; size as usize];
        AudioObjectGetPropertyData(
            id,
            &property.address,
            0,
            null(),
            &mut size,
            buffer.as_mut_ptr() as *mut c_void,
        ).check()?;

        (property.read)(&buffer)
    }
}

fn set_property_internal<V, D, A, L>(
    id: AudioObjectID,
    property: Property<V, D, A, L>,
    value: V,
) -> Result<(), CoreAudioError> {
    unsafe {
        let out_data;
        if let Some(function) = property.encode {
            out_data = function(value)
        } else {
            return Err(CoreAudioError::from(kAudioHardwareUnsupportedOperationError as i32));
        }

        let size = out_data.len() as u32;
        AudioObjectSetPropertyData(
            id,
            &property.address,
            0,
            null(),
            size,
            out_data.as_ptr() as *const c_void,
        ).check()?;

        Ok(())
    }
}
