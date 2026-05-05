//! # coreaudio
//!
//! A safe, idiomatic Rust wrapper around the macOS CoreAudio Hardware Abstraction Layer (HAL).
//!
//! This crate models the CoreAudio object hierarchy — system, devices, and streams — as
//! generic [`AudioObject<T>`] types, where the type parameter determines which properties
//! and operations are available. Properties carry compile-time metadata encoding their
//! value type, access mode (read-only vs read-write), and whether they support listeners,
//! so invalid operations are caught at compile time rather than at runtime.
//!
//! ## Core concepts
//!
//! - **[`AudioObject<System>`]** is the entry point. Use it to enumerate devices or
//!   query system-wide audio state.
//! - **[`AudioObject<Device>`]** represents a single audio device. Read and write
//!   properties like sample rate and buffer size, enumerate streams, or register
//!   IO procs for audio rendering.
//! - **[`AudioObject<Stream>`]** represents an individual stream on a device. Inspect
//!   its virtual/physical format, direction, and latency.
//! - **[`Property`] constants** (e.g. [`DEVICE_NAME`], [`DEVICE_NOMINAL_SAMPLE_RATE`])
//!   are passed to `get_property`, `set_property`, and `add_listener`. The type system
//!   enforces that you can only write to read-write properties and only listen to
//!   listenable ones.
//! - **[`PropertyListener`]** watches a property for changes, offering non-blocking,
//!   blocking, and timeout-based polling.
//! - **[`IOProc`]** wraps a CoreAudio I/O procedure, letting you register an audio
//!   render callback and control playback.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use coreaudio::{AudioObject, System, Scope, DEVICE_NAME};
//!
//! let system = AudioObject::<System>::default();
//! let devices = system.devices_with_scope(Scope::Output)?;
//!
//! for device in &devices {
//!     let name: String = device.get_property(DEVICE_NAME)?;
//!     println!("{}", name);
//! }
//! # Ok::<(), coreaudio::CoreAudioError>(())
//! ```

#![cfg(target_os = "macos")]

// ---- Modules ------------
pub mod data_types;
pub mod errors;
pub mod io_proc;
pub mod listener;
pub mod object;
pub mod property;
mod traits;

// ---- Re-exports ------------

// Data types
pub use data_types::{
    AACFormat,
    BufferFrameSizeRange,
    ChannelPair,
    DBRange,
    FormatFlags,
    FormatId,
    HogMode,
    PowerHint,
    SampleEncoding,
    SampleFormat,
    SampleRateRange,
    Scope,
    StreamDescription,
    StreamRangedDescription,
    TerminalType,
    TransportType,
};

// Errors
pub use errors::{
    CoreAudioError,
    ErrorKind,
};

// Objects
pub use object::{
    AudioObject,
    Device,
    Stream,
    System,
    Global,
};

// Listener
pub use listener::PropertyListener;

// IO Proc
pub use io_proc::{
    AudioBuffer,
    IOProc,
};

// Property type and markers
pub use property::{
    Property,
    ReadOnly,
    ReadWrite,
    Listenable,
    Silent,
    NoExtra,
    NeedElement,
    NeedQualifier,
    NeedBoth,
};

// Traits
pub use traits::{
    CanListen,
    HasAllData,
    IntoQualifierBytes,
    MissingElement,
    MissingQualifier,
    ObjectCompatibleWith,
    Writeable,
};

// Property constants — AudioObject (global)
pub use property::{
    OBJECT_BASE_CLASS,
    OBJECT_CLASS,
    OBJECT_CREATOR,
    OBJECT_ELEMENT_CATEGORY_NAME,
    OBJECT_ELEMENT_NAME,
    OBJECT_ELEMENT_NUMBER_NAME,
    OBJECT_MANUFACTURER,
    OBJECT_MODEL_NAME,
    OBJECT_OWNED_OBJECTS,
    OBJECT_OWNER,
};

// Property constants — Device
pub use property::{
    DEVICE_AVAILABLE_SAMPLE_RATES,
    DEVICE_BUFFER_FRAME_SIZE,
    DEVICE_BUFFER_FRAME_SIZE_RANGE,
    DEVICE_CAN_BE_DEFAULT,
    DEVICE_CAN_BE_DEFAULT_SYSTEM,
    DEVICE_CHANNEL_NOMINAL_LINE_LEVEL,
    DEVICE_CHANNEL_NOMINAL_LINE_LEVEL_NAME,
    DEVICE_CHANNEL_NOMINAL_LINE_LEVELS,
    DEVICE_CLIP_LIGHT,
    DEVICE_CLOCK_DOMAIN,
    DEVICE_CLOCK_SOURCE,
    DEVICE_CLOCK_SOURCE_NAME,
    DEVICE_CLOCK_SOURCES,
    DEVICE_CONFIGURATION_APPLICATION,
    DEVICE_DATA_SOURCE,
    DEVICE_DATA_SOURCE_NAME,
    DEVICE_DATA_SOURCES,
    DEVICE_HIGH_PASS_FILTER_SETTING,
    DEVICE_HIGH_PASS_FILTER_SETTING_NAME,
    DEVICE_HIGH_PASS_FILTER_SETTINGS,
    DEVICE_HOG_MODE,
    DEVICE_INPUT_LATENCY,
    DEVICE_IO_CYCLE_USAGE,
    DEVICE_IO_STOPPED_ABNORMALLY,
    DEVICE_IS_ALIVE,
    DEVICE_IS_HIDDEN,
    DEVICE_IS_RUNNING,
    DEVICE_JACK_IS_CONNECTED,
    DEVICE_LISTENBACK,
    DEVICE_MODEL_UID,
    DEVICE_MUTE,
    DEVICE_NAME,
    DEVICE_NOMINAL_SAMPLE_RATE,
    DEVICE_OUTPUT_LATENCY,
    DEVICE_PHANTOM_POWER,
    DEVICE_PHASE_INVERT,
    DEVICE_PLAY_THRU_DESTINATION,
    DEVICE_PLAY_THRU_DESTINATION_NAME,
    DEVICE_PLAY_THRU_DESTINATIONS,
    DEVICE_PREFERRED_CHANNELS_FOR_STEREO,
    DEVICE_PROCESSOR_OVERLOAD,
    DEVICE_RELATED_DEVICES,
    DEVICE_SAFETY_OFFSET,
    DEVICE_SOLO,
    DEVICE_STEREO_PAN,
    DEVICE_STEREO_PAN_CHANNELS,
    DEVICE_SUB_MUTE,
    DEVICE_SUB_VOLUME_DECIBELS,
    DEVICE_SUB_VOLUME_DECIBELS_TO_SCALAR,
    DEVICE_SUB_VOLUME_RANGE_DECIBELS,
    DEVICE_SUB_VOLUME_SCALAR,
    DEVICE_SUB_VOLUME_SCALAR_TO_DECIBELS,
    DEVICE_TALKBACK,
    DEVICE_TRANSPORT_TYPE,
    DEVICE_UID,
    DEVICE_USES_VARIABLE_BUFFER_FRAME_SIZES,
    DEVICE_VOLUME_DECIBELS,
    DEVICE_VOLUME_DECIBELS_TO_SCALAR,
    DEVICE_VOLUME_RANGE_DECIBELS,
    DEVICE_VOLUME_SCALAR,
    DEVICE_VOLUME_SCALAR_TO_DECIBELS,
};

// Property constants — Stream
pub use property::{
    STARTING_CHANNEL,
    STREAM_AVAILABLE_PHYSICAL_FORMATS,
    STREAM_AVAILABLE_VIRTUAL_FORMATS,
    STREAM_DIRECTION,
    STREAM_IS_ACTIVE,
    STREAM_LATENCY,
    STREAM_NAME,
    STREAM_PHYSICAL_FORMAT,
    STREAM_VIRTUAL_FORMAT,
    TERMINAL_TYPE,
};

// Property constants — System
pub use property::{
    SYSTEM_BOX_LIST,
    SYSTEM_CLOCK_DEVICE_LIST,
    SYSTEM_DEFAULT_SYSTEM_OUTPUT,
    SYSTEM_HOG_MODE_IS_ALLOWED,
    SYSTEM_IS_INITING_OR_EXITING,
    SYSTEM_MIX_STEREO_TO_MONO,
    SYSTEM_NAME,
    SYSTEM_PLUGIN_LIST,
    SYSTEM_POWER_HINT,
    SYSTEM_PROCESS_IS_AUDIBLE,
    SYSTEM_PROCESS_IS_MASTER,
    SYSTEM_SERVICE_RESTARTED,
    SYSTEM_SLEEPING_IS_ALLOWED,
    SYSTEM_TAP_LIST,
    SYSTEM_TRANSPORT_MANAGER_LIST,
    SYSTEM_TRANSLATE_BUNDLE_ID_TO_PLUGIN,
    SYSTEM_TRANSLATE_BUNDLE_ID_TO_TRANSPORT_MANAGER,
    SYSTEM_TRANSLATE_UID_TO_BOX,
    SYSTEM_TRANSLATE_UID_TO_CLOCK_DEVICE,
    SYSTEM_TRANSLATE_UID_TO_DEVICE,
    SYSTEM_UNLOADING_IS_ALLOWED,
    SYSTEM_USER_ID_CHANGED,
    SYSTEM_USER_SESSION_IS_ACTIVE_OR_HEADLESS,
};
