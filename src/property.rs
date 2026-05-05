//! # Properties

#![allow(unsafe_code)]

// ----- Imports ------------
use crate::{data_types::{BufferFrameSizeRange, ChannelPair, DBRange, SampleRateRange, StreamDescription, StreamRangedDescription}, errors::{CoreAudioError, ErrorKind}, object::{Device, Global, Stream, System}};
use std::marker::PhantomData;
use core_foundation::{base::TCFType, string::{CFString, CFStringRef}};
use coreaudio_sys::{
    AudioObjectID, AudioObjectPropertyAddress, AudioObjectPropertyScope, AudioObjectPropertySelector, AudioStreamBasicDescription, AudioStreamRangedDescription, AudioValueRange, kAudioDeviceProcessorOverload, kAudioDevicePropertyAvailableNominalSampleRates, kAudioDevicePropertyBufferFrameSize, kAudioDevicePropertyBufferFrameSizeRange, kAudioDevicePropertyChannelNominalLineLevel, kAudioDevicePropertyChannelNominalLineLevelNameForIDCFString, kAudioDevicePropertyChannelNominalLineLevels, kAudioDevicePropertyClipLight, kAudioDevicePropertyClockDomain, kAudioDevicePropertyClockSource, kAudioDevicePropertyClockSourceNameForIDCFString, kAudioDevicePropertyClockSources, kAudioDevicePropertyConfigurationApplication, kAudioDevicePropertyDataSource, kAudioDevicePropertyDataSourceNameForIDCFString, kAudioDevicePropertyDataSources, kAudioDevicePropertyDeviceCanBeDefaultDevice, kAudioDevicePropertyDeviceCanBeDefaultSystemDevice, kAudioDevicePropertyDeviceIsAlive, kAudioDevicePropertyDeviceIsRunning, kAudioDevicePropertyDeviceUID, kAudioDevicePropertyHighPassFilterSetting, kAudioDevicePropertyHighPassFilterSettingNameForIDCFString, kAudioDevicePropertyHighPassFilterSettings, kAudioDevicePropertyHogMode, kAudioDevicePropertyIOCycleUsage, kAudioDevicePropertyIOStoppedAbnormally, kAudioDevicePropertyIsHidden, kAudioDevicePropertyJackIsConnected, kAudioDevicePropertyLatency, kAudioDevicePropertyListenback, kAudioDevicePropertyModelUID, kAudioDevicePropertyMute, kAudioDevicePropertyNominalSampleRate, kAudioDevicePropertyPhantomPower, kAudioDevicePropertyPhaseInvert, kAudioDevicePropertyPlayThruDestination, kAudioDevicePropertyPlayThruDestinationNameForIDCFString, kAudioDevicePropertyPlayThruDestinations, kAudioDevicePropertyPreferredChannelLayout, kAudioDevicePropertyPreferredChannelsForStereo, kAudioDevicePropertyRelatedDevices, kAudioDevicePropertySafetyOffset, kAudioDevicePropertySolo, kAudioDevicePropertyStereoPan, kAudioDevicePropertyStereoPanChannels, kAudioDevicePropertyStreams, kAudioDevicePropertySubMute, kAudioDevicePropertySubVolumeDecibels, kAudioDevicePropertySubVolumeDecibelsToScalar, kAudioDevicePropertySubVolumeRangeDecibels, kAudioDevicePropertySubVolumeScalar, kAudioDevicePropertySubVolumeScalarToDecibels, kAudioDevicePropertyTalkback, kAudioDevicePropertyTransportType, kAudioDevicePropertyUsesVariableBufferFrameSizes, kAudioDevicePropertyVolumeDecibels, kAudioDevicePropertyVolumeDecibelsToScalar, kAudioDevicePropertyVolumeRangeDecibels, kAudioDevicePropertyVolumeScalar, kAudioDevicePropertyVolumeScalarToDecibels, kAudioHardwarePropertyBoxList, kAudioHardwarePropertyClockDeviceList, kAudioHardwarePropertyDefaultInputDevice, kAudioHardwarePropertyDefaultOutputDevice, kAudioHardwarePropertyDefaultSystemOutputDevice, kAudioHardwarePropertyDevices, kAudioHardwarePropertyHogModeIsAllowed, kAudioHardwarePropertyIsInitingOrExiting, kAudioHardwarePropertyMixStereoToMono, kAudioHardwarePropertyPlugInList, kAudioHardwarePropertyPowerHint, kAudioHardwarePropertyProcessIsAudible, kAudioHardwarePropertyProcessIsMaster, kAudioHardwarePropertyServiceRestarted, kAudioHardwarePropertySleepingIsAllowed, kAudioHardwarePropertyTapList, kAudioHardwarePropertyTranslateBundleIDToPlugIn, kAudioHardwarePropertyTranslateBundleIDToTransportManager, kAudioHardwarePropertyTranslateUIDToBox, kAudioHardwarePropertyTranslateUIDToClockDevice, kAudioHardwarePropertyTranslateUIDToDevice, kAudioHardwarePropertyTransportManagerList, kAudioHardwarePropertyUnloadingIsAllowed, kAudioHardwarePropertyUserIDChanged, kAudioHardwarePropertyUserSessionIsActiveOrHeadless, kAudioObjectPropertyBaseClass, kAudioObjectPropertyClass, kAudioObjectPropertyCreator, kAudioObjectPropertyElementCategoryName, kAudioObjectPropertyElementMain, kAudioObjectPropertyElementName, kAudioObjectPropertyElementNumberName, kAudioObjectPropertyManufacturer, kAudioObjectPropertyModelName, kAudioObjectPropertyName, kAudioObjectPropertyOwnedObjects, kAudioObjectPropertyOwner, kAudioObjectPropertyScopeGlobal, kAudioObjectPropertyScopeInput, kAudioObjectPropertyScopeOutput, kAudioStreamPropertyAvailablePhysicalFormats, kAudioStreamPropertyAvailableVirtualFormats, kAudioStreamPropertyDirection, kAudioStreamPropertyIsActive, kAudioStreamPropertyLatency, kAudioStreamPropertyPhysicalFormat, kAudioStreamPropertyStartingChannel, kAudioStreamPropertyTerminalType, kAudioStreamPropertyVirtualFormat
};

// ---- Structs -------------
/// Indicates a property to be read only
pub struct ReadOnly;

/// Indicates a property to be readable and writeable
pub struct ReadWrite;

/// Indicates a property to be listenable
pub struct Listenable;

/// Indicates a property to be unlistenable
pub struct Silent;

/// Indicates a property needs no extra data
pub struct NoExtra;

/// Indicates a property still needs an element
pub struct NeedElement;

/// Indicates a property still needs qualifier data
pub struct NeedQualifier<T>(T);

/// Indicates a property still needs both an element and qualifier data
pub struct NeedBoth<T>(T);

pub struct Property<T, Object, Access, L, E> {
    pub(crate) address: AudioObjectPropertyAddress,
    pub(crate) read: fn(&[u8]) -> Result<T, CoreAudioError>,
    pub(crate) encode: Option<fn(T) -> Vec<u8>>,
    pub(crate) qualifier: Option<Vec<u8>>,
    _object: PhantomData<Object>,
    _access: PhantomData<Access>,
    _listenable: PhantomData<L>,
    _extra_data: PhantomData<E>,
}

impl<T, Object, Access, L, E> Property<T, Object, Access, L, E> {
    pub(crate) const fn new(
        address: AudioObjectPropertyAddress,
        read: fn(&[u8]) -> Result<T, CoreAudioError>,
        encode: Option<fn(T) -> Vec<u8>>,
    ) -> Self {
        Self {
            address,
            read,
            encode,
            qualifier: None,
            _object: PhantomData,
            _access: PhantomData,
            _listenable: PhantomData,
            _extra_data: PhantomData,
        }
    }
}

// ---- Read functions (private) ----
fn read_string(bytes: &[u8]) -> Result<String, CoreAudioError> {
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

fn read_bool(bytes: &[u8]) -> Result<bool, CoreAudioError> {
    let value = u32::from_ne_bytes(
        match bytes[..4].try_into() {
            Ok(value) => value,
            Err(_) => return Err(CoreAudioError::from_error_kind(ErrorKind::BoolConversion)),
        }
    ) != 0;

    Ok(value)
}

fn read_f64(bytes: &[u8]) -> Result<f64, CoreAudioError> {
    let value = f64::from_ne_bytes(
        match bytes[..8].try_into() {
            Ok(value) => value,
            Err(_) => return Err(CoreAudioError::from_error_kind(ErrorKind::FPConversion)),
        }
    );

    Ok(value)
}

fn read_u32(bytes: &[u8]) -> Result<u32, CoreAudioError> {
    let value = u32::from_ne_bytes(
        match bytes[..4].try_into() {
            Ok(value) => value,
            Err(_) => return Err(CoreAudioError::from_error_kind(ErrorKind::U32Conversion)),
        }
    );

    Ok(value)
}

fn read_i32(bytes: &[u8]) -> Result<i32, CoreAudioError> {
    let value = i32::from_ne_bytes(
        match bytes[..4].try_into() {
            Ok(value) => value,
            Err(_) => return Err(CoreAudioError::from_error_kind(ErrorKind::I32Conversion)),
        }
    );

    Ok(value)
}

fn read_audio_object_id(bytes: &[u8]) -> Result<AudioObjectID, CoreAudioError> {
    let value = u32::from_ne_bytes(
        match bytes[..4].try_into() {
            Ok(value) => value,
            Err(_) => return Err(CoreAudioError::from_error_kind(ErrorKind::AudioObjectIdConversion)),
        }
    );

    Ok(value)
}

fn read_stream_description(bytes: &[u8]) -> Result<StreamDescription, CoreAudioError> {
    if bytes.len() != size_of::<AudioStreamBasicDescription>() {
        return Err(CoreAudioError::from_error_kind(ErrorKind::StreamDescriptionConversion));
    }

    let asbd = unsafe { 
        std::ptr::read(bytes.as_ptr() as *const AudioStreamBasicDescription) 
    };
    
    StreamDescription::try_from(asbd)
        .map_err(|_| CoreAudioError::from_error_kind(ErrorKind::StreamDescriptionConversion))
}

fn read_buffer_size_range(bytes: &[u8]) -> Result<BufferFrameSizeRange, CoreAudioError> {
    if bytes.len() != size_of::<AudioValueRange>() {
        return Err(CoreAudioError::from_error_kind(ErrorKind::ValueRangeConversion));
    }

    let range = unsafe {
        std::ptr::read(bytes.as_ptr() as *const AudioValueRange)
    };

    Ok(BufferFrameSizeRange::from(range))
}

fn read_vec_audio_object_id(bytes: &[u8]) -> Result<Vec<AudioObjectID>, CoreAudioError> {
    if bytes.len() % size_of::<AudioObjectID>() != 0 {
        return Err(CoreAudioError::from_error_kind(ErrorKind::AudioObjectIdConversion));
    }

    Ok(bytes.chunks(size_of::<AudioObjectID>())
        .map(|chunk| unsafe { std::ptr::read(chunk.as_ptr() as *const AudioObjectID) })
        .collect())
}

fn read_vec_sample_rate_range(bytes: &[u8]) -> Result<Vec<SampleRateRange>, CoreAudioError> {
    if bytes.len() % size_of::<AudioValueRange>() != 0 {
        return Err(CoreAudioError::from_error_kind(ErrorKind::ValueRangeConversion));
    }

    Ok(bytes.chunks(size_of::<AudioValueRange>())
        .map(|chunk| unsafe { 
            SampleRateRange::from(std::ptr::read(chunk.as_ptr() as *const AudioValueRange)) 
        })
        .collect())
}

fn read_vec_stream_ranged_description(bytes: &[u8]) -> Result<Vec<StreamRangedDescription>, CoreAudioError> {
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

fn encode_f64(value: f64) -> Vec<u8> {
    value.to_ne_bytes().to_vec()
}

pub(crate) fn encode_u32(value: u32) -> Vec<u8> {
    value.to_ne_bytes().to_vec()
}

fn encode_i32(value: i32) -> Vec<u8> {
    value.to_ne_bytes().to_vec()
}

fn encode_bool(value: bool) -> Vec<u8> {
    encode_u32(value as u32)
}

fn encode_audio_object_id(value: AudioObjectID) -> Vec<u8> {
    encode_u32(value)
}

fn encode_stream_description(value: StreamDescription) -> Vec<u8> {
    let asbd: AudioStreamBasicDescription = value.into();
    let size = size_of::<AudioStreamBasicDescription>();
    let mut bytes = vec![0u8; size];
    unsafe {
        std::ptr::write(bytes.as_mut_ptr() as *mut AudioStreamBasicDescription, asbd);
    }
    bytes
}

fn read_f32(bytes: &[u8]) -> Result<f32, CoreAudioError> {
    let value = f32::from_ne_bytes(
        match bytes[..4].try_into() {
            Ok(value) => value,
            Err(_) => return Err(CoreAudioError::from_error_kind(ErrorKind::FPConversion)),
        }
    );

    Ok(value)
}

fn encode_f32(value: f32) -> Vec<u8> {
    value.to_ne_bytes().to_vec()
}

fn read_channel_pair(bytes: &[u8]) -> Result<ChannelPair, CoreAudioError> {
    if bytes.len() < 8 {
        return Err(CoreAudioError::from_error_kind(ErrorKind::U32Conversion));
    }

    let a = u32::from_ne_bytes(bytes[..4].try_into()
        .map_err(|_| CoreAudioError::from_error_kind(ErrorKind::U32Conversion))?);
    let b = u32::from_ne_bytes(bytes[4..8].try_into()
        .map_err(|_| CoreAudioError::from_error_kind(ErrorKind::U32Conversion))?);

    Ok([a, b].into())
}

fn encode_channel_pair(value: ChannelPair) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(8);
    bytes.extend_from_slice(&value.left().to_ne_bytes());
    bytes.extend_from_slice(&value.right().to_ne_bytes());
    bytes
}

fn read_db_range(bytes: &[u8]) -> Result<DBRange, CoreAudioError> {
    if bytes.len() != size_of::<AudioValueRange>() {
        return Err(CoreAudioError::from_error_kind(ErrorKind::ValueRangeConversion));
    }

    Ok(unsafe { std::ptr::read(bytes.as_ptr() as *const AudioValueRange) }.into())
}

// ---- Helper ----
const fn address(
    selector: AudioObjectPropertySelector,
    scope: AudioObjectPropertyScope,
) -> AudioObjectPropertyAddress {
    AudioObjectPropertyAddress {
        mSelector: selector,
        mScope: scope,
        mElement: kAudioObjectPropertyElementMain,
    }
}

// ---- AudioObject constants ----
// These properties are defined on the base AudioObject type and apply to any audio object.

/// The class that the class of the AudioObject is a subclass of
pub const OBJECT_BASE_CLASS: Property<u32, Global, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioObjectPropertyBaseClass,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    None,
);

/// The class of the AudioObject
pub const OBJECT_CLASS: Property<u32, Global, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioObjectPropertyClass,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    None,
);

/// The AudioObjectID of the owning AudioObject
pub const OBJECT_OWNER: Property<AudioObjectID, Global, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioObjectPropertyOwner,
        kAudioObjectPropertyScopeGlobal
    ),
    read_audio_object_id,
    None,
);

/// The human-readable name of the model of the AudioObject
pub const OBJECT_MODEL_NAME: Property<String, Global, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioObjectPropertyModelName,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// The human-readable name of the manufacturer of the AudioObject
pub const OBJECT_MANUFACTURER: Property<String, Global, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioObjectPropertyManufacturer,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// The human-readable name of the given element in the given scope
pub const OBJECT_ELEMENT_NAME: Property<String, Global, ReadOnly, Silent, NeedElement> =
Property::new(
    address(
        kAudioObjectPropertyElementName,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// The human-readable name of the category of the given element in the given scope
pub const OBJECT_ELEMENT_CATEGORY_NAME: Property<String, Global, ReadOnly, Silent, NeedElement> =
Property::new(
    address(
        kAudioObjectPropertyElementCategoryName,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// The human-readable name of the number of the given element in the given scope
pub const OBJECT_ELEMENT_NUMBER_NAME: Property<String, Global, ReadOnly, Silent, NeedElement> =
Property::new(
    address(
        kAudioObjectPropertyElementNumberName,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// An array of the AudioObjectIDs of the AudioObjects owned by this object
///
/// Requires a qualifier specifying the class(es) to filter by.
pub const OBJECT_OWNED_OBJECTS: Property<Vec<AudioObjectID>, Global, ReadOnly, Silent, NeedQualifier<Vec<u32>>> =
Property::new(
    address(
        kAudioObjectPropertyOwnedObjects,
        kAudioObjectPropertyScopeGlobal
    ),
    read_vec_audio_object_id,
    None,
);

/// The bundle ID of the plug-in that instantiated the AudioObject
pub const OBJECT_CREATOR: Property<String, Global, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioObjectPropertyCreator,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

// ---- Device constants ----

/// Human readable name of the device
pub const DEVICE_NAME: Property<String, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioObjectPropertyName,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// Persistent unique identifier for the device
/// 
/// Not the same as the device id
pub const DEVICE_UID: Property<String, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyDeviceUID,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// Whether the device is still alive and connected
pub const DEVICE_IS_ALIVE: Property<bool, Device, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyDeviceIsAlive,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    None,
);

/// Whether the device is currently running I/O
pub const DEVICE_IS_RUNNING: Property<bool, Device, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyDeviceIsRunning,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    None,
);

/// The current nominal sample rate of the device
pub const DEVICE_NOMINAL_SAMPLE_RATE: Property<f64, Device, ReadWrite, Listenable, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyNominalSampleRate,
        kAudioObjectPropertyScopeGlobal
    ),
    read_f64,
    Some(encode_f64),
);

/// All sample rates supported by the device
pub const DEVICE_AVAILABLE_SAMPLE_RATES: Property<Vec<SampleRateRange>, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyAvailableNominalSampleRates,
        kAudioObjectPropertyScopeGlobal
    ),
    read_vec_sample_rate_range,
    None,
);

/// The number of frames in the I/O buffer
pub const DEVICE_BUFFER_FRAME_SIZE: Property<u32, Device, ReadWrite, Listenable, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyBufferFrameSize,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    Some(encode_u32)
);

/// The valid range of buffer frame sizes for the device
pub const DEVICE_BUFFER_FRAME_SIZE_RANGE: Property<BufferFrameSizeRange, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyBufferFrameSizeRange,
        kAudioObjectPropertyScopeGlobal
    ),
    read_buffer_size_range,
    None,
);

/// Input latency of the device in frames
pub const DEVICE_INPUT_LATENCY: Property<u32, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyLatency,
        kAudioObjectPropertyScopeInput
    ),
    read_u32,
    None,
);

/// Output latency of the device in frames
pub const DEVICE_OUTPUT_LATENCY: Property<u32, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyLatency,
        kAudioObjectPropertyScopeOutput
    ),
    read_u32,
    None
);

/// All input streams on the device
pub(crate) const DEVICE_INPUT_STREAMS: Property<Vec<AudioObjectID>, Device, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyStreams,
        kAudioObjectPropertyScopeInput
    ),
    read_vec_audio_object_id,
    None,
);

/// All output streams on the device
pub(crate) const DEVICE_OUTPUT_STREAMS: Property<Vec<AudioObjectID>, Device, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyStreams,
        kAudioObjectPropertyScopeOutput
    ),
    read_vec_audio_object_id,
    None,
);

pub const DEVICE_HOG_MODE: Property<i32, Device, ReadWrite, Listenable, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyHogMode,
        kAudioObjectPropertyScopeGlobal
    ),
    read_i32,
    Some(encode_i32)
);

// ---- Device core constants (AudioHardwareBase.h) ----

/// Bundle ID of the application that currently holds exclusive access
pub const DEVICE_CONFIGURATION_APPLICATION: Property<String, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyConfigurationApplication,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// Persistent model-level unique identifier for the device
pub const DEVICE_MODEL_UID: Property<String, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyModelUID,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// The transport type of the device (USB, FireWire, PCI, etc.)
pub const DEVICE_TRANSPORT_TYPE: Property<u32, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyTransportType,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    None,
);

/// Other devices in the same transport group as this device
pub const DEVICE_RELATED_DEVICES: Property<Vec<AudioObjectID>, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyRelatedDevices,
        kAudioObjectPropertyScopeGlobal
    ),
    read_vec_audio_object_id,
    None,
);

/// The clock domain of the device; 0 means not in a clock domain
pub const DEVICE_CLOCK_DOMAIN: Property<u32, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyClockDomain,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    None,
);

/// Whether the device can be the default device for its scope
pub const DEVICE_CAN_BE_DEFAULT: Property<bool, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyDeviceCanBeDefaultDevice,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    None,
);

/// Whether the device can be the system default output device
pub const DEVICE_CAN_BE_DEFAULT_SYSTEM: Property<bool, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyDeviceCanBeDefaultSystemDevice,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    None,
);

/// The number of frames of safety offset added by the hardware for its scope
pub const DEVICE_SAFETY_OFFSET: Property<u32, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertySafetyOffset,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    None,
);

/// Whether the device is hidden from normal clients
pub const DEVICE_IS_HIDDEN: Property<bool, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyIsHidden,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    None,
);

/// The preferred stereo channel pair (left index, right index) for the given scope
pub const DEVICE_PREFERRED_CHANNELS_FOR_STEREO: Property<ChannelPair, Device, ReadWrite, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyPreferredChannelsForStereo,
        kAudioObjectPropertyScopeGlobal
    ),
    read_channel_pair,
    Some(encode_channel_pair),
);

// ---- Device convenience constants (AudioHardware.h) ----

/// Whether an I/O overload occurred; listen to detect overloads
pub const DEVICE_PROCESSOR_OVERLOAD: Property<u32, Device, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioDeviceProcessorOverload,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    None,
);

/// Whether I/O stopped unexpectedly; listen to detect abnormal stops
pub const DEVICE_IO_STOPPED_ABNORMALLY: Property<u32, Device, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyIOStoppedAbnormally,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    None,
);

/// Fraction of the I/O cycle the HAL is allowed to use
pub const DEVICE_IO_CYCLE_USAGE: Property<f32, Device, ReadWrite, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyIOCycleUsage,
        kAudioObjectPropertyScopeGlobal
    ),
    read_f32,
    Some(encode_f32),
);

/// Whether the device uses variable-length I/O buffer frames
pub const DEVICE_USES_VARIABLE_BUFFER_FRAME_SIZES: Property<u32, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyUsesVariableBufferFrameSizes,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    None,
);

/// The volume as a linear scalar for the given scope and element
pub const DEVICE_VOLUME_SCALAR: Property<f32, Device, ReadWrite, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyVolumeScalar,
        kAudioObjectPropertyScopeGlobal
    ),
    read_f32,
    Some(encode_f32),
);

/// The volume in dB for the given scope and element
pub const DEVICE_VOLUME_DECIBELS: Property<f32, Device, ReadWrite, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyVolumeDecibels,
        kAudioObjectPropertyScopeGlobal
    ),
    read_f32,
    Some(encode_f32),
);

/// The valid dB range for volume for the given scope and element
pub const DEVICE_VOLUME_RANGE_DECIBELS: Property<DBRange, Device, ReadOnly, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyVolumeRangeDecibels,
        kAudioObjectPropertyScopeGlobal
    ),
    read_db_range,
    None,
);

/// Convert a volume scalar to dB
pub const DEVICE_VOLUME_SCALAR_TO_DECIBELS: Property<f32, Device, ReadOnly, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyVolumeScalarToDecibels,
        kAudioObjectPropertyScopeGlobal
    ),
    read_f32,
    None,
);

/// Convert a volume in dB to scalar
pub const DEVICE_VOLUME_DECIBELS_TO_SCALAR: Property<f32, Device, ReadOnly, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyVolumeDecibelsToScalar,
        kAudioObjectPropertyScopeGlobal
    ),
    read_f32,
    None,
);

/// The stereo pan position (0.0 = full left, 1.0 = full right)
pub const DEVICE_STEREO_PAN: Property<f32, Device, ReadWrite, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyStereoPan,
        kAudioObjectPropertyScopeGlobal
    ),
    read_f32,
    Some(encode_f32),
);

/// The two channel indices used for stereo panning (left, right)
pub const DEVICE_STEREO_PAN_CHANNELS: Property<ChannelPair, Device, ReadOnly, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyStereoPanChannels,
        kAudioObjectPropertyScopeGlobal
    ),
    read_channel_pair,
    None,
);

/// Whether the given element is muted for the given scope
pub const DEVICE_MUTE: Property<bool, Device, ReadWrite, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyMute,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    Some(encode_bool),
);

/// Whether the given element is soloed for the given scope
pub const DEVICE_SOLO: Property<bool, Device, ReadWrite, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertySolo,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    Some(encode_bool),
);

/// Whether phantom power is enabled for the given element
pub const DEVICE_PHANTOM_POWER: Property<bool, Device, ReadWrite, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyPhantomPower,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    Some(encode_bool),
);

/// Whether the signal phase is inverted for the given element
pub const DEVICE_PHASE_INVERT: Property<bool, Device, ReadWrite, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyPhaseInvert,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    Some(encode_bool),
);

/// Whether the clip light is currently lit
pub const DEVICE_CLIP_LIGHT: Property<bool, Device, ReadWrite, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyClipLight,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    Some(encode_bool),
);

/// Whether talkback is enabled
pub const DEVICE_TALKBACK: Property<bool, Device, ReadWrite, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyTalkback,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    Some(encode_bool),
);

/// Whether listenback is enabled
pub const DEVICE_LISTENBACK: Property<bool, Device, ReadWrite, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyListenback,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    Some(encode_bool),
);

/// Whether a jack is connected to the given scope and element
pub const DEVICE_JACK_IS_CONNECTED: Property<bool, Device, ReadOnly, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyJackIsConnected,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    None,
);

/// The ID of the currently selected data source for the given scope
pub const DEVICE_DATA_SOURCE: Property<u32, Device, ReadWrite, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyDataSource,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    Some(encode_u32),
);

/// All available data source IDs for the given scope
pub const DEVICE_DATA_SOURCES: Property<Vec<AudioObjectID>, Device, ReadOnly, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyDataSources,
        kAudioObjectPropertyScopeGlobal
    ),
    read_vec_audio_object_id,
    None,
);

/// Human-readable name for a data source ID; requires qualifier with source ID
pub const DEVICE_DATA_SOURCE_NAME: Property<String, Device, ReadOnly, Silent, NeedBoth<u32>> =
Property::new(
    address(
        kAudioDevicePropertyDataSourceNameForIDCFString,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// The ID of the currently selected clock source
pub const DEVICE_CLOCK_SOURCE: Property<u32, Device, ReadWrite, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyClockSource,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    Some(encode_u32),
);

/// All available clock source IDs
pub const DEVICE_CLOCK_SOURCES: Property<Vec<AudioObjectID>, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyClockSources,
        kAudioObjectPropertyScopeGlobal
    ),
    read_vec_audio_object_id,
    None,
);

/// Human-readable name for a clock source ID; requires qualifier with source ID
pub const DEVICE_CLOCK_SOURCE_NAME: Property<String, Device, ReadOnly, Silent, NeedQualifier<u32>> =
Property::new(
    address(
        kAudioDevicePropertyClockSourceNameForIDCFString,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// The ID of the currently selected play-through destination
pub const DEVICE_PLAY_THRU_DESTINATION: Property<u32, Device, ReadWrite, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyPlayThruDestination,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    Some(encode_u32),
);

/// All available play-through destination IDs
pub const DEVICE_PLAY_THRU_DESTINATIONS: Property<Vec<AudioObjectID>, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyPlayThruDestinations,
        kAudioObjectPropertyScopeGlobal
    ),
    read_vec_audio_object_id,
    None,
);

/// Human-readable name for a play-through destination ID; requires qualifier
pub const DEVICE_PLAY_THRU_DESTINATION_NAME: Property<String, Device, ReadOnly, Silent, NeedQualifier<u32>> =
Property::new(
    address(
        kAudioDevicePropertyPlayThruDestinationNameForIDCFString,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// The ID of the nominal line level for the given channel
pub const DEVICE_CHANNEL_NOMINAL_LINE_LEVEL: Property<u32, Device, ReadWrite, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyChannelNominalLineLevel,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    Some(encode_u32),
);

/// All available nominal line level IDs for the given channel
pub const DEVICE_CHANNEL_NOMINAL_LINE_LEVELS: Property<Vec<AudioObjectID>, Device, ReadOnly, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyChannelNominalLineLevels,
        kAudioObjectPropertyScopeGlobal
    ),
    read_vec_audio_object_id,
    None,
);

/// Human-readable name for a nominal line level ID; requires qualifier
pub const DEVICE_CHANNEL_NOMINAL_LINE_LEVEL_NAME: Property<String, Device, ReadOnly, Silent, NeedBoth<u32>> =
Property::new(
    address(
        kAudioDevicePropertyChannelNominalLineLevelNameForIDCFString,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// The ID of the current high-pass filter setting
pub const DEVICE_HIGH_PASS_FILTER_SETTING: Property<u32, Device, ReadWrite, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyHighPassFilterSetting,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    Some(encode_u32),
);

/// All available high-pass filter setting IDs
pub const DEVICE_HIGH_PASS_FILTER_SETTINGS: Property<Vec<AudioObjectID>, Device, ReadOnly, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyHighPassFilterSettings,
        kAudioObjectPropertyScopeGlobal
    ),
    read_vec_audio_object_id,
    None,
);

/// Human-readable name for a high-pass filter setting ID; requires qualifier
pub const DEVICE_HIGH_PASS_FILTER_SETTING_NAME: Property<String, Device, ReadOnly, Silent, NeedBoth<u32>> =
Property::new(
    address(
        kAudioDevicePropertyHighPassFilterSettingNameForIDCFString,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// The LFE channel volume as a linear scalar
pub const DEVICE_SUB_VOLUME_SCALAR: Property<f32, Device, ReadWrite, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertySubVolumeScalar,
        kAudioObjectPropertyScopeGlobal
    ),
    read_f32,
    Some(encode_f32),
);

/// The LFE channel volume in dB
pub const DEVICE_SUB_VOLUME_DECIBELS: Property<f32, Device, ReadWrite, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertySubVolumeDecibels,
        kAudioObjectPropertyScopeGlobal
    ),
    read_f32,
    Some(encode_f32),
);

/// The valid dB range for the LFE channel volume
pub const DEVICE_SUB_VOLUME_RANGE_DECIBELS: Property<DBRange, Device, ReadOnly, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertySubVolumeRangeDecibels,
        kAudioObjectPropertyScopeGlobal
    ),
    read_db_range,
    None,
);

/// Convert an LFE volume scalar to dB
pub const DEVICE_SUB_VOLUME_SCALAR_TO_DECIBELS: Property<f32, Device, ReadOnly, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertySubVolumeScalarToDecibels,
        kAudioObjectPropertyScopeGlobal
    ),
    read_f32,
    None,
);

/// Convert an LFE volume in dB to scalar
pub const DEVICE_SUB_VOLUME_DECIBELS_TO_SCALAR: Property<f32, Device, ReadOnly, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertySubVolumeDecibelsToScalar,
        kAudioObjectPropertyScopeGlobal
    ),
    read_f32,
    None,
);

/// Whether the LFE channel is muted
pub const DEVICE_SUB_MUTE: Property<bool, Device, ReadWrite, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertySubMute,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    Some(encode_bool),
);

// ---- Stream constants ----

/// Human readable name of the stream
pub const STREAM_NAME: Property<String, Stream, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioObjectPropertyName,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// Whether the stream is currently active
pub const STREAM_IS_ACTIVE: Property<bool, Stream, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioStreamPropertyIsActive,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    None,
);

/// Direction of the stream — 0 for output, 1 for input
pub const STREAM_DIRECTION: Property<u32, Stream, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioStreamPropertyDirection,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    None,
);

/// The virtual format of the stream as presented to the client
pub const STREAM_VIRTUAL_FORMAT: Property<StreamDescription, Stream, ReadWrite, Listenable, NoExtra> =
Property::new(
    address(
        kAudioStreamPropertyVirtualFormat,
        kAudioObjectPropertyScopeGlobal
    ),
    read_stream_description,
    Some(encode_stream_description)
);

/// The physical format of the stream as presented to the hardware
pub const STREAM_PHYSICAL_FORMAT: Property<StreamDescription, Stream, ReadWrite, Listenable, NoExtra> =
Property::new(
    address(
        kAudioStreamPropertyPhysicalFormat,
        kAudioObjectPropertyScopeGlobal
    ),
    read_stream_description,
    Some(encode_stream_description)
);

/// The device the stream is outputting through
pub const TERMINAL_TYPE: Property<u32, Stream, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioStreamPropertyTerminalType,
        kAudioObjectPropertyScopeGlobal,
    ),
    read_u32,
    None,
);

/// The first element of the stream that maps to element 1
pub const STARTING_CHANNEL: Property<u32, Stream, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioStreamPropertyStartingChannel,
        kAudioObjectPropertyScopeGlobal,
    ),
    read_u32,
    None,
);

/// All data formats the stream can present to clients, each with a sample rate range
pub const STREAM_AVAILABLE_VIRTUAL_FORMATS: Property<Vec<StreamRangedDescription>, Stream, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioStreamPropertyAvailableVirtualFormats,
        kAudioObjectPropertyScopeGlobal
    ),
    read_vec_stream_ranged_description,
    None,
);

/// All data formats the hardware actually supports, each with a sample rate range
pub const STREAM_AVAILABLE_PHYSICAL_FORMATS: Property<Vec<StreamRangedDescription>, Stream, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioStreamPropertyAvailablePhysicalFormats,
        kAudioObjectPropertyScopeGlobal
    ),
    read_vec_stream_ranged_description,
    None,
);

/// Latency of the stream in frames
pub const STREAM_LATENCY: Property<u32, Stream, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioStreamPropertyLatency,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    None,
);

// ---- System constants ----

/// Human readable name of the system object
pub const SYSTEM_NAME: Property<String, System, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioObjectPropertyName,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// All devices currently known to the HAL
pub(crate) const SYSTEM_DEVICES: Property<Vec<AudioObjectID>, System, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyDevices,
        kAudioObjectPropertyScopeGlobal
    ), read_vec_audio_object_id,
    None,
);

/// The current default input device
pub(crate) const SYSTEM_DEFAULT_INPUT: Property<AudioObjectID, System, ReadWrite, Listenable, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyDefaultInputDevice,
        kAudioObjectPropertyScopeGlobal
    ),
    read_audio_object_id,
    Some(encode_audio_object_id)
);

/// The current default output device
pub(crate) const SYSTEM_DEFAULT_OUTPUT: Property<AudioObjectID, System, ReadWrite, Listenable, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyDefaultOutputDevice,
        kAudioObjectPropertyScopeGlobal
    ),
    read_audio_object_id,
    Some(encode_audio_object_id)
);

/// All audio boxes known to the HAL
pub const SYSTEM_BOX_LIST: Property<Vec<AudioObjectID>, System, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyBoxList,
        kAudioObjectPropertyScopeGlobal
    ),
    read_vec_audio_object_id,
    None,
);

/// All clock devices known to the HAL
pub const SYSTEM_CLOCK_DEVICE_LIST: Property<Vec<AudioObjectID>, System, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyClockDeviceList,
        kAudioObjectPropertyScopeGlobal
    ),
    read_vec_audio_object_id,
    None,
);

/// Whether the HAL is currently initialising or shutting down
pub const SYSTEM_IS_INITING_OR_EXITING: Property<bool, System, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyIsInitingOrExiting,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    None,
);

/// Whether the system is permitted to sleep while audio is running
pub const SYSTEM_SLEEPING_IS_ALLOWED: Property<bool, System, ReadWrite, Listenable, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertySleepingIsAllowed,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    Some(encode_bool)
);

/// All HAL plugins currently loaded
pub const SYSTEM_PLUGIN_LIST: Property<Vec<AudioObjectID>, System, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyPlugInList,
        kAudioObjectPropertyScopeGlobal
    ),
    read_vec_audio_object_id,
    None,
);

/// Hints to the HAL about the current power situation
pub const SYSTEM_POWER_HINT: Property<u32, System, ReadWrite, Silent, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyPowerHint,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    Some(encode_u32)
);

/// All audio taps known to the HAL
pub const SYSTEM_TAP_LIST: Property<Vec<AudioObjectID>, System, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyTapList,
        kAudioObjectPropertyScopeGlobal
    ), read_vec_audio_object_id,
    None,
);

/// The default device used by the system for alert and UI sounds
pub const SYSTEM_DEFAULT_SYSTEM_OUTPUT: Property<AudioObjectID, System, ReadWrite, Listenable, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyDefaultSystemOutputDevice,
        kAudioObjectPropertyScopeGlobal
    ),
    read_audio_object_id,
    Some(encode_audio_object_id),
);

/// Translate a device UID string to an AudioObjectID; requires qualifier with UID
pub const SYSTEM_TRANSLATE_UID_TO_DEVICE: Property<AudioObjectID, System, ReadOnly, Silent, NeedQualifier<String>> =
Property::new(
    address(
        kAudioHardwarePropertyTranslateUIDToDevice,
        kAudioObjectPropertyScopeGlobal
    ),
    read_audio_object_id,
    None,
);

/// Translate a box UID string to an AudioObjectID; requires qualifier with UID
pub const SYSTEM_TRANSLATE_UID_TO_BOX: Property<AudioObjectID, System, ReadOnly, Silent, NeedQualifier<String>> =
Property::new(
    address(
        kAudioHardwarePropertyTranslateUIDToBox,
        kAudioObjectPropertyScopeGlobal
    ),
    read_audio_object_id,
    None,
);

/// Translate a clock device UID to an AudioObjectID; requires qualifier with UID
pub const SYSTEM_TRANSLATE_UID_TO_CLOCK_DEVICE: Property<AudioObjectID, System, ReadOnly, Silent, NeedQualifier<String>> =
Property::new(
    address(
        kAudioHardwarePropertyTranslateUIDToClockDevice,
        kAudioObjectPropertyScopeGlobal
    ),
    read_audio_object_id,
    None,
);

/// Translate a plug-in bundle ID to an AudioObjectID; requires qualifier with bundle ID
pub const SYSTEM_TRANSLATE_BUNDLE_ID_TO_PLUGIN: Property<AudioObjectID, System, ReadOnly, Silent, NeedQualifier<String>> =
Property::new(
    address(
        kAudioHardwarePropertyTranslateBundleIDToPlugIn,
        kAudioObjectPropertyScopeGlobal
    ),
    read_audio_object_id,
    None,
);

/// Translate a transport manager bundle ID to an AudioObjectID; requires qualifier
pub const SYSTEM_TRANSLATE_BUNDLE_ID_TO_TRANSPORT_MANAGER: Property<AudioObjectID, System, ReadOnly, Silent, NeedQualifier<String>> =
Property::new(
    address(
        kAudioHardwarePropertyTranslateBundleIDToTransportManager,
        kAudioObjectPropertyScopeGlobal
    ),
    read_audio_object_id,
    None,
);

/// All transport manager AudioObjectIDs known to the HAL
pub const SYSTEM_TRANSPORT_MANAGER_LIST: Property<Vec<AudioObjectID>, System, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyTransportManagerList,
        kAudioObjectPropertyScopeGlobal
    ),
    read_vec_audio_object_id,
    None,
);

/// Whether stereo pairs are mixed down to mono for output
pub const SYSTEM_MIX_STEREO_TO_MONO: Property<bool, System, ReadWrite, Silent, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyMixStereoToMono,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    Some(encode_bool),
);

/// Whether this process is the master of the HAL
///
/// Deprecated in macOS 12 in favour of `kAudioHardwarePropertyProcessIsMain`.
pub const SYSTEM_PROCESS_IS_MASTER: Property<bool, System, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyProcessIsMaster,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    None,
);

/// Writing any value triggers a user-ID-changed notification
pub const SYSTEM_USER_ID_CHANGED: Property<u32, System, ReadWrite, Silent, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyUserIDChanged,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    Some(encode_u32),
);

/// Whether audio from this process is audible (not muted at the system level)
pub const SYSTEM_PROCESS_IS_AUDIBLE: Property<bool, System, ReadWrite, Silent, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyProcessIsAudible,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    Some(encode_bool),
);

/// Whether the HAL is allowed to unload itself after the last client disconnects
pub const SYSTEM_UNLOADING_IS_ALLOWED: Property<bool, System, ReadWrite, Silent, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyUnloadingIsAllowed,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    Some(encode_bool),
);

/// Whether processes are allowed to take hog mode
pub const SYSTEM_HOG_MODE_IS_ALLOWED: Property<bool, System, ReadWrite, Silent, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyHogModeIsAllowed,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    Some(encode_bool),
);

/// Whether the current user session is active or the system is running headless
pub const SYSTEM_USER_SESSION_IS_ACTIVE_OR_HEADLESS: Property<bool, System, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyUserSessionIsActiveOrHeadless,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    None,
);

/// Listen to detect when the HAL daemon has restarted
pub const SYSTEM_SERVICE_RESTARTED: Property<u32, System, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyServiceRestarted,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    None,
);
