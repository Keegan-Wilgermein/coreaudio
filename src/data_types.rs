//! Audio data types used throughout the crate.
//!
//! This module defines enums and structs that map CoreAudio's C types to
//! idiomatic Rust equivalents. Most types implement `From`/`Into` conversions
//! to and from their underlying C representations.

#![forbid(unsafe_code)]

// ---- Imports ------------
use std::ops::RangeInclusive;
use coreaudio_sys::{
    AudioStreamBasicDescription, AudioStreamRangedDescription, AudioValueRange, kAudioDeviceTransportTypeAVB, kAudioDeviceTransportTypeAggregate, kAudioDeviceTransportTypeAirPlay, kAudioDeviceTransportTypeAutoAggregate, kAudioDeviceTransportTypeBluetooth, kAudioDeviceTransportTypeBluetoothLE, kAudioDeviceTransportTypeBuiltIn, kAudioDeviceTransportTypeContinuityCapture, kAudioDeviceTransportTypeContinuityCaptureWired, kAudioDeviceTransportTypeContinuityCaptureWireless, kAudioDeviceTransportTypeDisplayPort, kAudioDeviceTransportTypeFireWire, kAudioDeviceTransportTypeHDMI, kAudioDeviceTransportTypePCI, kAudioDeviceTransportTypeThunderbolt, kAudioDeviceTransportTypeUSB, kAudioDeviceTransportTypeVirtual, kAudioFormatAC3, kAudioFormatAES3, kAudioFormatALaw, kAudioFormatAMR, kAudioFormatAMR_WB, kAudioFormatAPAC, kAudioFormatAppleLossless, kAudioFormatEnhancedAC3, kAudioFormatFlagIsBigEndian, kAudioFormatFlagIsFloat, kAudioFormatFlagIsNonInterleaved, kAudioFormatFlagIsNonMixable, kAudioFormatFlagIsPacked, kAudioFormatFlagIsSignedInteger, kAudioFormatLinearPCM, kAudioFormatMPEG4AAC, kAudioFormatMPEG4AAC_ELD, kAudioFormatMPEG4AAC_ELD_SBR, kAudioFormatMPEG4AAC_ELD_V2, kAudioFormatMPEG4AAC_HE, kAudioFormatMPEG4AAC_HE_V2, kAudioFormatMPEG4AAC_LD, kAudioFormatMPEG4AAC_Spatial, kAudioFormatMPEGLayer3, kAudioFormatOpus, kAudioStreamTerminalTypeDigitalAudioInterface, kAudioStreamTerminalTypeDisplayPort, kAudioStreamTerminalTypeHDMI, kAudioStreamTerminalTypeHeadphones, kAudioStreamTerminalTypeHeadsetMicrophone, kAudioStreamTerminalTypeLFESpeaker, kAudioStreamTerminalTypeLine, kAudioStreamTerminalTypeMicrophone, kAudioStreamTerminalTypeReceiverMicrophone, kAudioStreamTerminalTypeReceiverSpeaker, kAudioStreamTerminalTypeSpeaker, kAudioStreamTerminalTypeTTY
};
use num_traits::AsPrimitive;
use crate::errors::{CoreAudioError, ErrorKind};

// ---- Constants ------------

/// Maximum value for a 20-bit signed integer sample.
const I20_MAX: i32 = 2i32.pow(20 - 1) - 1;

/// Maximum value for a 24-bit signed integer sample.
const I24_MAX: i32 = 2i32.pow(24 - 1) - 1;

// ---- Enums ------------

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

/// Numeric encoding of audio samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleEncoding {
    /// IEEE 754 floating-point.
    Float,
    /// Two's-complement signed integer.
    SignedInt,
    /// Unsigned integer (offset binary).
    UnSignedInt,
}

/// Whether this process holds exclusive access to a device.
///
/// CoreAudio represents hog mode as a PID: `-1` means the device is free,
/// any other value is the PID of the owning process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HogMode {
    /// No process holds exclusive access.
    Released,
    /// The device is owned by the process with this PID.
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

/// A hint to the HAL about the system's current power state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerHint {
    /// Normal operation; no power-saving constraint.
    None,
    /// Prefer lower power consumption in scheduling decisions.
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

/// The physical connection type between a device and the host system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportType {
    /// Integrated hardware (e.g. the MacBook speaker or internal mic).
    BuiltIn,
    /// An aggregate device composed of multiple logical devices.
    Aggregate,
    /// An automatically created aggregate device.
    AutoAggregate,
    /// A software-only virtual device with no physical hardware.
    Virtual,
    /// PCI or PCIe expansion card.
    PCIe,
    /// Universal Serial Bus.
    USB,
    /// FireWire (IEEE 1394).
    FireWire,
    /// Bluetooth Classic.
    Bluetooth,
    /// Bluetooth Low Energy.
    BluetoothLE,
    /// HDMI audio channel.
    HDMI,
    /// DisplayPort audio channel.
    DisplayPort,
    /// AirPlay wireless streaming.
    AirPlay,
    /// Audio Video Bridging (IEEE 802.1 AVB).
    AVB,
    /// Thunderbolt (includes USB4 tunnelled Thunderbolt).
    Thunderbolt,
    /// Continuity Camera (iPhone used as webcam/mic).
    ContinuityCapture,
    /// Continuity Camera over a wired USB connection.
    ContinuityCaptureWired,
    /// Continuity Camera over a wireless connection.
    ContinuityCaptureWireless,
    /// An unrecognised transport type; contains the raw transport ID.
    Unknown(u32),
}

impl From<u32> for TransportType {
    #[allow(non_upper_case_globals)]
    fn from(value: u32) -> Self {
        match value {
            kAudioDeviceTransportTypeBuiltIn => Self::BuiltIn,
            kAudioDeviceTransportTypeAggregate => Self::Aggregate,
            kAudioDeviceTransportTypeAutoAggregate => Self::AutoAggregate,
            kAudioDeviceTransportTypeVirtual => Self::Virtual,
            kAudioDeviceTransportTypePCI => Self::PCIe,
            kAudioDeviceTransportTypeUSB => Self::USB,
            kAudioDeviceTransportTypeFireWire => Self::FireWire,
            kAudioDeviceTransportTypeBluetooth => Self::Bluetooth,
            kAudioDeviceTransportTypeBluetoothLE => Self::BluetoothLE,
            kAudioDeviceTransportTypeHDMI => Self::HDMI,
            kAudioDeviceTransportTypeDisplayPort => Self::DisplayPort,
            kAudioDeviceTransportTypeAirPlay => Self::AirPlay,
            kAudioDeviceTransportTypeAVB => Self::AVB,
            kAudioDeviceTransportTypeThunderbolt => Self::Thunderbolt,
            kAudioDeviceTransportTypeContinuityCapture => Self::ContinuityCapture,
            kAudioDeviceTransportTypeContinuityCaptureWired => Self::ContinuityCaptureWired,
            kAudioDeviceTransportTypeContinuityCaptureWireless => Self::ContinuityCaptureWireless,
            id => Self::Unknown(id),
        }
    }
}

/// The type of physical endpoint a stream connects to.
///
/// Reported by `TERMINAL_TYPE` and maps to the `kAudioStreamTerminalType*`
/// constants in CoreAudio.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalType {
    /// Analog line-level input or output.
    Line,
    /// Digital audio interface such as S/PDIF or AES3.
    DigitalAudioInterface,
    /// A loudspeaker.
    Speaker,
    /// Headphone output.
    Headphones,
    /// Low-frequency effects (subwoofer) speaker.
    LFESpeaker,
    /// A speaker inside a handset receiver.
    ReceiverSpeaker,
    /// A microphone input.
    Microphone,
    /// A combined headset microphone (headphone + mic combo jack).
    HeadsetMicrophone,
    /// A microphone in a handset receiver.
    ReceiverMicrophone,
    /// Telephone TTY/TDD interface.
    TTY,
    /// HDMI audio endpoint.
    HDMI,
    /// DisplayPort audio endpoint.
    DisplayPort,
    /// An unrecognised terminal type; contains the raw value.
    Unknown(u32),
}

#[allow(non_upper_case_globals)]
impl From<u32> for TerminalType {
    fn from(value: u32) -> Self {
        match value {
            kAudioStreamTerminalTypeLine => Self::Line,
            kAudioStreamTerminalTypeDigitalAudioInterface => Self::DigitalAudioInterface,
            kAudioStreamTerminalTypeSpeaker => Self::Speaker,
            kAudioStreamTerminalTypeHeadphones => Self::Headphones,
            kAudioStreamTerminalTypeLFESpeaker => Self::LFESpeaker,
            kAudioStreamTerminalTypeReceiverSpeaker => Self::ReceiverSpeaker,
            kAudioStreamTerminalTypeMicrophone => Self::Microphone,
            kAudioStreamTerminalTypeHeadsetMicrophone => Self::HeadsetMicrophone,
            kAudioStreamTerminalTypeReceiverMicrophone => Self::ReceiverMicrophone,
            kAudioStreamTerminalTypeTTY => Self::TTY,
            kAudioStreamTerminalTypeHDMI => Self::HDMI,
            kAudioStreamTerminalTypeDisplayPort => Self::DisplayPort,
            id => Self::Unknown(id),
        }
    }
}

/// The concrete bit layout of an audio sample.
///
/// Constructed from a `bits_per_channel` value and a [`SampleEncoding`].
/// Used by [`SampleFormat::resample`] to scale a normalised `f32` value into
/// the target numeric type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleFormat {
    // Unsigned
    /// 8-bit unsigned integer.
    U8,
    /// 16-bit unsigned integer.
    U16,
    /// 32-bit unsigned integer.
    U32,
    // Signed
    /// 8-bit signed integer.
    I8,
    /// 16-bit signed integer.
    I16,
    /// 20-bit signed integer (packed into a larger word by the driver).
    I20,
    /// 24-bit signed integer.
    I24,
    /// 32-bit signed integer.
    I32,
    /// 64-bit signed integer.
    I64,
    // Floating point
    /// 32-bit IEEE 754 float.
    F32,
    /// 64-bit IEEE 754 float.
    F64,
}

impl SampleFormat {
    /// Constructs a `SampleFormat` from a bit depth and encoding, returning
    /// `None` for combinations that have no known mapping.
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

    /// Converts a normalised `f32` sample in `[-1.0, 1.0]` into the target
    /// numeric type `T` using the full range of this format.
    ///
    /// Unsigned formats are mapped to `[0, MAX]`; signed and float formats are
    /// mapped to `[-MAX, MAX]` and `[-1.0, 1.0]` respectively. Values outside
    /// `[-1.0, 1.0]` are clamped before conversion.
    pub fn resample<T>(&self, sample: f32) -> T
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

/// MPEG-4 AAC sub-format variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AACFormat {
    /// Standard MPEG-4 AAC (LC profile).
    Standard,
    /// AAC Enhanced Low Delay.
    ELD,
    /// AAC Enhanced Low Delay v2.
    ELDv2,
    /// AAC Enhanced Low Delay with Spectral Band Replication.
    ELDSBR,
    /// AAC High Efficiency (HE-AAC v1, with SBR).
    HE,
    /// AAC High Efficiency v2 (HE-AAC v2, with SBR + PS).
    HEv2,
    /// AAC Low Delay.
    LD,
    /// AAC Spatial Audio (Apple-specific spatial variant).
    Spatial,
}

/// Audio codec or compression format identifier.
///
/// Maps to the `mFormatID` field of `AudioStreamBasicDescription`. Implements
/// `From<u32>` and `Into<u32>` to round-trip through the CoreAudio four-char
/// code representation.
#[derive(Debug, Clone, Copy)]
pub enum FormatId {
    /// Uncompressed linear PCM.
    LinearPCM,
    /// MPEG-4 AAC in one of several sub-formats.
    AAC(AACFormat),
    /// Apple Lossless Audio Codec.
    ALAC,
    /// Dolby AC-3.
    AC3,
    /// Apple Positional Audio Codec (spatial audio).
    APAC,
    /// AES3 / S/PDIF digital audio.
    AES3,
    /// ITU-T G.711 A-law companding.
    ALaw,
    /// Adaptive Multi-Rate (GSM narrowband).
    AMR,
    /// Adaptive Multi-Rate Wideband.
    AMRWB,
    /// Dolby Digital Plus (Enhanced AC-3).
    EnhancedAC3,
    /// Opus interactive audio codec.
    Opus,
    /// MPEG-1 Layer III (MP3).
    MP3,
    /// An unrecognised format; contains the raw four-char code as a `u32`.
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

/// Decoded `mFormatFlags` from an `AudioStreamBasicDescription`.
///
/// The raw CoreAudio flags field is a bitmask; this struct breaks it out into
/// named boolean fields for easier inspection.
#[derive(Debug, Clone, Copy)]
pub struct FormatFlags {
    /// Whether samples are float, signed integer, or unsigned integer.
    encoding: SampleEncoding,
    /// Whether multi-byte samples are stored big-endian.
    is_big_endian: bool,
    /// Whether samples fill all bits of their container with no padding.
    is_packed: bool,
    /// Whether channel data is interleaved within each buffer.
    is_interleaved: bool,
    /// Whether mixing this stream with others is prohibited.
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
    /// The numeric encoding of each sample.
    pub fn encoding(&self) -> SampleEncoding {
        self.encoding
    }

    /// `true` if multi-byte samples are stored in big-endian byte order.
    pub fn is_big_endian(&self) -> bool {
        self.is_big_endian
    }

    /// `true` if sample bits fill their container word with no padding.
    pub fn is_packed(&self) -> bool {
        self.is_packed
    }

    /// `true` if channel data is interleaved within a single buffer.
    pub fn is_interleaved(&self) -> bool {
        self.is_interleaved
    }

    /// `true` if this stream cannot be mixed with others by the HAL.
    pub fn is_non_mixable(&self) -> bool {
        self.is_non_mixable
    }
}

/// A Rust-friendly view of `AudioStreamBasicDescription`.
///
/// Describes the complete audio format of a stream: sample rate, codec,
/// channel count, bit depth, and layout. Obtain one via
/// [`STREAM_VIRTUAL_FORMAT`](crate::property::STREAM_VIRTUAL_FORMAT) or
/// [`STREAM_PHYSICAL_FORMAT`](crate::property::STREAM_PHYSICAL_FORMAT).
///
/// ## Terminology
///
/// - **Sample** — a single amplitude value for a single channel at a single
///   point in time.
/// - **Frame** — one sample per channel, all captured at the same instant.
///   This is the fundamental unit of audio time: advancing by one frame
///   advances playback or recording by `1 / sample_rate` seconds.
/// - **Packet** — the smallest indivisible chunk of compressed data. For
///   uncompressed PCM, one packet always equals one frame
///   (`frames_per_packet == 1`). For compressed formats such as AAC, one
///   packet typically contains 1024 or more frames.
///
/// ## Relationship to [`AudioBuffer`](crate::AudioBuffer)
///
/// Each [`AudioBuffer`](crate::AudioBuffer) delivered to an `IOProc` callback
/// represents one I/O cycle's worth of PCM audio. Its fields map directly onto
/// this description:
///
/// - `frame_count` — the number of frames in the cycle; equals
///   `DEVICE_BUFFER_FRAME_SIZE` under normal conditions.
/// - `channels` — matches `channels_per_frame`.
/// - `data.len()` — for **interleaved** buffers (`is_interleaved == true`):
///   `frame_count * channels_per_frame` elements. For **non-interleaved**
///   buffers, each buffer covers a single channel and contains `frame_count`
///   elements, with one `AudioBuffer` per channel in the slice.
///
/// `IOProc` always delivers PCM, so `bytes_per_packet`, `frames_per_packet`,
/// and `bytes_per_frame` reflect the uncompressed layout. The HAL performs any
/// necessary decompression before handing data to the callback.
#[derive(Debug, Clone, Copy)]
pub struct StreamDescription {
    /// Sample rate in Hz (e.g. `44100.0` or `48000.0`).
    sample_rate: f64,
    /// Codec or compression scheme.
    format_id: FormatId,
    /// Decoded format flags describing byte order, packing, and interleaving.
    flags: FormatFlags,
    /// Concrete sample format derived from `bits_per_channel` and encoding,
    /// or `None` for compressed formats where bit depth is not meaningful.
    sample_format: Option<SampleFormat>,
    /// Bytes in one packet. For PCM equals `bytes_per_frame`; for compressed
    /// formats equals the encoded size of one packet (which spans
    /// `frames_per_packet` frames). Zero if the packet size is variable.
    bytes_per_packet: u32,
    /// Frames per packet. Always `1` for uncompressed PCM. For compressed
    /// formats this is the number of frames encoded into each packet — e.g.
    /// `1024` for AAC-LC.
    frames_per_packet: u32,
    /// Bytes in one frame: `(bits_per_channel / 8) * channels_per_frame` for
    /// packed PCM. Zero for compressed formats where frames are not
    /// independently addressable.
    bytes_per_frame: u32,
    /// Number of channels per frame. Matches `AudioBuffer::channels` for the
    /// corresponding interleaved buffer, or the total channel count across all
    /// non-interleaved buffers in an I/O cycle.
    channels_per_frame: u32,
    /// Valid audio bits per channel. May be less than the container word size —
    /// for example, 20-bit audio packed into a 32-bit word has
    /// `bits_per_channel == 20` and `bytes_per_frame == 4 * channels`.
    bits_per_channel: u32,
    /// Reserved; always zero.
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
    /// Sample rate in Hz.
    pub fn sample_rate(&self) -> f64 {
        self.sample_rate
    }

    /// Codec or compression scheme.
    pub fn format_id(&self) -> FormatId {
        self.format_id
    }

    /// Decoded format flags.
    pub fn flags(&self) -> FormatFlags {
        self.flags
    }

    /// Concrete sample format, or `None` for compressed audio.
    pub fn sample_format(&self) -> Option<SampleFormat> {
        self.sample_format
    }

    /// Bytes per compressed packet (equals `bytes_per_frame` for PCM).
    pub fn bytes_per_packet(&self) -> u32 {
        self.bytes_per_packet
    }

    /// Frames per compressed packet (always `1` for PCM).
    pub fn frames_per_packet(&self) -> u32 {
        self.frames_per_packet
    }

    /// Bytes in one audio frame (one sample per channel).
    pub fn bytes_per_frame(&self) -> u32 {
        self.bytes_per_frame
    }

    /// Number of channels per frame.
    pub fn channels_per_frame(&self) -> u32 {
        self.channels_per_frame
    }
}

/// The range of buffer frame sizes a device supports.
///
/// Both ends of the range are inclusive. Use [`valid_sizes`](Self::valid_sizes)
/// to enumerate only power-of-two sizes within the range, as required by most
/// CoreAudio devices.
#[derive(Debug)]
pub struct BufferFrameSizeRange {
    /// Smallest supported buffer size in frames.
    min: u32,
    /// Largest supported buffer size in frames.
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
    /// Returns all power-of-two values between `min` and `max` inclusive.
    ///
    /// CoreAudio devices typically only accept power-of-two buffer sizes.
    /// The returned list is sorted in ascending order.
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

/// A [`StreamDescription`] paired with the sample rate range it supports.
///
/// Used by `STREAM_AVAILABLE_VIRTUAL_FORMATS` and
/// `STREAM_AVAILABLE_PHYSICAL_FORMATS` to describe each format the stream can
/// operate in.
#[derive(Debug)]
pub struct StreamRangedDescription {
    /// The audio format parameters for this entry.
    stream_description: StreamDescription,
    /// The range of sample rates valid for this format.
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
    /// The audio format description.
    pub fn stream_description(&self) -> StreamDescription {
        self.stream_description
    }

    /// The sample rate range supported for this format.
    pub fn sample_rate_range(&self) -> SampleRateRange {
        self.sample_rate_range
    }
}

/// A continuous range of sample rates in Hz.
///
/// Some devices report discrete rates as ranges where `min == max`; others
/// report a true continuous range. Use [`as_range`](Self::as_range) to get a
/// standard Rust `RangeInclusive<f64>`.
#[derive(Debug, Clone, Copy)]
pub struct SampleRateRange {
    /// Lowest supported sample rate in Hz.
    min: f64,
    /// Highest supported sample rate in Hz.
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
    /// Lowest sample rate in the range, in Hz.
    pub fn min(&self) -> f64 {
        self.min
    }

    /// Highest sample rate in the range, in Hz.
    pub fn max(&self) -> f64 {
        self.max
    }

    /// Returns the range as a standard `RangeInclusive<f64>`.
    pub fn as_range(&self) -> RangeInclusive<f64> {
        self.min..=self.max
    }
}

/// A continuous range of volume values in dBFS.
///
/// Used by `DEVICE_VOLUME_RANGE_DECIBELS` and related properties to describe
/// the hardware's supported volume range on a given channel.
#[derive(Debug, Clone, Copy)]
pub struct DBRange {
    /// Minimum volume in dBFS (typically a large negative number).
    min: f64,
    /// Maximum volume in dBFS (typically `0.0`).
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
    /// Minimum volume in dBFS.
    pub fn min(&self) -> f64 {
        self.min
    }

    /// Maximum volume in dBFS.
    pub fn max(&self) -> f64 {
        self.max
    }

    /// Returns the range as a standard `RangeInclusive<f64>`.
    pub fn as_range(&self) -> RangeInclusive<f64> {
        self.min..=self.max
    }
}

/// A pair of channel indices representing a left/right stereo assignment.
///
/// Used by `DEVICE_PREFERRED_CHANNELS_FOR_STEREO` and
/// `DEVICE_STEREO_PAN_CHANNELS` to identify which device channels carry the
/// left and right signals.
#[derive(Debug, Clone, Copy)]
pub struct ChannelPair {
    /// One-based device channel index for the left signal.
    left: u32,
    /// One-based device channel index for the right signal.
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
    /// One-based device channel index for the left signal.
    pub fn left(&self) -> u32 {
        self.left
    }

    /// One-based device channel index for the right signal.
    pub fn right(&self) -> u32 {
        self.right
    }

    /// Returns the pair as a two-element array `[left, right]`.
    pub fn as_array(&self) -> [u32; 2] {
        [self.left, self.right]
    }
}
