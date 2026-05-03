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
/// Identifier for an unsupported format error from coreaudio
/// 
/// Code for '!dat'
const kAudioHardwareUnsupportedFormatError: u32 = 0x21646174;

#[allow(non_upper_case_globals)]
/// Identifier for insufficient permissions to access
/// a device - such as if another process is hogging it
/// 
/// Code for '!hog'
const kAudioHardwarePermissionsError: u32 = 0x21686F67;

// ---- Enums ------------
#[derive(Debug, Clone, Copy)]
/// Wrappers for codes returned by coreaudio
pub enum ErrorKind {
    // ---- CoreAudioErrors ------------
    /// HAL not running -
    /// typically due to there being no active devices avaliable
    NotRunning,
    /// Something ambiguous with no extra detail
    Unspecified,
    /// The requested property doesn't exist for this object
    UnknownProperty,
    /// Incorrect buffer size provided for property
    BadPropertySize,
    /// Operation not permitted on this object
    /// or in this scenario
    IllegalOperation,
    /// Invalid object ID
    BadObject,
    /// Invalid device ID
    BadDevice,
    /// Invalid stream ID
    BadStream,
    /// Object doesn't support this operation
    UnsupportedOperation,
    /// The device isn't ready to be used
    /// 
    /// There is no way to know when it will be ready
    NotReady,
    /// The requested format is not supported by the device
    UnsupportedFormat,
    /// Object can't be accessed due
    /// to another process having exclusive access
    Permissions,
    /// Unrecognised error codes
    /// 
    /// Check the `code` field in `CoreAudioError`
    /// for more info
    Unknown,

    // ---- Other errors ------------
    CFStringConversion,
    BoolConversion,
    F64Conversion,
    I32Conversion,
    U32Conversion,
    ScopeConversion,
    AudioObjectIdConversion,
    StreamDescriptionConversion,
    ValueRangeConversion,

    // ---- Listener errors ------------
    ListenerHangUp,
    ListenerTimeOut,
    
    // ---- IO Proc errors ------------
    AlreadyRunning,
    AlreadyPaused,
}

// ---- Structs ------------
#[derive(Debug)]
/// Error information
/// 
/// `kind` contains an `ErrorKind` to be matched against
/// 
/// `code` contains an `OSStatus` for the exact code
/// incase the error was unrecognised
pub struct CoreAudioError {
    /// The error kind
    kind: ErrorKind,
    /// The error code
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
    pub(crate) fn from_error_kind(kind: ErrorKind) -> Self {
        Self {
            kind,
            code: -1,
        }
    }

    /// Returns the contained `ErrorKind` value
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// Returns the contained `OSStatus` as an `i32`
    pub fn code(&self) -> i32 {
        self.code
    }

    /// Returns both the `ErrorKind` and `OSStatus` values
    pub fn as_tuple(&self) -> (ErrorKind, i32) {
        (
            self.kind,
            self.code,
        )
    }

    /// Returns the four character code that the contained `OSStatus` represents
    pub fn stringify_code(&self) -> String {
        self.code.stringify_bytes()
    }
}

// ---- Traits ------------
pub(crate) trait OSStatusCheck {
    fn check(self) -> Result<(), CoreAudioError>;
}

impl OSStatusCheck for OSStatus {
    /// Matches an `OSStatus` and returns an
    /// `OK(())` if `0`
    /// or a
    /// `Err(CoreAudioError)` if anything else
    fn check(self) -> Result<(), CoreAudioError> {
        match self {
            0 => Ok(()),
            _ => Err(CoreAudioError::from(self))
        }
    }
}

pub(crate) trait OSStatusStringify {
    fn stringify_bytes(self) -> String;
}

impl OSStatusStringify for OSStatus {
    /// Returns a `String` representation of an `OSStatus`
    fn stringify_bytes(self) -> String {
        let bytes = &self.to_be_bytes();
        String::from_utf8_lossy(bytes).into_owned()
    }
}
