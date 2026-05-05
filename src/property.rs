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

/// The class ID of the superclass of this object's class.
///
/// Used to walk the CoreAudio class hierarchy. For example, an `AudioDevice`'s
/// base class is `AudioObject`.
pub const OBJECT_BASE_CLASS: Property<u32, Global, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioObjectPropertyBaseClass,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    None,
);

/// The class ID identifying what kind of object this is.
///
/// For example, `kAudioDeviceClassID` for a device or `kAudioStreamClassID`
/// for a stream. Use this alongside `OBJECT_BASE_CLASS` to inspect the full
/// type hierarchy of an object.
pub const OBJECT_CLASS: Property<u32, Global, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioObjectPropertyClass,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    None,
);

/// The `AudioObjectID` of the object that owns this one.
///
/// For a stream, this is the device it belongs to. For a device, this is the
/// system object. Useful for traversing the object tree upward.
pub const OBJECT_OWNER: Property<AudioObjectID, Global, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioObjectPropertyOwner,
        kAudioObjectPropertyScopeGlobal
    ),
    read_audio_object_id,
    None,
);

/// The model name of the hardware this object represents, as provided by the manufacturer.
///
/// Unlike the display name, this reflects the hardware model rather than any
/// user-assigned label — e.g. "Scarlett 2i2" rather than "My Interface".
pub const OBJECT_MODEL_NAME: Property<String, Global, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioObjectPropertyModelName,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// The name of the company that manufactured the hardware this object represents.
pub const OBJECT_MANUFACTURER: Property<String, Global, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioObjectPropertyManufacturer,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// The display name of the specified element (channel) within the specified scope.
///
/// Elements correspond to individual channels — for example, "Front Left" or
/// "Rear Right". Element 0 (`kAudioObjectPropertyElementMain`) names the object
/// as a whole rather than a specific channel.
pub const OBJECT_ELEMENT_NAME: Property<String, Global, ReadOnly, Silent, NeedElement> =
Property::new(
    address(
        kAudioObjectPropertyElementName,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// The category name of the specified element, describing the type of signal it carries.
///
/// Describes what role the channel plays — for example, "Headphones",
/// "Microphone", or "Line In" — rather than just its position.
pub const OBJECT_ELEMENT_CATEGORY_NAME: Property<String, Global, ReadOnly, Silent, NeedElement> =
Property::new(
    address(
        kAudioObjectPropertyElementCategoryName,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// The ordinal name of the specified element — typically its channel number as a string.
///
/// Where `OBJECT_ELEMENT_NAME` gives a descriptive label like "Front Left",
/// this gives a positional label like "1" or "2".
pub const OBJECT_ELEMENT_NUMBER_NAME: Property<String, Global, ReadOnly, Silent, NeedElement> =
Property::new(
    address(
        kAudioObjectPropertyElementNumberName,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// The `AudioObjectID`s of all objects directly owned by this object, filtered by class.
///
/// The qualifier is a list of `AudioClassID` values (`u32`). Only objects whose
/// class matches one of the provided IDs are returned. Pass an empty list to
/// return all owned objects regardless of class.
pub const OBJECT_OWNED_OBJECTS: Property<Vec<AudioObjectID>, Global, ReadOnly, Silent, NeedQualifier<Vec<u32>>> =
Property::new(
    address(
        kAudioObjectPropertyOwnedObjects,
        kAudioObjectPropertyScopeGlobal
    ),
    read_vec_audio_object_id,
    None,
);

/// The bundle ID of the HAL plug-in that created this object.
///
/// For Apple built-in hardware this is the Apple HAL plug-in; for third-party
/// devices it identifies the driver. Useful for associating an object with its
/// vendor's software.
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

/// The display name of the device, as shown in Audio MIDI Setup and system UIs.
///
/// This can change if the user renames the device. Listen to be notified of
/// name changes.
pub const DEVICE_NAME: Property<String, Device, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioObjectPropertyName,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// A persistent string that uniquely identifies this specific hardware unit.
///
/// Unlike the numeric `AudioObjectID` which changes across boots and
/// reconnects, this UID is stable and suitable for storing in preferences to
/// refer to a specific device later. It is unique per physical unit, unlike
/// `DEVICE_MODEL_UID` which is the same for all devices of the same model.
pub const DEVICE_UID: Property<String, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyDeviceUID,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// Whether the device is still physically present and operational.
///
/// Becomes `false` when a USB or Bluetooth device is disconnected. Listen to
/// this property to detect device removal without polling.
pub const DEVICE_IS_ALIVE: Property<bool, Device, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyDeviceIsAlive,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    None,
);

/// Whether the device currently has any active I/O clients.
///
/// `true` while any process holds an `IOProc` or is otherwise driving I/O on
/// the device. A device may still be alive but not running if no client has
/// started audio.
pub const DEVICE_IS_RUNNING: Property<bool, Device, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyDeviceIsRunning,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    None,
);

/// The sample rate the device is currently running at, in Hz.
///
/// Writing this requests a sample rate change; the actual change may happen
/// asynchronously. Listen to confirm when the new rate takes effect. The
/// requested rate must appear in `DEVICE_AVAILABLE_SAMPLE_RATES`.
pub const DEVICE_NOMINAL_SAMPLE_RATE: Property<f64, Device, ReadWrite, Listenable, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyNominalSampleRate,
        kAudioObjectPropertyScopeGlobal
    ),
    read_f64,
    Some(encode_f64),
);

/// The set of sample rates — or continuous ranges — that the device hardware supports.
///
/// Some devices expose discrete values; others expose a range with a min and
/// max. Use this to populate a sample rate selector and to validate a rate
/// before writing `DEVICE_NOMINAL_SAMPLE_RATE`.
pub const DEVICE_AVAILABLE_SAMPLE_RATES: Property<Vec<SampleRateRange>, Device, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyAvailableNominalSampleRates,
        kAudioObjectPropertyScopeGlobal
    ),
    read_vec_sample_rate_range,
    None,
);

/// The number of frames in each I/O cycle.
///
/// Smaller values reduce latency but increase CPU overhead. Must fall within
/// the bounds reported by `DEVICE_BUFFER_FRAME_SIZE_RANGE`. If
/// `DEVICE_USES_VARIABLE_BUFFER_FRAME_SIZES` is set, the device ignores this
/// and delivers variable-length cycles instead.
pub const DEVICE_BUFFER_FRAME_SIZE: Property<u32, Device, ReadWrite, Listenable, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyBufferFrameSize,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    Some(encode_u32)
);

/// The minimum and maximum buffer frame sizes this device supports.
///
/// Use this to clamp or validate a requested buffer size before writing
/// `DEVICE_BUFFER_FRAME_SIZE`.
pub const DEVICE_BUFFER_FRAME_SIZE_RANGE: Property<BufferFrameSizeRange, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyBufferFrameSizeRange,
        kAudioObjectPropertyScopeGlobal
    ),
    read_buffer_size_range,
    None,
);

/// Frames of latency added by the device hardware on the input path.
///
/// Does not include the I/O buffer size. Add this to `STREAM_LATENCY` and the
/// buffer size when computing total input latency.
pub const DEVICE_INPUT_LATENCY: Property<u32, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyLatency,
        kAudioObjectPropertyScopeInput
    ),
    read_u32,
    None,
);

/// Frames of latency added by the device hardware on the output path.
///
/// Does not include the I/O buffer size. Add this to `STREAM_LATENCY` and the
/// buffer size when computing total output latency.
pub const DEVICE_OUTPUT_LATENCY: Property<u32, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyLatency,
        kAudioObjectPropertyScopeOutput
    ),
    read_u32,
    None
);

/// The `AudioObjectID`s of all input streams on this device.
///
/// Each stream covers one or more input channels. Listen to detect streams
/// being added or removed.
pub(crate) const DEVICE_INPUT_STREAMS: Property<Vec<AudioObjectID>, Device, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyStreams,
        kAudioObjectPropertyScopeInput
    ),
    read_vec_audio_object_id,
    None,
);

/// The `AudioObjectID`s of all output streams on this device.
///
/// Each stream covers one or more output channels. Listen to detect streams
/// being added or removed.
pub(crate) const DEVICE_OUTPUT_STREAMS: Property<Vec<AudioObjectID>, Device, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyStreams,
        kAudioObjectPropertyScopeOutput
    ),
    read_vec_audio_object_id,
    None,
);

/// The PID of the process that currently holds exclusive access to this device, or `-1` if none.
///
/// Write the calling process's PID to claim hog mode (exclusive access),
/// preventing all other clients from using the device. Write `-1` to release
/// it. Listen to detect when another process takes or releases exclusive access.
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

/// Bundle ID of the application responsible for configuring this device.
///
/// Typically a manufacturer-supplied control panel. If present, this app
/// should be launched when the user requests device settings. Distinct from
/// `DEVICE_HOG_MODE` — this identifies the configuration app, not the
/// exclusive-access holder.
pub const DEVICE_CONFIGURATION_APPLICATION: Property<String, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyConfigurationApplication,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// A persistent identifier shared by all hardware units of the same model.
///
/// Where `DEVICE_UID` is unique per physical unit, this is the same for every
/// device of the same type. Useful for applying model-specific settings
/// without targeting a particular unit.
pub const DEVICE_MODEL_UID: Property<String, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyModelUID,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// A four-character code describing how the device connects to the system.
///
/// Matches one of the `kAudioDeviceTransportType*` constants — e.g. USB,
/// FireWire, Bluetooth, PCI, or Built-In. Useful for filtering devices by
/// connection type or adjusting latency expectations.
pub const DEVICE_TRANSPORT_TYPE: Property<u32, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyTransportType,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    None,
);

/// `AudioObjectID`s of other devices sharing the same underlying transport or clock.
///
/// For example, the separate input and output sides of a USB interface that
/// appear as two logical devices will list each other here. Useful for
/// coordinating clock settings across related devices.
pub const DEVICE_RELATED_DEVICES: Property<Vec<AudioObjectID>, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyRelatedDevices,
        kAudioObjectPropertyScopeGlobal
    ),
    read_vec_audio_object_id,
    None,
);

/// An opaque integer grouping devices that share the same hardware clock.
///
/// Two devices with the same non-zero value are phase-locked to each other
/// and can be used together without sample-rate conversion. A value of `0`
/// means the device is not part of any shared clock domain.
pub const DEVICE_CLOCK_DOMAIN: Property<u32, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyClockDomain,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    None,
);

/// Whether this device is eligible to be selected as the system default device.
///
/// Some devices — such as aggregate devices or certain virtual devices — cannot
/// serve as the system default even though they are usable programmatically.
pub const DEVICE_CAN_BE_DEFAULT: Property<bool, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyDeviceCanBeDefaultDevice,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    None,
);

/// Whether this device can be selected as the system sound output device.
///
/// This is a separate designation from the standard default output. The system
/// sound output is used for UI alerts and notifications, and a device can be
/// eligible for one but not the other.
pub const DEVICE_CAN_BE_DEFAULT_SYSTEM: Property<bool, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyDeviceCanBeDefaultSystemDevice,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    None,
);

/// Extra frames the hardware adds as a safety margin to prevent glitches.
///
/// This is a fixed hardware characteristic beyond the declared buffer size.
/// Add this to the buffer size and `DEVICE_INPUT_LATENCY` or
/// `DEVICE_OUTPUT_LATENCY` when computing absolute end-to-end latency.
pub const DEVICE_SAFETY_OFFSET: Property<u32, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertySafetyOffset,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    None,
);

/// Whether this device is intentionally hidden from user-facing device lists.
///
/// Hidden devices are still fully functional but should not be presented to
/// the user in UI. Typically set on virtual or internal routing devices.
pub const DEVICE_IS_HIDDEN: Property<bool, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyIsHidden,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    None,
);

/// The pair of channel indices the device prefers to use for stereo output or input.
///
/// The first element is the left channel index, the second is right. Typically
/// `[1, 2]` but may differ on multi-channel devices. Write to override the
/// preferred stereo mapping.
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

/// Notification property that fires when the device's I/O cycle misses its deadline.
///
/// The value itself carries no meaning — listen to this property to detect CPU
/// overload events in real time. A sustained stream of notifications indicates
/// the buffer size is too small for the current CPU load.
pub const DEVICE_PROCESSOR_OVERLOAD: Property<u32, Device, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioDeviceProcessorOverload,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    None,
);

/// Notification property that fires when the device's I/O cycle stops unexpectedly.
///
/// Indicates a hardware fault, driver crash, or other abnormal termination.
/// The value itself carries no meaning — listen to detect abnormal stops so
/// your app can recover or report the failure.
pub const DEVICE_IO_STOPPED_ABNORMALLY: Property<u32, Device, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyIOStoppedAbnormally,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    None,
);

/// The fraction of each I/O cycle, from `0.0` to `1.0`, the HAL may use for scheduling.
///
/// Values below `1.0` leave headroom for other real-time work on the same
/// thread. Reduce this if your audio callback shares a thread with other
/// time-critical tasks.
pub const DEVICE_IO_CYCLE_USAGE: Property<f32, Device, ReadWrite, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyIOCycleUsage,
        kAudioObjectPropertyScopeGlobal
    ),
    read_f32,
    Some(encode_f32),
);

/// Non-zero if the device delivers I/O cycles with varying frame counts.
///
/// When set, `DEVICE_BUFFER_FRAME_SIZE` is ignored and the actual frame count
/// is supplied per cycle. Your `IOProc` must handle whatever count the
/// hardware delivers rather than assuming a fixed size.
pub const DEVICE_USES_VARIABLE_BUFFER_FRAME_SIZES: Property<u32, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyUsesVariableBufferFrameSizes,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    None,
);

/// The volume of the specified channel as a linear scalar (`0.0` = silence, `1.0` = full scale).
///
/// Linear scaling is suitable for mathematical operations but not perceptually
/// uniform — prefer `DEVICE_VOLUME_DECIBELS` for UI controls. Listen to
/// detect volume changes made by other processes or system UI.
pub const DEVICE_VOLUME_SCALAR: Property<f32, Device, ReadWrite, Listenable, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyVolumeScalar,
        kAudioObjectPropertyScopeGlobal
    ),
    read_f32,
    Some(encode_f32),
);

/// The volume of the specified channel in dBFS (`0.0` = full scale, negative = attenuation).
///
/// dB steps are perceptually uniform, making this the preferred representation
/// for UI controls. See `DEVICE_VOLUME_RANGE_DECIBELS` for the valid range on
/// this channel.
pub const DEVICE_VOLUME_DECIBELS: Property<f32, Device, ReadWrite, Listenable, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyVolumeDecibels,
        kAudioObjectPropertyScopeGlobal
    ),
    read_f32,
    Some(encode_f32),
);

/// The minimum and maximum volume values in dB that the specified channel supports.
///
/// Use this to set the bounds of a volume slider. The minimum is typically a
/// large negative number representing near-silence rather than true `-inf dB`.
pub const DEVICE_VOLUME_RANGE_DECIBELS: Property<DBRange, Device, ReadOnly, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyVolumeRangeDecibels,
        kAudioObjectPropertyScopeGlobal
    ),
    read_db_range,
    None,
);

/// Translates a linear scalar volume to its dB equivalent for the specified channel.
///
/// The hardware applies its own non-linear mapping, so this gives a more
/// accurate conversion than a generic formula. Set the element to the channel
/// and read back the converted dB value.
pub const DEVICE_VOLUME_SCALAR_TO_DECIBELS: Property<f32, Device, ReadOnly, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyVolumeScalarToDecibels,
        kAudioObjectPropertyScopeGlobal
    ),
    read_f32,
    None,
);

/// Translates a dB volume to its linear scalar equivalent for the specified channel.
///
/// The hardware applies its own non-linear mapping, so this gives a more
/// accurate conversion than a generic formula. Set the element to the channel
/// and read back the converted scalar.
pub const DEVICE_VOLUME_DECIBELS_TO_SCALAR: Property<f32, Device, ReadOnly, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyVolumeDecibelsToScalar,
        kAudioObjectPropertyScopeGlobal
    ),
    read_f32,
    None,
);

/// The stereo pan position of the specified channel (`0.0` = full left, `1.0` = full right).
///
/// The channel indices between which the signal is panned are reported by
/// `DEVICE_STEREO_PAN_CHANNELS`. Not all devices or channels support panning.
pub const DEVICE_STEREO_PAN: Property<f32, Device, ReadWrite, Listenable, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyStereoPan,
        kAudioObjectPropertyScopeGlobal
    ),
    read_f32,
    Some(encode_f32),
);

/// The pair of channel indices between which `DEVICE_STEREO_PAN` applies.
///
/// The first element is the left channel and the second is right. Use these
/// indices when displaying which channels are affected by the pan control.
pub const DEVICE_STEREO_PAN_CHANNELS: Property<ChannelPair, Device, ReadOnly, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyStereoPanChannels,
        kAudioObjectPropertyScopeGlobal
    ),
    read_channel_pair,
    None,
);

/// Whether the specified channel is muted.
///
/// When muted the channel produces silence regardless of its volume setting.
/// Listen to detect mute changes made by other processes or hardware buttons.
pub const DEVICE_MUTE: Property<bool, Device, ReadWrite, Listenable, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyMute,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    Some(encode_bool),
);

/// Whether the specified channel is soloed.
///
/// When any channel is soloed, all non-soloed channels on the same device are
/// silenced. Typically used in monitoring contexts on multi-channel interfaces.
pub const DEVICE_SOLO: Property<bool, Device, ReadWrite, Listenable, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertySolo,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    Some(encode_bool),
);

/// Whether phantom power (+48V) is enabled on the specified input channel.
///
/// Required by condenser microphones. Should be left off for dynamic
/// microphones, ribbon microphones, and line-level sources, as it can damage
/// some equipment.
pub const DEVICE_PHANTOM_POWER: Property<bool, Device, ReadWrite, Listenable, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyPhantomPower,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    Some(encode_bool),
);

/// Whether the polarity of the signal on the specified channel is inverted (180° phase flip).
///
/// Used to correct phase cancellation when combining signals from multiple
/// microphones placed on opposite sides of a source.
pub const DEVICE_PHASE_INVERT: Property<bool, Device, ReadWrite, Listenable, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyPhaseInvert,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    Some(encode_bool),
);

/// Whether the clip indicator for the specified channel is lit.
///
/// `true` means the signal has exceeded full scale at some point since the
/// last reset. Writing `false` clears the indicator; writing `true` has no
/// effect on most hardware.
pub const DEVICE_CLIP_LIGHT: Property<bool, Device, ReadWrite, Listenable, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyClipLight,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    Some(encode_bool),
);

/// Whether the talkback path is active on the specified channel.
///
/// Talkback routes a monitor mix or microphone signal back to performers
/// in a recording scenario, allowing the engineer to communicate with the
/// talent without interrupting the recording.
pub const DEVICE_TALKBACK: Property<bool, Device, ReadWrite, Listenable, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyTalkback,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    Some(encode_bool),
);

/// Whether the listenback path is active on the specified channel.
///
/// Listenback routes the performer's input back to the engineer's monitor
/// mix, letting the engineer hear exactly what the performer is sending
/// without switching source.
pub const DEVICE_LISTENBACK: Property<bool, Device, ReadWrite, Listenable, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyListenback,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    Some(encode_bool),
);

/// Whether a physical cable is detected in the jack for the specified channel.
///
/// Not all hardware supports per-jack detection — on devices that do not,
/// this may always return `true`. Listen to respond to plug/unplug events.
pub const DEVICE_JACK_IS_CONNECTED: Property<bool, Device, ReadOnly, Listenable, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyJackIsConnected,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    None,
);

/// The ID of the currently active input or output data source on the specified channel.
///
/// Data sources represent physical signal paths — for example "Line In",
/// "Optical", or "Headphone". Write a source ID from `DEVICE_DATA_SOURCES` to
/// switch the active source. Listen for changes triggered by other processes
/// or hardware switches.
pub const DEVICE_DATA_SOURCE: Property<u32, Device, ReadWrite, Listenable, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyDataSource,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    Some(encode_u32),
);

/// All data source IDs available on the specified channel.
///
/// Use `DEVICE_DATA_SOURCE_NAME` to get a display label for each ID, and
/// write one of these IDs to `DEVICE_DATA_SOURCE` to switch the active source.
pub const DEVICE_DATA_SOURCES: Property<Vec<AudioObjectID>, Device, ReadOnly, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyDataSources,
        kAudioObjectPropertyScopeGlobal
    ),
    read_vec_audio_object_id,
    None,
);

/// The display name of a data source, looked up by its ID.
///
/// Provide the channel as the element and the source ID (`u32`) as the
/// qualifier. Use the IDs from `DEVICE_DATA_SOURCES` to resolve names for
/// all available sources.
pub const DEVICE_DATA_SOURCE_NAME: Property<String, Device, ReadOnly, Silent, NeedBoth<u32>> =
Property::new(
    address(
        kAudioDevicePropertyDataSourceNameForIDCFString,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// The ID of the clock source the device is currently synchronised to.
///
/// Examples include "Internal", "S/PDIF", or "Word Clock". Write a source ID
/// from `DEVICE_CLOCK_SOURCES` to switch the active clock. Listen to detect
/// clock source changes, which also trigger a sample-rate notification.
pub const DEVICE_CLOCK_SOURCE: Property<u32, Device, ReadWrite, Listenable, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyClockSource,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    Some(encode_u32),
);

/// All clock source IDs available on this device.
///
/// Use `DEVICE_CLOCK_SOURCE_NAME` to get a display label for each ID, and
/// write one of these IDs to `DEVICE_CLOCK_SOURCE` to switch the active clock.
pub const DEVICE_CLOCK_SOURCES: Property<Vec<AudioObjectID>, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyClockSources,
        kAudioObjectPropertyScopeGlobal
    ),
    read_vec_audio_object_id,
    None,
);

/// The display name of a clock source, looked up by its ID.
///
/// Provide the source ID (`u32`) as the qualifier. Use the IDs from
/// `DEVICE_CLOCK_SOURCES` to resolve names for all available sources.
pub const DEVICE_CLOCK_SOURCE_NAME: Property<String, Device, ReadOnly, Silent, NeedQualifier<u32>> =
Property::new(
    address(
        kAudioDevicePropertyClockSourceNameForIDCFString,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// The ID of the currently active play-through destination.
///
/// Play-through routes the device's input signal directly to an output without
/// going through the client's audio graph. Write a destination ID from
/// `DEVICE_PLAY_THRU_DESTINATIONS` to change it.
pub const DEVICE_PLAY_THRU_DESTINATION: Property<u32, Device, ReadWrite, Listenable, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyPlayThruDestination,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    Some(encode_u32),
);

/// All play-through destination IDs available on this device.
///
/// Use `DEVICE_PLAY_THRU_DESTINATION_NAME` to get a display label for each
/// ID, and write one to `DEVICE_PLAY_THRU_DESTINATION` to switch the active
/// destination.
pub const DEVICE_PLAY_THRU_DESTINATIONS: Property<Vec<AudioObjectID>, Device, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioDevicePropertyPlayThruDestinations,
        kAudioObjectPropertyScopeGlobal
    ),
    read_vec_audio_object_id,
    None,
);

/// The display name of a play-through destination, looked up by its ID.
///
/// Provide the destination ID (`u32`) as the qualifier. Use the IDs from
/// `DEVICE_PLAY_THRU_DESTINATIONS` to resolve names for all destinations.
pub const DEVICE_PLAY_THRU_DESTINATION_NAME: Property<String, Device, ReadOnly, Silent, NeedQualifier<u32>> =
Property::new(
    address(
        kAudioDevicePropertyPlayThruDestinationNameForIDCFString,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// The ID of the nominal line level selected for the specified channel.
///
/// Sets the expected signal level for connected equipment — for example
/// "+4 dBu" for professional gear or "-10 dBV" for consumer gear. Write a
/// level ID from `DEVICE_CHANNEL_NOMINAL_LINE_LEVELS` to change it.
pub const DEVICE_CHANNEL_NOMINAL_LINE_LEVEL: Property<u32, Device, ReadWrite, Listenable, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyChannelNominalLineLevel,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    Some(encode_u32),
);

/// All nominal line level IDs available on the specified channel.
///
/// Use `DEVICE_CHANNEL_NOMINAL_LINE_LEVEL_NAME` to get a display label for
/// each ID, and write one to `DEVICE_CHANNEL_NOMINAL_LINE_LEVEL` to change the
/// active level.
pub const DEVICE_CHANNEL_NOMINAL_LINE_LEVELS: Property<Vec<AudioObjectID>, Device, ReadOnly, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyChannelNominalLineLevels,
        kAudioObjectPropertyScopeGlobal
    ),
    read_vec_audio_object_id,
    None,
);

/// The display name of a nominal line level, looked up by its ID.
///
/// Provide the channel as the element and the level ID (`u32`) as the
/// qualifier. Use the IDs from `DEVICE_CHANNEL_NOMINAL_LINE_LEVELS`.
pub const DEVICE_CHANNEL_NOMINAL_LINE_LEVEL_NAME: Property<String, Device, ReadOnly, Silent, NeedBoth<u32>> =
Property::new(
    address(
        kAudioDevicePropertyChannelNominalLineLevelNameForIDCFString,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// The ID of the high-pass filter setting currently active on the specified channel.
///
/// A high-pass filter cuts low-frequency rumble from microphone inputs.
/// Write a setting ID from `DEVICE_HIGH_PASS_FILTER_SETTINGS` to change the
/// cutoff frequency or disable the filter.
pub const DEVICE_HIGH_PASS_FILTER_SETTING: Property<u32, Device, ReadWrite, Listenable, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyHighPassFilterSetting,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    Some(encode_u32),
);

/// All high-pass filter setting IDs available on the specified channel.
///
/// Use `DEVICE_HIGH_PASS_FILTER_SETTING_NAME` to get a display label for each
/// ID, and write one to `DEVICE_HIGH_PASS_FILTER_SETTING` to change the active
/// setting.
pub const DEVICE_HIGH_PASS_FILTER_SETTINGS: Property<Vec<AudioObjectID>, Device, ReadOnly, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertyHighPassFilterSettings,
        kAudioObjectPropertyScopeGlobal
    ),
    read_vec_audio_object_id,
    None,
);

/// The display name of a high-pass filter setting, looked up by its ID.
///
/// Provide the channel as the element and the setting ID (`u32`) as the
/// qualifier. Use the IDs from `DEVICE_HIGH_PASS_FILTER_SETTINGS`.
pub const DEVICE_HIGH_PASS_FILTER_SETTING_NAME: Property<String, Device, ReadOnly, Silent, NeedBoth<u32>> =
Property::new(
    address(
        kAudioDevicePropertyHighPassFilterSettingNameForIDCFString,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// The LFE (subwoofer) channel volume as a linear scalar (`0.0` = silence, `1.0` = full scale).
///
/// This controls the dedicated low-frequency effects channel separately from
/// the main channel volumes. See `DEVICE_VOLUME_SCALAR` for the main channels.
pub const DEVICE_SUB_VOLUME_SCALAR: Property<f32, Device, ReadWrite, Listenable, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertySubVolumeScalar,
        kAudioObjectPropertyScopeGlobal
    ),
    read_f32,
    Some(encode_f32),
);

/// The LFE (subwoofer) channel volume in dBFS.
///
/// Controls the dedicated low-frequency effects channel. See
/// `DEVICE_SUB_VOLUME_RANGE_DECIBELS` for the valid range on this device.
pub const DEVICE_SUB_VOLUME_DECIBELS: Property<f32, Device, ReadWrite, Listenable, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertySubVolumeDecibels,
        kAudioObjectPropertyScopeGlobal
    ),
    read_f32,
    Some(encode_f32),
);

/// The minimum and maximum LFE channel volume values in dB that the device supports.
pub const DEVICE_SUB_VOLUME_RANGE_DECIBELS: Property<DBRange, Device, ReadOnly, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertySubVolumeRangeDecibels,
        kAudioObjectPropertyScopeGlobal
    ),
    read_db_range,
    None,
);

/// Translates a linear scalar LFE volume to its dB equivalent for the specified channel.
///
/// Uses the hardware's own mapping, which may be non-linear. Set the element
/// to the channel and read back the converted dB value.
pub const DEVICE_SUB_VOLUME_SCALAR_TO_DECIBELS: Property<f32, Device, ReadOnly, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertySubVolumeScalarToDecibels,
        kAudioObjectPropertyScopeGlobal
    ),
    read_f32,
    None,
);

/// Translates a dB LFE volume to its linear scalar equivalent for the specified channel.
///
/// Uses the hardware's own mapping, which may be non-linear. Set the element
/// to the channel and read back the converted scalar.
pub const DEVICE_SUB_VOLUME_DECIBELS_TO_SCALAR: Property<f32, Device, ReadOnly, Silent, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertySubVolumeDecibelsToScalar,
        kAudioObjectPropertyScopeGlobal
    ),
    read_f32,
    None,
);

/// Whether the LFE (subwoofer) channel is muted.
///
/// Mutes only the low-frequency effects channel, leaving the main channels
/// unaffected. See `DEVICE_MUTE` for per-channel muting.
pub const DEVICE_SUB_MUTE: Property<bool, Device, ReadWrite, Listenable, NeedElement> =
Property::new(
    address(
        kAudioDevicePropertySubMute,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    Some(encode_bool),
);

// ---- Stream constants ----

/// The display name of this stream.
///
/// Usually reflects the stream's direction and channel layout — for example
/// "Built-in Microphone" or "Headphones". Listen to be notified of name changes.
pub const STREAM_NAME: Property<String, Stream, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioObjectPropertyName,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// Whether this stream is currently active and processing audio.
///
/// A stream can exist but be inactive — for example when the device is idle or
/// when the stream has been deactivated by the driver. Listen to track changes
/// in stream activity.
pub const STREAM_IS_ACTIVE: Property<bool, Stream, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioStreamPropertyIsActive,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    None,
);

/// The direction of this stream: `0` for output, `1` for input.
///
/// Fixed at stream creation — a stream cannot change direction at runtime.
/// Use this to determine whether the stream carries data to or from the
/// hardware without having to track which device scope it came from.
pub const STREAM_DIRECTION: Property<u32, Stream, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioStreamPropertyDirection,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    None,
);

/// The audio format this stream presents to the client application.
///
/// This is the format your app sends or receives — sample rate, bit depth,
/// channel count, and encoding. The hardware may use a different physical
/// format internally (see `STREAM_PHYSICAL_FORMAT`) and convert. Writing this
/// requests a format change; listen to confirm when it takes effect.
pub const STREAM_VIRTUAL_FORMAT: Property<StreamDescription, Stream, ReadWrite, Listenable, NoExtra> =
Property::new(
    address(
        kAudioStreamPropertyVirtualFormat,
        kAudioObjectPropertyScopeGlobal
    ),
    read_stream_description,
    Some(encode_stream_description)
);

/// The audio format the hardware actually uses on this stream.
///
/// May differ from `STREAM_VIRTUAL_FORMAT` when the driver performs sample
/// rate conversion or bit-depth expansion. Changing the physical format
/// affects the underlying hardware configuration and may impact all clients
/// sharing the device.
pub const STREAM_PHYSICAL_FORMAT: Property<StreamDescription, Stream, ReadWrite, Listenable, NoExtra> =
Property::new(
    address(
        kAudioStreamPropertyPhysicalFormat,
        kAudioObjectPropertyScopeGlobal
    ),
    read_stream_description,
    Some(encode_stream_description)
);

/// A code describing the physical endpoint this stream connects to.
///
/// Matches one of the `kAudioStreamTerminalType*` constants — for example
/// `kAudioStreamTerminalTypeMicrophone`, `kAudioStreamTerminalTypeHeadphones`,
/// or `kAudioStreamTerminalTypeSpeaker`. Useful for choosing an appropriate
/// icon or routing hint in UI.
pub const TERMINAL_TYPE: Property<u32, Stream, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioStreamPropertyTerminalType,
        kAudioObjectPropertyScopeGlobal,
    ),
    read_u32,
    None,
);

/// The one-based device channel number that corresponds to element 1 of this stream.
///
/// Used to map stream-relative channel positions to absolute device channel
/// numbers. For example, if a stream starts at channel 3, its element 1 is
/// device channel 3, element 2 is device channel 4, and so on.
pub const STARTING_CHANNEL: Property<u32, Stream, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioStreamPropertyStartingChannel,
        kAudioObjectPropertyScopeGlobal,
    ),
    read_u32,
    None,
);

/// All virtual formats this stream can present to clients, each paired with a sample rate range.
///
/// Use this to populate a format selector for the client-facing audio format.
/// Each entry combines a `StreamDescription` with the range of sample rates
/// supported at that format. Listen to detect format availability changes.
pub const STREAM_AVAILABLE_VIRTUAL_FORMATS: Property<Vec<StreamRangedDescription>, Stream, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioStreamPropertyAvailableVirtualFormats,
        kAudioObjectPropertyScopeGlobal
    ),
    read_vec_stream_ranged_description,
    None,
);

/// All physical formats the hardware natively supports, each paired with a sample rate range.
///
/// Unlike the virtual formats, these reflect what the hardware DAC or ADC can
/// actually run at. Selecting a physical format that matches your target
/// sample rate avoids driver-level sample rate conversion.
pub const STREAM_AVAILABLE_PHYSICAL_FORMATS: Property<Vec<StreamRangedDescription>, Stream, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioStreamPropertyAvailablePhysicalFormats,
        kAudioObjectPropertyScopeGlobal
    ),
    read_vec_stream_ranged_description,
    None,
);

/// Frames of latency introduced by this stream, in addition to the device latency.
///
/// Add this to `DEVICE_INPUT_LATENCY` or `DEVICE_OUTPUT_LATENCY` and the
/// buffer size for the total hardware latency on this audio path. Listen to
/// detect latency changes when the format or clock changes.
pub const STREAM_LATENCY: Property<u32, Stream, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioStreamPropertyLatency,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    None,
);

// ---- System constants ----

/// The name of the HAL system object — always "HAL".
pub const SYSTEM_NAME: Property<String, System, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioObjectPropertyName,
        kAudioObjectPropertyScopeGlobal
    ),
    read_string,
    None,
);

/// The `AudioObjectID`s of all audio devices currently known to the HAL.
///
/// Includes built-in and all connected external hardware. Listen to detect
/// devices being plugged in or removed — the list changes on any
/// connect/disconnect event.
pub(crate) const SYSTEM_DEVICES: Property<Vec<AudioObjectID>, System, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyDevices,
        kAudioObjectPropertyScopeGlobal
    ), read_vec_audio_object_id,
    None,
);

/// The `AudioObjectID` of the device currently selected as the system default input.
///
/// Write a device's ID to change the system default input. Listen to detect
/// when the user or another process changes it — for example via System
/// Settings or the menu bar volume control.
pub(crate) const SYSTEM_DEFAULT_INPUT: Property<AudioObjectID, System, ReadWrite, Listenable, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyDefaultInputDevice,
        kAudioObjectPropertyScopeGlobal
    ),
    read_audio_object_id,
    Some(encode_audio_object_id)
);

/// The `AudioObjectID` of the device currently selected as the system default output.
///
/// Write a device's ID to change the system default output. Listen to detect
/// when the user or another process changes it.
pub(crate) const SYSTEM_DEFAULT_OUTPUT: Property<AudioObjectID, System, ReadWrite, Listenable, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyDefaultOutputDevice,
        kAudioObjectPropertyScopeGlobal
    ),
    read_audio_object_id,
    Some(encode_audio_object_id)
);

/// The `AudioObjectID`s of all audio box objects known to the HAL.
///
/// Boxes represent external hardware enclosures and may contain one or more
/// devices. Listen to detect boxes being connected or disconnected.
pub const SYSTEM_BOX_LIST: Property<Vec<AudioObjectID>, System, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyBoxList,
        kAudioObjectPropertyScopeGlobal
    ),
    read_vec_audio_object_id,
    None,
);

/// The `AudioObjectID`s of all standalone clock devices known to the HAL.
///
/// Clock devices are dedicated word-clock or sync sources that carry timing
/// but no audio. Listen to detect clock devices being connected or removed.
pub const SYSTEM_CLOCK_DEVICE_LIST: Property<Vec<AudioObjectID>, System, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyClockDeviceList,
        kAudioObjectPropertyScopeGlobal
    ),
    read_vec_audio_object_id,
    None,
);

/// Whether the HAL daemon is currently starting up or shutting down.
///
/// Attempting to use audio during this window may fail. Check this if your app
/// initialises audio at launch and needs to handle the case where the HAL is
/// not yet ready.
pub const SYSTEM_IS_INITING_OR_EXITING: Property<bool, System, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyIsInitingOrExiting,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    None,
);

/// Whether the HAL will allow the system to sleep while audio I/O is active.
///
/// Write `false` to hold a sleep assertion during low-latency recording or
/// playback. Remember to restore it to `true` when audio stops, or the system
/// will never sleep while your process is running.
pub const SYSTEM_SLEEPING_IS_ALLOWED: Property<bool, System, ReadWrite, Listenable, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertySleepingIsAllowed,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    Some(encode_bool)
);

/// The `AudioObjectID`s of all HAL plug-ins currently loaded.
///
/// Plug-ins provide the driver layer between the HAL and hardware. Listen to
/// detect plug-ins being loaded or unloaded, which typically happens when a
/// third-party audio device's driver is installed or removed.
pub const SYSTEM_PLUGIN_LIST: Property<Vec<AudioObjectID>, System, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyPlugInList,
        kAudioObjectPropertyScopeGlobal
    ),
    read_vec_audio_object_id,
    None,
);

/// A hint to the HAL about the system's current power mode.
///
/// Writing `kAudioHardwarePowerHintFavorSavingPower` allows the HAL to make
/// power-optimised scheduling decisions. Writing `kAudioHardwarePowerHintNone`
/// restores normal behaviour.
pub const SYSTEM_POWER_HINT: Property<u32, System, ReadWrite, Silent, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyPowerHint,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    Some(encode_u32)
);

/// The `AudioObjectID`s of all audio tap objects currently known to the HAL.
///
/// Taps intercept audio streams for monitoring, recording, or processing
/// without requiring exclusive device access. Listen to detect taps being
/// created or destroyed.
pub const SYSTEM_TAP_LIST: Property<Vec<AudioObjectID>, System, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyTapList,
        kAudioObjectPropertyScopeGlobal
    ), read_vec_audio_object_id,
    None,
);

/// The `AudioObjectID` of the device used for system alert and UI sounds.
///
/// May be different from the default output device. Write a device's ID to
/// redirect system sounds to a specific output. Listen to detect when the
/// user changes it in System Settings.
pub const SYSTEM_DEFAULT_SYSTEM_OUTPUT: Property<AudioObjectID, System, ReadWrite, Listenable, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyDefaultSystemOutputDevice,
        kAudioObjectPropertyScopeGlobal
    ),
    read_audio_object_id,
    Some(encode_audio_object_id),
);

/// Translates a device UID string to the `AudioObjectID` of the matching device.
///
/// Provide the UID string as the qualifier. Returns `kAudioObjectUnknown` if
/// no currently connected device has that UID. Use this to restore a saved
/// device reference across sessions.
pub const SYSTEM_TRANSLATE_UID_TO_DEVICE: Property<AudioObjectID, System, ReadOnly, Silent, NeedQualifier<String>> =
Property::new(
    address(
        kAudioHardwarePropertyTranslateUIDToDevice,
        kAudioObjectPropertyScopeGlobal
    ),
    read_audio_object_id,
    None,
);

/// Translates a box UID string to the `AudioObjectID` of the matching box.
///
/// Provide the UID string as the qualifier. Returns `kAudioObjectUnknown` if
/// no connected box has that UID.
pub const SYSTEM_TRANSLATE_UID_TO_BOX: Property<AudioObjectID, System, ReadOnly, Silent, NeedQualifier<String>> =
Property::new(
    address(
        kAudioHardwarePropertyTranslateUIDToBox,
        kAudioObjectPropertyScopeGlobal
    ),
    read_audio_object_id,
    None,
);

/// Translates a clock device UID string to the `AudioObjectID` of the matching clock device.
///
/// Provide the UID string as the qualifier. Returns `kAudioObjectUnknown` if
/// no connected clock device has that UID.
pub const SYSTEM_TRANSLATE_UID_TO_CLOCK_DEVICE: Property<AudioObjectID, System, ReadOnly, Silent, NeedQualifier<String>> =
Property::new(
    address(
        kAudioHardwarePropertyTranslateUIDToClockDevice,
        kAudioObjectPropertyScopeGlobal
    ),
    read_audio_object_id,
    None,
);

/// Translates a HAL plug-in bundle ID to the `AudioObjectID` of the loaded plug-in.
///
/// Provide the bundle ID string as the qualifier. Returns `kAudioObjectUnknown`
/// if no plug-in with that bundle ID is currently loaded.
pub const SYSTEM_TRANSLATE_BUNDLE_ID_TO_PLUGIN: Property<AudioObjectID, System, ReadOnly, Silent, NeedQualifier<String>> =
Property::new(
    address(
        kAudioHardwarePropertyTranslateBundleIDToPlugIn,
        kAudioObjectPropertyScopeGlobal
    ),
    read_audio_object_id,
    None,
);

/// Translates a transport manager bundle ID to the `AudioObjectID` of the matching manager.
///
/// Provide the bundle ID string as the qualifier. Returns `kAudioObjectUnknown`
/// if no transport manager with that bundle ID is currently loaded.
pub const SYSTEM_TRANSLATE_BUNDLE_ID_TO_TRANSPORT_MANAGER: Property<AudioObjectID, System, ReadOnly, Silent, NeedQualifier<String>> =
Property::new(
    address(
        kAudioHardwarePropertyTranslateBundleIDToTransportManager,
        kAudioObjectPropertyScopeGlobal
    ),
    read_audio_object_id,
    None,
);

/// The `AudioObjectID`s of all transport manager objects known to the HAL.
///
/// Transport managers handle discovery and configuration of audio devices over
/// a particular protocol — for example, the AVB or Thunderbolt transport
/// managers.
pub const SYSTEM_TRANSPORT_MANAGER_LIST: Property<Vec<AudioObjectID>, System, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyTransportManagerList,
        kAudioObjectPropertyScopeGlobal
    ),
    read_vec_audio_object_id,
    None,
);

/// Whether the HAL is currently downmixing stereo output to mono.
///
/// Useful for accessibility scenarios or devices with a single speaker. Note
/// that this is a global system setting and affects all audio output, not just
/// your process.
pub const SYSTEM_MIX_STEREO_TO_MONO: Property<bool, System, ReadWrite, Silent, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyMixStereoToMono,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    Some(encode_bool),
);

/// Whether this process has elevated privileges over the HAL.
///
/// The HAL master can perform configuration operations that other processes
/// cannot. Deprecated in macOS 12 in favour of
/// `kAudioHardwarePropertyProcessIsMain`.
pub const SYSTEM_PROCESS_IS_MASTER: Property<bool, System, ReadOnly, Silent, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyProcessIsMaster,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    None,
);

/// Write-only property that broadcasts a user-identity-changed notification system-wide.
///
/// Writing any value signals to all HAL clients that the current user has
/// changed — for example after fast-user switching — allowing them to refresh
/// any user-specific audio state. Listen to receive these notifications from
/// other processes.
pub const SYSTEM_USER_ID_CHANGED: Property<u32, System, ReadWrite, Listenable, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyUserIDChanged,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    Some(encode_u32),
);

/// Whether audio from this process is currently audible at the system level.
///
/// `false` means this process is muted system-wide, independently of any
/// per-device volume or mute settings. Another process (or the system itself)
/// can mute this process. Listen to detect when the system mutes or unmutes
/// your process.
pub const SYSTEM_PROCESS_IS_AUDIBLE: Property<bool, System, ReadWrite, Listenable, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyProcessIsAudible,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    Some(encode_bool),
);

/// Whether the HAL daemon may unload itself when the last client disconnects.
///
/// Write `false` if your process needs the HAL to persist between I/O sessions
/// — for example to preserve listener registrations or shared state. Restore
/// to `true` when no longer needed.
pub const SYSTEM_UNLOADING_IS_ALLOWED: Property<bool, System, ReadWrite, Silent, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyUnloadingIsAllowed,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    Some(encode_bool),
);

/// Whether any process is allowed to take exclusive (hog mode) access to a device.
///
/// Write `false` to prevent any process from claiming hog mode system-wide —
/// useful in environments where shared audio access must be enforced. Note
/// that this requires appropriate privileges to set.
pub const SYSTEM_HOG_MODE_IS_ALLOWED: Property<bool, System, ReadWrite, Silent, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyHogModeIsAllowed,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    Some(encode_bool),
);

/// Whether this process's user session is currently active, or the system is headless.
///
/// `false` during fast-user switching when another user's session is in the
/// foreground. Listen to pause or resume audio when your session moves in or
/// out of the foreground — playing audio in a background session is typically
/// disallowed or undesirable.
pub const SYSTEM_USER_SESSION_IS_ACTIVE_OR_HEADLESS: Property<bool, System, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyUserSessionIsActiveOrHeadless,
        kAudioObjectPropertyScopeGlobal
    ),
    read_bool,
    None,
);

/// Notification property that fires whenever the HAL daemon restarts.
///
/// The value carries no meaning. Listen to this property to detect daemon
/// restarts and re-initialise any HAL state that does not survive a restart
/// — such as IOProcs, listeners, and hog mode claims.
pub const SYSTEM_SERVICE_RESTARTED: Property<u32, System, ReadOnly, Listenable, NoExtra> =
Property::new(
    address(
        kAudioHardwarePropertyServiceRestarted,
        kAudioObjectPropertyScopeGlobal
    ),
    read_u32,
    None,
);
