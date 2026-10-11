//! Time stamps delivered with each I/O cycle
//!
//! CoreAudio describes a single moment in several clocks at once: the sample
//! clock, the host clock, a word clock and SMPTE time
//! [`Time`] holds all of them, and [`TimeFlags`] says which of them the device
//! actually filled in
//! The accessors on [`Time`] check the flags for you and return `None` for
//! any clock that isn't valid

use objc2_core_audio_types::{
    AudioTimeStamp, AudioTimeStampFlags, SMPTETime as RawSMPTETime, SMPTETimeFlags, SMPTETimeType,
};

// ---- Structs ------------

/// Rust representation of CoreAudio's `AudioTimeStamp`, one point in time
/// expressed in several clocks
///
/// Not every clock is filled in by every device
/// Use the accessors, which return `None` for a clock that
/// [`flags`](Self::flags) marks as invalid, rather than trusting a zero
///
/// The default has every flag cleared, so every accessor returns `None`
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Time {
    /// The absolute sample frame time
    /// Only meaningful if `flags.sample_time_valid`
    sample_time: f64,
    /// The host machine's clock in `mach_absolute_time` ticks
    /// Only meaningful if `flags.host_time_valid`
    host_time: u64,
    /// Actual host ticks per sample frame divided by the nominal amount
    /// Only meaningful if `flags.rate_scalar_valid`
    rate_scalar: f64,
    /// The word clock time
    /// Only meaningful if `flags.word_clock_time_valid`
    world_clock_time: u64,
    /// The SMPTE time
    /// Only meaningful if `flags.smpte_time_valid`
    smpte_time: SMPTETime,
    /// Which of the other fields hold valid data
    flags: TimeFlags,
}

impl Time {
    /// The absolute sample frame time, if the device reports one
    pub fn sample_time(&self) -> Option<f64> {
        self.flags.sample_time_valid.then_some(self.sample_time)
    }

    /// The host machine's clock (`mach_absolute_time` ticks), if reported
    pub fn host_time(&self) -> Option<u64> {
        self.flags.host_time_valid.then_some(self.host_time)
    }

    /// The ratio of actual host ticks per sample frame to the nominal host
    /// ticks per sample frame, if reported
    /// `1.0` means the device clock is running at exactly its nominal rate
    pub fn rate_scalar(&self) -> Option<f64> {
        self.flags.rate_scalar_valid.then_some(self.rate_scalar)
    }

    /// The word clock time, if reported
    pub fn world_clock_time(&self) -> Option<u64> {
        self.flags
            .word_clock_time_valid
            .then_some(self.world_clock_time)
    }

    /// The SMPTE time, if reported
    pub fn smpte_time(&self) -> Option<SMPTETime> {
        self.flags.smpte_time_valid.then_some(self.smpte_time)
    }

    /// Which of the clocks in this time stamp are valid
    pub fn flags(&self) -> TimeFlags {
        self.flags
    }
}

impl From<AudioTimeStamp> for Time {
    fn from(value: AudioTimeStamp) -> Self {
        Self {
            sample_time: value.mSampleTime,
            host_time: value.mHostTime,
            rate_scalar: value.mRateScalar,
            world_clock_time: value.mWordClockTime,
            smpte_time: value.mSMPTETime.into(),
            flags: value.mFlags.into(),
        }
    }
}

impl From<&AudioTimeStamp> for Time {
    fn from(value: &AudioTimeStamp) -> Self {
        (*value).into()
    }
}

/// A SMPTE time, used to align audio with video
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SMPTETime {
    /// The number of subframes in the full message
    pub sub_frames: i16,
    /// The number of subframes per frame (typically 80)
    pub sub_frame_divisor: i16,
    /// The total number of messages received
    pub counter: u32,
    /// The frame rate this time is counted in
    pub kind: SMPTETimeKind,
    /// Whether this time is valid and whether it's running
    pub flags: SMPTETimeStatus,
    /// The hours field of the time
    pub hours: i16,
    /// The minutes field of the time
    pub minutes: i16,
    /// The seconds field of the time
    pub seconds: i16,
    /// The frames field of the time
    pub frames: i16,
}

impl From<RawSMPTETime> for SMPTETime {
    fn from(value: RawSMPTETime) -> Self {
        Self {
            sub_frames: value.mSubframes,
            sub_frame_divisor: value.mSubframeDivisor,
            counter: value.mCounter,
            kind: value.mType.into(),
            flags: value.mFlags.into(),
            hours: value.mHours,
            minutes: value.mMinutes,
            seconds: value.mSeconds,
            frames: value.mFrames,
        }
    }
}

/// The flags which indicate which fields are valid for this callback
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TimeFlags {
    /// The sample frame time is valid
    pub sample_time_valid: bool,
    /// The host time is valid
    pub host_time_valid: bool,
    /// The rate scalar is valid
    pub rate_scalar_valid: bool,
    /// The word clock time is valid
    pub word_clock_time_valid: bool,
    /// The SMPTE time is valid
    pub smpte_time_valid: bool,
}

impl From<AudioTimeStampFlags> for TimeFlags {
    fn from(value: AudioTimeStampFlags) -> Self {
        Self {
            sample_time_valid: value.contains(AudioTimeStampFlags::SampleTimeValid),
            host_time_valid: value.contains(AudioTimeStampFlags::HostTimeValid),
            rate_scalar_valid: value.contains(AudioTimeStampFlags::RateScalarValid),
            word_clock_time_valid: value.contains(AudioTimeStampFlags::WordClockTimeValid),
            smpte_time_valid: value.contains(AudioTimeStampFlags::SMPTETimeValid),
        }
    }
}

/// Whether a [`SMPTETime`] is valid and whether its clock is running
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SMPTETimeStatus {
    /// The time holds valid data
    pub valid: bool,
    /// The time is advancing
    pub running: bool,
}

impl From<SMPTETimeFlags> for SMPTETimeStatus {
    fn from(value: SMPTETimeFlags) -> Self {
        Self {
            valid: value.contains(SMPTETimeFlags::Valid),
            running: value.contains(SMPTETimeFlags::Running),
        }
    }
}

// ---- Enums ------------

/// The frame rate a [`SMPTETime`] is counted in
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SMPTETimeKind {
    /// 23.98 frames per second
    Fps2398,
    /// 24 frames per second
    #[default]
    Fps24,
    /// 25 frames per second
    Fps25,
    /// 30 frames per second, drop frame
    Fps30Drop,
    /// 30 frames per second
    Fps30,
    /// 29.97 frames per second
    Fps2997,
    /// 29.97 frames per second, drop frame
    Fps2997Drop,
    /// 60 frames per second
    Fps60,
    /// 59.94 frames per second
    Fps5994,
    /// 60 frames per second, drop frame
    Fps60Drop,
    /// 59.94 frames per second, drop frame
    Fps5994Drop,
    /// 50 frames per second
    Fps50,
    /// A frame rate this crate doesn't know
    /// Holds the raw CoreAudio value
    Unknown(u32),
}

impl From<SMPTETimeType> for SMPTETimeKind {
    fn from(value: SMPTETimeType) -> Self {
        match value {
            SMPTETimeType::Type2398 => Self::Fps2398,
            SMPTETimeType::Type24 => Self::Fps24,
            SMPTETimeType::Type25 => Self::Fps25,
            SMPTETimeType::Type30Drop => Self::Fps30Drop,
            SMPTETimeType::Type30 => Self::Fps30,
            SMPTETimeType::Type2997 => Self::Fps2997,
            SMPTETimeType::Type2997Drop => Self::Fps2997Drop,
            SMPTETimeType::Type60 => Self::Fps60,
            SMPTETimeType::Type5994 => Self::Fps5994,
            SMPTETimeType::Type60Drop => Self::Fps60Drop,
            SMPTETimeType::Type5994Drop => Self::Fps5994Drop,
            SMPTETimeType::Type50 => Self::Fps50,
            SMPTETimeType(other) => Self::Unknown(other),
        }
    }
}
