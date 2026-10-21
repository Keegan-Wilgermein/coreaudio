//! Custom properties: the ones a HAL plug-in adds to its objects, described
//! by `OBJECT_CUSTOM_PROPERTY_INFO_LIST`, and the values they hold

use crate::errors::OSStatusStringify;
use objc2_core_audio::{
    AudioServerPlugInCustomPropertyInfo, kAudioServerPlugInCustomPropertyDataTypeCFPropertyList,
    kAudioServerPlugInCustomPropertyDataTypeCFString, kAudioServerPlugInCustomPropertyDataTypeNone,
};
use std::{
    collections::BTreeMap,
    time::{Duration, SystemTime},
};

/// The kind of data a custom property, or its qualifier, carries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomPropertyDataType {
    /// No data (only valid for a qualifier)
    None,
    /// A `CFString`
    CFString,
    /// A `CFPropertyList`
    CFPropertyList,
    /// An unrecognised data type; contains the raw value
    Unknown(u32),
}

#[allow(non_upper_case_globals)]
impl From<u32> for CustomPropertyDataType {
    fn from(value: u32) -> Self {
        match value {
            kAudioServerPlugInCustomPropertyDataTypeNone => Self::None,
            kAudioServerPlugInCustomPropertyDataTypeCFString => Self::CFString,
            kAudioServerPlugInCustomPropertyDataTypeCFPropertyList => Self::CFPropertyList,
            id => Self::Unknown(id),
        }
    }
}

impl From<CustomPropertyDataType> for u32 {
    fn from(value: CustomPropertyDataType) -> Self {
        match value {
            CustomPropertyDataType::None => kAudioServerPlugInCustomPropertyDataTypeNone,
            CustomPropertyDataType::CFString => kAudioServerPlugInCustomPropertyDataTypeCFString,
            CustomPropertyDataType::CFPropertyList => {
                kAudioServerPlugInCustomPropertyDataTypeCFPropertyList
            }
            CustomPropertyDataType::Unknown(id) => id,
        }
    }
}

/// A custom property a HAL plug-in object publishes
///
/// Reported by `OBJECT_CUSTOM_PROPERTY_INFO_LIST`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CustomPropertyInfo {
    selector: u32,
    property_data_type: CustomPropertyDataType,
    qualifier_data_type: CustomPropertyDataType,
}

impl CustomPropertyInfo {
    /// The property's selector, a four-character code
    pub fn selector(&self) -> u32 {
        self.selector
    }

    /// The 4cc that the property is represented as
    pub fn four_char_code(&self) -> String {
        self.selector.stringify_bytes()
    }

    /// The kind of data the property's value is
    pub fn property_data_type(&self) -> CustomPropertyDataType {
        self.property_data_type
    }

    /// The kind of data the property's qualifier is
    pub fn qualifier_data_type(&self) -> CustomPropertyDataType {
        self.qualifier_data_type
    }
}

impl From<AudioServerPlugInCustomPropertyInfo> for CustomPropertyInfo {
    fn from(value: AudioServerPlugInCustomPropertyInfo) -> Self {
        Self {
            selector: value.mSelector,
            property_data_type: value.mPropertyDataType.into(),
            qualifier_data_type: value.mQualifierDataType.into(),
        }
    }
}

impl From<CustomPropertyInfo> for AudioServerPlugInCustomPropertyInfo {
    fn from(value: CustomPropertyInfo) -> Self {
        Self {
            mSelector: value.selector,
            mPropertyDataType: value.property_data_type.into(),
            mQualifierDataType: value.qualifier_data_type.into(),
        }
    }
}

/// The value of a custom property, or the qualifier passed with one
///
/// Which variant a property takes follows its declared
/// [`CustomPropertyDataType`]
#[derive(Debug, Clone, PartialEq)]
pub enum CustomValue {
    /// For [`CustomPropertyDataType::CFString`]
    String(String),
    /// For [`CustomPropertyDataType::CFPropertyList`]
    ///
    /// A string is a property list too, so these properties also take
    /// [`CustomValue::String`]
    PropertyList(PropertyList),
    /// For any data type the crate doesn't know, such as raw data (`'raww'`)
    ///
    /// CoreAudio copies these bytes between processes unchanged, so they are
    /// the value itself
    Raw(Vec<u8>),
}

/// A property list value: what a `CFPropertyList` holds
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyList {
    /// A `CFString`
    String(String),
    /// A `CFNumber` holding an integer
    Integer(i64),
    /// A `CFNumber` holding a floating point number
    Float(f64),
    /// A `CFBoolean`
    Bool(bool),
    /// A `CFData`
    Data(Vec<u8>),
    /// A `CFDate`
    Date(SystemTime),
    /// A `CFArray`
    Array(Vec<PropertyList>),
    /// A `CFDictionary`; property list keys are always strings
    Dictionary(BTreeMap<String, PropertyList>),
}

/// Seconds from the Unix epoch to CoreFoundation's, 1 January 2001
const CF_EPOCH: f64 = 978_307_200.0;

/// The time a `CFAbsoluteTime` (seconds since 2001) stands for
pub(crate) fn time_from_absolute(absolute: f64) -> SystemTime {
    let unix = absolute + CF_EPOCH;
    if unix >= 0.0 {
        SystemTime::UNIX_EPOCH + Duration::from_secs_f64(unix)
    } else {
        SystemTime::UNIX_EPOCH - Duration::from_secs_f64(-unix)
    }
}

/// The `CFAbsoluteTime` (seconds since 2001) for a time
pub(crate) fn absolute_from_time(time: SystemTime) -> f64 {
    let unix = match time.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(after) => after.as_secs_f64(),
        Err(before) => -before.duration().as_secs_f64(),
    };
    unix - CF_EPOCH
}
