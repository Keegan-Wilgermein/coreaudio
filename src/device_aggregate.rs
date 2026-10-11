//! Aggregate devices built from other devices
//!
//! An aggregate combines several devices into one, with the channels of every
//! device side by side, so they can be read and written through a single
//! [`AudioObject<Device>`] and a single IO proc
//! CoreAudio keeps the devices in sync by resampling all but the first, which
//! is the aggregate's clock source

#![allow(unsafe_code)]

// ---- Imports ------------
use crate::{AudioObject, CoreAudioError, Device, ErrorKind, errors::OSStatusCheck, property::DEVICE_UID};
use std::{ffi::CStr, sync::atomic::{AtomicU32, Ordering}};
use core_foundation::{array::CFArray, base::{CFType, TCFType}, boolean::CFBoolean, dictionary::CFDictionary, string::CFString};
use objc2_core_audio::{
    AudioHardwareCreateAggregateDevice, AudioHardwareDestroyAggregateDevice, AudioObjectID,
    kAudioAggregateDeviceIsPrivateKey, kAudioAggregateDeviceIsStackedKey,
    kAudioAggregateDeviceMainSubDeviceKey, kAudioAggregateDeviceNameKey,
    kAudioAggregateDeviceSubDeviceListKey, kAudioAggregateDeviceUIDKey,
    kAudioSubDeviceDriftCompensationKey, kAudioSubDeviceUIDKey,
};

// ---- Constants ------------

/// Counts the aggregates made by this process, so every one gets its own UID
static NEXT_UID: AtomicU32 = AtomicU32::new(0);

// ---- Structs ------------

/// An aggregate device made of other devices, destroyed when dropped
///
/// A private aggregate is only visible to the process that created it and
/// macOS destroys it if the process exits without dropping it
/// 
/// A public aggregate is visible system-wide while it exists
pub struct DeviceAggregateDevice {
    /// The aggregate as a device
    device: AudioObject<Device>,
}

impl DeviceAggregateDevice {
    /// Creates an aggregate of `devices`
    ///
    /// The first device is the clock source, and every other device is drift
    /// corrected to it
    /// The aggregate gets a UID that is unique within this process
    ///
    /// Fails with [`ErrorKind::BadDevice`] if `devices` is empty
    /// Drop the aggregate before the devices it contains
    pub fn new(
        name: &str,
        devices: Vec<AudioObject<Device>>,
        private: bool,
    ) -> Result<Self, CoreAudioError> {
        let uids = devices
            .iter()
            .map(|device| device.get_property(DEVICE_UID))
            .collect::<Result<Vec<String>, CoreAudioError>>()?;

        let description = aggregate_description(name, &next_uid(), &uids, private)
            .ok_or_else(|| CoreAudioError::from_error_kind(ErrorKind::BadDevice))?;

        let mut id: AudioObjectID = 0;
        unsafe {
            AudioHardwareCreateAggregateDevice(
                &*description.as_concrete_TypeRef().cast(),
                (&mut id).into(),
            ).check()?;
        }

        Ok(Self { device: AudioObject::<Device>::from(id) })
    }

    /// The aggregate as a device, to read properties from and add an IO proc to
    ///
    /// This is not the underlying devices that this aggregate contains
    pub fn device(&self) -> AudioObject<Device> {
        self.device
    }


}

impl Drop for DeviceAggregateDevice {
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

/// A UID no other aggregate made by this process has
fn next_uid() -> String {
    format!(
        "coreaudio.aggregate.{}.{}",
        std::process::id(),
        NEXT_UID.fetch_add(1, Ordering::Relaxed),
    )
}

/// Creates an aggregate device description from the UIDs of its devices
///
/// The first UID becomes the main sub-device that clocks the aggregate
/// Returns `None` if `device_uids` is empty
fn aggregate_description(
    name: &str,
    uid: &str,
    device_uids: &[String],
    private: bool,
) -> Option<CFDictionary<CFString, CFType>> {
    let main = device_uids.first()?;

    let sub_devices: Vec<CFDictionary<CFString, CFType>> = device_uids
        .iter()
        .enumerate()
        .map(|(index, device_uid)| {
            // The main device is the clock, so it has nothing to correct to
            let drift_corrected = if index == 0 {
                CFBoolean::false_value()
            } else {
                CFBoolean::true_value()
            };

            CFDictionary::from_CFType_pairs(&[
                (key(kAudioSubDeviceUIDKey), CFString::new(device_uid).as_CFType()),
                (key(kAudioSubDeviceDriftCompensationKey), drift_corrected.as_CFType()),
            ])
        })
        .collect();

    let private = if private {
        CFBoolean::true_value().as_CFType()
    } else {
        CFBoolean::false_value().as_CFType()
    };

    Some(CFDictionary::from_CFType_pairs(&[
        (key(kAudioAggregateDeviceNameKey), CFString::new(name).as_CFType()),
        (key(kAudioAggregateDeviceUIDKey), CFString::new(uid).as_CFType()),
        (key(kAudioAggregateDeviceMainSubDeviceKey), CFString::new(main).as_CFType()),
        (key(kAudioAggregateDeviceIsPrivateKey), private),
        (key(kAudioAggregateDeviceIsStackedKey), CFBoolean::false_value().as_CFType()),
        (key(kAudioAggregateDeviceSubDeviceListKey), CFArray::from_CFTypes(&sub_devices).as_CFType()),
    ]))
}
