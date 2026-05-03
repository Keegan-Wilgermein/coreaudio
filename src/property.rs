//! # Properties

// ----- Imports ------------
use crate::{data_types::{BufferFrameSizeRange, SampleRateRange, StreamDescription}, errors::{CoreAudioError, ErrorKind}, object::{Device, Stream, System}};
use std::marker::PhantomData;
use core_foundation::{base::TCFType, string::{CFString, CFStringRef}};
use coreaudio_sys::{
    AudioObjectID,
    AudioObjectPropertyAddress,
    AudioObjectPropertyScope,
    AudioObjectPropertySelector,
    AudioStreamBasicDescription,
    AudioValueRange,
    kAudioDevicePropertyAvailableNominalSampleRates,
    kAudioDevicePropertyBufferFrameSize,
    kAudioDevicePropertyBufferFrameSizeRange,
    kAudioDevicePropertyDeviceIsAlive,
    kAudioDevicePropertyDeviceIsRunning,
    kAudioDevicePropertyDeviceUID,
    kAudioDevicePropertyLatency,
    kAudioDevicePropertyNominalSampleRate,
    kAudioDevicePropertyStreams,
    kAudioDevicePropertyHogMode,
    kAudioHardwarePropertyBoxList,
    kAudioHardwarePropertyClockDeviceList,
    kAudioHardwarePropertyDefaultInputDevice,
    kAudioHardwarePropertyDefaultOutputDevice,
    kAudioHardwarePropertyDevices,
    kAudioHardwarePropertyIsInitingOrExiting,
    kAudioHardwarePropertyPlugInList,
    kAudioHardwarePropertyPowerHint,
    kAudioHardwarePropertySleepingIsAllowed,
    kAudioHardwarePropertyTapList,
    kAudioObjectPropertyElementMain,
    kAudioObjectPropertyName,
    kAudioObjectPropertyScopeGlobal,
    kAudioObjectPropertyScopeInput,
    kAudioObjectPropertyScopeOutput,
    kAudioStreamPropertyDirection,
    kAudioStreamPropertyIsActive,
    kAudioStreamPropertyLatency,
    kAudioStreamPropertyPhysicalFormat,
    kAudioStreamPropertyVirtualFormat,
};

// ---- Structs -------------
/// Indicates a property to be read only
pub(crate) struct ReadOnly;

/// Indicates a property to be readable and writeable
pub(crate) struct ReadWrite;

/// Indicates a property to be listenable
pub(crate) struct Listenable;

/// Indicates a property to be unlistenable
pub(crate) struct Silent;

pub struct Property<T, Object, Access, L> {
    pub(crate) address: AudioObjectPropertyAddress,
    pub(crate) read: fn(&[u8]) -> Result<T, CoreAudioError>,
    pub(crate) encode: Option<fn(T) -> Vec<u8>>,
    _object: PhantomData<Object>,
    _access: PhantomData<Access>,
    _listenable: PhantomData<L>,
}

impl<T, Object, Access, L> Property<T, Object, Access, L> {
    pub(crate) const fn new(
        address: AudioObjectPropertyAddress,
        read: fn(&[u8]) -> Result<T, CoreAudioError>,
        encode: Option<fn(T) -> Vec<u8>>,
    ) -> Self {
        Self {
            address,
            read,
            encode,
            _object: PhantomData,
            _access: PhantomData,
            _listenable: PhantomData,
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
            Err(_) => return Err(CoreAudioError::from_error_kind(ErrorKind::F64Conversion)),
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

fn read_vec_buffer_size_range(bytes: &[u8]) -> Result<Vec<BufferFrameSizeRange>, CoreAudioError> {
    if bytes.len() % size_of::<AudioValueRange>() != 0 {
        return Err(CoreAudioError::from_error_kind(ErrorKind::ValueRangeConversion));
    }

    Ok(bytes.chunks(size_of::<AudioValueRange>())
        .map(|chunk| unsafe { 
            BufferFrameSizeRange::from(std::ptr::read(chunk.as_ptr() as *const AudioValueRange)) 
        })
        .collect())
}

fn encode_f64(value: f64) -> Vec<u8> {
    value.to_ne_bytes().to_vec()
}

fn encode_u32(value: u32) -> Vec<u8> {
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

// ---- Device constants ----

/// Human readable name of the device
pub const DEVICE_NAME: Property<String, Device, ReadOnly, Silent> =
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
pub const DEVICE_UID: Property<String, Device, ReadOnly, Silent> =
    Property::new(
        address(
            kAudioDevicePropertyDeviceUID,
            kAudioObjectPropertyScopeGlobal
        ),
        read_string,
        None,
    );

/// Whether the device is still alive and connected
pub const DEVICE_IS_ALIVE: Property<bool, Device, ReadOnly, Listenable> =
    Property::new(
        address(
            kAudioDevicePropertyDeviceIsAlive,
            kAudioObjectPropertyScopeGlobal
        ),
        read_bool,
        None,
    );

/// Whether the device is currently running I/O
pub const DEVICE_IS_RUNNING: Property<bool, Device, ReadOnly, Listenable> =
    Property::new(
        address(
            kAudioDevicePropertyDeviceIsRunning,
            kAudioObjectPropertyScopeGlobal
        ),
        read_bool,
        None,
    );

/// The current nominal sample rate of the device
pub const DEVICE_NOMINAL_SAMPLE_RATE: Property<f64, Device, ReadWrite, Listenable> =
    Property::new(
        address(
            kAudioDevicePropertyNominalSampleRate,
            kAudioObjectPropertyScopeGlobal
        ),
        read_f64,
        Some(encode_f64),
    );

/// All sample rates supported by the device
pub(crate) const DEVICE_AVAILABLE_SAMPLE_RATES: Property<Vec<SampleRateRange>, Device, ReadOnly, Silent> =
    Property::new(
        address(
            kAudioDevicePropertyAvailableNominalSampleRates,
            kAudioObjectPropertyScopeGlobal
        ),
        read_vec_sample_rate_range,
        None,
    );

/// The number of frames in the I/O buffer
pub const DEVICE_BUFFER_FRAME_SIZE: Property<u32, Device, ReadWrite, Listenable> =
    Property::new(
        address(
            kAudioDevicePropertyBufferFrameSize,
            kAudioObjectPropertyScopeGlobal
        ),
        read_u32,
        Some(encode_u32)
    );

/// The valid range of buffer frame sizes for the device
pub(crate) const DEVICE_BUFFER_FRAME_SIZE_RANGE: Property<BufferFrameSizeRange, Device, ReadOnly, Silent> =
    Property::new(
        address(
            kAudioDevicePropertyBufferFrameSizeRange,
            kAudioObjectPropertyScopeGlobal
        ),
        read_buffer_size_range,
        None,
    );

/// Input latency of the device in frames
pub const DEVICE_INPUT_LATENCY: Property<u32, Device, ReadOnly, Silent> =
    Property::new(
        address(
            kAudioDevicePropertyLatency,
            kAudioObjectPropertyScopeInput
        ),
        read_u32,
        None,
    );

/// Output latency of the device in frames
pub const DEVICE_OUTPUT_LATENCY: Property<u32, Device, ReadOnly, Silent> =
    Property::new(
        address(
            kAudioDevicePropertyLatency,
            kAudioObjectPropertyScopeOutput
        ),
        read_u32,
        None
    );

/// All input streams on the device
pub(crate) const DEVICE_INPUT_STREAMS: Property<Vec<AudioObjectID>, Device, ReadOnly, Listenable> =
    Property::new(
        address(
            kAudioDevicePropertyStreams,
            kAudioObjectPropertyScopeInput
        ),
        read_vec_audio_object_id,
        None,
    );

/// All output streams on the device
pub(crate) const DEVICE_OUTPUT_STREAMS: Property<Vec<AudioObjectID>, Device, ReadOnly, Listenable> =
    Property::new(
        address(
            kAudioDevicePropertyStreams,
            kAudioObjectPropertyScopeOutput
        ),
        read_vec_audio_object_id,
        None,
    );

pub const DEVICE_HOG_MODE: Property<i32, Device, ReadWrite, Listenable> =
    Property::new(
        address(
            kAudioDevicePropertyHogMode,
            kAudioObjectPropertyScopeGlobal
        ),
        read_i32,
        Some(encode_i32)
    );

// ---- Stream constants ----

/// Human readable name of the stream
pub const STREAM_NAME: Property<String, Stream, ReadOnly, Silent> =
    Property::new(
        address(
            kAudioObjectPropertyName,
            kAudioObjectPropertyScopeGlobal
        ),
        read_string,
        None,
    );

/// Whether the stream is currently active
pub const STREAM_IS_ACTIVE: Property<bool, Stream, ReadOnly, Listenable> =
    Property::new(
        address(
            kAudioStreamPropertyIsActive,
            kAudioObjectPropertyScopeGlobal
        ),
        read_bool,
        None,
    );

/// Direction of the stream — 0 for output, 1 for input
pub const STREAM_DIRECTION: Property<u32, Stream, ReadOnly, Silent> =
    Property::new(
        address(
            kAudioStreamPropertyDirection,
            kAudioObjectPropertyScopeGlobal
        ),
        read_u32,
        None,
    );

/// The virtual format of the stream as presented to the client
pub(crate) const STREAM_VIRTUAL_FORMAT: Property<StreamDescription, Stream, ReadWrite, Listenable> =
    Property::new(
        address(
            kAudioStreamPropertyVirtualFormat,
            kAudioObjectPropertyScopeGlobal
        ),
        read_stream_description,
        Some(encode_stream_description)
    );

/// The physical format of the stream as presented to the hardware
pub(crate) const STREAM_PHYSICAL_FORMAT: Property<StreamDescription, Stream, ReadWrite, Listenable> =
    Property::new(
        address(
            kAudioStreamPropertyPhysicalFormat,
            kAudioObjectPropertyScopeGlobal
        ),
        read_stream_description,
        Some(encode_stream_description)
    );

/// Latency of the stream in frames
pub const STREAM_LATENCY: Property<u32, Stream, ReadOnly, Silent> =
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
pub const SYSTEM_NAME: Property<String, System, ReadOnly, Silent> =
    Property::new(
        address(
            kAudioObjectPropertyName,
            kAudioObjectPropertyScopeGlobal
        ),
        read_string,
        None,
    );

/// All devices currently known to the HAL
pub(crate) const SYSTEM_DEVICES: Property<Vec<AudioObjectID>, System, ReadOnly, Listenable> =
    Property::new(
        address(
            kAudioHardwarePropertyDevices,
            kAudioObjectPropertyScopeGlobal
        ), read_vec_audio_object_id,
        None,
    );

/// The current default input device
pub(crate) const SYSTEM_DEFAULT_INPUT: Property<AudioObjectID, System, ReadWrite, Listenable> =
    Property::new(
        address(
            kAudioHardwarePropertyDefaultInputDevice,
            kAudioObjectPropertyScopeGlobal
        ),
        read_audio_object_id,
        Some(encode_audio_object_id)
    );

/// The current default output device
pub(crate) const SYSTEM_DEFAULT_OUTPUT: Property<AudioObjectID, System, ReadWrite, Listenable> =
    Property::new(
        address(
            kAudioHardwarePropertyDefaultOutputDevice,
            kAudioObjectPropertyScopeGlobal
        ),
        read_audio_object_id,
        Some(encode_audio_object_id)
    );

/// All audio boxes known to the HAL
pub(crate) const SYSTEM_BOX_LIST: Property<Vec<AudioObjectID>, System, ReadOnly, Listenable> =
    Property::new(
        address(
            kAudioHardwarePropertyBoxList,
            kAudioObjectPropertyScopeGlobal
        ),
        read_vec_audio_object_id,
        None,
    );

/// All clock devices known to the HAL
pub(crate) const SYSTEM_CLOCK_DEVICE_LIST: Property<Vec<AudioObjectID>, System, ReadOnly, Listenable> =
    Property::new(
        address(
            kAudioHardwarePropertyClockDeviceList,
            kAudioObjectPropertyScopeGlobal
        ),
        read_vec_audio_object_id,
        None,
    );

/// Whether the HAL is currently initialising or shutting down
pub const SYSTEM_IS_INITING_OR_EXITING: Property<bool, System, ReadOnly, Silent> =
    Property::new(
        address(
            kAudioHardwarePropertyIsInitingOrExiting,
            kAudioObjectPropertyScopeGlobal
        ),
        read_bool,
        None,
    );

/// Whether the system is permitted to sleep while audio is running
pub const SYSTEM_SLEEPING_IS_ALLOWED: Property<bool, System, ReadWrite, Listenable> =
    Property::new(
        address(
            kAudioHardwarePropertySleepingIsAllowed,
            kAudioObjectPropertyScopeGlobal
        ),
        read_bool,
        Some(encode_bool)
    );

/// All HAL plugins currently loaded
pub(crate) const SYSTEM_PLUGIN_LIST: Property<Vec<AudioObjectID>, System, ReadOnly, Silent> =
    Property::new(
        address(
            kAudioHardwarePropertyPlugInList,
            kAudioObjectPropertyScopeGlobal
        ),
        read_vec_audio_object_id,
        None,
    );

/// Hints to the HAL about the current power situation
pub const SYSTEM_POWER_HINT: Property<u32, System, ReadWrite, Silent> =
    Property::new(
        address(
            kAudioHardwarePropertyPowerHint,
            kAudioObjectPropertyScopeGlobal
        ),
        read_u32,
        Some(encode_u32)
    );

/// All audio taps known to the HAL
pub(crate) const SYSTEM_TAP_LIST: Property<Vec<AudioObjectID>, System, ReadOnly, Listenable> =
    Property::new(
        address(
            kAudioHardwarePropertyTapList,
            kAudioObjectPropertyScopeGlobal
        ), read_vec_audio_object_id,
        None,
    );