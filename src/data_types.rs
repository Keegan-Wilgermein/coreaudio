//! # Data types

#![forbid(unsafe_code)]

// ---- Imports ------------
use std::ops::RangeInclusive;
use coreaudio_sys::{
    AudioStreamBasicDescription, AudioStreamRangedDescription, AudioValueRange, kAudioFormatAC3, kAudioFormatAES3, kAudioFormatALaw, kAudioFormatAMR, kAudioFormatAMR_WB, kAudioFormatAPAC, kAudioFormatAppleLossless, kAudioFormatEnhancedAC3, kAudioFormatFlagIsBigEndian, kAudioFormatFlagIsFloat, kAudioFormatFlagIsNonInterleaved, kAudioFormatFlagIsNonMixable, kAudioFormatFlagIsPacked, kAudioFormatFlagIsSignedInteger, kAudioFormatLinearPCM, kAudioFormatMPEG4AAC, kAudioFormatMPEG4AAC_ELD, kAudioFormatMPEG4AAC_ELD_SBR, kAudioFormatMPEG4AAC_ELD_V2, kAudioFormatMPEG4AAC_HE, kAudioFormatMPEG4AAC_HE_V2, kAudioFormatMPEG4AAC_LD, kAudioFormatMPEG4AAC_Spatial, kAudioFormatMPEGLayer3, kAudioFormatOpus
};
use num_traits::AsPrimitive;
use crate::errors::{CoreAudioError, ErrorKind};

// ---- Constants ------------
const I20_MAX: i32 = 2i32.pow(20 - 1) - 1;
const I24_MAX: i32 = 2i32.pow(24 - 1) - 1;

// ---- Enums ------------
/// Input or output device selector
pub enum Scope {
    /// Input devices
    Input,
    /// Output devices
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

#[derive(Debug, Clone, Copy)]
pub enum SampleEncoding {
    Float,
    SignedInt,
    UnSignedInt,
}

#[derive(Debug, Clone, Copy)]
pub enum HogMode {
    Released,
    Owned(i32),
}

impl From<i32> for HogMode {
    fn from(value: i32) -> Self {
        match value {
            -1 => Self::Released,
            pid => Self::Owned(pid),
        }
    }
}

impl Into<i32> for HogMode {
    fn into(self) -> i32 {
        match self {
            Self::Released => -1,
            Self::Owned(pid) => pid,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PowerHint {
    None,
    PowerSaving,
}

impl TryFrom<u32> for PowerHint {
    type Error = CoreAudioError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::PowerSaving),
            _ => Err(CoreAudioError::from_error_kind(ErrorKind::PowerHintConversion)),
        }
    }
}

impl Into<u32> for PowerHint {
    fn into(self) -> u32 {
        match self {
            Self::None => 0,
            Self::PowerSaving => 1,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SampleFormat {
// Unsigned
    U8,
    U16,
    U32,
// Signed
    I8,
    I16,
    I20,
    I24,
    I32,
    I64,
// Floating point
    F32,
    F64,
}

impl SampleFormat {
    fn from(bits_per_channel: u32, encoding: SampleEncoding) -> Option<Self> {
        match (encoding, bits_per_channel) {
            (SampleEncoding::UnSignedInt, 8) => Some(SampleFormat::U8),
            (SampleEncoding::UnSignedInt, 16) => Some(SampleFormat::U16),
            (SampleEncoding::UnSignedInt, 32) => Some(SampleFormat::U32),
            (SampleEncoding::SignedInt, 8) => Some(SampleFormat::I8),
            (SampleEncoding::SignedInt, 16) => Some(SampleFormat::I16),
            (SampleEncoding::SignedInt, 20) => Some(SampleFormat::I20),
            (SampleEncoding::SignedInt, 24) => Some(SampleFormat::I24),
            (SampleEncoding::SignedInt, 32) => Some(SampleFormat::I32),
            (SampleEncoding::SignedInt, 64) => Some(SampleFormat::I64),
            (SampleEncoding::Float, 32) => Some(SampleFormat::F32),
            (SampleEncoding::Float, 64) => Some(SampleFormat::F64),
            _ => None,
        }
    }

    pub fn resample<T>(self, sample: f32) -> T
    where
        T: Copy + 'static,
        f64: AsPrimitive<T>,
    {
        let min = -1.0;
        let max = 1.0;
        let half_max: f64 = max as f64 / 2.0;
        let sample = sample.clamp(min, max) as f64;

        match self {
            SampleFormat::U8 => ((sample * half_max + half_max) * u8::MAX as f64).as_(),
            SampleFormat::U16 => ((sample * half_max + half_max) * u16::MAX as f64).as_(),
            SampleFormat::U32 => ((sample * half_max + half_max) * u32::MAX as f64).as_(),
            SampleFormat::I8 => (sample * i8::MAX as f64).as_(),
            SampleFormat::I16 => (sample * i16::MAX as f64).as_(),
            SampleFormat::I20 => (sample * I20_MAX as f64).as_(),
            SampleFormat::I24 => (sample * I24_MAX as f64).as_(),
            SampleFormat::I32 => (sample * i32::MAX as f64).as_(),
            SampleFormat::I64 => (sample * i64::MAX as f64).as_(),
            SampleFormat::F32 => sample.as_(),
            SampleFormat::F64 => sample.as_(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AACFormat {
    Standard,
    ELD,
    ELDv2,
    ELDSBR,
    HE,
    HEv2,
    LD,
    Spatial,
}

#[derive(Debug, Clone, Copy)]
pub enum FormatId {
    LinearPCM,
    AAC(AACFormat),
    ALAC,
    AC3,
    APAC,
    AES3,
    ALaw,
    AMR,
    AMRWB,
    EnhancedAC3,
    Opus,
    MP3,
    Unknown(u32),
}

impl From<u32> for FormatId {
    #[allow(non_upper_case_globals)]
    fn from(value: u32) -> Self {
        match value {
            kAudioFormatLinearPCM => Self::LinearPCM,
            kAudioFormatMPEG4AAC => Self::AAC(AACFormat::Standard),
            kAudioFormatMPEG4AAC_ELD => Self::AAC(AACFormat::ELD),
            kAudioFormatMPEG4AAC_ELD_SBR => Self::AAC(AACFormat::ELDSBR),
            kAudioFormatMPEG4AAC_ELD_V2 => Self::AAC(AACFormat::ELDv2),
            kAudioFormatMPEG4AAC_HE => Self::AAC(AACFormat::HE),
            kAudioFormatMPEG4AAC_HE_V2 => Self::AAC(AACFormat::HEv2),
            kAudioFormatMPEG4AAC_LD => Self::AAC(AACFormat::LD),
            kAudioFormatMPEG4AAC_Spatial => Self::AAC(AACFormat::Spatial),
            kAudioFormatAppleLossless => Self::ALAC,
            kAudioFormatAC3 => Self::AC3,
            kAudioFormatAPAC => Self::APAC,
            kAudioFormatAES3 => Self::AES3,
            kAudioFormatALaw => Self::ALaw,
            kAudioFormatAMR => Self::AMR,
            kAudioFormatAMR_WB => Self::AMRWB,
            kAudioFormatEnhancedAC3 => Self::EnhancedAC3,
            kAudioFormatOpus => Self::Opus,
            kAudioFormatMPEGLayer3 => Self::MP3,
            _ => Self::Unknown(value)
        }
    }
}

impl Into<u32> for FormatId {
    fn into(self) -> u32 {
        match self {
            FormatId::LinearPCM => kAudioFormatLinearPCM,
            FormatId::AAC(aacformat) => {
                match aacformat {
                    AACFormat::Standard => kAudioFormatMPEG4AAC,
                    AACFormat::ELD => kAudioFormatMPEG4AAC_ELD,
                    AACFormat::ELDv2 => kAudioFormatMPEG4AAC_ELD_V2,
                    AACFormat::ELDSBR => kAudioFormatMPEG4AAC_ELD_SBR,
                    AACFormat::HE => kAudioFormatMPEG4AAC_HE,
                    AACFormat::HEv2 => kAudioFormatMPEG4AAC_HE_V2,
                    AACFormat::LD => kAudioFormatMPEG4AAC_LD,
                    AACFormat::Spatial => kAudioFormatMPEG4AAC_Spatial,
                }
            },
            FormatId::ALAC => kAudioFormatAppleLossless,
            FormatId::AC3 => kAudioFormatAC3,
            FormatId::APAC => kAudioFormatAPAC,
            FormatId::AES3 => kAudioFormatAES3,
            FormatId::ALaw => kAudioFormatALaw,
            FormatId::AMR => kAudioFormatAMR,
            FormatId::AMRWB => kAudioFormatAMR_WB,
            FormatId::EnhancedAC3 => kAudioFormatEnhancedAC3,
            FormatId::Opus => kAudioFormatOpus,
            FormatId::MP3 => kAudioFormatMPEGLayer3,
            FormatId::Unknown(value) => value,
        }
    }
}

// ----- Structs ------------
#[derive(Debug, Clone, Copy)]
pub struct FormatFlags {
    encoding: SampleEncoding,
    is_big_endian: bool,
    is_packed: bool,
    is_interleaved: bool,
    is_non_mixable: bool,
}

impl From<u32> for FormatFlags {
    fn from(value: u32) -> Self {
        let encoding = if value & kAudioFormatFlagIsFloat != 0 {
            SampleEncoding::Float
        } else if value & kAudioFormatFlagIsSignedInteger != 0 {
            SampleEncoding::SignedInt
        } else {
            SampleEncoding::UnSignedInt
        };

        Self {
            encoding,
            is_big_endian: value & kAudioFormatFlagIsBigEndian != 0,
            is_packed: value & kAudioFormatFlagIsPacked != 0,
            is_interleaved: value & kAudioFormatFlagIsNonInterleaved == 0,
            is_non_mixable: value & kAudioFormatFlagIsNonMixable != 0,
        }
    }
}

impl Into<u32> for FormatFlags {
    fn into(self) -> u32 {
        let mut result = 0u32;
        
        match self.encoding {
            SampleEncoding::Float => result |= kAudioFormatFlagIsFloat,
            SampleEncoding::SignedInt => result |= kAudioFormatFlagIsSignedInteger,
            SampleEncoding::UnSignedInt => (),
        }
        
        if self.is_big_endian  { result |= kAudioFormatFlagIsBigEndian }
        if self.is_packed      { result |= kAudioFormatFlagIsPacked }
        if !self.is_interleaved { result |= kAudioFormatFlagIsNonInterleaved }
        if self.is_non_mixable { result |= kAudioFormatFlagIsNonMixable }
        
        result
    }
}

impl FormatFlags {
    pub fn encoding(&self) -> SampleEncoding {
        self.encoding
    }

    pub fn is_big_endian(&self) -> bool {
        self.is_big_endian
    }

    pub fn is_packed(&self) -> bool {
        self.is_packed
    }

    pub fn is_interleaved(&self) -> bool {
        self.is_interleaved
    }

    pub fn is_non_mixable(&self) -> bool {
        self.is_non_mixable
    }
}

/// Audio stream description
#[derive(Debug, Clone, Copy)]
pub struct StreamDescription {
    sample_rate: f64,
    format_id: FormatId,
    flags: FormatFlags,
    sample_format: Option<SampleFormat>,
    bytes_per_packet: u32,
    frames_per_packet: u32,
    bytes_per_frame: u32,
    channels_per_frame: u32,
    bits_per_channel: u32,
    reserved: u32,
}

impl From<AudioStreamBasicDescription> for StreamDescription {
    fn from(value: AudioStreamBasicDescription) -> Self {
        let format_id = FormatId::from(value.mFormatID);
        let flags = FormatFlags::from(value.mFormatFlags);
        let sample_format = SampleFormat::from(value.mBitsPerChannel, flags.encoding);

        Self {
            sample_rate: value.mSampleRate,
            format_id,
            flags,
            sample_format,
            bytes_per_packet: value.mBytesPerPacket,
            frames_per_packet: value.mFramesPerPacket,
            bytes_per_frame: value.mBytesPerFrame,
            channels_per_frame: value.mChannelsPerFrame,
            bits_per_channel: value.mBitsPerChannel,
            reserved: value.mReserved,
        }
    }
}

impl Into<AudioStreamBasicDescription> for StreamDescription {
    fn into(self) -> AudioStreamBasicDescription {
        AudioStreamBasicDescription {
            mSampleRate: self.sample_rate,
            mFormatID: self.format_id.into(),
            mFormatFlags: self.flags.into(),
            mBytesPerPacket: self.bytes_per_packet,
            mFramesPerPacket: self.frames_per_packet,
            mBytesPerFrame: self.bytes_per_frame,
            mChannelsPerFrame: self.channels_per_frame,
            mBitsPerChannel: self.bits_per_channel,
            mReserved: self.reserved,
        }
    }
}

impl StreamDescription {
    pub fn sample_rate(&self) -> f64 {
        self.sample_rate
    }

    pub fn format_id(&self) -> FormatId {
        self.format_id
    }

    pub fn flags(&self) -> FormatFlags {
        self.flags
    }

    pub fn sample_format(&self) -> Option<SampleFormat> {
        self.sample_format
    }

    pub fn bytes_per_packet(&self) -> u32 {
        self.bytes_per_packet
    }

    pub fn frames_per_packet(&self) -> u32 {
        self.frames_per_packet
    }

    pub fn bytes_per_frame(&self) -> u32 {
        self.bytes_per_frame
    }

    pub fn channels_per_frame(&self) -> u32 {
        self.channels_per_frame
    }
}

/// A range of sizes supported by a device to be used as a buffer size
#[derive(Debug)]
pub struct BufferFrameSizeRange {
    min: u32,
    max: u32,
}

impl From<AudioValueRange> for BufferFrameSizeRange {
    fn from(value: AudioValueRange) -> Self {
        Self {
            min: value.mMinimum as u32,
            max: value.mMaximum as u32,
        }
    }
}

impl BufferFrameSizeRange {
    /// Returns all values between `min` and `max` that are a power of 2
    pub fn valid_sizes(&self) -> Vec<u32> {
        let mut size = self.min.next_power_of_two();
        let mut sizes = Vec::new();
        while size <= self.max {
            sizes.push(size);
            size *= 2;
        }
        sizes
    }
}

#[derive(Debug)]
pub struct StreamRangedDescription {
    stream_description: StreamDescription,
    sample_rate_range: SampleRateRange,
}

impl From<AudioStreamRangedDescription> for StreamRangedDescription {
    fn from(value: AudioStreamRangedDescription) -> Self {
        Self {
            stream_description: value.mFormat.into(),
            sample_rate_range: value.mSampleRateRange.into(),
        }
    }
}

impl StreamRangedDescription {
    pub fn stream_description(&self) -> StreamDescription {
        self.stream_description
    }

    pub fn sample_rate_range(&self) -> SampleRateRange {
        self.sample_rate_range
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SampleRateRange {
    min: f64,
    max: f64,
}

impl From<AudioValueRange> for SampleRateRange {
    fn from(value: AudioValueRange) -> Self {
        Self {
            min: value.mMinimum,
            max: value.mMaximum,
        }
    }
}

impl SampleRateRange {
    pub fn min(&self) -> f64 {
        self.min
    }

    pub fn max(&self) -> f64 {
        self.max
    }

    pub fn as_range(&self) -> RangeInclusive<f64> {
        self.min..=self.max
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DBRange {
    min: f64,
    max: f64,
}

impl From<AudioValueRange> for DBRange {
    fn from(value: AudioValueRange) -> Self {
        Self {
            min: value.mMinimum,
            max: value.mMaximum,
        }
    }
}

impl DBRange {
    pub fn min(&self) -> f64 {
        self.min
    }

    pub fn max(&self) -> f64 {
        self.max
    }

    pub fn as_range(&self) -> RangeInclusive<f64> {
        self.min..=self.max
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ChannelPair {
    left: u32,
    right: u32,
}

impl From<[u32; 2]> for ChannelPair {
    fn from(value: [u32; 2]) -> Self {
        Self {
            left: value[0],
            right: value[1],
        }
    }
}

impl ChannelPair {
    pub fn left(&self) -> u32 {
        self.left
    }

    pub fn right(&self) -> u32 {
        self.right
    }

    pub fn as_array(&self) -> [u32; 2] {
        [self.left, self.right]
    }
}
