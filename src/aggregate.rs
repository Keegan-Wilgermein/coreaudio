//! Private aggregate devices built around a process tap.
//!
//! A tap on its own has no IO; putting it in an aggregate device turns it into
//! an ordinary [`AudioObject<Device>`] with input streams, so its audio is read
//! with [`AudioObject::<Device>::add_io_proc`] like any input device.

#![allow(unsafe_code)]

// ---- Imports ------------
use crate::{errors::{CoreAudioError, OSStatusCheck}, object::{AudioObject, Device}};
use core_foundation::{
    array::CFArray,
    base::TCFType,
    boolean::CFBoolean,
    dictionary::CFDictionary,
    string::CFString,
};
use coreaudio_sys::{
    AudioHardwareCreateAggregateDevice, AudioHardwareDestroyAggregateDevice, AudioObjectID,
    kAudioAggregateDeviceIsPrivateKey, kAudioAggregateDeviceIsStackedKey,
    kAudioAggregateDeviceNameKey, kAudioAggregateDeviceTapAutoStartKey,
    kAudioAggregateDeviceTapListKey, kAudioAggregateDeviceUIDKey,
    kAudioSubTapDriftCompensationKey, kAudioSubTapUIDKey,
};

// ---- Structs ------------

/// A private aggregate device, destroyed when dropped.
///
/// Private aggregates are only visible to the process that created them: they
/// never show in Audio MIDI Setup, System Settings or any other app, and macOS
/// destroys them if the process exits without dropping them.
pub struct AggregateDevice {
    device: AudioObject<Device>,
}

impl AggregateDevice {
    /// Creates a private aggregate whose only content is the tap with UID
    /// `tap_uid` (see [`TAP_UID`](crate::TAP_UID)), started along with the
    /// aggregate and clocked by the tap itself.
    ///
    /// `uid` must be unique among the process's aggregates. Drop the
    /// aggregate before the tap it reads.
    pub fn with_tap(name: &str, uid: &str, tap_uid: &str) -> Result<Self, CoreAudioError> {
        let tap = CFDictionary::from_CFType_pairs(&[
            (key(kAudioSubTapUIDKey), CFString::new(tap_uid).as_CFType()),
            (key(kAudioSubTapDriftCompensationKey), CFBoolean::true_value().as_CFType()),
        ]);
        let taps = CFArray::from_CFTypes(&[tap]);

        let description = CFDictionary::from_CFType_pairs(&[
            (key(kAudioAggregateDeviceNameKey), CFString::new(name).as_CFType()),
            (key(kAudioAggregateDeviceUIDKey), CFString::new(uid).as_CFType()),
            (key(kAudioAggregateDeviceIsPrivateKey), CFBoolean::true_value().as_CFType()),
            (key(kAudioAggregateDeviceIsStackedKey), CFBoolean::false_value().as_CFType()),
            (key(kAudioAggregateDeviceTapAutoStartKey), CFBoolean::true_value().as_CFType()),
            (key(kAudioAggregateDeviceTapListKey), taps.as_CFType()),
        ]);

        let mut id: AudioObjectID = 0;
        unsafe {
            AudioHardwareCreateAggregateDevice(
                description.as_concrete_TypeRef() as *const _,
                &mut id,
            ).check()?;
        }

        Ok(Self { device: AudioObject::<Device>::from(id) })
    }

    /// The aggregate as a device, to read properties from and add an IO proc to.
    pub fn device(&self) -> AudioObject<Device> {
        self.device
    }
}

impl Drop for AggregateDevice {
    fn drop(&mut self) {
        unsafe {
            AudioHardwareDestroyAggregateDevice(self.device.id());
        }
    }
}

// ---- Functions -----------

/// A dictionary key from one of CoreAudio's NUL-terminated key constants.
fn key(bytes: &[u8]) -> CFString {
    let text = std::str::from_utf8(bytes.strip_suffix(&[0]).unwrap_or(bytes)).unwrap_or_default();
    CFString::new(text)
}

