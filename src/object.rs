//! CoreAudio object hierarchy: system, device, and stream.
//!
//! The central type is [`AudioObject<T>`], where `T` is a marker that selects
//! which methods are available. The markers are [`System`], [`Device`],
//! [`Stream`], and [`Global`]. Property access, listener registration, and I/O
//! proc creation are all gated through the type system so invalid operations
//! are rejected at compile time.

#![allow(unsafe_code)]

// ---- Imports ------------
use crate::{data_types::Scope, errors::{CoreAudioError, OSStatusCheck}, io_proc::{AudioBuffer, IOProc}, listener::PropertyListener, property::{DEVICE_INPUT_STREAMS, DEVICE_OUTPUT_STREAMS, Property, SYSTEM_DEFAULT_INPUT, SYSTEM_DEFAULT_OUTPUT, SYSTEM_DEVICES}, traits::{CanListen, HasAllData, ObjectCompatibleWith, Writeable}};

use std::{ffi::c_void, marker::PhantomData, ptr::null};
use coreaudio_sys::{AudioObjectGetPropertyData, AudioObjectGetPropertyDataSize, AudioObjectID, AudioObjectSetPropertyData, kAudioHardwareUnsupportedOperationError, kAudioObjectSystemObject};

// ---- Structs ------------

/// Marker for [`AudioObject`] granting access to properties on any object type.
///
/// `Global` properties are defined on the base `AudioObject` class in CoreAudio
/// and apply equally to system objects, devices, and streams. A
/// `Property<_, Global, _, _, _>` constant can be used with any `AudioObject`.
pub struct Global;

/// Marker for [`AudioObject`] granting access to system-wide HAL properties.
///
/// Construct the system object with `AudioObject::<System>::default()`. It
/// exposes device enumeration, default device selection, and global HAL state.
pub struct System;

/// Marker for [`AudioObject`] granting access to device-specific properties.
///
/// Obtain device objects from [`AudioObject::<System>::devices`] or
/// [`AudioObject::<System>::devices_with_scope`].
pub struct Device;

/// Marker for [`AudioObject`] granting access to stream-specific properties.
///
/// Obtain stream objects from [`AudioObject::<Device>::streams`] or
/// [`AudioObject::<Device>::streams_with_scope`].
pub struct Stream;

/// A CoreAudio HAL object identified by an `AudioObjectID`.
///
/// The type parameter `T` is one of [`System`], [`Device`], [`Stream`],
/// and determines which `impl` blocks — and therefore which methods
/// and properties — are available on this value.
pub struct AudioObject<T> {
    /// Raw CoreAudio object identifier.
    id: AudioObjectID,
    _marker: PhantomData<T>,
}

impl Default for AudioObject<System> {
    /// Returns the singleton system object (`kAudioObjectSystemObject`).
    fn default() -> Self {
        Self {
            id: kAudioObjectSystemObject,
            _marker: PhantomData,
        }
    }
}

impl From<u32> for AudioObject<Device> {
    /// Wraps a raw `AudioObjectID` as a device object.
    fn from(value: u32) -> Self {
        Self {
            id: value,
            _marker: PhantomData,
        }
    }
}

impl From<u32> for AudioObject<Stream> {
    /// Wraps a raw `AudioObjectID` as a stream object.
    fn from(value: u32) -> Self {
        Self {
            id: value,
            _marker: PhantomData,
        }
    }
}

impl<T> AudioObject<T> {
    /// Returns the raw `AudioObjectID` for this object.
    pub fn id(&self) -> u32 {
        self.id
    }
}

// ---- Implementation on `AudioObject<System>` --------------------------
impl AudioObject<System> {
    /// Returns all audio devices currently known to the HAL, regardless of scope.
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

    /// Returns all audio devices that have at least one stream in the given scope.
    ///
    /// Devices with no streams in `scope` (e.g. an output-only device queried
    /// with `Scope::Input`) are excluded from the result.
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

    /// Returns the device currently selected as the system default for `scope`.
    pub fn current_device(&self, scope: Scope) ->
    Result<AudioObject<Device>, CoreAudioError> {
        let id = match scope {
            Scope::Input => self.get_property(SYSTEM_DEFAULT_INPUT)?,
            Scope::Output => self.get_property(SYSTEM_DEFAULT_OUTPUT)?,
        };

        Ok(AudioObject::<Device>::from(id))
    }

    /// Reads a system-scoped property value.
    ///
    /// The property constant's object type must be [`System`] or [`Global`].
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

    /// Writes a system-scoped property value.
    ///
    /// The property constant must be [`ReadWrite`](crate::property::ReadWrite)
    /// and its object type must be [`System`] or [`Global`].
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

    /// Registers a listener for a system-scoped property.
    ///
    /// The property constant must be
    /// [`Listenable`](crate::property::Listenable) and its object type must be
    /// [`System`] or [`Global`]. Drop the returned [`PropertyListener`] to
    /// unregister.
    pub fn add_listener<V, D, A, L, E>(
        &self,
        property: Property<V, D, A, L, E>,
    ) -> Result<PropertyListener<V, D, A>, CoreAudioError>
    where
        D: ObjectCompatibleWith<System>,
        L: CanListen,
        E: HasAllData,
    {
        PropertyListener::try_new(self.id, property.address, property.read)
    }
}

// ---- Implementation on `AudioObject<Device>` --------------------------
impl AudioObject<Device> {
    /// Returns all streams on this device, regardless of scope.
    ///
    /// Input and output streams are combined into a single list.
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

    /// Returns all streams on this device in the given scope.
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

    /// Reads a device-scoped property value.
    ///
    /// The property constant's object type must be [`Device`] or [`Global`].
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

    /// Writes a device-scoped property value.
    ///
    /// The property constant must be [`ReadWrite`](crate::property::ReadWrite)
    /// and its object type must be [`Device`] or [`Global`].
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

    /// Registers a listener for a device-scoped property.
    ///
    /// The property constant must be
    /// [`Listenable`](crate::property::Listenable) and its object type must be
    /// [`Device`] or [`Global`]. Drop the returned [`PropertyListener`] to
    /// unregister.
    pub fn add_listener<V, D, A, L, E>(
        &self,
        property: Property<V, D, A, L, E>,
    ) -> Result<PropertyListener<V, D, A>, CoreAudioError>
    where
        D: ObjectCompatibleWith<Device>,
        L: CanListen,
        E: HasAllData,
    {
        PropertyListener::try_new(self.id, property.address, property.read)
    }

    /// Registers an audio render callback on this device and returns the
    /// resulting [`IOProc`].
    ///
    /// The callback receives a slice of [`AudioBuffer`]s once per I/O cycle.
    /// Call [`IOProc::play`] to start audio delivery.
    pub fn add_io_proc<F>(
        &self,
        callback: F,
    ) -> Result<IOProc, CoreAudioError>
    where
        F: Fn(&[AudioBuffer]) + Send + 'static,
    {
        IOProc::try_new(&self, callback)
    }
}

// ---- Implementation on `AudioObject<Stream>` --------------------------
impl AudioObject<Stream> {
    /// Reads a stream-scoped property value.
    ///
    /// The property constant's object type must be [`Stream`] or [`Global`].
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

    /// Writes a stream-scoped property value.
    ///
    /// The property constant must be [`ReadWrite`](crate::property::ReadWrite)
    /// and its object type must be [`Stream`] or [`Global`].
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

    /// Registers a listener for a stream-scoped property.
    ///
    /// The property constant must be
    /// [`Listenable`](crate::property::Listenable) and its object type must be
    /// [`Stream`] or [`Global`]. Drop the returned [`PropertyListener`] to
    /// unregister.
    pub fn add_listener<V, D, A, L, E>(
        &self,
        property: Property<V, D, A, L, E>,
    ) -> Result<PropertyListener<V, D, A>, CoreAudioError>
    where
        D: ObjectCompatibleWith<Stream>,
        L: CanListen,
        E: HasAllData,
    {
        PropertyListener::try_new(self.id, property.address, property.read)
    }
}

// ---- Functions -----------

/// Reads a property from the object identified by `id`.
///
/// Calls `AudioObjectGetPropertyDataSize` to determine the required buffer
/// size, allocates the buffer, then calls `AudioObjectGetPropertyData` and
/// deserialises the result using `property.read`.
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

/// Writes a property value to the object identified by `id`.
///
/// Calls `property.encode` to serialise `value` to bytes, then passes those
/// bytes to `AudioObjectSetPropertyData`. Returns an error if the property has
/// no encoder (i.e. is read-only).
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
