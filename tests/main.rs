//! # Main.rs
//! 
//! The main test file; Used to decide which device to use
//! before passing that on to the other files and functions
//! 

use std::{thread, time::Duration};

use coreaudio::{AudioObject, DEVICE_NAME, Device, DeviceAggregateDevice, Scope, System};

/// Aggregate device called `Test Subject`
static TEST_DEVICE_NAME: &str = "Test Subject";

#[test]
fn main() {
    let system = AudioObject::default();
    let test = get_test_device(system, Scope::Output);

    let aggregate = DeviceAggregateDevice::new(
        "Hello",
        vec![test],
        false,
    ).unwrap();

    thread::sleep(Duration::from_secs(10));
}

/// Helper function to find a device
/// matching the name of the test device
/// 
/// This function will panic if something
/// goes wrong and that is by design
fn get_test_device(
    sys: AudioObject<System>,
    scope: Scope
) -> AudioObject<Device> {
    sys.devices_with_scope(scope)
        .unwrap()
        .iter()
        .find(|d| {
            d.get_property(DEVICE_NAME)
                .unwrap_or("".to_owned())
                .eq(TEST_DEVICE_NAME)
        }).unwrap()
        .clone()
}
