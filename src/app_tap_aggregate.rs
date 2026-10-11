//! Private aggregate devices built around a process tap
//!
//! A tap on its own has no IO; putting it in an aggregate device turns it into
//! an ordinary [`AudioObject<Device>`] with input streams, so its audio is read
//! with [`AudioObject::<Device>::add_io_proc`] like any input device

#![allow(unsafe_code)]

// ---- Imports ------------
use crate::{errors::{CoreAudioError, OSStatusCheck}, object::{AudioObject, Device}};
use std::ffi::CStr;
use core_foundation::{
    array::CFArray, base::{CFType, TCFType}, boolean::CFBoolean, dictionary::CFDictionary, string::CFString,
};
use objc2_core_audio::{
    AudioHardwareCreateAggregateDevice, AudioHardwareDestroyAggregateDevice, AudioObjectID,
    kAudioAggregateDeviceIsPrivateKey, kAudioAggregateDeviceIsStackedKey,
    kAudioAggregateDeviceNameKey, kAudioAggregateDeviceTapAutoStartKey,
    kAudioAggregateDeviceTapListKey, kAudioAggregateDeviceUIDKey,
    kAudioSubTapDriftCompensationKey, kAudioSubTapUIDKey,
};

// ---- Structs ------------

/// A private aggregate device, destroyed when dropped
///
/// Private aggregates are only visible to the process that created them: they
/// never show in Audio MIDI Setup, System Settings or any other app, and macOS
/// destroys them if the process exits without dropping them
pub struct AppTapAggregateDevice {
    device: AudioObject<Device>,
}

impl AppTapAggregateDevice {
    /// Creates a private aggregate whose only content is the tap with UID
    /// `tap_uid` (see [`TAP_UID`](crate::TAP_UID)), started along with the
    /// aggregate and clocked by the tap itself
    ///
    /// `uid` must be unique among the process's aggregates
    /// Drop the aggregate before the tap it reads
    pub fn with_tap(name: &str, uid: &str, tap_uid: &str) -> Result<Self, CoreAudioError> {
        Self::new_tap_internal(name, uid, tap_uid, true)
    }

    /// Creates a public aggregate whose only content is the tap with UID
    /// `tap_uid` (see [`TAP_UID`](crate::TAP_UID)), started along with the
    /// aggregate and clocked by the tap itself
    ///
    /// `uid` must be unique among the process's aggregates
    /// Drop the aggregate before the tap it reads
    pub fn with_tap_public(name: &str, uid: &str, tap_uid: &str) -> Result<Self, CoreAudioError> {
        Self::new_tap_internal(name, uid, tap_uid, false)
    }

    /// The aggregate as a device, to read properties from and add an IO proc to
    /// 
    /// This is the aggregate itself, not any underlying
    /// devices that it contains as an `AppTapAggregateDevice`
    /// cannot contain any devices
    pub fn device(&self) -> AudioObject<Device> {
        self.device
    }

    /// Internal function for creating a aggregate device
    /// from a device tap
    fn new_tap_internal(
        name: &str,
        uid: &str,
        tap_uid: &str,
        private: bool,
    ) -> Result<Self, CoreAudioError> {
        let tap = CFDictionary::from_CFType_pairs(&[
            (key(kAudioSubTapUIDKey), CFString::new(tap_uid).as_CFType()),
            (key(kAudioSubTapDriftCompensationKey), CFBoolean::true_value().as_CFType()),
        ]);

        let taps = CFArray::from_CFTypes(&[tap]);

        let description = tap_description(
            name,
            uid,
            taps,
            private,
        );

        let mut id: AudioObjectID = 0;
        unsafe {
            AudioHardwareCreateAggregateDevice(
                &*description.as_concrete_TypeRef().cast(),
                (&mut id).into(),
            ).check()?;
        }

        Ok(Self { device: AudioObject::<Device>::from(id) })
    }
}

impl Drop for AppTapAggregateDevice {
    fn drop(&mut self) {
        unsafe {
            AudioHardwareDestroyAggregateDevice(self.device.id());
        }
    }
}

// ---- Functions -----------

/// A dictionary key from one of CoreAudio's key constants
fn key(key: &CStr) -> CFString {
    CFString::new(&key.to_string_lossy())
}

/// Creates an aggregate device description
/// for app tapping
fn tap_description(
    name: &str,
    uid: &str,
    taps: CFArray<CFDictionary<CFString, CFType>>,
    private: bool,
) -> CFDictionary<CFString, CFType> {
    let private_key = if private {
        CFBoolean::true_value().as_CFType()
    } else {
        CFBoolean::false_value().as_CFType()
    };

    CFDictionary::from_CFType_pairs(&[
        (key(kAudioAggregateDeviceNameKey), CFString::new(name).as_CFType()),
        (key(kAudioAggregateDeviceUIDKey), CFString::new(uid).as_CFType()),
        (key(kAudioAggregateDeviceIsPrivateKey), private_key),
        (key(kAudioAggregateDeviceIsStackedKey), CFBoolean::false_value().as_CFType()),
        (key(kAudioAggregateDeviceTapAutoStartKey), CFBoolean::true_value().as_CFType()),
        (key(kAudioAggregateDeviceTapListKey), taps.as_CFType()),
    ])
}
