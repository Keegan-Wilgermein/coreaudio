//! # Errors
//! A collection of errors that can occur along with conversions between `OSStatus`
//! and `ErrorKind`

// ---- Imports ------------
use std::{error::Error, fmt::Display};
use coreaudio_sys::{
    OSStatus,
    kAudioHardwareBadDeviceError,
    kAudioHardwareBadObjectError,
    kAudioHardwareBadPropertySizeError,
    kAudioHardwareBadStreamError,
    kAudioHardwareIllegalOperationError,
    kAudioHardwareNotReadyError,
    kAudioHardwareNotRunningError,
    kAudioHardwareUnknownPropertyError,
    kAudioHardwareUnspecifiedError,
    kAudioHardwareUnsupportedOperationError
};

// ---- Constants ------------
#[allow(non_upper_case_globals)]
/// Code for '!dat'
const kAudioHardwareUnsupportedFormatError: u32 = 0x21646174;

#[allow(non_upper_case_globals)]
/// Code for '!hog'
const kAudioHardwarePermissionsError: u32 = 0x21686F67;

// ---- Enums ------------
#[derive(Debug, Clone, Copy)]
pub enum ErrorKind {
    NotRunning,
    Unspecified,
    UnknownProperty,
    BadPropertySize,
    IllegalOperation,
    BadObject,
    BadDevice,
    BadStream,
    UnsupportedOperation,
    NotReady,
    UnsupportedFormat,
    Permissions,
    Unknown,
}

// ---- Structs ------------
#[derive(Debug)]
pub struct CoreAudioError {
    kind: ErrorKind,
    code: OSStatus,
}

impl Error for CoreAudioError {}

impl Display for CoreAudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} (code: '{}' | {})", self.kind, self.stringify_code(), self.code,
        )
    }
}

impl From<OSStatus> for CoreAudioError {
    #[allow(non_upper_case_globals)]
    fn from(value: OSStatus) -> Self {
        let kind = match value as u32 {
            kAudioHardwareNotRunningError => ErrorKind::NotRunning,
            kAudioHardwareUnspecifiedError => ErrorKind::Unspecified,
            kAudioHardwareUnknownPropertyError => ErrorKind::UnknownProperty,
            kAudioHardwareBadPropertySizeError => ErrorKind::BadPropertySize,
            kAudioHardwareIllegalOperationError => ErrorKind::IllegalOperation,
            kAudioHardwareBadObjectError => ErrorKind::BadObject,
            kAudioHardwareBadDeviceError => ErrorKind::BadDevice,
            kAudioHardwareBadStreamError => ErrorKind::BadStream,
            kAudioHardwareUnsupportedOperationError => ErrorKind::UnsupportedOperation,
            kAudioHardwareNotReadyError => ErrorKind::NotReady,
            kAudioHardwareUnsupportedFormatError => ErrorKind::UnsupportedFormat,
            kAudioHardwarePermissionsError => ErrorKind::Permissions,
            _ => ErrorKind::Unknown,
        };

        Self {
            kind,
            code: value,
        }
    }
}

impl CoreAudioError {
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    pub fn code(&self) -> i32 {
        self.code
    }

    pub fn as_tuple(&self) -> (ErrorKind, i32) {
        (
            self.kind,
            self.code,
        )
    }

    pub fn stringify_code(&self) -> String {
        self.code.stringify_bytes()
    }
}

// ---- Traits ------------
pub trait OSStatusCheck {
    fn check(self) -> Result<(), CoreAudioError>;
}

impl OSStatusCheck for OSStatus {
    fn check(self) -> Result<(), CoreAudioError> {
        match self {
            0 => Ok(()),
            _ => Err(CoreAudioError::from(self))
        }
    }
}

pub trait OSStatusStringify {
    fn stringify_bytes(self) -> String;
}

impl OSStatusStringify for OSStatus {
    fn stringify_bytes(self) -> String {
        let bytes = &self.to_be_bytes();
        String::from_utf8_lossy(bytes).into_owned()
    }
}
