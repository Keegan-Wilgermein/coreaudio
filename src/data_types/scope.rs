//! Scopes and property sizes.

use crate::errors::{CoreAudioError, ErrorKind};

/// Whether audio flows into or out of a device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// Audio captured from the outside world (microphone, line-in, etc.).
    Input,
    /// Audio sent to speakers or a line-out.
    Output,
}

impl TryFrom<u32> for Scope {
    type Error = CoreAudioError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Output),
            1 => Ok(Self::Input),
            _ => Err(CoreAudioError::from_error_kind(ErrorKind::ScopeConversion)),
        }
    }
}

/// Whether the size of the propery is known
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum PropertySize {
    /// The size of a known property
    Known(u32),
    /// The size is unknown
    /// and needs to be checked
    Unknown,
}
