//! Scopes and property sizes

use crate::errors::{CoreAudioError, ErrorKind};
use objc2_core_audio::{
    AudioObjectPropertyScope, kAudioObjectPropertyScopeGlobal, kAudioObjectPropertyScopeInput,
    kAudioObjectPropertyScopeOutput,
};

/// Whether audio flows into or out of a device
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// Global scope for property selectors
    ///
    /// Useless for anything else
    Global,
    /// Audio captured from the outside world (microphone, line-in, etc)
    Input,
    /// Audio sent to speakers or a line-out
    Output,
}

impl Scope {
    /// The `AudioObjectPropertyScope` a property address takes for this scope
    pub(crate) fn property_scope(self) -> AudioObjectPropertyScope {
        match self {
            Self::Global => kAudioObjectPropertyScopeGlobal,
            Self::Input => kAudioObjectPropertyScopeInput,
            Self::Output => kAudioObjectPropertyScopeOutput,
        }
    }
}

impl TryFrom<u32> for Scope {
    type Error = CoreAudioError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            u32::MAX => Ok(Scope::Global),
            0 => Ok(Self::Output),
            1 => Ok(Self::Input),
            _ => Err(CoreAudioError::from_error_kind(ErrorKind::ScopeConversion)),
        }
    }
}

impl From<Scope> for u32 {
    fn from(value: Scope) -> Self {
        match value {
            Scope::Global => u32::MAX,
            Scope::Output => 0,
            Scope::Input => 1,
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
