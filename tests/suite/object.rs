//! # Object
//!
//! Properties every `AudioObject` has, read from the test device

use coreaudio::{
    AudioObject, DEVICE_NAME, Device, MissingElement, MissingQualifier, OBJECT_BASE_CLASS,
    OBJECT_CLASS, OBJECT_CREATOR, OBJECT_ELEMENT_NAME, OBJECT_MANUFACTURER, OBJECT_NAME,
    OBJECT_OWNED_OBJECTS, OBJECT_OWNER,
};
use objc2_core_audio::{
    kAudioAggregateDeviceClassID, kAudioDeviceClassID, kAudioObjectSystemObject,
};

/// Runs every object test
pub fn run(device: AudioObject<Device>) {
    class(device);
    owner(device);
    names(device);
    owned_objects(device);
}

/// An aggregate device is an `AudioAggregateDevice`, a subclass of `AudioDevice`
fn class(device: AudioObject<Device>) {
    assert_eq!(
        device.get_property(OBJECT_CLASS).unwrap(),
        kAudioAggregateDeviceClassID
    );
    assert_eq!(
        device.get_property(OBJECT_BASE_CLASS).unwrap(),
        kAudioDeviceClassID
    );
}

/// Devices are owned by the system object
fn owner(device: AudioObject<Device>) {
    assert_eq!(
        device.get_property(OBJECT_OWNER).unwrap(),
        kAudioObjectSystemObject as u32
    );
}

fn names(device: AudioObject<Device>) {
    // The global name and the device-specific one read the same thing
    assert_eq!(
        device.get_property(OBJECT_NAME).unwrap(),
        device.get_property(DEVICE_NAME).unwrap()
    );

    // CoreAudio itself makes aggregate devices
    assert_eq!(
        device.get_property(OBJECT_CREATOR).unwrap(),
        "com.apple.audio.CoreAudio"
    );
    assert!(!device.get_property(OBJECT_MANUFACTURER).unwrap().is_empty());

    // Aggregates don't name their channels on the global scope, but the
    // property still has to read
    device
        .get_property(OBJECT_ELEMENT_NAME.for_element(0))
        .unwrap();
}

/// An aggregate owns its streams, so asking for every owned object has to
/// include them
fn owned_objects(device: AudioObject<Device>) {
    let owned = device
        .get_property(OBJECT_OWNED_OBJECTS.with_qualifier(Vec::new()))
        .unwrap();

    for stream in device.streams().unwrap() {
        assert!(
            owned.contains(&stream.id()),
            "stream {} isn't in the device's owned objects {owned:?}",
            stream.id()
        );
    }
}
