//! Readers: turn the raw bytes CoreAudio returns for a property into Rust values

#![allow(unsafe_code)]

use core_foundation::{base::TCFType, string::{CFString, CFStringRef}};
use objc2_core_audio_types::{AudioStreamBasicDescription, AudioValueRange};
use objc2_core_audio::{AudioObjectID, AudioStreamRangedDescription};
use crate::{Scope, data_types::{BufferFrameSizeRange, ChannelLayout, ChannelPair, DBRange, HogMode, PowerHint, SampleRateRange, StreamDescription, StreamRangedDescription, TerminalType, TransportType}, errors::{CoreAudioError, ErrorKind}};

/// Reads a variable-length `AudioChannelLayout` from `bytes`
pub(crate) fn read_channel_layout(bytes: &[u8]) -> Result<ChannelLayout, CoreAudioError> {
    ChannelLayout::from_bytes(bytes)
}

/// Reads a `CFStringRef` from `bytes` and converts it to a Rust `String`
pub(crate) fn read_string(bytes: &[u8]) -> Result<String, CoreAudioError> {
    if bytes.len() != size_of::<CFStringRef>() {
        return Err(CoreAudioError::from_error_kind(ErrorKind::CFStringConversion));
    }

    let ptr = usize::from_ne_bytes(bytes[..size_of::<usize>()].try_into()
        .map_err(|_| CoreAudioError::from_error_kind(ErrorKind::CFStringConversion))?
    ) as CFStringRef;

    if ptr.is_null() {
        return Err(CoreAudioError::from_error_kind(ErrorKind::CFStringConversion));
    }

    let cf_string = unsafe { CFString::wrap_under_get_rule(ptr) };
    Ok(cf_string.to_string())
}

/// Reads a `u32` from `bytes` and returns `true` if it is non-zero
pub(crate) fn read_bool(bytes: &[u8]) -> Result<bool, CoreAudioError> {
    let value = u32::from_ne_bytes(
        match bytes[..4].try_into() {
            Ok(value) => value,
            Err(_) => return Err(CoreAudioError::from_error_kind(ErrorKind::BoolConversion)),
        }
    ) != 0;

    Ok(value)
}

/// Reads an `f64` from `bytes` in native byte order
pub(crate) fn read_f64(bytes: &[u8]) -> Result<f64, CoreAudioError> {
    let value = f64::from_ne_bytes(
        match bytes[..8].try_into() {
            Ok(value) => value,
            Err(_) => return Err(CoreAudioError::from_error_kind(ErrorKind::FPConversion)),
        }
    );

    Ok(value)
}

/// Reads a `u32` from `bytes` in native byte order
pub(crate) fn read_u32(bytes: &[u8]) -> Result<u32, CoreAudioError> {
    let value = u32::from_ne_bytes(
        match bytes[..4].try_into() {
            Ok(value) => value,
            Err(_) => return Err(CoreAudioError::from_error_kind(ErrorKind::U32Conversion)),
        }
    );

    Ok(value)
}

/// Reads an `i32` from `bytes` in native byte order
pub(crate) fn read_i32(bytes: &[u8]) -> Result<i32, CoreAudioError> {
    let value = i32::from_ne_bytes(
        match bytes[..4].try_into() {
            Ok(value) => value,
            Err(_) => return Err(CoreAudioError::from_error_kind(ErrorKind::I32Conversion)),
        }
    );

    Ok(value)
}

/// Reads a single `AudioObjectID` (`u32`) from `bytes`
pub(crate) fn read_audio_object_id(bytes: &[u8]) -> Result<AudioObjectID, CoreAudioError> {
    let value = u32::from_ne_bytes(
        match bytes[..4].try_into() {
            Ok(value) => value,
            Err(_) => return Err(CoreAudioError::from_error_kind(ErrorKind::AudioObjectIdConversion)),
        }
    );

    Ok(value)
}

/// Reads an `AudioStreamBasicDescription` from `bytes` and converts it to
/// a [`StreamDescription`]
pub(crate) fn read_stream_description(bytes: &[u8]) -> Result<StreamDescription, CoreAudioError> {
    if bytes.len() != size_of::<AudioStreamBasicDescription>() {
        return Err(CoreAudioError::from_error_kind(ErrorKind::StreamDescriptionConversion));
    }

    let asbd = unsafe { 
        std::ptr::read(bytes.as_ptr() as *const AudioStreamBasicDescription) 
    };
    
    StreamDescription::try_from(asbd)
        .map_err(|_| CoreAudioError::from_error_kind(ErrorKind::StreamDescriptionConversion))
}

/// Reads an `AudioValueRange` from `bytes` and converts it to a
/// [`BufferFrameSizeRange`]
pub(crate) fn read_buffer_size_range(bytes: &[u8]) -> Result<BufferFrameSizeRange, CoreAudioError> {
    if bytes.len() != size_of::<AudioValueRange>() {
        return Err(CoreAudioError::from_error_kind(ErrorKind::ValueRangeConversion));
    }

    let range = unsafe {
        std::ptr::read(bytes.as_ptr() as *const AudioValueRange)
    };

    Ok(BufferFrameSizeRange::from(range))
}

/// Reads a packed array of `AudioObjectID` values from `bytes`
pub(crate) fn read_vec_audio_object_id(bytes: &[u8]) -> Result<Vec<AudioObjectID>, CoreAudioError> {
    if bytes.len() % size_of::<AudioObjectID>() != 0 {
        return Err(CoreAudioError::from_error_kind(ErrorKind::AudioObjectIdConversion));
    }

    Ok(bytes.chunks(size_of::<AudioObjectID>())
    .map(|chunk| AudioObjectID::from_ne_bytes(chunk.try_into().unwrap()))
    .collect())
}

/// Reads a packed array of `AudioValueRange` values and converts each to a
/// [`SampleRateRange`]
pub(crate) fn read_vec_sample_rate_range(bytes: &[u8]) -> Result<Vec<SampleRateRange>, CoreAudioError> {
    if bytes.len() % size_of::<AudioValueRange>() != 0 {
        return Err(CoreAudioError::from_error_kind(ErrorKind::ValueRangeConversion));
    }

    Ok(bytes.chunks(size_of::<AudioValueRange>())
        .map(|chunk| unsafe { 
            SampleRateRange::from(std::ptr::read(chunk.as_ptr() as *const AudioValueRange)) 
        })
        .collect())
}

/// Reads a packed array of `AudioStreamRangedDescription` values and converts
/// each to a [`StreamRangedDescription`]
pub(crate) fn read_vec_stream_ranged_description(bytes: &[u8]) -> Result<Vec<StreamRangedDescription>, CoreAudioError> {
    if bytes.len() % size_of::<AudioStreamRangedDescription>() != 0 {
        return Err(CoreAudioError::from_error_kind(ErrorKind::StreamDescriptionConversion));
    }

    Ok(bytes.chunks(size_of::<AudioStreamRangedDescription>())
    .map(|chunk| unsafe {
        StreamRangedDescription::from(
            std::ptr::read(chunk.as_ptr() as *const AudioStreamRangedDescription)
        )
    })
    .collect())
}

/// Reads an `f32` from `bytes` in native byte order
pub(crate) fn read_f32(bytes: &[u8]) -> Result<f32, CoreAudioError> {
    let value = f32::from_ne_bytes(
        match bytes[..4].try_into() {
            Ok(value) => value,
            Err(_) => return Err(CoreAudioError::from_error_kind(ErrorKind::FPConversion)),
        }
    );

    Ok(value)
}

/// Reads two consecutive `u32` values from `bytes` and returns them as a
/// [`ChannelPair`]
pub(crate) fn read_channel_pair(bytes: &[u8]) -> Result<ChannelPair, CoreAudioError> {
    if bytes.len() < 8 {
        return Err(CoreAudioError::from_error_kind(ErrorKind::U32Conversion));
    }

    let a = u32::from_ne_bytes(bytes[..4].try_into()
        .map_err(|_| CoreAudioError::from_error_kind(ErrorKind::U32Conversion))?);
    let b = u32::from_ne_bytes(bytes[4..8].try_into()
        .map_err(|_| CoreAudioError::from_error_kind(ErrorKind::U32Conversion))?);

    Ok([a, b].into())
}

/// Reads an `AudioValueRange` from `bytes` and converts it to a [`DBRange`]
pub(crate) fn read_db_range(bytes: &[u8]) -> Result<DBRange, CoreAudioError> {
    if bytes.len() != size_of::<AudioValueRange>() {
        return Err(CoreAudioError::from_error_kind(ErrorKind::ValueRangeConversion));
    }

    Ok(unsafe { std::ptr::read(bytes.as_ptr() as *const AudioValueRange) }.into())
}

/// Reads an `i32` from `bytes` and converts it to a [`HogMode`]
pub(crate) fn read_hog_mode(bytes: &[u8]) -> Result<HogMode, CoreAudioError> {
    if bytes.len() != size_of::<i32>() {
        return Err(CoreAudioError::from_error_kind(ErrorKind::HogModeConversion));
    }

    Ok(unsafe { std::ptr::read(bytes.as_ptr() as *const i32) }.into())
}

/// Reads a `u32` from `bytes` and converts it to a [`Scope`]
pub(crate) fn read_scope(bytes: &[u8]) -> Result<Scope, CoreAudioError> {
    if bytes.len() != size_of::<u32>() {
        return Err(CoreAudioError::from_error_kind(ErrorKind::ScopeConversion));
    }

    unsafe { std::ptr::read(bytes.as_ptr() as *const u32) }.try_into()
}

/// Reads a `u32` from `bytes` and converts it to a [`PowerHint`]
pub(crate) fn read_power_hint(bytes: &[u8]) -> Result<PowerHint, CoreAudioError> {
    if bytes.len() != size_of::<u32>() {
        return Err(CoreAudioError::from_error_kind(ErrorKind::PowerHintConversion));
    }

    unsafe { std::ptr::read(bytes.as_ptr() as *const u32) }.try_into()
}

/// Reads a `u32` from `bytes` and converts it to a [`TerminalType`]
pub(crate) fn read_terminal_type(bytes: &[u8]) -> Result<TerminalType, CoreAudioError> {
    if bytes.len() != size_of::<u32>() {
        return Err(CoreAudioError::from_error_kind(ErrorKind::PowerHintConversion));
    }

    Ok(unsafe { std::ptr::read(bytes.as_ptr() as *const u32) }.into())
}

/// Reads a `u32` from `bytes` and converts it to a [`TransportType`]
pub(crate) fn read_transport_type(bytes: &[u8]) -> Result<TransportType, CoreAudioError> {
    if bytes.len() != size_of::<u32>() {
        return Err(CoreAudioError::from_error_kind(ErrorKind::PowerHintConversion));
    }

    Ok(unsafe { std::ptr::read(bytes.as_ptr() as *const u32) }.into())
}
