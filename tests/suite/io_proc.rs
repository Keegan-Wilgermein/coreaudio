//! # IO proc
//!
//! Runs an IO proc on each side of the test device

use super::stream_channels;
use coreaudio::{AudioObject, DEVICE_IS_RUNNING, Device, ErrorKind, Scope};
use std::{
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

/// How many cycles to wait for before calling the proc working
const CYCLES: u32 = 10;

/// Runs every IO proc test
pub fn run(output: AudioObject<Device>, input: AudioObject<Device>) {
    cycles(output, Scope::Output);
    cycles(input, Scope::Input);
}

/// The proc is called with one buffer per stream, the device runs while it
/// plays, and it can't be played or paused twice
fn cycles(device: AudioObject<Device>, scope: Scope) {
    let expected = stream_channels(device, scope);
    let calls = Arc::new(AtomicU32::new(0));
    let bad_buffers = Arc::new(AtomicU32::new(0));

    let mut proc = {
        let calls = calls.clone();
        let bad_buffers = bad_buffers.clone();
        device
            .add_io_proc(scope, move |buffers| {
                let channels: Vec<u32> = buffers.iter().map(|b| b.get_channels()).collect();
                let sized = buffers
                    .iter()
                    .all(|b| b.data.len() == (b.get_channels() * b.get_frame_count()) as usize);
                if channels != expected || !sized {
                    bad_buffers.fetch_add(1, Ordering::Relaxed);
                }

                // Output silence rather than whatever was in the buffer
                if scope == Scope::Output {
                    buffers.iter_mut().for_each(|b| b.data.fill(0.0));
                }
                calls.fetch_add(1, Ordering::Relaxed);
            })
            .unwrap()
    };

    // Created stopped
    thread::sleep(Duration::from_millis(100));
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    assert!(matches!(
        proc.pause().unwrap_err().kind(),
        ErrorKind::AlreadyPaused
    ));

    proc.play().unwrap();
    assert!(matches!(
        proc.play().unwrap_err().kind(),
        ErrorKind::AlreadyRunning
    ));
    assert!(device.get_property(DEVICE_IS_RUNNING).unwrap());

    let start = Instant::now();
    while calls.load(Ordering::Relaxed) < CYCLES {
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "the {scope:?} proc was only called {} times",
            calls.load(Ordering::Relaxed)
        );
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(bad_buffers.load(Ordering::Relaxed), 0);

    proc.pause().unwrap();
    proc.remove();
}
