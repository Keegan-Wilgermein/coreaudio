//! # Objects

// ---- Imports ------------
use crate::{data_types::Scope, errors::{CoreAudioError, OSStatusCheck}, property::{Listenable, Property, ReadWrite}};

use std::{ffi::c_void, marker::PhantomData, ptr::null};
use coreaudio_sys::{AudioObjectGetPropertyData, AudioObjectGetPropertyDataSize, AudioObjectID, AudioObjectSetPropertyData, kAudioObjectSystemObject};

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
        todo!()
    }

    /// Gets all avaliable devices with scope
    pub fn devices_with_scope(&self, scope: Scope) ->
    Result<Vec<AudioObject<Device>>, CoreAudioError> {
        todo!()
    }

    /// Gets the current device with scope
    pub fn current_device(&self, scope: Scope) ->
    Result<AudioObject<Device>, CoreAudioError> {
        todo!()
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
        todo!()
    }

    /// Gets all avaliable streams with scope
    pub fn streams_with_scope(&self, scope: Scope) ->
    Result<Vec<AudioObject<Stream>>, CoreAudioError> {
        todo!()
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
        let size = size_of::<V>() as u32;
        AudioObjectSetPropertyData(
            id,
            &property.address,
            0,
            null(),
            size,
            &value as *const _ as *mut c_void,
        ).check()?;

        Ok(())
    }
}
