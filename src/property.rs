//! Property descriptors and constants for CoreAudio HAL objects
//!
//! Each property constant (e.g. [`DEVICE_NAME`]) is a [`Property`] value that
//! encodes the property's value type, target object kind, access mode, and
//! whether it supports listeners — all at the type level
//! Access-mode and listener markers are enforced by the trait bounds on
//! `get_property`, `set_property`, and `add_listener`
//!
//! The marker types ([`ReadOnly`], [`ReadWrite`], [`Listenable`], [`Silent`],
//! [`NoExtra`], [`NeedElement`], [`NeedQualifier`], [`NeedBoth`]) carry no
//! runtime data; they exist solely to drive compile-time checks

#![allow(unsafe_code)]

// ----- Imports ------------
use crate::{
    Scope,
    data_types::{
        BufferFrameSizeRange, ChannelLayout, ChannelPair, DBRange, HogMode, PowerHint,
        PropertySize, SampleRateRange, StreamDescription, StreamRangedDescription, TerminalType,
        TransportType,
    },
    errors::CoreAudioError,
    object::{Device, Global, Process, Stream, System, Tap},
};
use crate::{encode::*, read::*};
use core_foundation::{base::TCFType, string::CFString};
use objc2_core_audio::{
    AudioObjectID, AudioObjectPropertyAddress, AudioObjectPropertyScope,
    AudioObjectPropertySelector, kAudioDeviceProcessorOverload,
    kAudioDevicePropertyAvailableNominalSampleRates, kAudioDevicePropertyBufferFrameSize,
    kAudioDevicePropertyBufferFrameSizeRange, kAudioDevicePropertyChannelNominalLineLevel,
    kAudioDevicePropertyChannelNominalLineLevelNameForIDCFString,
    kAudioDevicePropertyChannelNominalLineLevels, kAudioDevicePropertyClipLight,
    kAudioDevicePropertyClockDomain, kAudioDevicePropertyClockSource,
    kAudioDevicePropertyClockSourceNameForIDCFString, kAudioDevicePropertyClockSources,
    kAudioDevicePropertyConfigurationApplication, kAudioDevicePropertyDataSource,
    kAudioDevicePropertyDataSourceNameForIDCFString, kAudioDevicePropertyDataSources,
    kAudioDevicePropertyDeviceCanBeDefaultDevice,
    kAudioDevicePropertyDeviceCanBeDefaultSystemDevice, kAudioDevicePropertyDeviceIsAlive,
    kAudioDevicePropertyDeviceIsRunning, kAudioDevicePropertyDeviceUID,
    kAudioDevicePropertyHighPassFilterSetting,
    kAudioDevicePropertyHighPassFilterSettingNameForIDCFString,
    kAudioDevicePropertyHighPassFilterSettings, kAudioDevicePropertyHogMode,
    kAudioDevicePropertyIOCycleUsage, kAudioDevicePropertyIOStoppedAbnormally,
    kAudioDevicePropertyIsHidden, kAudioDevicePropertyJackIsConnected, kAudioDevicePropertyLatency,
    kAudioDevicePropertyListenback, kAudioDevicePropertyModelUID, kAudioDevicePropertyMute,
    kAudioDevicePropertyNominalSampleRate, kAudioDevicePropertyPhantomPower,
    kAudioDevicePropertyPhaseInvert, kAudioDevicePropertyPlayThruDestination,
    kAudioDevicePropertyPlayThruDestinationNameForIDCFString,
    kAudioDevicePropertyPlayThruDestinations, kAudioDevicePropertyPreferredChannelLayout,
    kAudioDevicePropertyPreferredChannelsForStereo, kAudioDevicePropertyRelatedDevices,
    kAudioDevicePropertySafetyOffset, kAudioDevicePropertySolo, kAudioDevicePropertyStereoPan,
    kAudioDevicePropertyStereoPanChannels, kAudioDevicePropertyStreamConfiguration,
    kAudioDevicePropertyStreams, kAudioDevicePropertySubMute,
    kAudioDevicePropertySubVolumeDecibels, kAudioDevicePropertySubVolumeDecibelsToScalar,
    kAudioDevicePropertySubVolumeRangeDecibels, kAudioDevicePropertySubVolumeScalar,
    kAudioDevicePropertySubVolumeScalarToDecibels, kAudioDevicePropertyTalkback,
    kAudioDevicePropertyTransportType, kAudioDevicePropertyUsesVariableBufferFrameSizes,
    kAudioDevicePropertyVolumeDecibels, kAudioDevicePropertyVolumeDecibelsToScalar,
    kAudioDevicePropertyVolumeRangeDecibels, kAudioDevicePropertyVolumeScalar,
    kAudioDevicePropertyVolumeScalarToDecibels, kAudioHardwarePropertyBoxList,
    kAudioHardwarePropertyClockDeviceList, kAudioHardwarePropertyDefaultInputDevice,
    kAudioHardwarePropertyDefaultOutputDevice, kAudioHardwarePropertyDefaultSystemOutputDevice,
    kAudioHardwarePropertyDevices, kAudioHardwarePropertyHogModeIsAllowed,
    kAudioHardwarePropertyIsInitingOrExiting, kAudioHardwarePropertyMixStereoToMono,
    kAudioHardwarePropertyPlugInList, kAudioHardwarePropertyPowerHint,
    kAudioHardwarePropertyProcessIsAudible, kAudioHardwarePropertyProcessIsMain,
    kAudioHardwarePropertyProcessObjectList, kAudioHardwarePropertyServiceRestarted,
    kAudioHardwarePropertySleepingIsAllowed, kAudioHardwarePropertyTapList,
    kAudioHardwarePropertyTranslateBundleIDToPlugIn,
    kAudioHardwarePropertyTranslateBundleIDToTransportManager,
    kAudioHardwarePropertyTranslateUIDToBox, kAudioHardwarePropertyTranslateUIDToClockDevice,
    kAudioHardwarePropertyTranslateUIDToDevice, kAudioHardwarePropertyTransportManagerList,
    kAudioHardwarePropertyUnloadingIsAllowed, kAudioHardwarePropertyUserIDChanged,
    kAudioHardwarePropertyUserSessionIsActiveOrHeadless, kAudioObjectPropertyBaseClass,
    kAudioObjectPropertyClass, kAudioObjectPropertyCreator,
    kAudioObjectPropertyElementCategoryName, kAudioObjectPropertyElementMain,
    kAudioObjectPropertyElementName, kAudioObjectPropertyElementNumberName,
    kAudioObjectPropertyManufacturer, kAudioObjectPropertyModelName, kAudioObjectPropertyName,
    kAudioObjectPropertyOwnedObjects, kAudioObjectPropertyOwner, kAudioObjectPropertyScopeGlobal,
    kAudioObjectPropertyScopeInput, kAudioObjectPropertyScopeOutput, kAudioProcessPropertyBundleID,
    kAudioProcessPropertyDevices, kAudioProcessPropertyIsRunning,
    kAudioProcessPropertyIsRunningInput, kAudioProcessPropertyIsRunningOutput,
    kAudioProcessPropertyPID, kAudioStreamPropertyAvailablePhysicalFormats,
    kAudioStreamPropertyAvailableVirtualFormats, kAudioStreamPropertyDirection,
    kAudioStreamPropertyIsActive, kAudioStreamPropertyLatency, kAudioStreamPropertyPhysicalFormat,
    kAudioStreamPropertyStartingChannel, kAudioStreamPropertyTerminalType,
    kAudioStreamPropertyVirtualFormat, kAudioTapPropertyFormat, kAudioTapPropertyUID,
};
use objc2_core_audio_types::{AudioStreamBasicDescription, AudioValueRange};
use std::marker::PhantomData;

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

/// A typed descriptor for a single CoreAudio property
///
/// The type parameters encode everything the compiler needs to enforce correct
/// usage:
/// - `T` — the Rust type the property value deserialises to
/// - `Object` — which [`AudioObject`](crate::AudioObject) kind owns this property
/// - `Access` — [`ReadOnly`] or [`ReadWrite`]; gates `set_property`
/// - `L` — [`Listenable`] or [`Silent`]; gates `add_listener`
/// - `E` — readiness marker: [`NoExtra`] means the property is fully
///   specified; [`NeedElement`], [`NeedQualifier`], and [`NeedBoth`] indicate
///   that `.for_element()` or `.with_qualifier()` must be called first
#[derive(Clone)]
pub struct Property<T, Object, Access, L, E> {
    /// CoreAudio address (selector, scope, element)
    pub(crate) address: AudioObjectPropertyAddress,
    /// Whether the size of the property data is known or must be queried
    pub(crate) size: PropertySize,
    /// Deserialises the raw byte buffer returned by CoreAudio into `T`
    pub(crate) read: fn(&[u8]) -> Result<T, CoreAudioError>,
    /// Serialises a value of type `T` into bytes for `AudioObjectSetPropertyData`
    /// The result also owns anything those bytes point to (a string's
    /// `CFString`), so it must outlive the call
    ///
    /// Every property has one, including [`ReadOnly`] ones: the access marker
    /// gates `set_property`, but `force_set_property` can still write them
    pub(crate) encode: fn(T) -> Qualifier,
    /// Optional qualifier passed alongside the property address
    pub(crate) qualifier: Option<Qualifier>,
    _object: PhantomData<Object>,
    _access: PhantomData<Access>,
    _listenable: PhantomData<L>,
    _extra_data: PhantomData<E>,
}

impl<T, Object, Access, L, E> Property<T, Object, Access, L, E> {
    /// Constructs a `Property` from its address, size, read function, and
    /// encode function
    /// The qualifier starts as `None`
    pub(crate) const fn new(
        address: AudioObjectPropertyAddress,
        size: PropertySize,
        read: fn(&[u8]) -> Result<T, CoreAudioError>,
        encode: fn(T) -> Qualifier,
    ) -> Self {
        Self {
            address,
            size,
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

/// Qualifier data as CoreAudio takes it: the bytes, plus anything they point
/// to, kept alive for as long as the property that carries them
#[derive(Clone)]
pub struct Qualifier {
    pub(crate) bytes: Vec<u8>,
    /// A string qualifier is a `CFStringRef`: `bytes` holds the pointer, and
    /// this holds the string it points to
    _string: Option<CFString>,
}

impl Qualifier {
    /// Qualifier data that's just these bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            _string: None,
        }
    }

    /// A string qualifier: CoreAudio takes a `CFStringRef`, not the text
    pub(crate) fn from_string(value: &str) -> Self {
        let string = CFString::new(value);
        let pointer = string.as_concrete_TypeRef() as usize;
        Self {
            bytes: pointer.to_ne_bytes().to_vec(),
            _string: Some(string),
        }
    }
}

// ---- Helpers ----

/// Constructs an `AudioObjectPropertyAddress` with `mElement` set to
/// `kAudioObjectPropertyElementMain`
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
// These properties are defined on the base AudioObject type and apply to any audio object

/// The class ID of the superclass of this object's class
///
/// Used to walk the CoreAudio class hierarchy
/// For example, an `AudioDevice`'s base class is `AudioObject`
pub const OBJECT_BASE_CLASS: Property<u32, Global, ReadOnly, Listenable, NoExtra> = Property::new(
    address(
        kAudioObjectPropertyBaseClass,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Known(size_of::<u32>() as u32),
    read_u32,
    encode_u32,
);

/// The class ID identifying what kind of object this is
///
/// For example, `kAudioDeviceClassID` for a device or `kAudioStreamClassID`
/// for a stream
/// Use this alongside `OBJECT_BASE_CLASS` to inspect the full type hierarchy of
/// an object
pub const OBJECT_CLASS: Property<u32, Global, ReadOnly, Listenable, NoExtra> = Property::new(
    address(kAudioObjectPropertyClass, kAudioObjectPropertyScopeGlobal),
    PropertySize::Known(size_of::<u32>() as u32),
    read_u32,
    encode_u32,
);

/// The `AudioObjectID` of the object that owns this one
///
/// For a stream, this is the device it belongs to
/// For a device, this is the system object
/// Useful for traversing the object tree upward
pub const OBJECT_OWNER: Property<AudioObjectID, Global, ReadOnly, Listenable, NoExtra> =
    Property::new(
        address(kAudioObjectPropertyOwner, kAudioObjectPropertyScopeGlobal),
        PropertySize::Known(size_of::<AudioObjectID>() as u32),
        read_audio_object_id,
        encode_audio_object_id,
    );

/// The model name of the hardware this object represents, as provided by the manufacturer
///
/// Unlike the display name, this reflects the hardware model rather than any
/// user-assigned label — e.g. "Scarlett 2i2" rather than "My Interface"
pub const OBJECT_MODEL_NAME: Property<String, Global, ReadOnly, Silent, NoExtra> = Property::new(
    address(
        kAudioObjectPropertyModelName,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Unknown,
    read_string,
    encode_string,
);

/// The name of the company that manufactured the hardware this object represents
pub const OBJECT_MANUFACTURER: Property<String, Global, ReadOnly, Listenable, NoExtra> =
    Property::new(
        address(
            kAudioObjectPropertyManufacturer,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Unknown,
        read_string,
        encode_string,
    );

/// The display name of the specified element (channel) within the specified scope
///
/// Elements correspond to individual channels — for example, "Front Left" or
/// "Rear Right"
/// Element 0 (`kAudioObjectPropertyElementMain`) names the object as a whole
/// rather than a specific channel
pub const OBJECT_ELEMENT_NAME: Property<String, Global, ReadWrite, Listenable, NeedElement> =
    Property::new(
        address(
            kAudioObjectPropertyElementName,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Unknown,
        read_string,
        encode_string,
    );

/// The category name of the specified element, describing the type of signal it carries
///
/// Describes what role the channel plays — for example, "Headphones",
/// "Microphone", or "Line In" — rather than just its position
pub const OBJECT_ELEMENT_CATEGORY_NAME: Property<String, Global, ReadOnly, Silent, NeedElement> =
    Property::new(
        address(
            kAudioObjectPropertyElementCategoryName,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Unknown,
        read_string,
        encode_string,
    );

/// The ordinal name of the specified element — typically its channel number as a string
///
/// Where `OBJECT_ELEMENT_NAME` gives a descriptive label like "Front Left",
/// this gives a positional label like "1" or "2"
pub const OBJECT_ELEMENT_NUMBER_NAME: Property<String, Global, ReadOnly, Silent, NeedElement> =
    Property::new(
        address(
            kAudioObjectPropertyElementNumberName,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Unknown,
        read_string,
        encode_string,
    );

/// The display name of the specified input channel on a device
///
/// Devices usually name their channels per scope, so the global
/// `OBJECT_ELEMENT_NAME` often reads empty where this returns a name — e.g.
/// "Channel 1" on an iPhone, or "Mic/Line 1" on an audio interface
/// Element 0 names the device's input side as a whole
pub const DEVICE_INPUT_ELEMENT_NAME: Property<String, Device, ReadWrite, Listenable, NeedElement> =
    Property::new(
        address(
            kAudioObjectPropertyElementName,
            kAudioObjectPropertyScopeInput,
        ),
        PropertySize::Unknown,
        read_string,
        encode_string,
    );

/// The display name of the specified output channel on a device
///
/// The output-side counterpart of `DEVICE_INPUT_ELEMENT_NAME`
pub const DEVICE_OUTPUT_ELEMENT_NAME: Property<String, Device, ReadWrite, Listenable, NeedElement> =
    Property::new(
        address(
            kAudioObjectPropertyElementName,
            kAudioObjectPropertyScopeOutput,
        ),
        PropertySize::Unknown,
        read_string,
        encode_string,
    );

/// The `AudioObjectID`s of all objects directly owned by this object, filtered by class
///
/// The qualifier is a list of `AudioClassID` values (`u32`)
/// Only objects whose class matches one of the provided IDs are returned
/// Pass an empty list to return all owned objects regardless of class
pub const OBJECT_OWNED_OBJECTS: Property<
    Vec<AudioObjectID>,
    Global,
    ReadOnly,
    Listenable,
    NeedQualifier<Vec<u32>>,
> = Property::new(
    address(
        kAudioObjectPropertyOwnedObjects,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Unknown,
    read_vec_audio_object_id,
    encode_vec_u32,
);

/// The bundle ID of the HAL plug-in that created this object
///
/// For Apple built-in hardware this is the Apple HAL plug-in; for third-party
/// devices it identifies the driver
/// Useful for associating an object with its vendor's software
pub const OBJECT_CREATOR: Property<String, Global, ReadOnly, Listenable, NoExtra> = Property::new(
    address(kAudioObjectPropertyCreator, kAudioObjectPropertyScopeGlobal),
    PropertySize::Unknown,
    read_string,
    encode_string,
);

// ---- Device constants ----

/// The display name of the device, as shown in Audio MIDI Setup and system UIs
///
/// This can change if the user renames the device
/// Listen to be notified of name changes
pub const DEVICE_NAME: Property<String, Device, ReadOnly, Listenable, NoExtra> = Property::new(
    address(kAudioObjectPropertyName, kAudioObjectPropertyScopeGlobal),
    PropertySize::Unknown,
    read_string,
    encode_string,
);

/// A persistent string that uniquely identifies this specific hardware unit
///
/// Unlike the numeric `AudioObjectID` which changes across boots and
/// reconnects, this UID is stable and suitable for storing in preferences to
/// refer to a specific device later
/// It is unique per physical unit, unlike `DEVICE_MODEL_UID` which is the same
/// for all devices of the same model
pub const DEVICE_UID: Property<String, Device, ReadOnly, Listenable, NoExtra> = Property::new(
    address(
        kAudioDevicePropertyDeviceUID,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Unknown,
    read_string,
    encode_string,
);

/// Whether the device is still physically present and operational
///
/// Becomes `false` when a USB or Bluetooth device is disconnected
/// Listen to this property to detect device removal without polling
pub const DEVICE_IS_ALIVE: Property<bool, Device, ReadOnly, Listenable, NoExtra> = Property::new(
    address(
        kAudioDevicePropertyDeviceIsAlive,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Known(size_of::<u32>() as u32),
    read_bool,
    encode_bool,
);

/// Whether the device currently has any active I/O clients
///
/// `true` while any process holds an `IOProc` or is otherwise driving I/O on
/// the device
/// A device may still be alive but not running if no client has started audio
pub const DEVICE_IS_RUNNING: Property<bool, Device, ReadOnly, Listenable, NoExtra> = Property::new(
    address(
        kAudioDevicePropertyDeviceIsRunning,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Known(size_of::<u32>() as u32),
    read_bool,
    encode_bool,
);

/// The sample rate the device is currently running at, in Hz
///
/// Writing this requests a sample rate change; the actual change may happen
/// asynchronously
/// Listen to confirm when the new rate takes effect
/// The requested rate must appear in `DEVICE_AVAILABLE_SAMPLE_RATES`
pub const DEVICE_NOMINAL_SAMPLE_RATE: Property<f64, Device, ReadWrite, Listenable, NoExtra> =
    Property::new(
        address(
            kAudioDevicePropertyNominalSampleRate,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<f64>() as u32),
        read_f64,
        encode_f64,
    );

/// The set of sample rates — or continuous ranges — that the device hardware supports
///
/// Some devices expose discrete values; others expose a range with a min and
/// max
/// Use this to populate a sample rate selector and to validate a rate before
/// writing `DEVICE_NOMINAL_SAMPLE_RATE`
pub const DEVICE_AVAILABLE_SAMPLE_RATES: Property<
    Vec<SampleRateRange>,
    Device,
    ReadOnly,
    Listenable,
    NoExtra,
> = Property::new(
    address(
        kAudioDevicePropertyAvailableNominalSampleRates,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Unknown,
    read_vec_sample_rate_range,
    encode_vec_sample_rate_range,
);

/// The number of frames in each I/O cycle
///
/// Smaller values reduce latency but increase CPU overhead
/// Must fall within the bounds reported by `DEVICE_BUFFER_FRAME_SIZE_RANGE`
/// If `DEVICE_USES_VARIABLE_BUFFER_FRAME_SIZES` is set, the device ignores this
/// and delivers variable-length cycles instead
pub const DEVICE_BUFFER_FRAME_SIZE: Property<u32, Device, ReadWrite, Listenable, NoExtra> =
    Property::new(
        address(
            kAudioDevicePropertyBufferFrameSize,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_u32,
        encode_u32,
    );

/// The minimum and maximum buffer frame sizes this device supports
///
/// Use this to clamp or validate a requested buffer size before writing
/// `DEVICE_BUFFER_FRAME_SIZE`
pub const DEVICE_BUFFER_FRAME_SIZE_RANGE: Property<
    BufferFrameSizeRange,
    Device,
    ReadOnly,
    Listenable,
    NoExtra,
> = Property::new(
    address(
        kAudioDevicePropertyBufferFrameSizeRange,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Known(size_of::<AudioValueRange>() as u32),
    read_buffer_size_range,
    encode_buffer_size_range,
);

/// Frames of latency added by the device hardware on the input path
///
/// Does not include the I/O buffer size
/// Add this to `STREAM_LATENCY` and the buffer size when computing total input
/// latency
pub const DEVICE_INPUT_LATENCY: Property<u32, Device, ReadOnly, Listenable, NoExtra> =
    Property::new(
        address(kAudioDevicePropertyLatency, kAudioObjectPropertyScopeInput),
        PropertySize::Known(size_of::<u32>() as u32),
        read_u32,
        encode_u32,
    );

/// Frames of latency added by the device hardware on the output path
///
/// Does not include the I/O buffer size
/// Add this to `STREAM_LATENCY` and the buffer size when computing total output
/// latency
pub const DEVICE_OUTPUT_LATENCY: Property<u32, Device, ReadOnly, Listenable, NoExtra> =
    Property::new(
        address(kAudioDevicePropertyLatency, kAudioObjectPropertyScopeOutput),
        PropertySize::Known(size_of::<u32>() as u32),
        read_u32,
        encode_u32,
    );

/// The `AudioObjectID`s of all input streams on this device
///
/// Each stream covers one or more input channels
/// Listen to detect streams being added or removed
pub(crate) const DEVICE_INPUT_STREAMS: Property<
    Vec<AudioObjectID>,
    Device,
    ReadOnly,
    Listenable,
    NoExtra,
> = Property::new(
    address(kAudioDevicePropertyStreams, kAudioObjectPropertyScopeInput),
    PropertySize::Unknown,
    read_vec_audio_object_id,
    encode_vec_u32,
);

/// The `AudioObjectID`s of all output streams on this device
///
/// Each stream covers one or more output channels
/// Listen to detect streams being added or removed
pub(crate) const DEVICE_OUTPUT_STREAMS: Property<
    Vec<AudioObjectID>,
    Device,
    ReadOnly,
    Listenable,
    NoExtra,
> = Property::new(
    address(kAudioDevicePropertyStreams, kAudioObjectPropertyScopeOutput),
    PropertySize::Unknown,
    read_vec_audio_object_id,
    encode_vec_u32,
);

/// The PID of the process that currently holds exclusive access to this device, or `-1` if none
///
/// Write the calling process's PID to claim hog mode (exclusive access),
/// preventing all other clients from using the device
/// Write `-1` to release it
/// Listen to detect when another process takes or releases exclusive access
pub const DEVICE_HOG_MODE: Property<HogMode, Device, ReadWrite, Listenable, NoExtra> =
    Property::new(
        address(kAudioDevicePropertyHogMode, kAudioObjectPropertyScopeGlobal),
        PropertySize::Known(size_of::<i32>() as u32),
        read_hog_mode,
        encode_hog_mode,
    );

// ---- Device core constants (AudioHardwareBase.h) ----

/// Bundle ID of the application responsible for configuring this device
///
/// Typically a manufacturer-supplied control panel
/// If present, this app should be launched when the user requests device
/// settings
/// Distinct from `DEVICE_HOG_MODE` — this identifies the configuration app, not
/// the exclusive-access holder
pub const DEVICE_CONFIGURATION_APPLICATION: Property<
    String,
    Device,
    ReadOnly,
    Listenable,
    NoExtra,
> = Property::new(
    address(
        kAudioDevicePropertyConfigurationApplication,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Unknown,
    read_string,
    encode_string,
);

/// A persistent identifier shared by all hardware units of the same model
///
/// Where `DEVICE_UID` is unique per physical unit, this is the same for every
/// device of the same type
/// Useful for applying model-specific settings without targeting a particular
/// unit
pub const DEVICE_MODEL_UID: Property<String, Device, ReadOnly, Listenable, NoExtra> = Property::new(
    address(
        kAudioDevicePropertyModelUID,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Unknown,
    read_string,
    encode_string,
);

/// A four-character code describing how the device connects to the system
///
/// Matches one of the `kAudioDeviceTransportType*` constants — e.g. USB,
/// FireWire, Bluetooth, PCI, or Built-In
/// Useful for filtering devices by connection type or adjusting latency
/// expectations
pub const DEVICE_TRANSPORT_TYPE: Property<TransportType, Device, ReadOnly, Listenable, NoExtra> =
    Property::new(
        address(
            kAudioDevicePropertyTransportType,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_transport_type,
        encode_transport_type,
    );

/// `AudioObjectID`s of other devices sharing the same underlying transport or clock
///
/// For example, the separate input and output sides of a USB interface that
/// appear as two logical devices will list each other here
/// Useful for coordinating clock settings across related devices
pub const DEVICE_RELATED_DEVICES: Property<
    Vec<AudioObjectID>,
    Device,
    ReadOnly,
    Listenable,
    NoExtra,
> = Property::new(
    address(
        kAudioDevicePropertyRelatedDevices,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Unknown,
    read_vec_audio_object_id,
    encode_vec_u32,
);

/// An opaque integer grouping devices that share the same hardware clock
///
/// Two devices with the same non-zero value are phase-locked to each other
/// and can be used together without sample-rate conversion
/// A value of `0` means the device is not part of any shared clock domain
pub const DEVICE_CLOCK_DOMAIN: Property<u32, Device, ReadOnly, Listenable, NoExtra> = Property::new(
    address(
        kAudioDevicePropertyClockDomain,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Known(size_of::<u32>() as u32),
    read_u32,
    encode_u32,
);

/// Whether this device is eligible to be selected as the system default device
///
/// Some devices — such as aggregate devices or certain virtual devices — cannot
/// serve as the system default even though they are usable programmatically
pub const DEVICE_CAN_BE_DEFAULT: Property<bool, Device, ReadOnly, Silent, NoExtra> = Property::new(
    address(
        kAudioDevicePropertyDeviceCanBeDefaultDevice,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Known(size_of::<u32>() as u32),
    read_bool,
    encode_bool,
);

/// Whether this device can be selected as the system sound output device
///
/// This is a separate designation from the standard default output
/// The system sound output is used for UI alerts and notifications, and a
/// device can be eligible for one but not the other
pub const DEVICE_CAN_BE_DEFAULT_SYSTEM: Property<bool, Device, ReadOnly, Silent, NoExtra> =
    Property::new(
        address(
            kAudioDevicePropertyDeviceCanBeDefaultSystemDevice,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_bool,
        encode_bool,
    );

/// Extra frames the hardware adds as a safety margin to prevent glitches
///
/// This is a fixed hardware characteristic beyond the declared buffer size
/// Add this to the buffer size and `DEVICE_INPUT_LATENCY` or
/// `DEVICE_OUTPUT_LATENCY` when computing absolute end-to-end latency
pub const DEVICE_SAFETY_OFFSET_INPUT: Property<u32, Device, ReadOnly, Silent, NoExtra> =
    Property::new(
        address(
            kAudioDevicePropertySafetyOffset,
            kAudioObjectPropertyScopeInput,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_u32,
        encode_u32,
    );

/// Extra frames the hardware adds as a safety margin to prevent glitches
///
/// This is a fixed hardware characteristic beyond the declared buffer size
/// Add this to the buffer size and `DEVICE_INPUT_LATENCY` or
/// `DEVICE_OUTPUT_LATENCY` when computing absolute end-to-end latency
pub const DEVICE_SAFETY_OFFSET_OUTPUT: Property<u32, Device, ReadOnly, Silent, NoExtra> =
    Property::new(
        address(
            kAudioDevicePropertySafetyOffset,
            kAudioObjectPropertyScopeOutput,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_u32,
        encode_u32,
    );

/// Whether this device is intentionally hidden from user-facing device lists
///
/// Hidden devices are still fully functional but should not be presented to
/// the user in UI
/// Typically set on virtual or internal routing devices
pub const DEVICE_IS_HIDDEN: Property<bool, Device, ReadOnly, Listenable, NoExtra> = Property::new(
    address(
        kAudioDevicePropertyIsHidden,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Known(size_of::<u32>() as u32),
    read_bool,
    encode_bool,
);

/// The pair of channel indices the device prefers to use for stereo output or input
///
/// The first element is the left channel index, the second is right
/// Typically `[1, 2]` but may differ on multi-channel devices
/// Write to override the preferred stereo mapping
pub const DEVICE_PREFERRED_CHANNELS_FOR_STEREO: Property<
    ChannelPair,
    Device,
    ReadWrite,
    Silent,
    NoExtra,
> = Property::new(
    address(
        kAudioDevicePropertyPreferredChannelsForStereo,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Known(size_of::<[u32; 2]>() as u32),
    read_channel_pair,
    encode_channel_pair,
);

/// The speaker each output channel feeds, as set in Audio MIDI Setup's
/// "Configure Speakers"
///
/// Apps read this to decide which channel carries which part of a
/// multichannel mix
/// Write a layout with one [`ChannelLabel`](crate::ChannelLabel) per channel
/// (use `UNUSED` for spare channels) to change it
pub const DEVICE_PREFERRED_CHANNEL_LAYOUT_OUTPUT: Property<
    ChannelLayout,
    Device,
    ReadWrite,
    Listenable,
    NoExtra,
> = Property::new(
    address(
        kAudioDevicePropertyPreferredChannelLayout,
        kAudioObjectPropertyScopeOutput,
    ),
    PropertySize::Unknown,
    read_channel_layout,
    encode_channel_layout,
);

/// The speaker each input channel comes from
///
/// The input counterpart of [`DEVICE_PREFERRED_CHANNEL_LAYOUT_OUTPUT`]
pub const DEVICE_PREFERRED_CHANNEL_LAYOUT_INPUT: Property<
    ChannelLayout,
    Device,
    ReadWrite,
    Listenable,
    NoExtra,
> = Property::new(
    address(
        kAudioDevicePropertyPreferredChannelLayout,
        kAudioObjectPropertyScopeInput,
    ),
    PropertySize::Unknown,
    read_channel_layout,
    encode_channel_layout,
);

// ---- Device convenience constants (AudioHardware.h) ----

/// Notification property that fires when the device's I/O cycle misses its deadline
///
/// The value itself carries no meaning — listen to this property to detect CPU
/// overload events in real time
/// A sustained stream of notifications indicates the buffer size is too small
/// for the current CPU load
pub const DEVICE_PROCESSOR_OVERLOAD: Property<u32, Device, ReadOnly, Listenable, NoExtra> =
    Property::new(
        address(
            kAudioDeviceProcessorOverload,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_u32,
        encode_u32,
    );

/// Notification property that fires when the device's I/O cycle stops unexpectedly
///
/// Indicates a hardware fault, driver crash, or other abnormal termination
/// The value itself carries no meaning — listen to detect abnormal stops so
/// your app can recover or report the failure
pub const DEVICE_IO_STOPPED_ABNORMALLY: Property<u32, Device, ReadOnly, Listenable, NoExtra> =
    Property::new(
        address(
            kAudioDevicePropertyIOStoppedAbnormally,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_u32,
        encode_u32,
    );

/// The fraction of each I/O cycle, from `0.0` to `1.0`, the HAL may use for scheduling
///
/// Values below `1.0` leave headroom for other real-time work on the same
/// thread
/// Reduce this if your audio callback shares a thread with other time-critical
/// tasks
pub const DEVICE_IO_CYCLE_USAGE: Property<f32, Device, ReadWrite, Listenable, NoExtra> =
    Property::new(
        address(
            kAudioDevicePropertyIOCycleUsage,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<f32>() as u32),
        read_f32,
        encode_f32,
    );

/// Non-zero if the device delivers I/O cycles with varying frame counts
///
/// When set, `DEVICE_BUFFER_FRAME_SIZE` is ignored and the actual frame count
/// is supplied per cycle
/// Your `IOProc` must handle whatever count the hardware delivers rather than
/// assuming a fixed size
pub const DEVICE_USES_VARIABLE_BUFFER_FRAME_SIZES: Property<
    u32,
    Device,
    ReadOnly,
    Silent,
    NoExtra,
> = Property::new(
    address(
        kAudioDevicePropertyUsesVariableBufferFrameSizes,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Known(size_of::<u32>() as u32),
    read_u32,
    encode_u32,
);

/// The volume of the specified channel as a linear scalar (`0.0` = silence, `1.0` = full scale)
///
/// Linear scaling is suitable for mathematical operations but not perceptually
/// uniform — prefer `DEVICE_VOLUME_DECIBELS` for UI controls
/// Listen to detect volume changes made by other processes or system UI
pub const DEVICE_GLOBAL_VOLUME_SCALAR: Property<f32, Device, ReadWrite, Listenable, NeedElement> =
    Property::new(
        address(
            kAudioDevicePropertyVolumeScalar,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<f32>() as u32),
        read_f32,
        encode_f32,
    );

/// The volume of the specified channel as a linear scalar (`0.0` = silence, `1.0` = full scale)
/// Only affects a devices input stream
///
/// Linear scaling is suitable for mathematical operations but not perceptually
/// uniform — prefer `DEVICE_VOLUME_DECIBELS` for UI controls
/// Listen to detect volume changes made by other processes or system UI
pub const DEVICE_INPUT_VOLUME_SCALAR: Property<f32, Device, ReadWrite, Listenable, NeedElement> =
    Property::new(
        address(
            kAudioDevicePropertyVolumeScalar,
            kAudioObjectPropertyScopeInput,
        ),
        PropertySize::Known(size_of::<f32>() as u32),
        read_f32,
        encode_f32,
    );

/// The volume of the specified channel as a linear scalar (`0.0` = silence, `1.0` = full scale)
/// Only affects a devices output stream
///
/// Linear scaling is suitable for mathematical operations but not perceptually
/// uniform — prefer `DEVICE_VOLUME_DECIBELS` for UI controls
/// Listen to detect volume changes made by other processes or system UI
pub const DEVICE_OUTPUT_VOLUME_SCALAR: Property<f32, Device, ReadWrite, Listenable, NeedElement> =
    Property::new(
        address(
            kAudioDevicePropertyVolumeScalar,
            kAudioObjectPropertyScopeOutput,
        ),
        PropertySize::Known(size_of::<f32>() as u32),
        read_f32,
        encode_f32,
    );

/// The volume of the specified channel in dBFS (`0.0` = full scale, negative = attenuation)
///
/// dB steps are perceptually uniform, making this the preferred representation
/// for UI controls
/// See `DEVICE_VOLUME_RANGE_DECIBELS` for the valid range on this channel
pub const DEVICE_VOLUME_DECIBELS: Property<f32, Device, ReadWrite, Listenable, NeedElement> =
    Property::new(
        address(
            kAudioDevicePropertyVolumeDecibels,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<f32>() as u32),
        read_f32,
        encode_f32,
    );

/// The volume of the specified channel in dBFS (`0.0` = full scale, negative = attenuation)
///
/// dB steps are perceptually uniform, making this the preferred representation
/// for UI controls
/// See `DEVICE_VOLUME_RANGE_DECIBELS` for the valid range on this channel
/// 
/// Output scoped
pub const DEVICE_OUTPUT_VOLUME_DECIBELS: Property<f32, Device, ReadWrite, Listenable, NeedElement> =
    Property::new(
        address(
            kAudioDevicePropertyVolumeDecibels,
            kAudioObjectPropertyScopeOutput,
        ),
        PropertySize::Known(size_of::<f32>() as u32),
        read_f32,
        encode_f32,
    );

/// The volume of the specified channel in dBFS (`0.0` = full scale, negative = attenuation)
///
/// dB steps are perceptually uniform, making this the preferred representation
/// for UI controls
/// See `DEVICE_VOLUME_RANGE_DECIBELS` for the valid range on this channel
/// 
/// Input scoped
pub const DEVICE_INPUT_VOLUME_DECIBELS: Property<f32, Device, ReadWrite, Listenable, NeedElement> =
    Property::new(
        address(
            kAudioDevicePropertyVolumeDecibels,
            kAudioObjectPropertyScopeInput,
        ),
        PropertySize::Known(size_of::<f32>() as u32),
        read_f32,
        encode_f32,
    );

/// The minimum and maximum volume values in dB that the specified channel supports
///
/// Use this to set the bounds of a volume slider
/// The minimum is typically a large negative number representing near-silence
/// rather than true `-inf dB`
pub const DEVICE_VOLUME_RANGE_DECIBELS: Property<DBRange, Device, ReadOnly, Silent, NeedElement> =
    Property::new(
        address(
            kAudioDevicePropertyVolumeRangeDecibels,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<AudioValueRange>() as u32),
        read_db_range,
        encode_db_range,
    );

/// The minimum and maximum volume values in dB that the specified channel supports
///
/// Use this to set the bounds of a volume slider
/// The minimum is typically a large negative number representing near-silence
/// rather than true `-inf dB`
/// 
/// Output scoped
pub const DEVICE_OUTPUT_VOLUME_RANGE_DECIBELS: Property<DBRange, Device, ReadOnly, Silent, NeedElement> =
    Property::new(
        address(
            kAudioDevicePropertyVolumeRangeDecibels,
            kAudioObjectPropertyScopeOutput,
        ),
        PropertySize::Known(size_of::<AudioValueRange>() as u32),
        read_db_range,
        encode_db_range,
    );

/// The minimum and maximum volume values in dB that the specified channel supports
///
/// Use this to set the bounds of a volume slider
/// The minimum is typically a large negative number representing near-silence
/// rather than true `-inf dB`
/// 
/// Input scoped
pub const DEVICE_INPUT_VOLUME_RANGE_DECIBELS: Property<DBRange, Device, ReadOnly, Silent, NeedElement> =
    Property::new(
        address(
            kAudioDevicePropertyVolumeRangeDecibels,
            kAudioObjectPropertyScopeInput,
        ),
        PropertySize::Known(size_of::<AudioValueRange>() as u32),
        read_db_range,
        encode_db_range,
    );

/// Translates a linear scalar volume to its dB equivalent for the specified channel
///
/// The hardware applies its own non-linear mapping, so this gives a more
/// accurate conversion than a generic formula
/// Set the element to the channel and read back the converted dB value
pub const DEVICE_VOLUME_SCALAR_TO_DECIBELS: Property<f32, Device, ReadOnly, Silent, NeedElement> =
    Property::new(
        address(
            kAudioDevicePropertyVolumeScalarToDecibels,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<f32>() as u32),
        read_f32,
        encode_f32,
    );

/// Translates a linear scalar volume to its dB equivalent for the specified channel
///
/// The hardware applies its own non-linear mapping, so this gives a more
/// accurate conversion than a generic formula
/// Set the element to the channel and read back the converted dB value
/// 
/// Output scoped
pub const DEVICE_OUTPUT_VOLUME_SCALAR_TO_DECIBELS: Property<f32, Device, ReadOnly, Silent, NeedElement> =
    Property::new(
        address(
            kAudioDevicePropertyVolumeScalarToDecibels,
            kAudioObjectPropertyScopeOutput,
        ),
        PropertySize::Known(size_of::<f32>() as u32),
        read_f32,
        encode_f32,
    );

/// Translates a linear scalar volume to its dB equivalent for the specified channel
///
/// The hardware applies its own non-linear mapping, so this gives a more
/// accurate conversion than a generic formula
/// Set the element to the channel and read back the converted dB value
/// 
/// Input scoped
pub const DEVICE_INPUT_VOLUME_SCALAR_TO_DECIBELS: Property<f32, Device, ReadOnly, Silent, NeedElement> =
    Property::new(
        address(
            kAudioDevicePropertyVolumeScalarToDecibels,
            kAudioObjectPropertyScopeInput,
        ),
        PropertySize::Known(size_of::<f32>() as u32),
        read_f32,
        encode_f32,
    );

/// Translates a dB volume to its linear scalar equivalent for the specified channel
///
/// The hardware applies its own non-linear mapping, so this gives a more
/// accurate conversion than a generic formula
/// Set the element to the channel and read back the converted scalar
pub const DEVICE_VOLUME_DECIBELS_TO_SCALAR: Property<f32, Device, ReadOnly, Silent, NeedElement> =
    Property::new(
        address(
            kAudioDevicePropertyVolumeDecibelsToScalar,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<f32>() as u32),
        read_f32,
        encode_f32,
    );

/// Translates a dB volume to its linear scalar equivalent for the specified channel
///
/// The hardware applies its own non-linear mapping, so this gives a more
/// accurate conversion than a generic formula
/// Set the element to the channel and read back the converted scalar
/// 
/// Output scoped
pub const DEVICE_OUTPUT_VOLUME_DECIBELS_TO_SCALAR: Property<f32, Device, ReadOnly, Silent, NeedElement> =
    Property::new(
        address(
            kAudioDevicePropertyVolumeDecibelsToScalar,
            kAudioObjectPropertyScopeOutput,
        ),
        PropertySize::Known(size_of::<f32>() as u32),
        read_f32,
        encode_f32,
    );

/// Translates a dB volume to its linear scalar equivalent for the specified channel
///
/// The hardware applies its own non-linear mapping, so this gives a more
/// accurate conversion than a generic formula
/// Set the element to the channel and read back the converted scalar
/// 
/// Input scoped
pub const DEVICE_INPUT_VOLUME_DECIBELS_TO_SCALAR: Property<f32, Device, ReadOnly, Silent, NeedElement> =
    Property::new(
        address(
            kAudioDevicePropertyVolumeDecibelsToScalar,
            kAudioObjectPropertyScopeInput,
        ),
        PropertySize::Known(size_of::<f32>() as u32),
        read_f32,
        encode_f32,
    );

/// The stereo pan position of the specified channel (`0.0` = full left, `1.0` = full right)
///
/// The channel indices between which the signal is panned are reported by
/// `DEVICE_STEREO_PAN_CHANNELS`
/// Not all devices or channels support panning
pub const DEVICE_STEREO_PAN: Property<f32, Device, ReadWrite, Listenable, NeedElement> =
    Property::new(
        address(
            kAudioDevicePropertyStereoPan,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<f32>() as u32),
        read_f32,
        encode_f32,
    );

/// The pair of channel indices between which `DEVICE_STEREO_PAN` applies
///
/// The first element is the left channel and the second is right
/// Use these indices when displaying which channels are affected by the pan
/// control
pub const DEVICE_STEREO_PAN_CHANNELS: Property<ChannelPair, Device, ReadOnly, Silent, NeedElement> =
    Property::new(
        address(
            kAudioDevicePropertyStereoPanChannels,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<[u32; 2]>() as u32),
        read_channel_pair,
        encode_channel_pair,
    );

/// Whether the specified channel is muted
///
/// When muted the channel produces silence regardless of its volume setting
/// Listen to detect mute changes made by other processes or hardware buttons
pub const DEVICE_GLOBAL_MUTE: Property<bool, Device, ReadWrite, Listenable, NeedElement> =
    Property::new(
        address(kAudioDevicePropertyMute, kAudioObjectPropertyScopeGlobal),
        PropertySize::Known(size_of::<u32>() as u32),
        read_bool,
        encode_bool,
    );

/// Whether the specified channel is muted
///
/// When muted the channel produces silence regardless of its volume setting
/// Listen to detect mute changes made by other processes or hardware buttons
pub const DEVICE_OUTPUT_MUTE: Property<bool, Device, ReadWrite, Listenable, NeedElement> =
    Property::new(
        address(kAudioDevicePropertyMute, kAudioObjectPropertyScopeOutput),
        PropertySize::Known(size_of::<u32>() as u32),
        read_bool,
        encode_bool,
    );

/// Whether the specified channel is muted
///
/// When muted the channel produces silence regardless of its volume setting
/// Listen to detect mute changes made by other processes or hardware buttons
pub const DEVICE_INPUT_MUTE: Property<bool, Device, ReadWrite, Listenable, NeedElement> =
    Property::new(
        address(kAudioDevicePropertyMute, kAudioObjectPropertyScopeInput),
        PropertySize::Known(size_of::<u32>() as u32),
        read_bool,
        encode_bool,
    );

/// Whether the specified channel is soloed
///
/// When any channel is soloed, all non-soloed channels on the same device are
/// silenced
/// Typically used in monitoring contexts on multi-channel interfaces
pub const DEVICE_SOLO: Property<bool, Device, ReadWrite, Listenable, NeedElement> = Property::new(
    address(kAudioDevicePropertySolo, kAudioObjectPropertyScopeGlobal),
    PropertySize::Known(size_of::<u32>() as u32),
    read_bool,
    encode_bool,
);

/// Whether phantom power (+48V) is enabled on the specified input channel
///
/// Required by condenser microphones
/// Should be left off for dynamic microphones, ribbon microphones, and
/// line-level sources, as it can damage some equipment
pub const DEVICE_PHANTOM_POWER: Property<bool, Device, ReadWrite, Listenable, NeedElement> =
    Property::new(
        address(
            kAudioDevicePropertyPhantomPower,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_bool,
        encode_bool,
    );

/// Whether the polarity of the signal on the specified channel is inverted (180° phase flip)
///
/// Used to correct phase cancellation when combining signals from multiple
/// microphones placed on opposite sides of a source
pub const DEVICE_PHASE_INVERT: Property<bool, Device, ReadWrite, Listenable, NeedElement> =
    Property::new(
        address(
            kAudioDevicePropertyPhaseInvert,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_bool,
        encode_bool,
    );

/// Whether the clip indicator for the specified channel is lit
///
/// `true` means the signal has exceeded full scale at some point since the
/// last reset
/// Writing `false` clears the indicator; writing `true` has no effect on most
/// hardware
pub const DEVICE_CLIP_LIGHT: Property<bool, Device, ReadWrite, Listenable, NeedElement> =
    Property::new(
        address(
            kAudioDevicePropertyClipLight,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_bool,
        encode_bool,
    );

/// Whether the talkback path is active on the specified channel
///
/// Talkback routes a monitor mix or microphone signal back to performers
/// in a recording scenario, allowing the engineer to communicate with the
/// talent without interrupting the recording
pub const DEVICE_TALKBACK: Property<bool, Device, ReadWrite, Listenable, NeedElement> =
    Property::new(
        address(
            kAudioDevicePropertyTalkback,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_bool,
        encode_bool,
    );

/// Whether the listenback path is active on the specified channel
///
/// Listenback routes the performer's input back to the engineer's monitor
/// mix, letting the engineer hear exactly what the performer is sending
/// without switching source
pub const DEVICE_LISTENBACK: Property<bool, Device, ReadWrite, Listenable, NeedElement> =
    Property::new(
        address(
            kAudioDevicePropertyListenback,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_bool,
        encode_bool,
    );

/// Whether a physical cable is detected in the jack for the specified channel
///
/// Not all hardware supports per-jack detection — on devices that do not,
/// this may always return `true`
/// Listen to respond to plug/unplug events
pub const DEVICE_JACK_IS_CONNECTED: Property<bool, Device, ReadOnly, Listenable, NeedElement> =
    Property::new(
        address(
            kAudioDevicePropertyJackIsConnected,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_bool,
        encode_bool,
    );

/// The ID of the currently active input or output data source on the specified channel
///
/// Data sources represent physical signal paths — for example "Line In",
/// "Optical", or "Headphone"
/// Write a source ID from `DEVICE_DATA_SOURCES` to switch the active source
/// Listen for changes triggered by other processes or hardware switches
pub const DEVICE_DATA_SOURCE: Property<u32, Device, ReadWrite, Listenable, NeedElement> =
    Property::new(
        address(
            kAudioDevicePropertyDataSource,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_u32,
        encode_u32,
    );

/// All data source IDs available on the specified channel
///
/// Use `DEVICE_DATA_SOURCE_NAME` to get a display label for each ID, and
/// write one of these IDs to `DEVICE_DATA_SOURCE` to switch the active source
pub const DEVICE_DATA_SOURCES: Property<Vec<AudioObjectID>, Device, ReadOnly, Silent, NeedElement> =
    Property::new(
        address(
            kAudioDevicePropertyDataSources,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Unknown,
        read_vec_audio_object_id,
        encode_vec_u32,
    );

/// The display name of a data source, looked up by its ID
///
/// Provide the channel as the element and the source ID (`u32`) as the
/// qualifier
/// Use the IDs from `DEVICE_DATA_SOURCES` to resolve names for all available
/// sources
pub const DEVICE_DATA_SOURCE_NAME: Property<String, Device, ReadOnly, Silent, NeedBoth<u32>> =
    Property::new(
        address(
            kAudioDevicePropertyDataSourceNameForIDCFString,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Unknown,
        read_string,
        encode_string,
    );

/// The ID of the clock source the device is currently synchronised to
///
/// Examples include "Internal", "S/PDIF", or "Word Clock"
/// Write a source ID from `DEVICE_CLOCK_SOURCES` to switch the active clock
/// Listen to detect clock source changes, which also trigger a sample-rate
/// notification
pub const DEVICE_CLOCK_SOURCE: Property<u32, Device, ReadWrite, Listenable, NoExtra> =
    Property::new(
        address(
            kAudioDevicePropertyClockSource,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_u32,
        encode_u32,
    );

/// All clock source IDs available on this device
///
/// Use `DEVICE_CLOCK_SOURCE_NAME` to get a display label for each ID, and
/// write one of these IDs to `DEVICE_CLOCK_SOURCE` to switch the active clock
pub const DEVICE_CLOCK_SOURCES: Property<
    Vec<AudioObjectID>,
    Device,
    ReadOnly,
    Listenable,
    NoExtra,
> = Property::new(
    address(
        kAudioDevicePropertyClockSources,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Unknown,
    read_vec_audio_object_id,
    encode_vec_u32,
);

/// The display name of a clock source, looked up by its ID
///
/// Provide the source ID (`u32`) as the qualifier
/// Use the IDs from `DEVICE_CLOCK_SOURCES` to resolve names for all available
/// sources
pub const DEVICE_CLOCK_SOURCE_NAME: Property<String, Device, ReadOnly, Silent, NeedQualifier<u32>> =
    Property::new(
        address(
            kAudioDevicePropertyClockSourceNameForIDCFString,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Unknown,
        read_string,
        encode_string,
    );

/// The ID of the currently active play-through destination
///
/// Play-through routes the device's input signal directly to an output without
/// going through the client's audio graph
/// Write a destination ID from `DEVICE_PLAY_THRU_DESTINATIONS` to change it
pub const DEVICE_PLAY_THRU_DESTINATION: Property<u32, Device, ReadWrite, Listenable, NoExtra> =
    Property::new(
        address(
            kAudioDevicePropertyPlayThruDestination,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_u32,
        encode_u32,
    );

/// All play-through destination IDs available on this device
///
/// Use `DEVICE_PLAY_THRU_DESTINATION_NAME` to get a display label for each
/// ID, and write one to `DEVICE_PLAY_THRU_DESTINATION` to switch the active
/// destination
pub const DEVICE_PLAY_THRU_DESTINATIONS: Property<
    Vec<AudioObjectID>,
    Device,
    ReadOnly,
    Silent,
    NoExtra,
> = Property::new(
    address(
        kAudioDevicePropertyPlayThruDestinations,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Unknown,
    read_vec_audio_object_id,
    encode_vec_u32,
);

/// The display name of a play-through destination, looked up by its ID
///
/// Provide the destination ID (`u32`) as the qualifier
/// Use the IDs from `DEVICE_PLAY_THRU_DESTINATIONS` to resolve names for all
/// destinations
pub const DEVICE_PLAY_THRU_DESTINATION_NAME: Property<
    String,
    Device,
    ReadOnly,
    Silent,
    NeedQualifier<u32>,
> = Property::new(
    address(
        kAudioDevicePropertyPlayThruDestinationNameForIDCFString,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Unknown,
    read_string,
    encode_string,
);

/// The ID of the nominal line level selected for the specified channel
///
/// Sets the expected signal level for connected equipment — for example
/// "+4 dBu" for professional gear or "-10 dBV" for consumer gear
/// Write a level ID from `DEVICE_CHANNEL_NOMINAL_LINE_LEVELS` to change it
pub const DEVICE_CHANNEL_NOMINAL_LINE_LEVEL: Property<
    u32,
    Device,
    ReadWrite,
    Listenable,
    NeedElement,
> = Property::new(
    address(
        kAudioDevicePropertyChannelNominalLineLevel,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Known(size_of::<u32>() as u32),
    read_u32,
    encode_u32,
);

/// All nominal line level IDs available on the specified channel
///
/// Use `DEVICE_CHANNEL_NOMINAL_LINE_LEVEL_NAME` to get a display label for
/// each ID, and write one to `DEVICE_CHANNEL_NOMINAL_LINE_LEVEL` to change the
/// active level
pub const DEVICE_CHANNEL_NOMINAL_LINE_LEVELS: Property<
    Vec<AudioObjectID>,
    Device,
    ReadOnly,
    Silent,
    NeedElement,
> = Property::new(
    address(
        kAudioDevicePropertyChannelNominalLineLevels,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Unknown,
    read_vec_audio_object_id,
    encode_vec_u32,
);

/// The display name of a nominal line level, looked up by its ID
///
/// Provide the channel as the element and the level ID (`u32`) as the
/// qualifier
/// Use the IDs from `DEVICE_CHANNEL_NOMINAL_LINE_LEVELS`
pub const DEVICE_CHANNEL_NOMINAL_LINE_LEVEL_NAME: Property<
    String,
    Device,
    ReadOnly,
    Silent,
    NeedBoth<u32>,
> = Property::new(
    address(
        kAudioDevicePropertyChannelNominalLineLevelNameForIDCFString,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Unknown,
    read_string,
    encode_string,
);

/// The ID of the high-pass filter setting currently active on the specified channel
///
/// A high-pass filter cuts low-frequency rumble from microphone inputs
/// Write a setting ID from `DEVICE_HIGH_PASS_FILTER_SETTINGS` to change the
/// cutoff frequency or disable the filter
pub const DEVICE_HIGH_PASS_FILTER_SETTING: Property<
    u32,
    Device,
    ReadWrite,
    Listenable,
    NeedElement,
> = Property::new(
    address(
        kAudioDevicePropertyHighPassFilterSetting,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Known(size_of::<u32>() as u32),
    read_u32,
    encode_u32,
);

/// All high-pass filter setting IDs available on the specified channel
///
/// Use `DEVICE_HIGH_PASS_FILTER_SETTING_NAME` to get a display label for each
/// ID, and write one to `DEVICE_HIGH_PASS_FILTER_SETTING` to change the active
/// setting
pub const DEVICE_HIGH_PASS_FILTER_SETTINGS: Property<
    Vec<AudioObjectID>,
    Device,
    ReadOnly,
    Silent,
    NeedElement,
> = Property::new(
    address(
        kAudioDevicePropertyHighPassFilterSettings,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Unknown,
    read_vec_audio_object_id,
    encode_vec_u32,
);

/// The display name of a high-pass filter setting, looked up by its ID
///
/// Provide the channel as the element and the setting ID (`u32`) as the
/// qualifier
/// Use the IDs from `DEVICE_HIGH_PASS_FILTER_SETTINGS`
pub const DEVICE_HIGH_PASS_FILTER_SETTING_NAME: Property<
    String,
    Device,
    ReadOnly,
    Silent,
    NeedBoth<u32>,
> = Property::new(
    address(
        kAudioDevicePropertyHighPassFilterSettingNameForIDCFString,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Unknown,
    read_string,
    encode_string,
);

/// The LFE (subwoofer) channel volume as a linear scalar (`0.0` = silence, `1.0` = full scale)
///
/// This controls the dedicated low-frequency effects channel separately from
/// the main channel volumes
/// See `DEVICE_GLOBAL_VOLUME_SCALAR`, `DEVICE_INPUT_VOLUME_SCALAR` and
/// `DEVICE_OUTPUT_VOLUME_SCALAR` for the main channels
pub const DEVICE_SUB_VOLUME_SCALAR: Property<f32, Device, ReadWrite, Listenable, NeedElement> =
    Property::new(
        address(
            kAudioDevicePropertySubVolumeScalar,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<f32>() as u32),
        read_f32,
        encode_f32,
    );

/// The LFE (subwoofer) channel volume in dBFS
///
/// Controls the dedicated low-frequency effects channel
/// See `DEVICE_SUB_VOLUME_RANGE_DECIBELS` for the valid range on this device
pub const DEVICE_SUB_VOLUME_DECIBELS: Property<f32, Device, ReadWrite, Listenable, NeedElement> =
    Property::new(
        address(
            kAudioDevicePropertySubVolumeDecibels,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<f32>() as u32),
        read_f32,
        encode_f32,
    );

/// The minimum and maximum LFE channel volume values in dB that the device supports
pub const DEVICE_SUB_VOLUME_RANGE_DECIBELS: Property<
    DBRange,
    Device,
    ReadOnly,
    Silent,
    NeedElement,
> = Property::new(
    address(
        kAudioDevicePropertySubVolumeRangeDecibels,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Known(size_of::<AudioValueRange>() as u32),
    read_db_range,
    encode_db_range,
);

/// Translates a linear scalar LFE volume to its dB equivalent for the specified channel
///
/// Uses the hardware's own mapping, which may be non-linear
/// Set the element to the channel and read back the converted dB value
pub const DEVICE_SUB_VOLUME_SCALAR_TO_DECIBELS: Property<
    f32,
    Device,
    ReadOnly,
    Silent,
    NeedElement,
> = Property::new(
    address(
        kAudioDevicePropertySubVolumeScalarToDecibels,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Known(size_of::<f32>() as u32),
    read_f32,
    encode_f32,
);

/// Translates a dB LFE volume to its linear scalar equivalent for the specified channel
///
/// Uses the hardware's own mapping, which may be non-linear
/// Set the element to the channel and read back the converted scalar
pub const DEVICE_SUB_VOLUME_DECIBELS_TO_SCALAR: Property<
    f32,
    Device,
    ReadOnly,
    Silent,
    NeedElement,
> = Property::new(
    address(
        kAudioDevicePropertySubVolumeDecibelsToScalar,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Known(size_of::<f32>() as u32),
    read_f32,
    encode_f32,
);

/// Whether the LFE (subwoofer) channel is muted
///
/// Mutes only the low-frequency effects channel, leaving the main channels
/// unaffected
/// See `DEVICE_GLOBAL_MUTE`, `DEVICE_INPUT_MUTE` and `DEVICE_OUTPUT_MUTE` for
/// per-channel muting
pub const DEVICE_SUB_MUTE: Property<bool, Device, ReadWrite, Listenable, NeedElement> =
    Property::new(
        address(kAudioDevicePropertySubMute, kAudioObjectPropertyScopeGlobal),
        PropertySize::Known(size_of::<u32>() as u32),
        read_bool,
        encode_bool,
    );

/// The default stream of a device
pub const DEVICE_DEFAULT_STREAM: Property<
    StreamDescription,
    Device,
    ReadOnly,
    Listenable,
    NoExtra,
> = Property::new(
    address(
        kAudioDevicePropertyStreamConfiguration,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Known(size_of::<AudioStreamBasicDescription>() as u32),
    read_stream_description,
    encode_stream_description,
);

// ---- Stream constants ----

/// The display name of this stream
///
/// Usually reflects the stream's direction and channel layout — for example
/// "Built-in Microphone" or "Headphones"
/// Listen to be notified of name changes
pub const STREAM_NAME: Property<String, Stream, ReadOnly, Listenable, NoExtra> = Property::new(
    address(kAudioObjectPropertyName, kAudioObjectPropertyScopeGlobal),
    PropertySize::Unknown,
    read_string,
    encode_string,
);

/// Whether this stream is currently active and processing audio
///
/// A stream can exist but be inactive — for example when the device is idle or
/// when the stream has been deactivated by the driver
/// Listen to track changes in stream activity
pub const STREAM_IS_ACTIVE: Property<bool, Stream, ReadWrite, Listenable, NoExtra> = Property::new(
    address(
        kAudioStreamPropertyIsActive,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Known(size_of::<u32>() as u32),
    read_bool,
    encode_bool,
);

/// The direction of this stream: `0` for output, `1` for input
///
/// Fixed at stream creation — a stream cannot change direction at runtime
/// Use this to determine whether the stream carries data to or from the
/// hardware without having to track which device scope it came from
pub const STREAM_DIRECTION: Property<Scope, Stream, ReadOnly, Listenable, NoExtra> = Property::new(
    address(
        kAudioStreamPropertyDirection,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Known(size_of::<u32>() as u32),
    read_scope,
    encode_scope,
);

/// The audio format this stream presents to the client application
///
/// This is the format your app sends or receives — sample rate, bit depth,
/// channel count, and encoding
/// The hardware may use a different physical format internally (see
/// `STREAM_PHYSICAL_FORMAT`) and convert
/// Writing this requests a format change; listen to confirm when it takes
/// effect
pub const STREAM_VIRTUAL_FORMAT: Property<
    StreamDescription,
    Stream,
    ReadWrite,
    Listenable,
    NoExtra,
> = Property::new(
    address(
        kAudioStreamPropertyVirtualFormat,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Known(size_of::<AudioStreamBasicDescription>() as u32),
    read_stream_description,
    encode_stream_description,
);

/// The audio format the hardware actually uses on this stream
///
/// May differ from `STREAM_VIRTUAL_FORMAT` when the driver performs sample
/// rate conversion or bit-depth expansion
/// Changing the physical format affects the underlying hardware configuration
/// and may impact all clients sharing the device
pub const STREAM_PHYSICAL_FORMAT: Property<
    StreamDescription,
    Stream,
    ReadWrite,
    Listenable,
    NoExtra,
> = Property::new(
    address(
        kAudioStreamPropertyPhysicalFormat,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Known(size_of::<AudioStreamBasicDescription>() as u32),
    read_stream_description,
    encode_stream_description,
);

/// A code describing the physical endpoint this stream connects to
///
/// Matches one of the `kAudioStreamTerminalType*` constants — for example
/// `kAudioStreamTerminalTypeMicrophone`, `kAudioStreamTerminalTypeHeadphones`,
/// or `kAudioStreamTerminalTypeSpeaker`
/// Useful for choosing an appropriate icon or routing hint in UI
pub const TERMINAL_TYPE: Property<TerminalType, Stream, ReadOnly, Listenable, NoExtra> =
    Property::new(
        address(
            kAudioStreamPropertyTerminalType,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_terminal_type,
        encode_terminal_type,
    );

/// The one-based device channel number that corresponds to element 1 of this stream
///
/// Used to map stream-relative channel positions to absolute device channel
/// numbers
/// For example, if a stream starts at channel 3, its element 1 is device
/// channel 3, element 2 is device channel 4, and so on
pub const STARTING_CHANNEL: Property<u32, Stream, ReadOnly, Listenable, NoExtra> = Property::new(
    address(
        kAudioStreamPropertyStartingChannel,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Known(size_of::<u32>() as u32),
    read_u32,
    encode_u32,
);

/// All virtual formats this stream can present to clients, each paired with a sample rate range
///
/// Use this to populate a format selector for the client-facing audio format
/// Each entry combines a `StreamDescription` with the range of sample rates
/// supported at that format
/// Listen to detect format availability changes
pub const STREAM_AVAILABLE_VIRTUAL_FORMATS: Property<
    Vec<StreamRangedDescription>,
    Stream,
    ReadOnly,
    Listenable,
    NoExtra,
> = Property::new(
    address(
        kAudioStreamPropertyAvailableVirtualFormats,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Unknown,
    read_vec_stream_ranged_description,
    encode_vec_stream_ranged_description,
);

/// All physical formats the hardware natively supports, each paired with a sample rate range
///
/// Unlike the virtual formats, these reflect what the hardware DAC or ADC can
/// actually run at
/// Selecting a physical format that matches your target sample rate avoids
/// driver-level sample rate conversion
pub const STREAM_AVAILABLE_PHYSICAL_FORMATS: Property<
    Vec<StreamRangedDescription>,
    Stream,
    ReadOnly,
    Listenable,
    NoExtra,
> = Property::new(
    address(
        kAudioStreamPropertyAvailablePhysicalFormats,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Unknown,
    read_vec_stream_ranged_description,
    encode_vec_stream_ranged_description,
);

/// Frames of latency introduced by this stream, in addition to the device latency
///
/// Add this to `DEVICE_INPUT_LATENCY` or `DEVICE_OUTPUT_LATENCY` and the
/// buffer size for the total hardware latency on this audio path
/// Listen to detect latency changes when the format or clock changes
pub const STREAM_LATENCY: Property<u32, Stream, ReadOnly, Listenable, NoExtra> = Property::new(
    address(kAudioStreamPropertyLatency, kAudioObjectPropertyScopeGlobal),
    PropertySize::Known(size_of::<u32>() as u32),
    read_u32,
    encode_u32,
);

// ---- System constants ----

/// The name of the HAL system object — always "HAL"
pub const SYSTEM_NAME: Property<String, System, ReadOnly, Silent, NoExtra> = Property::new(
    address(kAudioObjectPropertyName, kAudioObjectPropertyScopeGlobal),
    PropertySize::Unknown,
    read_string,
    encode_string,
);

/// The `AudioObjectID`s of all audio devices currently known to the HAL
///
/// Includes built-in and all connected external hardware
/// Listen to detect devices being plugged in or removed — the list changes on
/// any connect/disconnect event
pub const SYSTEM_DEVICES: Property<Vec<AudioObjectID>, System, ReadOnly, Listenable, NoExtra> =
    Property::new(
        address(
            kAudioHardwarePropertyDevices,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Unknown,
        read_vec_audio_object_id,
        encode_vec_u32,
    );

/// The `AudioObjectID` of the device currently selected as the system default input
///
/// Write a device's ID to change the system default input
/// Listen to detect when the user or another process changes it — for example
/// via System Settings or the menu bar volume control
pub const SYSTEM_DEFAULT_INPUT: Property<AudioObjectID, System, ReadWrite, Listenable, NoExtra> =
    Property::new(
        address(
            kAudioHardwarePropertyDefaultInputDevice,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Unknown,
        read_audio_object_id,
        encode_audio_object_id,
    );

/// The `AudioObjectID` of the device currently selected as the system default output
///
/// Write a device's ID to change the system default output
/// Listen to detect when the user or another process changes it
pub const SYSTEM_DEFAULT_OUTPUT: Property<AudioObjectID, System, ReadWrite, Listenable, NoExtra> =
    Property::new(
        address(
            kAudioHardwarePropertyDefaultOutputDevice,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<AudioObjectID>() as u32),
        read_audio_object_id,
        encode_audio_object_id,
    );

/// The `AudioObjectID`s of all audio box objects known to the HAL
///
/// Boxes represent external hardware enclosures and may contain one or more
/// devices
/// Listen to detect boxes being connected or disconnected
pub const SYSTEM_BOX_LIST: Property<Vec<AudioObjectID>, System, ReadOnly, Listenable, NoExtra> =
    Property::new(
        address(
            kAudioHardwarePropertyBoxList,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Unknown,
        read_vec_audio_object_id,
        encode_vec_u32,
    );

/// The `AudioObjectID`s of all standalone clock devices known to the HAL
///
/// Clock devices are dedicated word-clock or sync sources that carry timing
/// but no audio
/// Listen to detect clock devices being connected or removed
pub const SYSTEM_CLOCK_DEVICE_LIST: Property<
    Vec<AudioObjectID>,
    System,
    ReadOnly,
    Listenable,
    NoExtra,
> = Property::new(
    address(
        kAudioHardwarePropertyClockDeviceList,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Unknown,
    read_vec_audio_object_id,
    encode_vec_u32,
);

/// Whether the HAL daemon is currently starting up or shutting down
///
/// Attempting to use audio during this window may fail
/// Check this if your app initialises audio at launch and needs to handle the
/// case where the HAL is not yet ready
pub const SYSTEM_IS_INITING_OR_EXITING: Property<bool, System, ReadOnly, Silent, NoExtra> =
    Property::new(
        address(
            kAudioHardwarePropertyIsInitingOrExiting,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_bool,
        encode_bool,
    );

/// Whether the HAL will allow the system to sleep while audio I/O is active
///
/// Write `false` to hold a sleep assertion during low-latency recording or
/// playback
/// Remember to restore it to `true` when audio stops, or the system will never
/// sleep while your process is running
pub const SYSTEM_SLEEPING_IS_ALLOWED: Property<bool, System, ReadWrite, Listenable, NoExtra> =
    Property::new(
        address(
            kAudioHardwarePropertySleepingIsAllowed,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_bool,
        encode_bool,
    );

/// The `AudioObjectID`s of all HAL plug-ins currently loaded
///
/// Plug-ins provide the driver layer between the HAL and hardware
/// Listen to detect plug-ins being loaded or unloaded, which typically happens
/// when a third-party audio device's driver is installed or removed
pub const SYSTEM_PLUGIN_LIST: Property<Vec<AudioObjectID>, System, ReadOnly, Listenable, NoExtra> =
    Property::new(
        address(
            kAudioHardwarePropertyPlugInList,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Unknown,
        read_vec_audio_object_id,
        encode_vec_u32,
    );

/// A hint to the HAL about the system's current power mode
///
/// Writing `kAudioHardwarePowerHintFavorSavingPower` allows the HAL to make
/// power-optimised scheduling decisions
/// Writing `kAudioHardwarePowerHintNone` restores normal behaviour
pub const SYSTEM_POWER_HINT: Property<PowerHint, System, ReadWrite, Silent, NoExtra> =
    Property::new(
        address(
            kAudioHardwarePropertyPowerHint,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_power_hint,
        encode_power_hint,
    );

/// The `AudioObjectID`s of all audio tap objects currently known to the HAL
///
/// Taps intercept audio streams for monitoring, recording, or processing
/// without requiring exclusive device access
/// Listen to detect taps being created or destroyed
pub const SYSTEM_TAP_LIST: Property<Vec<AudioObjectID>, System, ReadOnly, Listenable, NoExtra> =
    Property::new(
        address(
            kAudioHardwarePropertyTapList,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Unknown,
        read_vec_audio_object_id,
        encode_vec_u32,
    );

/// The `AudioObjectID` of the device used for system alert and UI sounds
///
/// May be different from the default output device
/// Write a device's ID to redirect system sounds to a specific output
/// Listen to detect when the user changes it in System Settings
pub const SYSTEM_DEFAULT_SYSTEM_OUTPUT: Property<
    AudioObjectID,
    System,
    ReadWrite,
    Listenable,
    NoExtra,
> = Property::new(
    address(
        kAudioHardwarePropertyDefaultSystemOutputDevice,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Unknown,
    read_audio_object_id,
    encode_audio_object_id,
);

/// Translates a device UID string to the `AudioObjectID` of the matching device
///
/// Provide the UID string as the qualifier
/// Returns `kAudioObjectUnknown` if no currently connected device has that UID
/// Use this to restore a saved device reference across sessions
pub const SYSTEM_TRANSLATE_UID_TO_DEVICE: Property<
    AudioObjectID,
    System,
    ReadOnly,
    Silent,
    NeedQualifier<String>,
> = Property::new(
    address(
        kAudioHardwarePropertyTranslateUIDToDevice,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Known(size_of::<AudioObjectID>() as u32),
    read_audio_object_id,
    encode_audio_object_id,
);

/// Translates a box UID string to the `AudioObjectID` of the matching box
///
/// Provide the UID string as the qualifier
/// Returns `kAudioObjectUnknown` if no connected box has that UID
pub const SYSTEM_TRANSLATE_UID_TO_BOX: Property<
    AudioObjectID,
    System,
    ReadOnly,
    Silent,
    NeedQualifier<String>,
> = Property::new(
    address(
        kAudioHardwarePropertyTranslateUIDToBox,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Known(size_of::<AudioObjectID>() as u32),
    read_audio_object_id,
    encode_audio_object_id,
);

/// Translates a clock device UID string to the `AudioObjectID` of the matching clock device
///
/// Provide the UID string as the qualifier
/// Returns `kAudioObjectUnknown` if no connected clock device has that UID
pub const SYSTEM_TRANSLATE_UID_TO_CLOCK_DEVICE: Property<
    AudioObjectID,
    System,
    ReadOnly,
    Silent,
    NeedQualifier<String>,
> = Property::new(
    address(
        kAudioHardwarePropertyTranslateUIDToClockDevice,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Known(size_of::<AudioObjectID>() as u32),
    read_audio_object_id,
    encode_audio_object_id,
);

/// Translates a HAL plug-in bundle ID to the `AudioObjectID` of the loaded plug-in
///
/// Provide the bundle ID string as the qualifier
/// Returns `kAudioObjectUnknown` if no plug-in with that bundle ID is currently
/// loaded
pub const SYSTEM_TRANSLATE_BUNDLE_ID_TO_PLUGIN: Property<
    AudioObjectID,
    System,
    ReadOnly,
    Silent,
    NeedQualifier<String>,
> = Property::new(
    address(
        kAudioHardwarePropertyTranslateBundleIDToPlugIn,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Known(size_of::<AudioObjectID>() as u32),
    read_audio_object_id,
    encode_audio_object_id,
);

/// Translates a transport manager bundle ID to the `AudioObjectID` of the matching manager
///
/// Provide the bundle ID string as the qualifier
/// Returns `kAudioObjectUnknown` if no transport manager with that bundle ID is
/// currently loaded
pub const SYSTEM_TRANSLATE_BUNDLE_ID_TO_TRANSPORT_MANAGER: Property<
    AudioObjectID,
    System,
    ReadOnly,
    Silent,
    NeedQualifier<String>,
> = Property::new(
    address(
        kAudioHardwarePropertyTranslateBundleIDToTransportManager,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Known(size_of::<AudioObjectID>() as u32),
    read_audio_object_id,
    encode_audio_object_id,
);

/// The `AudioObjectID`s of all transport manager objects known to the HAL
///
/// Transport managers handle discovery and configuration of audio devices over
/// a particular protocol — for example, the AVB or Thunderbolt transport
/// managers
pub const SYSTEM_TRANSPORT_MANAGER_LIST: Property<
    Vec<AudioObjectID>,
    System,
    ReadOnly,
    Silent,
    NoExtra,
> = Property::new(
    address(
        kAudioHardwarePropertyTransportManagerList,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Unknown,
    read_vec_audio_object_id,
    encode_vec_u32,
);

/// Whether the HAL is currently downmixing stereo output to mono
///
/// Useful for accessibility scenarios or devices with a single speaker
/// Note that this is a global system setting and affects all audio output, not
/// just your process
pub const SYSTEM_MIX_STEREO_TO_MONO: Property<bool, System, ReadWrite, Silent, NoExtra> =
    Property::new(
        address(
            kAudioHardwarePropertyMixStereoToMono,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_bool,
        encode_bool,
    );

/// Whether this process has elevated privileges over the HAL
///
/// The HAL master can perform configuration operations that other processes
/// cannot
/// Deprecated in macOS 12 in favour of `kAudioHardwarePropertyProcessIsMain`
pub const SYSTEM_PROCESS_IS_MASTER: Property<bool, System, ReadOnly, Silent, NoExtra> =
    Property::new(
        address(
            kAudioHardwarePropertyProcessIsMain,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_bool,
        encode_bool,
    );

/// Write-only property that broadcasts a user-identity-changed notification system-wide
///
/// Writing any value signals to all HAL clients that the current user has
/// changed — for example after fast-user switching — allowing them to refresh
/// any user-specific audio state
/// Listen to receive these notifications from other processes
pub const SYSTEM_USER_ID_CHANGED: Property<u32, System, ReadWrite, Listenable, NoExtra> =
    Property::new(
        address(
            kAudioHardwarePropertyUserIDChanged,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_u32,
        encode_u32,
    );

/// Whether audio from this process is currently audible at the system level
///
/// `false` means this process is muted system-wide, independently of any
/// per-device volume or mute settings
/// Another process (or the system itself) can mute this process
/// Listen to detect when the system mutes or unmutes your process
pub const SYSTEM_PROCESS_IS_AUDIBLE: Property<bool, System, ReadWrite, Listenable, NoExtra> =
    Property::new(
        address(
            kAudioHardwarePropertyProcessIsAudible,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_bool,
        encode_bool,
    );

/// Whether the HAL daemon may unload itself when the last client disconnects
///
/// Write `false` if your process needs the HAL to persist between I/O sessions
/// — for example to preserve listener registrations or shared state
/// Restore to `true` when no longer needed
pub const SYSTEM_UNLOADING_IS_ALLOWED: Property<bool, System, ReadWrite, Silent, NoExtra> =
    Property::new(
        address(
            kAudioHardwarePropertyUnloadingIsAllowed,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_bool,
        encode_bool,
    );

/// Whether any process is allowed to take exclusive (hog mode) access to a device
///
/// Write `false` to prevent any process from claiming hog mode system-wide —
/// useful in environments where shared audio access must be enforced
/// Note that this requires appropriate privileges to set
pub const SYSTEM_HOG_MODE_IS_ALLOWED: Property<bool, System, ReadWrite, Silent, NoExtra> =
    Property::new(
        address(
            kAudioHardwarePropertyHogModeIsAllowed,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_bool,
        encode_bool,
    );

/// Whether this process's user session is currently active, or the system is headless
///
/// `false` during fast-user switching when another user's session is in the
/// foreground
/// Listen to pause or resume audio when your session moves in or out of the
/// foreground — playing audio in a background session is typically disallowed
/// or undesirable
pub const SYSTEM_USER_SESSION_IS_ACTIVE_OR_HEADLESS: Property<
    bool,
    System,
    ReadOnly,
    Listenable,
    NoExtra,
> = Property::new(
    address(
        kAudioHardwarePropertyUserSessionIsActiveOrHeadless,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Known(size_of::<u32>() as u32),
    read_bool,
    encode_bool,
);

/// Notification property that fires whenever the HAL daemon restarts
///
/// The value carries no meaning
/// Listen to this property to detect daemon restarts and re-initialise any HAL
/// state that does not survive a restart — such as IOProcs, listeners, and hog
/// mode claims
pub const SYSTEM_SERVICE_RESTARTED: Property<u32, System, ReadOnly, Listenable, NoExtra> =
    Property::new(
        address(
            kAudioHardwarePropertyServiceRestarted,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_u32,
        encode_u32,
    );

// ---- Process constants ----
// Every process that has used CoreAudio has a process object (macOS 14.2+),
// listed by `SYSTEM_PROCESS_OBJECT_LIST`

/// The `AudioObjectID` of every process object: each process that has used
/// CoreAudio (macOS 14.2+)
///
/// Listen to hear processes come and go
/// Wrap the IDs with
/// [`AudioObject::<System>::processes`](crate::AudioObject::processes)
pub const SYSTEM_PROCESS_OBJECT_LIST: Property<
    Vec<AudioObjectID>,
    System,
    ReadOnly,
    Listenable,
    NoExtra,
> = Property::new(
    address(
        kAudioHardwarePropertyProcessObjectList,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Unknown,
    read_vec_audio_object_id,
    encode_vec_u32,
);

/// The PID of the process
pub const PROCESS_PID: Property<i32, Process, ReadOnly, Silent, NoExtra> = Property::new(
    address(kAudioProcessPropertyPID, kAudioObjectPropertyScopeGlobal),
    PropertySize::Known(size_of::<i32>() as u32),
    read_i32,
    encode_i32,
);

/// The bundle ID of the process
///
/// Empty for a process that isn't in a bundle, such as a command-line tool
pub const PROCESS_BUNDLE_ID: Property<String, Process, ReadOnly, Silent, NoExtra> = Property::new(
    address(
        kAudioProcessPropertyBundleID,
        kAudioObjectPropertyScopeGlobal,
    ),
    PropertySize::Unknown,
    read_string,
    encode_string,
);

/// Whether the process is running audio input or output on any device
pub const PROCESS_IS_RUNNING: Property<bool, Process, ReadOnly, Listenable, NoExtra> =
    Property::new(
        address(
            kAudioProcessPropertyIsRunning,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_bool,
        encode_bool,
    );

/// Whether the process is running audio input on any device
pub const PROCESS_IS_RUNNING_INPUT: Property<bool, Process, ReadOnly, Listenable, NoExtra> =
    Property::new(
        address(
            kAudioProcessPropertyIsRunningInput,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_bool,
        encode_bool,
    );

/// Whether the process is running audio output on any device
pub const PROCESS_IS_RUNNING_OUTPUT: Property<bool, Process, ReadOnly, Listenable, NoExtra> =
    Property::new(
        address(
            kAudioProcessPropertyIsRunningOutput,
            kAudioObjectPropertyScopeGlobal,
        ),
        PropertySize::Known(size_of::<u32>() as u32),
        read_bool,
        encode_bool,
    );

/// The `AudioObjectID`s of the devices the process is playing to
pub const PROCESS_OUTPUT_DEVICES: Property<
    Vec<AudioObjectID>,
    Process,
    ReadOnly,
    Listenable,
    NoExtra,
> = Property::new(
    address(
        kAudioProcessPropertyDevices,
        kAudioObjectPropertyScopeOutput,
    ),
    PropertySize::Unknown,
    read_vec_audio_object_id,
    encode_vec_u32,
);

/// The `AudioObjectID`s of the devices the process is recording from
pub const PROCESS_INPUT_DEVICES: Property<
    Vec<AudioObjectID>,
    Process,
    ReadOnly,
    Listenable,
    NoExtra,
> = Property::new(
    address(kAudioProcessPropertyDevices, kAudioObjectPropertyScopeInput),
    PropertySize::Unknown,
    read_vec_audio_object_id,
    encode_vec_u32,
);

// ---- Tap constants ----

/// The tap's UID: what an aggregate device's tap list refers to it by
pub const TAP_UID: Property<String, Tap, ReadOnly, Silent, NoExtra> = Property::new(
    address(kAudioTapPropertyUID, kAudioObjectPropertyScopeGlobal),
    PropertySize::Unknown,
    read_string,
    encode_string,
);

/// The format of the audio the tap delivers
pub const TAP_FORMAT: Property<StreamDescription, Tap, ReadOnly, Listenable, NoExtra> =
    Property::new(
        address(kAudioTapPropertyFormat, kAudioObjectPropertyScopeGlobal),
        PropertySize::Known(size_of::<AudioStreamBasicDescription>() as u32),
        read_stream_description,
        encode_stream_description,
    );
