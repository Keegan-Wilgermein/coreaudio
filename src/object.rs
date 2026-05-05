//! # Objects

#![allow(unsafe_code)]

// ---- Imports ------------
use crate::{data_types::Scope, errors::{CoreAudioError, OSStatusCheck}, io_proc::{AudioBuffer, IOProc}, listener::PropertyListener, property::{DEVICE_INPUT_STREAMS, DEVICE_OUTPUT_STREAMS, Property, SYSTEM_DEFAULT_INPUT, SYSTEM_DEFAULT_OUTPUT, SYSTEM_DEVICES}, traits::{CanListen, HasAllData, ObjectCompatibleWith, Writeable}};

use std::{ffi::c_void, marker::PhantomData, ptr::null};
use coreaudio_sys::{AudioObjectGetPropertyData, AudioObjectGetPropertyDataSize, AudioObjectID, AudioObjectSetPropertyData, kAudioHardwareUnsupportedOperationError, kAudioObjectSystemObject};

// ---- Structs ------------
/// Identifier for `AudioObject` to expose global property access
pub struct Global;

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

// ---- Implementation on `AudioDevice<System>` --------------------------
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
            Scope::Output => self.get_property(SYSTEM_DEFAULT_OUTPUT)?,
        };

        Ok(AudioObject::<Device>::from(id))
    }

    /// Gets the value of a property
    pub fn get_property<V, D, A, L, E>(
        &self,
        property: Property<V, D, A, L, E>,
    ) -> Result<V, CoreAudioError>
    where
        D: ObjectCompatibleWith<System>,
        E: HasAllData,
    {
        get_property_internal(self.id, property)
    }

    /// Sets the value of a property
    pub fn set_property<V, D, A, L, E>(
        &self,
        property: Property<V, D, A, L, E>,
        value: V,
    ) -> Result<(), CoreAudioError>
    where
        D: ObjectCompatibleWith<System>,
        A: Writeable,
        E: HasAllData,
    {
        set_property_internal(self.id, property, value)
    }

    /// Adds a listener to a property
    pub fn add_listener<V, D, A, L, E>(
        &self,
        property: Property<V, D, A, L, E>,
    ) -> Result<PropertyListener<V, D, A>, CoreAudioError>
    where
        D: ObjectCompatibleWith<System>,
        L: CanListen,
        E: HasAllData,
    {
        add_listener_internal(self.id, property)
    }
}

// ---- Implementation on `AudioDevice<Device>` --------------------------
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

    /// Gets the value of a property
    pub fn get_property<V, D, A, L, E>(
        &self,
        property: Property<V, D, A, L, E>,
    ) -> Result<V, CoreAudioError>
    where
        D: ObjectCompatibleWith<Device>,
        E: HasAllData,
    {
        get_property_internal(self.id, property)
    }

    /// Sets the value of a property
    pub fn set_property<V, D, A, L, E>(
        &self,
        property: Property<V, D, A, L, E>,
        value: V,
    ) -> Result<(), CoreAudioError>
    where
        D: ObjectCompatibleWith<Device>,
        A: Writeable,
        E: HasAllData,
    {
        set_property_internal(self.id, property, value)
    }

    /// Adds a listener to a property
    pub fn add_listener<V, D, A, L, E>(
        &self,
        property: Property<V, D, A, L, E>,
    ) -> Result<PropertyListener<V, D, A>, CoreAudioError>
    where
        D: ObjectCompatibleWith<Device>,
        L: CanListen,
        E: HasAllData,
    {
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

// ---- Implementation on `AudioDevice<Stream>` --------------------------
impl AudioObject<Stream> {
    /// Gets the value of a property
    pub fn get_property<V, D, A, L, E>(
        &self,
        property: Property<V, D, A, L, E>,
    ) -> Result<V, CoreAudioError>
    where
        D: ObjectCompatibleWith<Stream>,
        E: HasAllData,
    {
        get_property_internal(self.id, property)
    }

    /// Sets the value of a property
    pub fn set_property<V, D, A, L, E>(
        &self,
        property: Property<V, D, A, L, E>,
        value: V,
    ) -> Result<(), CoreAudioError>
    where
        D: ObjectCompatibleWith<Stream>,
        A: Writeable,
        E: HasAllData,
    {
        set_property_internal(self.id, property, value)
    }

    /// Adds a listener to a property
    pub fn add_listener<V, D, A, L, E>(
        &self,
        property: Property<V, D, A, L, E>,
    ) -> Result<PropertyListener<V, D, A>, CoreAudioError>
    where
        D: ObjectCompatibleWith<Stream>,
        L: CanListen,
        E: HasAllData,
    {
        add_listener_internal(self.id, property)
    }
}

// ---- Functions -----------
pub(crate) fn get_property_internal<V, D, A, L, E>(
    id: AudioObjectID,
    property: Property<V, D, A, L, E>,
 ) -> Result<V, CoreAudioError> {
    let (q_len, q_data) = property.qualifier.map_or(
        (0, null()),
        |q| {
            (q.len() as u32, q.as_ptr() as *const c_void)
        }
    );

    unsafe {
        let mut size = 0u32;
        AudioObjectGetPropertyDataSize(
            id,
            &property.address,
            q_len,
            q_data,
            &mut size,
        ).check()?;

        let mut buffer = vec![0u8; size as usize];
        AudioObjectGetPropertyData(
            id,
            &property.address,
            q_len,
            q_data,
            &mut size,
            buffer.as_mut_ptr() as *mut c_void,
        ).check()?;

        (property.read)(&buffer)
    }
 }

fn set_property_internal<V, D, A, L, E>(
    id: AudioObjectID,
    property: Property<V, D, A, L, E>,
    value: V,
) -> Result<(), CoreAudioError> {
    unsafe {
        let out_data;
        if let Some(function) = property.encode {
            out_data = function(value)
        } else {
            return Err(CoreAudioError::from(kAudioHardwareUnsupportedOperationError as i32));
        }

        let (q_len, q_data) = property.qualifier.map_or(
            (0, null()),
            |q| {
                (q.len() as u32, q.as_ptr() as *const c_void)
            }
        );

        let size = out_data.len() as u32;
        AudioObjectSetPropertyData(
            id,
            &property.address,
            q_len,
            q_data,
            size,
            out_data.as_ptr() as *const c_void,
        ).check()?;

        Ok(())
    }
}

fn add_listener_internal<V, D, A, L, E>(
    id: AudioObjectID,
    property: Property<V, D, A, L, E>,
) -> Result<PropertyListener<V, D, A>, CoreAudioError> {
    PropertyListener::try_new(id, property.address, property.read)
}
