//! Readers: turn the raw bytes CoreAudio returns for a property into Rust values

#![allow(unsafe_code)]

use crate::{
    Scope,
    data_types::{
        BufferFrameSizeRange, ChannelLayout, ChannelPair, ClockAlgorithm, CustomPropertyDataType,
        CustomPropertyInfo, CustomValue, DBRange, HogMode, PowerHint, PropertyList,
        SampleRateRange, StreamDescription, StreamRangedDescription, TerminalType, TransportType,
        time_from_absolute,
    },
    errors::{CoreAudioError, ErrorKind},
};
use core_foundation::{
    array::{CFArray, CFArrayRef},
    base::{CFType, CFTypeRef, TCFType},
    boolean::CFBoolean,
    data::CFData,
    date::CFDate,
    dictionary::CFDictionary,
    number::{CFNumber, CFNumberIsFloatType},
    string::{CFString, CFStringRef},
    url::{CFURL, CFURLRef},
};
use objc2_core_audio::{
    AudioObjectID, AudioServerPlugInCustomPropertyInfo, AudioStreamRangedDescription,
};
use objc2_core_audio_types::{
    AudioBuffer, AudioBufferList, AudioStreamBasicDescription, AudioValueRange,
};
use std::{mem::offset_of, path::PathBuf};

/// Reads a variable-length `AudioChannelLayout` from `bytes`
pub(crate) fn read_channel_layout(bytes: &[u8]) -> Result<ChannelLayout, CoreAudioError> {
    ChannelLayout::from_bytes(bytes)
}

/// Reads a `CFStringRef` from `bytes` and converts it to a Rust `String`
///
/// CoreAudio hands the caller ownership of the string, so it is released here
pub(crate) fn read_string(bytes: &[u8]) -> Result<String, CoreAudioError> {
    if bytes.len() != size_of::<CFStringRef>() {
        return Err(CoreAudioError::from_error_kind(
            ErrorKind::CFStringConversion,
        ));
    }

    let ptr = usize::from_ne_bytes(
        bytes[..size_of::<usize>()]
            .try_into()
            .map_err(|_| CoreAudioError::from_error_kind(ErrorKind::CFStringConversion))?,
    ) as CFStringRef;

    if ptr.is_null() {
        return Err(CoreAudioError::from_error_kind(
            ErrorKind::CFStringConversion,
        ));
    }

    let cf_string = unsafe { CFString::wrap_under_create_rule(ptr) };
    Ok(cf_string.to_string())
}

/// Reads a `u32` from `bytes` and returns `true` if it is non-zero
pub(crate) fn read_bool(bytes: &[u8]) -> Result<bool, CoreAudioError> {
    let value = u32::from_ne_bytes(match bytes[..4].try_into() {
        Ok(value) => value,
        Err(_) => return Err(CoreAudioError::from_error_kind(ErrorKind::BoolConversion)),
    }) != 0;

    Ok(value)
}

/// Reads an `f64` from `bytes` in native byte order
pub(crate) fn read_f64(bytes: &[u8]) -> Result<f64, CoreAudioError> {
    let value = f64::from_ne_bytes(match bytes[..8].try_into() {
        Ok(value) => value,
        Err(_) => return Err(CoreAudioError::from_error_kind(ErrorKind::FPConversion)),
    });

    Ok(value)
}

/// Reads a `u32` from `bytes` in native byte order
pub(crate) fn read_u32(bytes: &[u8]) -> Result<u32, CoreAudioError> {
    let value = u32::from_ne_bytes(match bytes[..4].try_into() {
        Ok(value) => value,
        Err(_) => return Err(CoreAudioError::from_error_kind(ErrorKind::U32Conversion)),
    });

    Ok(value)
}

/// Reads an `i32` from `bytes` in native byte order
pub(crate) fn read_i32(bytes: &[u8]) -> Result<i32, CoreAudioError> {
    let value = i32::from_ne_bytes(match bytes[..4].try_into() {
        Ok(value) => value,
        Err(_) => return Err(CoreAudioError::from_error_kind(ErrorKind::I32Conversion)),
    });

    Ok(value)
}

/// Reads a single `AudioObjectID` (`u32`) from `bytes`
pub(crate) fn read_audio_object_id(bytes: &[u8]) -> Result<AudioObjectID, CoreAudioError> {
    let value = u32::from_ne_bytes(match bytes[..4].try_into() {
        Ok(value) => value,
        Err(_) => {
            return Err(CoreAudioError::from_error_kind(
                ErrorKind::AudioObjectIdConversion,
            ));
        }
    });

    Ok(value)
}

/// Reads an `AudioStreamBasicDescription` from `bytes` and converts it to
/// a [`StreamDescription`]
pub(crate) fn read_stream_description(bytes: &[u8]) -> Result<StreamDescription, CoreAudioError> {
    if bytes.len() != size_of::<AudioStreamBasicDescription>() {
        return Err(CoreAudioError::from_error_kind(
            ErrorKind::StreamDescriptionConversion,
        ));
    }

    let asbd = unsafe { std::ptr::read(bytes.as_ptr() as *const AudioStreamBasicDescription) };

    Ok(StreamDescription::from(asbd))
}

/// Reads an `AudioValueRange` from `bytes` and converts it to a
/// [`BufferFrameSizeRange`]
pub(crate) fn read_buffer_size_range(bytes: &[u8]) -> Result<BufferFrameSizeRange, CoreAudioError> {
    if bytes.len() != size_of::<AudioValueRange>() {
        return Err(CoreAudioError::from_error_kind(
            ErrorKind::ValueRangeConversion,
        ));
    }

    let range = unsafe { std::ptr::read(bytes.as_ptr() as *const AudioValueRange) };

    Ok(BufferFrameSizeRange::from(range))
}

/// Reads a packed array of `AudioObjectID` values from `bytes`
pub(crate) fn read_vec_audio_object_id(bytes: &[u8]) -> Result<Vec<AudioObjectID>, CoreAudioError> {
    if !bytes.len().is_multiple_of(size_of::<AudioObjectID>()) {
        return Err(CoreAudioError::from_error_kind(
            ErrorKind::AudioObjectIdConversion,
        ));
    }

    Ok(bytes
        .chunks(size_of::<AudioObjectID>())
        .map(|chunk| AudioObjectID::from_ne_bytes(chunk.try_into().unwrap()))
        .collect())
}

/// Reads a packed array of `AudioValueRange` values and converts each to a
/// [`SampleRateRange`]
pub(crate) fn read_vec_sample_rate_range(
    bytes: &[u8],
) -> Result<Vec<SampleRateRange>, CoreAudioError> {
    if !bytes.len().is_multiple_of(size_of::<AudioValueRange>()) {
        return Err(CoreAudioError::from_error_kind(
            ErrorKind::ValueRangeConversion,
        ));
    }

    Ok(bytes
        .chunks(size_of::<AudioValueRange>())
        .map(|chunk| unsafe {
            SampleRateRange::from(std::ptr::read(chunk.as_ptr() as *const AudioValueRange))
        })
        .collect())
}

/// Reads a packed array of `AudioStreamRangedDescription` values and converts
/// each to a [`StreamRangedDescription`]
pub(crate) fn read_vec_stream_ranged_description(
    bytes: &[u8],
) -> Result<Vec<StreamRangedDescription>, CoreAudioError> {
    if !bytes
        .len()
        .is_multiple_of(size_of::<AudioStreamRangedDescription>())
    {
        return Err(CoreAudioError::from_error_kind(
            ErrorKind::StreamDescriptionConversion,
        ));
    }

    Ok(bytes
        .chunks(size_of::<AudioStreamRangedDescription>())
        .map(|chunk| unsafe {
            StreamRangedDescription::from(std::ptr::read(
                chunk.as_ptr() as *const AudioStreamRangedDescription
            ))
        })
        .collect())
}

/// Reads an `f32` from `bytes` in native byte order
pub(crate) fn read_f32(bytes: &[u8]) -> Result<f32, CoreAudioError> {
    let value = f32::from_ne_bytes(match bytes[..4].try_into() {
        Ok(value) => value,
        Err(_) => return Err(CoreAudioError::from_error_kind(ErrorKind::FPConversion)),
    });

    Ok(value)
}

/// Reads two consecutive `u32` values from `bytes` and returns them as a
/// [`ChannelPair`]
pub(crate) fn read_channel_pair(bytes: &[u8]) -> Result<ChannelPair, CoreAudioError> {
    if bytes.len() < 8 {
        return Err(CoreAudioError::from_error_kind(ErrorKind::U32Conversion));
    }

    let a = u32::from_ne_bytes(
        bytes[..4]
            .try_into()
            .map_err(|_| CoreAudioError::from_error_kind(ErrorKind::U32Conversion))?,
    );
    let b = u32::from_ne_bytes(
        bytes[4..8]
            .try_into()
            .map_err(|_| CoreAudioError::from_error_kind(ErrorKind::U32Conversion))?,
    );

    Ok([a, b].into())
}

/// Reads an `AudioValueRange` from `bytes` and converts it to a [`DBRange`]
pub(crate) fn read_db_range(bytes: &[u8]) -> Result<DBRange, CoreAudioError> {
    if bytes.len() != size_of::<AudioValueRange>() {
        return Err(CoreAudioError::from_error_kind(
            ErrorKind::ValueRangeConversion,
        ));
    }

    Ok(unsafe { std::ptr::read(bytes.as_ptr() as *const AudioValueRange) }.into())
}

/// Reads an `i32` from `bytes` and converts it to a [`HogMode`]
pub(crate) fn read_hog_mode(bytes: &[u8]) -> Result<HogMode, CoreAudioError> {
    if bytes.len() != size_of::<i32>() {
        return Err(CoreAudioError::from_error_kind(
            ErrorKind::HogModeConversion,
        ));
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
        return Err(CoreAudioError::from_error_kind(
            ErrorKind::PowerHintConversion,
        ));
    }

    unsafe { std::ptr::read(bytes.as_ptr() as *const u32) }.try_into()
}

/// Reads a `u32` from `bytes` and converts it to a [`TerminalType`]
pub(crate) fn read_terminal_type(bytes: &[u8]) -> Result<TerminalType, CoreAudioError> {
    if bytes.len() != size_of::<u32>() {
        return Err(CoreAudioError::from_error_kind(
            ErrorKind::PowerHintConversion,
        ));
    }

    Ok(unsafe { std::ptr::read(bytes.as_ptr() as *const u32) }.into())
}

/// Reads a `u32` from `bytes` and converts it to a [`TransportType`]
pub(crate) fn read_transport_type(bytes: &[u8]) -> Result<TransportType, CoreAudioError> {
    if bytes.len() != size_of::<u32>() {
        return Err(CoreAudioError::from_error_kind(
            ErrorKind::PowerHintConversion,
        ));
    }

    Ok(unsafe { std::ptr::read(bytes.as_ptr() as *const u32) }.into())
}

/// Reads a `u32` from `bytes` and converts it to a [`ClockAlgorithm`]
pub(crate) fn read_clock_algorithm(bytes: &[u8]) -> Result<ClockAlgorithm, CoreAudioError> {
    read_u32(bytes).map(ClockAlgorithm::from)
}

/// Reads a pointer-sized CoreFoundation reference from `bytes`, or `None`
/// if it isn't one or is null
fn read_cf_pointer(bytes: &[u8]) -> Option<usize> {
    let pointer = usize::from_ne_bytes(bytes.try_into().ok()?);
    (pointer != 0).then_some(pointer)
}

/// Reads a `CFURLRef` from `bytes` and converts it to a file system path
///
/// CoreAudio hands the caller ownership of the URL, so it is released here
pub(crate) fn read_path(bytes: &[u8]) -> Result<PathBuf, CoreAudioError> {
    let pointer = read_cf_pointer(bytes)
        .ok_or_else(|| CoreAudioError::from_error_kind(ErrorKind::CFStringConversion))?;

    let url = unsafe { CFURL::wrap_under_create_rule(pointer as CFURLRef) };
    url.to_path()
        .ok_or_else(|| CoreAudioError::from_error_kind(ErrorKind::CFStringConversion))
}

/// Reads a `CFArrayRef` of `CFString`s from `bytes` and converts it to a
/// `Vec<String>`
///
/// CoreAudio hands the caller ownership of the array, so it is released here
pub(crate) fn read_vec_string(bytes: &[u8]) -> Result<Vec<String>, CoreAudioError> {
    let pointer = read_cf_pointer(bytes)
        .ok_or_else(|| CoreAudioError::from_error_kind(ErrorKind::CFStringConversion))?;

    let array: CFArray<CFString> =
        unsafe { CFArray::wrap_under_create_rule(pointer as CFArrayRef) };
    Ok(array.iter().map(|string| string.to_string()).collect())
}

/// Reads an `AudioBufferList` from `bytes` and returns the channel count of
/// each buffer (stream) in it
pub(crate) fn read_buffer_list_channels(bytes: &[u8]) -> Result<Vec<u32>, CoreAudioError> {
    let buffers_start = offset_of!(AudioBufferList, mBuffers);
    let channels_offset = offset_of!(AudioBuffer, mNumberChannels);

    let count = read_u32(bytes)? as usize;
    let end = buffers_start + count * size_of::<AudioBuffer>();
    if bytes.len() < end {
        return Err(CoreAudioError::from_error_kind(ErrorKind::U32Conversion));
    }

    bytes[buffers_start..end]
        .chunks(size_of::<AudioBuffer>())
        .map(|buffer| read_u32(&buffer[channels_offset..]))
        .collect()
}

/// Reads a packed array of `AudioServerPlugInCustomPropertyInfo` values and
/// converts each to a [`CustomPropertyInfo`]
pub(crate) fn read_vec_custom_property_info(
    bytes: &[u8],
) -> Result<Vec<CustomPropertyInfo>, CoreAudioError> {
    if !bytes
        .len()
        .is_multiple_of(size_of::<AudioServerPlugInCustomPropertyInfo>())
    {
        return Err(CoreAudioError::from_error_kind(ErrorKind::U32Conversion));
    }

    Ok(bytes
        .chunks(size_of::<AudioServerPlugInCustomPropertyInfo>())
        .map(|chunk| unsafe {
            CustomPropertyInfo::from(std::ptr::read_unaligned(
                chunk.as_ptr() as *const AudioServerPlugInCustomPropertyInfo
            ))
        })
        .collect())
}

/// Returns `bytes` as they are, for data the crate doesn't interpret
pub(crate) fn read_bytes(bytes: &[u8]) -> Result<Vec<u8>, CoreAudioError> {
    Ok(bytes.to_vec())
}

/// Reads a custom property's data, decoding it by the data type the property
/// declares
///
/// CoreAudio hands the caller ownership of any CoreFoundation object, so it
/// is released here
pub(crate) fn read_custom(
    data_type: CustomPropertyDataType,
    bytes: &[u8],
) -> Result<CustomValue, CoreAudioError> {
    match data_type {
        CustomPropertyDataType::CFString => read_string(bytes).map(CustomValue::String),
        CustomPropertyDataType::CFPropertyList => {
            let pointer = read_cf_pointer(bytes)
                .ok_or_else(|| CoreAudioError::from_error_kind(ErrorKind::BadCustomData))?;
            let object = unsafe { CFType::wrap_under_create_rule(pointer as CFTypeRef) };
            read_property_list(&object).map(CustomValue::PropertyList)
        }
        _ => Ok(CustomValue::Raw(bytes.to_vec())),
    }
}

/// Converts a property list object into a [`PropertyList`]
///
/// Fails with [`ErrorKind::BadCustomData`] on anything a property list can't
/// hold
fn read_property_list(object: &CFType) -> Result<PropertyList, CoreAudioError> {
    let bad = || CoreAudioError::from_error_kind(ErrorKind::BadCustomData);

    if let Some(string) = object.downcast::<CFString>() {
        Ok(PropertyList::String(string.to_string()))
    } else if let Some(number) = object.downcast::<CFNumber>() {
        if unsafe { CFNumberIsFloatType(number.as_concrete_TypeRef()) } != 0 {
            number.to_f64().map(PropertyList::Float).ok_or_else(bad)
        } else {
            number.to_i64().map(PropertyList::Integer).ok_or_else(bad)
        }
    } else if let Some(boolean) = object.downcast::<CFBoolean>() {
        Ok(PropertyList::Bool(boolean.into()))
    } else if let Some(data) = object.downcast::<CFData>() {
        Ok(PropertyList::Data(data.bytes().to_vec()))
    } else if let Some(date) = object.downcast::<CFDate>() {
        Ok(PropertyList::Date(time_from_absolute(date.abs_time())))
    } else if let Some(array) = object.downcast::<CFArray>() {
        array
            .iter()
            .map(|item| read_property_list(&unsafe { CFType::wrap_under_get_rule(*item) }))
            .collect::<Result<_, _>>()
            .map(PropertyList::Array)
    } else if let Some(dictionary) = object.downcast::<CFDictionary>() {
        let (keys, values) = dictionary.get_keys_and_values();
        keys.into_iter()
            .zip(values)
            .map(|(key, value)| {
                let key = unsafe { CFType::wrap_under_get_rule(key) }
                    .downcast::<CFString>()
                    .ok_or_else(bad)?;
                let value = read_property_list(&unsafe { CFType::wrap_under_get_rule(value) })?;
                Ok((key.to_string(), value))
            })
            .collect::<Result<_, _>>()
            .map(PropertyList::Dictionary)
    } else {
        Err(bad())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::property_list_object;
    use std::{
        collections::BTreeMap,
        time::{Duration, SystemTime},
    };

    /// Every kind of property list value survives being built into
    /// CoreFoundation objects and read back
    #[test]
    fn property_list_round_trip() {
        let list = PropertyList::Dictionary(BTreeMap::from([
            (
                "string".to_string(),
                PropertyList::String("0.3.0".to_string()),
            ),
            ("integer".to_string(), PropertyList::Integer(-96000)),
            ("float".to_string(), PropertyList::Float(96000.5)),
            ("true".to_string(), PropertyList::Bool(true)),
            ("false".to_string(), PropertyList::Bool(false)),
            ("data".to_string(), PropertyList::Data(vec![0, 1, 2, 255])),
            (
                "date".to_string(),
                PropertyList::Date(SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000)),
            ),
            (
                "array".to_string(),
                PropertyList::Array(vec![
                    PropertyList::Integer(1),
                    PropertyList::Array(Vec::new()),
                    PropertyList::Dictionary(BTreeMap::new()),
                ]),
            ),
        ]));

        let object = property_list_object(&list);
        assert_eq!(read_property_list(&object).unwrap(), list);
    }

    /// A CoreFoundation value that can't be in a property list is refused
    #[test]
    fn property_list_rejects_other_types() {
        let url = CFURL::from_path("/", true).unwrap().into_CFType();
        assert!(matches!(
            read_property_list(&url).unwrap_err().kind(),
            ErrorKind::BadCustomData
        ));
    }
}
