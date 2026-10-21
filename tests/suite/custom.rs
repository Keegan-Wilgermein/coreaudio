//! # Custom
//!
//! Custom property access, on the test device
//!
//! The test device has no custom properties, so these point a custom
//! property's info at its buffer frame size instead, declared as raw data
//! The buffer size only applies to this process, so changing it disturbs
//! nothing else

use super::CHANGE_TIMEOUT;
use coreaudio::{
    AudioObject, CustomPropertyInfo, CustomValue, DEVICE_BUFFER_FRAME_SIZE,
    DEVICE_BUFFER_FRAME_SIZE_RANGE, Device, ErrorKind, Scope,
};
use objc2_core_audio::{
    AudioServerPlugInCustomPropertyInfo, kAudioDevicePropertyBufferFrameSize,
    kAudioServerPlugInCustomPropertyDataTypeCFString, kAudioServerPlugInCustomPropertyDataTypeNone,
};
use std::sync::mpsc;

/// Raw data, which the crate decodes as `CustomValue::Raw`
const RAW: u32 = u32::from_be_bytes(*b"raww");

/// Runs every custom property test
pub fn run(device: AudioObject<Device>) {
    let info = buffer_size_info(kAudioServerPlugInCustomPropertyDataTypeNone);
    let original = device.get_property(DEVICE_BUFFER_FRAME_SIZE).unwrap();
    let other = *device
        .get_property(DEVICE_BUFFER_FRAME_SIZE_RANGE)
        .unwrap()
        .valid_sizes()
        .iter()
        .find(|size| **size != original)
        .unwrap();

    read(device, info, original);
    write(device, info, original, other);
    listener(device, info, original, other);
    type_checks(device, info);

    assert_eq!(
        device.get_property(DEVICE_BUFFER_FRAME_SIZE).unwrap(),
        original
    );
}

/// An info for the buffer frame size as raw data, with `qualifier` as its
/// qualifier type
fn buffer_size_info(qualifier: u32) -> CustomPropertyInfo {
    AudioServerPlugInCustomPropertyInfo {
        mSelector: kAudioDevicePropertyBufferFrameSize,
        mPropertyDataType: RAW,
        mQualifierDataType: qualifier,
    }
    .into()
}

/// Raw data comes back as the bytes CoreAudio returned
fn read(device: AudioObject<Device>, info: CustomPropertyInfo, current: u32) {
    assert_eq!(info.four_char_code(), "fsiz");
    assert_eq!(
        device.get_custom(info, Scope::Global, 0, None).unwrap(),
        CustomValue::Raw(current.to_ne_bytes().to_vec())
    );
    assert_eq!(
        device
            .get_custom_data_size(info, Scope::Global, 0, None)
            .unwrap(),
        size_of::<u32>() as u32
    );
}

/// Raw data is written as given
fn write(device: AudioObject<Device>, info: CustomPropertyInfo, original: u32, other: u32) {
    device
        .set_custom(info, Scope::Global, 0, None, raw(other))
        .unwrap();
    assert_eq!(
        device.get_property(DEVICE_BUFFER_FRAME_SIZE).unwrap(),
        other
    );

    device
        .set_custom(info, Scope::Global, 0, None, raw(original))
        .unwrap();
    assert_eq!(
        device.get_property(DEVICE_BUFFER_FRAME_SIZE).unwrap(),
        original
    );
}

/// Each change reaches the closure decoded, and nothing does once it's removed
fn listener(device: AudioObject<Device>, info: CustomPropertyInfo, original: u32, other: u32) {
    let (sender, receiver) = mpsc::channel();
    let listener = device
        .add_custom_listener_with(info, Scope::Global, 0, move |value| {
            let _ = sender.send(value);
        })
        .unwrap();

    device
        .set_property(DEVICE_BUFFER_FRAME_SIZE, other)
        .unwrap();
    assert_eq!(
        receiver.recv_timeout(CHANGE_TIMEOUT).unwrap().unwrap(),
        raw(other)
    );

    device
        .set_property(DEVICE_BUFFER_FRAME_SIZE, original)
        .unwrap();
    assert_eq!(
        receiver.recv_timeout(CHANGE_TIMEOUT).unwrap().unwrap(),
        raw(original)
    );

    listener.remove();
    device
        .set_property(DEVICE_BUFFER_FRAME_SIZE, other)
        .unwrap();
    assert!(receiver.recv_timeout(CHANGE_TIMEOUT / 10).is_err());
    device
        .set_property(DEVICE_BUFFER_FRAME_SIZE, original)
        .unwrap();
}

/// Data that doesn't fit the declared types is refused before CoreAudio sees
/// it
fn type_checks(device: AudioObject<Device>, info: CustomPropertyInfo) {
    let bad = |error: coreaudio::CoreAudioError| matches!(error.kind(), ErrorKind::BadCustomData);

    // Raw data only takes raw bytes
    let string = CustomValue::String("512".to_string());
    assert!(bad(device
        .set_custom(info, Scope::Global, 0, None, string)
        .unwrap_err()));

    // A CoreFoundation qualifier can't be left out, and the listener reads
    // without one
    let needs_string = buffer_size_info(kAudioServerPlugInCustomPropertyDataTypeCFString);
    assert!(bad(device
        .get_custom(needs_string, Scope::Global, 0, None)
        .unwrap_err()));
    assert!(bad(device
        .add_custom_listener_with(needs_string, Scope::Global, 0, |_| {})
        .err()
        .unwrap()));

    // ...and has to be a string, not raw bytes
    let raw_qualifier = Some(CustomValue::Raw(vec![0; 8]));
    assert!(bad(device
        .get_custom(needs_string, Scope::Global, 0, raw_qualifier)
        .unwrap_err()));
}

/// A buffer size as the raw value CoreAudio holds
fn raw(size: u32) -> CustomValue {
    CustomValue::Raw(size.to_ne_bytes().to_vec())
}
