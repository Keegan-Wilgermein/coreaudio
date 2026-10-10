//! Audio data types used throughout the crate.
//!
//! This module defines enums and structs that map CoreAudio's C types to
//! idiomatic Rust equivalents. Most types implement `From`/`Into` conversions
//! to and from their underlying C representations.

#![forbid(unsafe_code)]

mod channel_layout;
mod device;
mod ranges;
mod scope;
mod stream_format;

pub use channel_layout::*;
pub use device::*;
pub use ranges::*;
pub use scope::*;
pub use stream_format::*;
