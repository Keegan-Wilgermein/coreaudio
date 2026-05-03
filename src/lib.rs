//! # coreaudio-hal
//! A safe, idiomatic Rust wrapper around the CoreAudio HAL.
//!
//! ## Quick start
//! ```rust
//! use coreaudio_hal::{AudioObject, System, Scope};
//!
//! let system = AudioObject::<System>::default();
//! let devices = system.devices()?;
//!
//! for device in devices {
//!     let name = device.get_property(coreaudio_hal::DEVICE_NAME)?;
//!     println!("{}", name);
//! }
//! ```

#![cfg(target_os = "macos")]

// ---- Modules ------------
pub mod data_types;
pub mod errors;
pub mod io_proc;
pub mod listener;
pub mod object;
pub mod property;

// ---- Re-exports ------------

// Data types
pub use data_types::{
    AACFormat,
    BufferFrameSizeRange,
    FormatFlags,
    FormatId,
    SampleEncoding,
    SampleFormat,
    SampleRateRange,
    Scope,
    StreamDescription,
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
};

// Listener
pub use listener::PropertyListener;

// IO Proc
pub use io_proc::{
    AudioBuffer,
    IOProc,
};

// Property
pub use property::Property;

// Property constants
pub use property::{
    // Device
    DEVICE_NAME,
    DEVICE_UID,
    DEVICE_IS_ALIVE,
    DEVICE_IS_RUNNING,
    DEVICE_NOMINAL_SAMPLE_RATE,
    DEVICE_BUFFER_FRAME_SIZE,
    DEVICE_INPUT_LATENCY,
    DEVICE_OUTPUT_LATENCY,
    DEVICE_HOG_MODE,
    // Stream
    STREAM_NAME,
    STREAM_IS_ACTIVE,
    STREAM_DIRECTION,
    STREAM_LATENCY,
    // System
    SYSTEM_NAME,
    SYSTEM_IS_INITING_OR_EXITING,
    SYSTEM_SLEEPING_IS_ALLOWED,
    SYSTEM_POWER_HINT,
};
