//! Error types for CoreAudio HAL operations.
//!
//! [`CoreAudioError`] is the single error type returned throughout the crate.
//! It wraps an [`ErrorKind`] for pattern matching and an `OSStatus` code for
//! the raw value reported by CoreAudio. Internal utility traits
//! (`OSStatusCheck`, `OSStatusStringify`) allow ergonomic handling of
//! `OSStatus` return values.

#![forbid(unsafe_code)]

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

/// `OSStatus` code for an unsupported data format error (`'!dat'`).
#[allow(non_upper_case_globals)]
const kAudioHardwareUnsupportedFormatError: u32 = 0x21646174;

/// `OSStatus` code for a permissions error caused by another process holding
/// exclusive (hog mode) access to the device (`'!hog'`).
#[allow(non_upper_case_globals)]
const kAudioHardwarePermissionsError: u32 = 0x21686F67;

// ---- Enums ------------

/// Categorised reason for a [`CoreAudioError`].
///
/// Match on this to distinguish error conditions without inspecting raw
/// `OSStatus` codes. Variants beginning with a CoreAudio prefix map directly
/// to `kAudioHardware*` constants; the remaining variants describe conversion
/// failures, listener teardown, and I/O proc state errors.
#[derive(Debug, Clone, Copy)]
pub enum ErrorKind {
    // ---- CoreAudio HAL errors ------------

    /// The HAL is not running, typically because no audio devices are available.
    NotRunning,
    /// An unspecified error with no additional detail from CoreAudio.
    Unspecified,
    /// The requested property does not exist on this object.
    UnknownProperty,
    /// The buffer size provided for a property operation was incorrect.
    BadPropertySize,
    /// The operation is not permitted on this object or in this context.
    IllegalOperation,
    /// The object ID is invalid or refers to a non-existent object.
    BadObject,
    /// The device ID is invalid or refers to a non-existent device.
    BadDevice,
    /// The stream ID is invalid or refers to a non-existent stream.
    BadStream,
    /// The object does not support this operation.
    UnsupportedOperation,
    /// The device exists but is not yet ready to use; there is no way to
    /// predict when it will become ready.
    NotReady,
    /// The requested audio format is not supported by the device.
    UnsupportedFormat,
    /// The device is held exclusively by another process (hog mode).
    Permissions,
    /// An unrecognised `OSStatus` code; inspect `CoreAudioError::code` for
    /// the raw value.
    Unknown,

    // ---- Conversion errors ------------

    /// Failed to interpret a `CFStringRef` as a Rust `String`.
    CFStringConversion,
    /// Failed to convert a `u32` to a `bool`.
    BoolConversion,
    /// Failed to convert bytes to a floating-point value.
    FPConversion,
    /// Failed to convert bytes to an `i32`.
    I32Conversion,
    /// Failed to convert bytes to a `u32`.
    U32Conversion,
    /// Failed to convert a `u32` to a [`Scope`](crate::Scope).
    ScopeConversion,
    /// Failed to convert bytes to an `AudioObjectID`.
    AudioObjectIdConversion,
    /// Failed to interpret bytes as a `StreamDescription`.
    StreamDescriptionConversion,
    /// Failed to interpret bytes as an `AudioValueRange`.
    ValueRangeConversion,
    /// Failed to convert a `u32` to a [`PowerHint`](crate::data_types::PowerHint).
    PowerHintConversion,
    /// Failed to convert an `i32` to a [`HogMode`](crate::data_types::HogMode).
    HogModeConversion,

    // ---- Listener errors ------------

    /// The listener's internal channel was closed unexpectedly.
    ListenerHangUp,
    /// A timed wait on a listener expired before a change was received.
    ListenerTimeout,

    // ---- IO Proc errors ------------

    /// `play()` was called on an `IOProc` that is already running.
    AlreadyRunning,
    /// `pause()` was called on an `IOProc` that is already paused.
    AlreadyPaused,
}

// ---- Structs ------------

/// An error returned by a CoreAudio HAL operation.
///
/// Contains an [`ErrorKind`] for structured matching and the raw `OSStatus`
/// code for cases where the kind is [`ErrorKind::Unknown`] or more detail is
/// needed.
#[derive(Debug)]
pub struct CoreAudioError {
    /// Categorised error kind for pattern matching.
    kind: ErrorKind,
    /// Raw `OSStatus` code as returned by CoreAudio, or `-1` for errors
    /// constructed without a CoreAudio code.
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
    /// Constructs a `CoreAudioError` from an `ErrorKind` without a CoreAudio
    /// `OSStatus` code. The `code` field is set to `-1`.
    pub(crate) fn from_error_kind(kind: ErrorKind) -> Self {
        Self {
            kind,
            code: -1,
        }
    }

    /// Returns the categorised [`ErrorKind`].
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// Returns the raw `OSStatus` code as an `i32`.
    pub fn code(&self) -> i32 {
        self.code
    }

    /// Returns the `ErrorKind` and raw `OSStatus` as a tuple `(kind, code)`.
    pub fn as_tuple(&self) -> (ErrorKind, i32) {
        (
            self.kind,
            self.code,
        )
    }

    /// Converts the raw `OSStatus` to its four-character-code string
    /// representation (e.g. `"!dat"` for an unsupported format error).
    pub fn stringify_code(&self) -> String {
        self.code.stringify_bytes()
    }
}

// ---- Traits ------------

/// Extension trait for checking an `OSStatus` return value.
pub(crate) trait OSStatusCheck {
    /// Returns `Ok(())` if the status is `0`, otherwise returns
    /// `Err(CoreAudioError)` with the status mapped to an [`ErrorKind`].
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

/// Extension trait for converting an `OSStatus` to its four-char-code string.
pub(crate) trait OSStatusStringify {
    /// Interprets the four bytes of the `OSStatus` as ASCII and returns them
    /// as a `String`, useful for human-readable error reporting.
    fn stringify_bytes(self) -> String;
}

impl OSStatusStringify for OSStatus {
    fn stringify_bytes(self) -> String {
        let bytes = &self.to_be_bytes();
        String::from_utf8_lossy(bytes).into_owned()
    }
}
