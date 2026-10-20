//! CoreAudio object hierarchy: system, device, and stream
//!
//! The central type is [`AudioObject<T>`], where `T` is a marker that selects
//! which methods are available
//! The markers are [`System`], [`Device`], [`Stream`], [`Process`], [`Tap`],
//! and [`Global`]
//! Property access, listener registration, and I/O proc creation are all gated
//! through the type system so invalid operations are rejected at compile time

#![allow(unsafe_code)]

// ---- Imports ------------
use crate::{
    data_types::{PropertySize, Scope},
    errors::{CoreAudioError, OSStatusCheck},
    io_proc::{AudioBuffer, IOProc},
    listener::{CallbackListener, PropertyListener},
    property::{
        DEVICE_INPUT_STREAMS, DEVICE_OUTPUT_STREAMS, Property, SYSTEM_DEFAULT_INPUT,
        SYSTEM_DEFAULT_OUTPUT, SYSTEM_DEVICES, SYSTEM_PROCESS_OBJECT_LIST,
    },
    traits::{CanListen, HasAllData, IntoNonNull, ObjectCompatibleWith, PtrIntoNonNull, Writeable},
};
use objc2_core_audio::{
    AudioObjectGetPropertyData, AudioObjectGetPropertyDataSize, AudioObjectID,
    AudioObjectSetPropertyData, kAudioObjectSystemObject,
};
use std::{ffi::c_void, marker::PhantomData, ptr::null};

// ---- Structs ------------

/// Marker for [`AudioObject`] granting access to properties on any object type
///
/// `Global` properties are defined on the base `AudioObject` class in CoreAudio
/// and apply equally to system objects, devices, and streams
///
/// A `Property<_, Global, _, _, _>` constant can be used with any `AudioObject`
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Global;

/// Marker for [`AudioObject`] granting access to system-wide HAL properties
///
/// Construct the system object with `AudioObject::<System>::default()`
///
/// It exposes device enumeration, default device selection, and global HAL
/// state
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct System;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Plugin;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Box;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Clock;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Unit;

/// Marker for [`AudioObject`] granting access to device-specific properties
///
/// Obtain device objects from [`AudioObject::<System>::devices`] or
/// [`AudioObject::<System>::devices_with_scope`]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Device;

/// Marker for [`AudioObject`] granting access to stream-specific properties
///
/// Obtain stream objects from [`AudioObject::<Device>::streams`] or
/// [`AudioObject::<Device>::streams_with_scope`]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Stream;

/// Marker for [`AudioObject`] granting access to process properties
///
/// Every process that has used CoreAudio has a process object (macOS 14.2+)
/// Obtain them from [`AudioObject::<System>::processes`]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Process;

/// Marker for [`AudioObject`] granting access to process tap properties
///
/// A tap captures the audio a set of processes output (macOS 14.2+)
/// Create one with `ProcessTap` (the `process-tap` feature)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tap;

/// A CoreAudio HAL object identified by an `AudioObjectID`
///
/// The type parameter `T` is one of [`System`], [`Device`], [`Stream`],
/// [`Process`], [`Tap`], [`Plugin`], [`Box`], [`Clock`], or [`Unit`]
/// and determines which `impl` blocks, and therefore which methods
/// and properties, are available on this value
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AudioObject<T> {
    /// Raw CoreAudio object identifier
    id: AudioObjectID,
    _marker: PhantomData<T>,
}

impl Default for AudioObject<System> {
    /// Returns the singleton system object (`kAudioObjectSystemObject`)
    fn default() -> Self {
        Self {
            id: kAudioObjectSystemObject as u32,
            _marker: PhantomData,
        }
    }
}

impl From<u32> for AudioObject<Device> {
    /// Wraps a raw `AudioObjectID` as a device object
    fn from(value: u32) -> Self {
        Self {
            id: value,
            _marker: PhantomData,
        }
    }
}

impl From<u32> for AudioObject<Stream> {
    /// Wraps a raw `AudioObjectID` as a stream object
    fn from(value: u32) -> Self {
        Self {
            id: value,
            _marker: PhantomData,
        }
    }
}

impl From<u32> for AudioObject<Process> {
    /// Wraps a raw `AudioObjectID` as a process object
    fn from(value: u32) -> Self {
        Self {
            id: value,
            _marker: PhantomData,
        }
    }
}

impl From<u32> for AudioObject<Tap> {
    /// Wraps a raw `AudioObjectID` as a tap object
    fn from(value: u32) -> Self {
        Self {
            id: value,
            _marker: PhantomData,
        }
    }
}

impl<C> AudioObject<C> {
    /// Returns the raw `AudioObjectID` for this object
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Reads a property value scoped to `C` compatible AudioObjects
    pub fn get_property<V, D, A, L, E>(
        &self,
        property: Property<V, D, A, L, E>,
    ) -> Result<V, CoreAudioError>
    where
        D: ObjectCompatibleWith<C>,
        E: HasAllData,
    {
        get_property_internal(self.id, property)
    }

    /// Reads a translation property, whose data starts as `value` and comes
    /// back converted in place
    ///
    /// For properties like `DEVICE_OUTPUT_VOLUME_DECIBELS_TO_SCALAR`, where
    /// CoreAudio reads its input from the data buffer itself
    pub fn translate<V, D, A, L, E>(
        &self,
        property: Property<V, D, A, L, E>,
        value: V,
    ) -> Result<V, CoreAudioError>
    where
        D: ObjectCompatibleWith<C>,
        E: HasAllData,
    {
        let buffer = (property.encode)(value).bytes;
        read_property_internal(self.id, property, buffer)
    }

    /// Returns the size in bytes of a property's data
    ///
    /// Uses the size recorded on the property when it is known, and otherwise
    /// asks CoreAudio
    pub fn get_property_data_size<V, D, A, L, E>(
        &self,
        property: Property<V, D, A, L, E>,
    ) -> Result<u32, CoreAudioError>
    where
        D: ObjectCompatibleWith<C>,
        E: HasAllData,
    {
        get_property_data_size_internal(self.id, &property)
    }

    /// Writes a property value scoped to `C` compatible AudioObjects
    ///
    /// The property constant must be [`ReadWrite`](crate::property::ReadWrite)
    pub fn set_property<V, D, A, L, E>(
        &self,
        property: Property<V, D, A, L, E>,
        value: V,
    ) -> Result<(), CoreAudioError>
    where
        D: ObjectCompatibleWith<C>,
        A: Writeable,
        E: HasAllData,
    {
        set_property_internal(self.id, property, value)
    }

    /// Registers a listener for a property scoped to `C` compatible AudioObjects
    ///
    /// The property constant must be
    /// [`Listenable`](crate::property::Listenable)
    ///
    /// Drop the returned [`PropertyListener`] to
    /// unregister
    pub fn add_listener<V, D, A, L, E>(
        &self,
        property: Property<V, D, A, L, E>,
    ) -> Result<PropertyListener<V, D, A>, CoreAudioError>
    where
        D: ObjectCompatibleWith<C>,
        L: CanListen,
        E: HasAllData,
    {
        PropertyListener::try_new(self.id, property.size, property.address, property.read)
    }

    /// Registers a listener for a property scoped to `C` compatible AudioObjects
    ///
    /// Unlike a normal listener, it calls `sink` on each change
    ///
    /// `sink` runs on CoreAudio's notification thread with the new value, or
    /// the error reading it
    /// The same property rules apply as for
    /// [`add_listener`](Self::add_listener)
    ///
    /// Drop the returned [`CallbackListener`] to unregister
    pub fn add_listener_with<V, D, A, L, E, S>(
        &self,
        property: Property<V, D, A, L, E>,
        sink: S,
    ) -> Result<CallbackListener<V, D, A>, CoreAudioError>
    where
        D: ObjectCompatibleWith<C>,
        L: CanListen,
        E: HasAllData,
        S: Fn(Result<V, CoreAudioError>) + Send + Sync + 'static,
    {
        CallbackListener::try_new(
            self.id,
            property.size,
            property.address,
            property.read,
            sink,
        )
    }

    /// Forcefully sets the property's value
    ///
    /// Use this if you have to interact with a
    /// device that doesn't follow the normal conventions
    ///
    /// # Safety
    /// Unlike the normal property setter, this version
    /// will ignore whether the property is
    /// - Writable
    /// - Has an element
    /// - Has qualifier data
    /// - Is being called on the correct kind of device
    ///
    /// This leaves it up to the caller to confirm all of these
    ///
    /// The function will still return a `CoreAudioError` if it
    /// fails for any reason instead of crashing,
    /// but it will not be able to say why
    pub unsafe fn force_set_property<V, D, A, L, E>(
        &self,
        property: Property<V, D, A, L, E>,
        value: V,
    ) -> Result<(), CoreAudioError> {
        set_property_internal(self.id, property, value)
    }

    /// Forcefully registers a listener for a property
    ///
    /// Use this if you have to interact with a
    /// device that doesn't follow the normal conventions
    ///
    /// # Safety
    /// Unlike the normal listener, this version
    /// will ignore whether the property is
    /// - [`Listenable`](crate::property::Listenable)
    /// - Has an element
    /// - Is being called on the correct kind of device
    ///
    /// CoreAudio accepts a listener on any property, so this
    /// can't misbehave, but it may never fire
    /// It's up to the caller to know the property really does notify
    ///
    /// The listener reads the property with no qualifier data, so
    /// properties that need a qualifier will fail to read
    /// Use [`force_add_listener_with`](Self::force_add_listener_with) to see
    /// those errors
    ///
    /// Drop the returned [`PropertyListener`] to unregister
    pub unsafe fn force_add_listener<V, D, A, L, E>(
        &self,
        property: Property<V, D, A, L, E>,
    ) -> Result<PropertyListener<V, D, A>, CoreAudioError> {
        PropertyListener::try_new(self.id, property.size, property.address, property.read)
    }

    /// Forcefully registers a listener for a property that calls `sink` on each change
    ///
    /// # Safety
    /// The same rules are ignored as for
    /// [`force_add_listener`](Self::force_add_listener)
    ///
    /// Drop the returned [`CallbackListener`] to unregister
    pub unsafe fn force_add_listener_with<V, D, A, L, E, S>(
        &self,
        property: Property<V, D, A, L, E>,
        sink: S,
    ) -> Result<CallbackListener<V, D, A>, CoreAudioError>
    where
        S: Fn(Result<V, CoreAudioError>) + Send + Sync + 'static,
    {
        CallbackListener::try_new(
            self.id,
            property.size,
            property.address,
            property.read,
            sink,
        )
    }
}

// ---- Implementation on `AudioObject<System>` --------------------------
impl AudioObject<System> {
    /// Returns all audio devices currently known to the HAL, regardless of scope
    pub fn devices(&self) -> Result<Vec<AudioObject<Device>>, CoreAudioError> {
        Ok(get_property_internal(self.id, SYSTEM_DEVICES)?
            .iter()
            .map(|id| AudioObject::<Device>::from(*id))
            .collect())
    }

    /// Returns all audio devices that have at least one stream in the given scope
    ///
    /// Devices with no streams in `scope` (e.g. an output-only device queried
    /// with `Scope::Input`) are excluded from the result
    pub fn devices_with_scope(
        &self,
        scope: Scope,
    ) -> Result<Vec<AudioObject<Device>>, CoreAudioError> {
        let all_devices = self.devices()?;

        Ok(all_devices
            .into_iter()
            .filter(|device| {
                let streams = match scope {
                    Scope::Input => device.get_property(DEVICE_INPUT_STREAMS),
                    Scope::Output => device.get_property(DEVICE_OUTPUT_STREAMS),
                };

                streams
                    .map(|s: Vec<AudioObjectID>| !s.is_empty())
                    .unwrap_or(false)
            })
            .collect())
    }

    /// Returns every process object the HAL knows: each process that has used
    /// CoreAudio (macOS 14.2+)
    /// Listen to [`crate::SYSTEM_PROCESS_OBJECT_LIST`] to hear processes come
    /// and go
    pub fn processes(&self) -> Result<Vec<AudioObject<Process>>, CoreAudioError> {
        Ok(get_property_internal(self.id, SYSTEM_PROCESS_OBJECT_LIST)?
            .iter()
            .map(|id| AudioObject::<Process>::from(*id))
            .collect())
    }

    /// Returns the device currently selected as the system default for `scope`
    pub fn current_device(&self, scope: Scope) -> Result<AudioObject<Device>, CoreAudioError> {
        let id = match scope {
            Scope::Input => self.get_property(SYSTEM_DEFAULT_INPUT)?,
            Scope::Output => self.get_property(SYSTEM_DEFAULT_OUTPUT)?,
        };

        Ok(AudioObject::<Device>::from(id))
    }
}

// ---- Implementation on `AudioObject<Device>` --------------------------
impl AudioObject<Device> {
    /// Returns all streams on this device, regardless of scope
    ///
    /// Input and output streams are combined into a single list
    pub fn streams(&self) -> Result<Vec<AudioObject<Stream>>, CoreAudioError> {
        let mut streams = self.get_property(DEVICE_INPUT_STREAMS)?;
        let output_streams = self.get_property(DEVICE_OUTPUT_STREAMS)?;
        output_streams.iter().for_each(|stream| {
            streams.push(*stream);
        });

        Ok(streams
            .iter()
            .map(|stream| AudioObject::<Stream>::from(*stream))
            .collect())
    }

    /// Returns all streams on this device in the given scope
    pub fn streams_with_scope(
        &self,
        scope: Scope,
    ) -> Result<Vec<AudioObject<Stream>>, CoreAudioError> {
        let streams = match scope {
            Scope::Input => self.get_property(DEVICE_INPUT_STREAMS)?,
            Scope::Output => self.get_property(DEVICE_OUTPUT_STREAMS)?,
        };

        Ok(streams
            .iter()
            .map(|stream| AudioObject::<Stream>::from(*stream))
            .collect())
    }

    /// Registers an audio render callback on this device and returns the
    /// resulting [`IOProc`]
    ///
    /// The callback receives a slice of [`AudioBuffer`]s once per I/O cycle
    /// Call [`IOProc::play`] to start audio delivery
    pub fn add_io_proc<F>(&self, scope: Scope, callback: F) -> Result<IOProc, CoreAudioError>
    where
        F: FnMut(&mut [AudioBuffer]) + Send + 'static,
    {
        IOProc::try_new(self, scope, callback)
    }
}

// ---- Implementation on `AudioObject<Stream>` --------------------------
impl AudioObject<Stream> {}

// ---- Implementation on `AudioObject<Process>` --------------------------
impl AudioObject<Process> {}

// ---- Implementation on `AudioObject<Tap>` --------------------------
impl AudioObject<Tap> {}

// ---- Functions -----------

/// Reads a property from the object identified by `id`
///
/// Calls `AudioObjectGetPropertyDataSize` to determine the required buffer
/// size, allocates the buffer, then calls `AudioObjectGetPropertyData` and
/// deserialises the result using `property.read`
pub(crate) fn get_property_internal<V, D, A, L, E>(
    id: AudioObjectID,
    property: Property<V, D, A, L, E>,
) -> Result<V, CoreAudioError> {
    let size = get_property_data_size_internal(id, &property)?;
    read_property_internal(id, property, vec![0u8; size as usize])
}

/// Reads a property into `buffer`, which CoreAudio sees as the data's
/// starting value, then deserialises it using `property.read`
fn read_property_internal<V, D, A, L, E>(
    id: AudioObjectID,
    mut property: Property<V, D, A, L, E>,
    mut buffer: Vec<u8>,
) -> Result<V, CoreAudioError> {
    let (q_len, q_data) = get_q_data_len(&property);
    let mut size = buffer.len() as u32;

    unsafe {
        AudioObjectGetPropertyData(
            id,
            property.address.non_null(),
            q_len,
            q_data,
            size.non_null(),
            buffer.as_mut_ptr().cast::<c_void>().non_null(),
        )
        .check()?;

        (property.read)(&buffer)
    }
}

/// Writes a property value to the object identified by `id`
///
/// Calls `property.encode` to serialise `value` to bytes, then passes those
/// bytes to `AudioObjectSetPropertyData`
/// Ignores the access marker: CoreAudio decides whether the write is allowed
pub(crate) fn set_property_internal<V, D, A, L, E>(
    id: AudioObjectID,
    mut property: Property<V, D, A, L, E>,
    value: V,
) -> Result<(), CoreAudioError> {
    unsafe {
        let out_data = (property.encode)(value);

        let (q_len, q_data) = get_q_data_len(&property);

        let size = out_data.bytes.len() as u32;
        AudioObjectSetPropertyData(
            id,
            property.address.non_null(),
            q_len,
            q_data,
            size,
            (out_data.bytes.as_ptr() as *const c_void).non_null(),
        )
        .check()?;

        Ok(())
    }
}

/// Gets the q_len and q_data as a tuple
/// of ( q_len, q_data )
fn get_q_data_len<V, D, A, L, E>(property: &Property<V, D, A, L, E>) -> (u32, *const c_void) {
    property.qualifier.as_ref().map_or((0, null()), |q| {
        (q.bytes.len() as u32, q.bytes.as_ptr() as *const c_void)
    })
}

/// Gets the size of the property data for a device
fn get_property_data_size_internal<V, D, A, L, E>(
    id: AudioObjectID,
    property: &Property<V, D, A, L, E>,
) -> Result<u32, CoreAudioError> {
    let (q_len, q_data) = get_q_data_len(property);

    match property.size {
        PropertySize::Known(size) => Ok(size),
        PropertySize::Unknown => unsafe {
            let mut address = property.address;
            let mut size = 0u32;
            AudioObjectGetPropertyDataSize(id, address.non_null(), q_len, q_data, size.non_null())
                .check()?;

            Ok(size)
        },
    }
}
