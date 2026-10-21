//! # Errors
//!
//! What comes back when the test device is asked for something it doesn't do

use coreaudio::{
    AudioObject, DEVICE_HOG_MODE, DEVICE_OUTPUT_VOLUME_SCALAR, Device, ErrorKind, MissingElement,
};

/// Runs every error test
pub fn run(device: AudioObject<Device>) {
    unknown_property(device);
}

/// Aggregates have no volume control and no hog mode of their own, so the
/// HAL's unknown property error has to come through as `UnknownProperty`
fn unknown_property(device: AudioObject<Device>) {
    let error = device
        .get_property(DEVICE_OUTPUT_VOLUME_SCALAR.for_element(0))
        .unwrap_err();
    assert!(matches!(error.kind(), ErrorKind::UnknownProperty));
    assert_eq!(error.stringify_code(), "who?");

    assert!(matches!(
        device.get_property(DEVICE_HOG_MODE).unwrap_err().kind(),
        ErrorKind::UnknownProperty
    ));
}
