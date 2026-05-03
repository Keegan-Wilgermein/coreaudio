//! # Objects

// ---- Imports ------------
use crate::{errors::{CoreAudioError}, property::{Property, ReadWrite, Listenable}, data_types::Scope};

use std::{marker::PhantomData};
use coreaudio_sys::{AudioObjectID, kAudioObjectSystemObject};

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

    /// Checks if a property exists on an object and returns `true` if it does
    pub fn has_property<V, A, L>(
        &self,
        property: Property<V, System, A, L>,
    ) -> bool {
        false
    }

    /// Gets the value of an objects property
    pub fn get_property<V, A, L>(
        &self,
        property: Property<V, System, A, L>,
    ) -> Result<V, CoreAudioError> {
        todo!()
    }

    /// Sets the value of an objects property
    pub fn set_property<V, L>(
        &self,
        property: Property<V, System, ReadWrite, L>,
        value: V,
    ) -> Result<(), CoreAudioError> {
        todo!()
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

    /// Checks if a property exists on an object and returns `true` if it does
    pub fn has_property<V, A, L>(
        &self,
        property: Property<V, Device, A, L>,
    ) -> bool {
        false
    }

    /// Gets the value of an objects property
    pub fn get_property<V, A, L>(
        &self,
        property: Property<V, Device, A, L>,
    ) -> Result<V, CoreAudioError> {
        todo!()
    }

    /// Sets the value of an objects property
    pub fn set_property<V, L>(
        &self,
        property: Property<V, Device, ReadWrite, L>,
        value: V,
    ) -> Result<(), CoreAudioError> {
        todo!()
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

}
