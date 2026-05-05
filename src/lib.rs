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
    Global,
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
