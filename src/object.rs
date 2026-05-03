//! # Objects

#![allow(unsafe_code)]

// ---- Imports ------------
use crate::{data_types::{BufferFrameSizeRange, SampleRateRange, Scope, StreamDescription}, errors::{CoreAudioError, OSStatusCheck}, io_proc::{AudioBuffer, IOProc}, listener::PropertyListener, property::{DEVICE_AVAILABLE_SAMPLE_RATES, DEVICE_BUFFER_FRAME_SIZE_RANGE, DEVICE_INPUT_STREAMS, DEVICE_OUTPUT_STREAMS, Listenable, Property, ReadWrite, STREAM_PHYSICAL_FORMAT, STREAM_VIRTUAL_FORMAT, SYSTEM_BOX_LIST, SYSTEM_CLOCK_DEVICE_LIST, SYSTEM_DEFAULT_INPUT, SYSTEM_DEFAULT_OUTPUT, SYSTEM_DEVICES, SYSTEM_PLUGIN_LIST, SYSTEM_TAP_LIST}};

use std::{ffi::c_void, marker::PhantomData, ptr::null};
use coreaudio_sys::{AudioObjectGetPropertyData, AudioObjectGetPropertyDataSize, AudioObjectID, AudioObjectPropertyAddress, AudioObjectSetPropertyData, kAudioHardwareUnsupportedOperationError, kAudioObjectSystemObject};

// ---- Structs ------------
/// Identifier for `AudioObject` to expose system object functions
pub struct System;

/// Identifier for `AudioObject` to expose device object functions
pub struct Device;

/// Identifier for `AudioObject` to expose stream object functions
pub struct Stream;

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
            get_property_internal(self.id, SYSTEM_DEVICES.address, SYSTEM_DEVICES.read)?
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
            Scope::Output => self.get_property(SYSTEM_DEFAULT_OUTPUT)?,
        };

        Ok(AudioObject::<Device>::from(id))
    }

    pub fn system_box_list(&self) -> Result<Vec<AudioObjectID>, CoreAudioError> {
        get_property_internal(self.id, SYSTEM_BOX_LIST.address, SYSTEM_BOX_LIST.read)
    }

    pub fn system_clock_device_list(&self) -> Result<Vec<AudioObjectID>, CoreAudioError> {
        get_property_internal(self.id, SYSTEM_CLOCK_DEVICE_LIST.address, SYSTEM_CLOCK_DEVICE_LIST.read)
    }

    pub fn system_plugin_list(&self) -> Result<Vec<AudioObjectID>, CoreAudioError> {
        get_property_internal(self.id, SYSTEM_PLUGIN_LIST.address, SYSTEM_PLUGIN_LIST.read)
    }

    pub fn system_tap_list(&self) -> Result<Vec<AudioObjectID>, CoreAudioError> {
        get_property_internal(self.id, SYSTEM_TAP_LIST.address, SYSTEM_TAP_LIST.read)
    }

    /// Gets the value of an objects property
    pub fn get_property<V, A, L>(
        &self,
        property: Property<V, System, A, L>,
    ) -> Result<V, CoreAudioError> {
        get_property_internal(self.id, property.address, property.read)
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
    ) -> Result<PropertyListener<V>, CoreAudioError> {
        add_listener_internal(self.id, property)
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

    pub fn avaliable_sample_rates(&self) -> Result<Vec<SampleRateRange>, CoreAudioError> {
        get_property_internal(self.id, DEVICE_AVAILABLE_SAMPLE_RATES.address, DEVICE_AVAILABLE_SAMPLE_RATES.read)
    }

    pub fn avaliable_buffer_sizes(&self) -> Result<BufferFrameSizeRange, CoreAudioError> {
        get_property_internal(self.id, DEVICE_BUFFER_FRAME_SIZE_RANGE.address, DEVICE_BUFFER_FRAME_SIZE_RANGE.read)
    }

    /// Gets the value of an objects property
    pub fn get_property<V, A, L>(
        &self,
        property: Property<V, Device, A, L>,
    ) -> Result<V, CoreAudioError> {
        get_property_internal(self.id, property.address, property.read)
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
    ) -> Result<PropertyListener<V>, CoreAudioError> {
        add_listener_internal(self.id, property)
    }

    pub fn add_io_proc<F>(
        &self,
        callback: F,
    ) -> Result<IOProc, CoreAudioError>
    where
        F: Fn(&mut [AudioBuffer]) + Send + 'static,
    {
        IOProc::try_new(&self, callback)
    }
}

impl AudioObject<Stream> {
    pub fn stream_virtual_format(&self) -> Result<StreamDescription, CoreAudioError> {
        get_property_internal(self.id, STREAM_VIRTUAL_FORMAT.address, STREAM_VIRTUAL_FORMAT.read)
    }

    pub fn stream_physical_format(&self) -> Result<StreamDescription, CoreAudioError> {
        get_property_internal(self.id, STREAM_PHYSICAL_FORMAT.address, STREAM_PHYSICAL_FORMAT.read)
    }

    /// Gets the value of an streams property
    pub fn get_property<V, A, L>(
        &self,
        property: Property<V, Stream, A, L>,
    ) -> Result<V, CoreAudioError> {
        get_property_internal(self.id, property.address, property.read)
    }

    /// Adds a listener to a property
    pub fn add_listener<V, A>(
        &self,
        property: Property<V, Stream, A, Listenable>,
    ) -> Result<PropertyListener<V>, CoreAudioError> {
        add_listener_internal(self.id, property)
    }
}

// ---- Functions -----------
pub(crate) fn get_property_internal<T>(
    id: AudioObjectID,
    address: AudioObjectPropertyAddress,
    read: fn(&[u8]) -> Result<T, CoreAudioError>,
 ) -> Result<T, CoreAudioError> {
    unsafe {
        let mut size = 0u32;
        AudioObjectGetPropertyDataSize(
            id,
            &address,
            0,
            null(),
            &mut size,
        ).check()?;

        let mut buffer = vec![0u8; size as usize];
        AudioObjectGetPropertyData(
            id,
            &address,
            0,
            null(),
            &mut size,
            buffer.as_mut_ptr() as *mut c_void,
        ).check()?;

        read(&buffer)
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

fn add_listener_internal<V, D, A, L>(
    id: AudioObjectID,
    property: Property<V, D, A, L>,
) -> Result<PropertyListener<V>, CoreAudioError> {
    PropertyListener::try_new(id, property.address, property.read)
}
