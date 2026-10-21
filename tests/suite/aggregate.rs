//! # Aggregate
//!
//! Properties only aggregate devices have
//!
//! Writes put back the value that was already there, so the test device's
//! setup never changes

use coreaudio::{
    AGGREGATE_ACTIVE_SUB_DEVICE_LIST, AGGREGATE_CLOCK_DEVICE, AGGREGATE_FULL_SUB_DEVICE_LIST,
    AGGREGATE_MAIN_SUB_DEVICE, AGGREGATE_SUB_TAP_LIST, AGGREGATE_TAP_LIST, AudioObject, DEVICE_UID,
    Device, END_POINT_LIST,
};

/// Runs every aggregate test
pub fn run(device: AudioObject<Device>) {
    sub_devices(device);
    main_sub_device(device);
    clock_device(device);
    taps(device);
    end_points(device);
}

/// Every present sub-device is one of the sub-devices the aggregate lists
fn sub_devices(device: AudioObject<Device>) {
    let full = device.get_property(AGGREGATE_FULL_SUB_DEVICE_LIST).unwrap();
    assert!(!full.is_empty());

    let active = device
        .get_property(AGGREGATE_ACTIVE_SUB_DEVICE_LIST)
        .unwrap();
    assert!(active.len() <= full.len());

    for id in active {
        let uid = AudioObject::<Device>::from(id)
            .get_property(DEVICE_UID)
            .unwrap();
        assert!(full.contains(&uid), "{uid} isn't in {full:?}");
    }

    // Writing the list back unchanged is accepted and changes nothing
    device
        .set_property(AGGREGATE_FULL_SUB_DEVICE_LIST, full.clone())
        .unwrap();
    assert_eq!(
        device.get_property(AGGREGATE_FULL_SUB_DEVICE_LIST).unwrap(),
        full
    );
}

/// The clock comes from one of the sub-devices
fn main_sub_device(device: AudioObject<Device>) {
    let main = device.get_property(AGGREGATE_MAIN_SUB_DEVICE).unwrap();
    let full = device.get_property(AGGREGATE_FULL_SUB_DEVICE_LIST).unwrap();
    assert!(full.contains(&main), "{main} isn't in {full:?}");

    device
        .set_property(AGGREGATE_MAIN_SUB_DEVICE, main.clone())
        .unwrap();
    assert_eq!(
        device.get_property(AGGREGATE_MAIN_SUB_DEVICE).unwrap(),
        main
    );
}

/// The test device has no separate clock device, so this reads empty
fn clock_device(device: AudioObject<Device>) {
    assert_eq!(device.get_property(AGGREGATE_CLOCK_DEVICE).unwrap(), "");
}

/// One sub-tap per tap that is present
fn taps(device: AudioObject<Device>) {
    let taps = device.get_property(AGGREGATE_TAP_LIST).unwrap();
    let sub_taps = device.get_property(AGGREGATE_SUB_TAP_LIST).unwrap();
    assert!(sub_taps.len() <= taps.len());

    device
        .set_property(AGGREGATE_TAP_LIST, taps.clone())
        .unwrap();
    assert_eq!(device.get_property(AGGREGATE_TAP_LIST).unwrap(), taps);
}

/// Aggregates report their present sub-devices as their end points
fn end_points(device: AudioObject<Device>) {
    assert_eq!(
        device.get_property(END_POINT_LIST).unwrap(),
        device
            .get_property(AGGREGATE_ACTIVE_SUB_DEVICE_LIST)
            .unwrap()
    );
}
