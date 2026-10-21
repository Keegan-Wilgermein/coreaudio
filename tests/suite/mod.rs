//! # Suite
//!
//! The tests `main.rs` runs against the test device, split up by
//! what they cover
//!
//! Each function panics on failure, so `main` stops at the first
//! property that misbehaves

pub mod aggregate;
pub mod device;
pub mod errors;
pub mod io_proc;
pub mod listener;
pub mod object;

use coreaudio::{AudioObject, Device, STREAM_VIRTUAL_FORMAT, Scope};
use std::time::Duration;

/// How long to wait for CoreAudio to report an asynchronous change
pub const CHANGE_TIMEOUT: Duration = Duration::from_secs(5);

/// Channel count of every stream on one side of the device, in stream order
///
/// Read from the streams themselves, so device properties that describe
/// the streams have something independent to be checked against
pub fn stream_channels(device: AudioObject<Device>, scope: Scope) -> Vec<u32> {
    device
        .streams_with_scope(scope)
        .unwrap()
        .iter()
        .map(|stream| {
            stream
                .get_property(STREAM_VIRTUAL_FORMAT)
                .unwrap()
                .channels_per_frame()
        })
        .collect()
}
