//! # Listener
//!
//! Both kinds of listener, driven by changing the test device's sample rate

use super::CHANGE_TIMEOUT;
use coreaudio::{
    AudioObject, DEVICE_AVAILABLE_SAMPLE_RATES, DEVICE_NOMINAL_SAMPLE_RATE, Device, ErrorKind,
};
use std::{sync::mpsc, time::Duration};

/// Runs every listener test
pub fn run(device: AudioObject<Device>) {
    let original = device.get_property(DEVICE_NOMINAL_SAMPLE_RATE).unwrap();
    let other = other_rate(device, original);

    property_listener(device, original, other);
    callback_listener(device, original, other);

    assert_eq!(
        device.get_property(DEVICE_NOMINAL_SAMPLE_RATE).unwrap(),
        original
    );
}

/// A supported rate that isn't `current`
fn other_rate(device: AudioObject<Device>, current: f64) -> f64 {
    let available = device.get_property(DEVICE_AVAILABLE_SAMPLE_RATES).unwrap();
    [48000.0, 44100.0]
        .into_iter()
        .find(|rate| {
            *rate != current
                && available
                    .iter()
                    .any(|range| range.get_min() <= *rate && *rate <= range.get_max())
        })
        .expect("the test device should support 44.1 or 48 kHz")
}

fn property_listener(device: AudioObject<Device>, original: f64, other: f64) {
    let listener = device.add_listener(DEVICE_NOMINAL_SAMPLE_RATE).unwrap();

    // Nothing has changed yet
    assert!(listener.latest().is_none());
    assert!(matches!(
        listener
            .block_for_duration(Duration::from_millis(100))
            .unwrap_err()
            .kind(),
        ErrorKind::ListenerTimeout
    ));

    device
        .set_property(DEVICE_NOMINAL_SAMPLE_RATE, other)
        .unwrap();
    assert_eq!(listener.block_for_duration(CHANGE_TIMEOUT).unwrap(), other);

    device
        .set_property(DEVICE_NOMINAL_SAMPLE_RATE, original)
        .unwrap();
    assert_eq!(
        listener.block_for_duration(CHANGE_TIMEOUT).unwrap(),
        original
    );

    // Everything has been taken off the queue
    assert!(listener.all_since_last_check().is_empty());
    listener.remove();
}

fn callback_listener(device: AudioObject<Device>, original: f64, other: f64) {
    let (sender, receiver) = mpsc::channel();
    let listener = device
        .add_listener_with(DEVICE_NOMINAL_SAMPLE_RATE, move |rate| {
            let _ = sender.send(rate);
        })
        .unwrap();

    device
        .set_property(DEVICE_NOMINAL_SAMPLE_RATE, other)
        .unwrap();
    assert_eq!(
        receiver.recv_timeout(CHANGE_TIMEOUT).unwrap().unwrap(),
        other
    );

    device
        .set_property(DEVICE_NOMINAL_SAMPLE_RATE, original)
        .unwrap();
    assert_eq!(
        receiver.recv_timeout(CHANGE_TIMEOUT).unwrap().unwrap(),
        original
    );

    // Once removed, changes no longer reach the closure
    listener.remove();
    device
        .set_property(DEVICE_NOMINAL_SAMPLE_RATE, other)
        .unwrap();
    assert!(receiver.recv_timeout(Duration::from_millis(500)).is_err());

    restore(device, original);
}

/// Puts the sample rate back and waits for it to take effect
fn restore(device: AudioObject<Device>, original: f64) {
    let listener = device.add_listener(DEVICE_NOMINAL_SAMPLE_RATE).unwrap();
    device
        .set_property(DEVICE_NOMINAL_SAMPLE_RATE, original)
        .unwrap();

    while device.get_property(DEVICE_NOMINAL_SAMPLE_RATE).unwrap() != original {
        listener.block_for_duration(CHANGE_TIMEOUT).unwrap();
    }
}
