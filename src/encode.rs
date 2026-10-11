//! Encoders: turn Rust values into the raw bytes CoreAudio takes when writing a property

#![allow(unsafe_code)]

use objc2_core_audio_types::{AudioStreamBasicDescription, AudioValueRange};
use objc2_core_audio::{AudioObjectID, AudioStreamRangedDescription};
use crate::{Scope, data_types::{BufferFrameSizeRange, ChannelLayout, ChannelPair, DBRange, HogMode, PowerHint, SampleRateRange, StreamDescription, StreamRangedDescription, TerminalType, TransportType}, property::Qualifier};

/// Serialises a [`PowerHint`] as a native-endian `u32`
pub(crate) fn encode_power_hint(value: PowerHint) -> Qualifier {
    let num: u32 = value.into();
    Qualifier::from_bytes(num.to_ne_bytes().to_vec())
}

/// Serialises a [`ChannelLayout`] as a variable-length `AudioChannelLayout`
pub(crate) fn encode_channel_layout(value: ChannelLayout) -> Qualifier {
    Qualifier::from_bytes(value.to_bytes())
}

/// Encodes a `String` as the `CFStringRef` CoreAudio takes
///
/// The returned value owns the `CFString` the bytes point to, so it has to
/// stay alive until `AudioObjectSetPropertyData` has returned
pub(crate) fn encode_string(value: String) -> Qualifier {
    Qualifier::from_string(&value)
}

/// Encodes an `i32` as native-endian bytes
pub(crate) fn encode_i32(value: i32) -> Qualifier {
    Qualifier::from_bytes(value.to_ne_bytes().to_vec())
}

/// Returns the raw bytes of an `AudioValueRange`
pub(crate) fn value_range_bytes(range: AudioValueRange) -> Vec<u8> {
    let mut bytes = vec![0u8; size_of::<AudioValueRange>()];
    unsafe { std::ptr::write(bytes.as_mut_ptr() as *mut AudioValueRange, range) };
    bytes
}

/// Serialises a [`BufferFrameSizeRange`] as an `AudioValueRange`
pub(crate) fn encode_buffer_size_range(value: BufferFrameSizeRange) -> Qualifier {
    Qualifier::from_bytes(value_range_bytes(value.into()))
}

/// Serialises a [`DBRange`] as an `AudioValueRange`
pub(crate) fn encode_db_range(value: DBRange) -> Qualifier {
    Qualifier::from_bytes(value_range_bytes(value.into()))
}

/// Serialises a [`Scope`] as a native-endian `u32`
pub(crate) fn encode_scope(value: Scope) -> Qualifier {
    Qualifier::from_bytes(bytes_u32(value.into()))
}

/// Serialises a [`TerminalType`] as a native-endian `u32`
pub(crate) fn encode_terminal_type(value: TerminalType) -> Qualifier {
    Qualifier::from_bytes(bytes_u32(value.into()))
}

/// Serialises a [`TransportType`] as a native-endian `u32`
pub(crate) fn encode_transport_type(value: TransportType) -> Qualifier {
    Qualifier::from_bytes(bytes_u32(value.into()))
}

/// Serialises a packed array of [`SampleRateRange`] as `AudioValueRange`s
pub(crate) fn encode_vec_sample_rate_range(value: Vec<SampleRateRange>) -> Qualifier {
    Qualifier::from_bytes(value.into_iter().flat_map(|range| value_range_bytes(range.into())).collect())
}

/// Serialises a packed array of [`StreamRangedDescription`] as
/// `AudioStreamRangedDescription`s
pub(crate) fn encode_vec_stream_ranged_description(value: Vec<StreamRangedDescription>) -> Qualifier {
    Qualifier::from_bytes(
        value.into_iter().flat_map(|description| {
            let raw: AudioStreamRangedDescription = description.into();
            let mut bytes = vec![0u8; size_of::<AudioStreamRangedDescription>()];
            unsafe { std::ptr::write(bytes.as_mut_ptr() as *mut AudioStreamRangedDescription, raw) };
            bytes
        }).collect()
    )
}

/// Encodes an `f64` as native-endian bytes
pub(crate) fn encode_f64(value: f64) -> Qualifier {
    Qualifier::from_bytes(value.to_ne_bytes().to_vec())
}

/// Native-endian bytes of a `u32`
pub(crate) fn bytes_u32(value: u32) -> Vec<u8> {
    value.to_ne_bytes().to_vec()
}

/// Encodes a `u32` as native-endian bytes
pub(crate) fn encode_u32(value: u32) -> Qualifier {
    Qualifier::from_bytes(bytes_u32(value))
}

/// Packed native-endian bytes of a `Vec<u32>`
pub(crate) fn bytes_vec_u32(value: Vec<u32>) -> Vec<u8> {
    value.iter().flat_map(|n| n.to_ne_bytes()).collect()
}

/// Encodes a `Vec<u32>` as a packed array of native-endian bytes
pub(crate) fn encode_vec_u32(value: Vec<u32>) -> Qualifier {
    Qualifier::from_bytes(bytes_vec_u32(value))
}

/// Encodes a `bool` as a `u32` (`0` or `1`) in native-endian bytes
pub(crate) fn encode_bool(value: bool) -> Qualifier {
    Qualifier::from_bytes(bytes_u32(value as u32))
}

/// Encodes an `AudioObjectID` (`u32`) as native-endian bytes
pub(crate) fn encode_audio_object_id(value: AudioObjectID) -> Qualifier {
    Qualifier::from_bytes(bytes_u32(value))
}

/// Serialises a [`StreamDescription`] into an `AudioStreamBasicDescription`
/// and returns its raw bytes
pub(crate) fn encode_stream_description(value: StreamDescription) -> Qualifier {
    let asbd: AudioStreamBasicDescription = value.into();
    let size = size_of::<AudioStreamBasicDescription>();
    let mut bytes = vec![0u8; size];
    unsafe {
        std::ptr::write(bytes.as_mut_ptr() as *mut AudioStreamBasicDescription, asbd);
    }
    Qualifier::from_bytes(bytes)
}

/// Encodes an `f32` as native-endian bytes
pub(crate) fn encode_f32(value: f32) -> Qualifier {
    Qualifier::from_bytes(value.to_ne_bytes().to_vec())
}

/// Serialises a [`ChannelPair`] as two consecutive native-endian `u32` values
pub(crate) fn encode_channel_pair(value: ChannelPair) -> Qualifier {
    let mut bytes = Vec::with_capacity(8);
    bytes.extend_from_slice(&value.left().to_ne_bytes());
    bytes.extend_from_slice(&value.right().to_ne_bytes());
    Qualifier::from_bytes(bytes)
}

/// Serialises a [`HogMode`] as a native-endian `i32`
pub(crate) fn encode_hog_mode(value: HogMode) -> Qualifier {
    let num: i32 = value.into();
    Qualifier::from_bytes(num.to_ne_bytes().to_vec())
}
