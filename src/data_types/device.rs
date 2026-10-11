//! Device-level enums: hog mode, power hints, transport and terminal types

#![allow(deprecated)] // `kAudioDeviceTransportTypeContinuityCapture` is still reported by older systems

use objc2_core_audio::{kAudioDeviceTransportTypeAVB, kAudioDeviceTransportTypeAggregate, kAudioDeviceTransportTypeAirPlay, kAudioDeviceTransportTypeAutoAggregate, kAudioDeviceTransportTypeBluetooth, kAudioDeviceTransportTypeBluetoothLE, kAudioDeviceTransportTypeBuiltIn, kAudioDeviceTransportTypeContinuityCapture, kAudioDeviceTransportTypeContinuityCaptureWired, kAudioDeviceTransportTypeContinuityCaptureWireless, kAudioDeviceTransportTypeDisplayPort, kAudioDeviceTransportTypeFireWire, kAudioDeviceTransportTypeHDMI, kAudioDeviceTransportTypePCI, kAudioDeviceTransportTypeThunderbolt, kAudioDeviceTransportTypeUSB, kAudioDeviceTransportTypeVirtual, kAudioStreamTerminalTypeDigitalAudioInterface, kAudioStreamTerminalTypeDisplayPort, kAudioStreamTerminalTypeHDMI, kAudioStreamTerminalTypeHeadphones, kAudioStreamTerminalTypeHeadsetMicrophone, kAudioStreamTerminalTypeLFESpeaker, kAudioStreamTerminalTypeLine, kAudioStreamTerminalTypeMicrophone, kAudioStreamTerminalTypeReceiverMicrophone, kAudioStreamTerminalTypeReceiverSpeaker, kAudioStreamTerminalTypeSpeaker, kAudioStreamTerminalTypeTTY};
use crate::errors::{CoreAudioError, ErrorKind};

/// Whether this process holds exclusive access to a device
///
/// CoreAudio represents hog mode as a PID: `-1` means the device is free,
/// any other value is the PID of the owning process
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HogMode {
    /// No process holds exclusive access
    Released,
    /// The device is owned by the process with this PID
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

/// A hint to the HAL about the system's current power state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerHint {
    /// Normal operation; no power-saving constraint
    None,
    /// Prefer lower power consumption in scheduling decisions
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

/// The physical connection type between a device and the host system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportType {
    /// Integrated hardware (e.g. the MacBook speaker or internal mic)
    BuiltIn,
    /// An aggregate device composed of multiple logical devices
    Aggregate,
    /// An automatically created aggregate device
    AutoAggregate,
    /// A software-only virtual device with no physical hardware
    Virtual,
    /// PCI or PCIe expansion card
    PCIe,
    /// Universal Serial Bus
    USB,
    /// FireWire (IEEE 1394)
    FireWire,
    /// Bluetooth Classic
    Bluetooth,
    /// Bluetooth Low Energy
    BluetoothLE,
    /// HDMI audio channel
    HDMI,
    /// DisplayPort audio channel
    DisplayPort,
    /// AirPlay wireless streaming
    AirPlay,
    /// Audio Video Bridging (IEEE 802.1 AVB)
    AVB,
    /// Thunderbolt (includes USB4 tunnelled Thunderbolt)
    Thunderbolt,
    /// Continuity Camera (iPhone used as webcam/mic)
    ContinuityCapture,
    /// Continuity Camera over a wired USB connection
    ContinuityCaptureWired,
    /// Continuity Camera over a wireless connection
    ContinuityCaptureWireless,
    /// An unrecognised transport type; contains the raw transport ID
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

/// The type of physical endpoint a stream connects to
///
/// Reported by `TERMINAL_TYPE` and maps to the `kAudioStreamTerminalType*`
/// constants in CoreAudio
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalType {
    /// Analog line-level input or output
    Line,
    /// Digital audio interface such as S/PDIF or AES3
    DigitalAudioInterface,
    /// A loudspeaker
    Speaker,
    /// Headphone output
    Headphones,
    /// Low-frequency effects (subwoofer) speaker
    LFESpeaker,
    /// A speaker inside a handset receiver
    ReceiverSpeaker,
    /// A microphone input
    Microphone,
    /// A combined headset microphone (headphone + mic combo jack)
    HeadsetMicrophone,
    /// A microphone in a handset receiver
    ReceiverMicrophone,
    /// Telephone TTY/TDD interface
    TTY,
    /// HDMI audio endpoint
    HDMI,
    /// DisplayPort audio endpoint
    DisplayPort,
    /// An unrecognised terminal type; contains the raw value
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

#[allow(non_upper_case_globals)]
impl From<TransportType> for u32 {
    fn from(value: TransportType) -> Self {
        match value {
            TransportType::BuiltIn => kAudioDeviceTransportTypeBuiltIn,
            TransportType::Aggregate => kAudioDeviceTransportTypeAggregate,
            TransportType::AutoAggregate => kAudioDeviceTransportTypeAutoAggregate,
            TransportType::Virtual => kAudioDeviceTransportTypeVirtual,
            TransportType::PCIe => kAudioDeviceTransportTypePCI,
            TransportType::USB => kAudioDeviceTransportTypeUSB,
            TransportType::FireWire => kAudioDeviceTransportTypeFireWire,
            TransportType::Bluetooth => kAudioDeviceTransportTypeBluetooth,
            TransportType::BluetoothLE => kAudioDeviceTransportTypeBluetoothLE,
            TransportType::HDMI => kAudioDeviceTransportTypeHDMI,
            TransportType::DisplayPort => kAudioDeviceTransportTypeDisplayPort,
            TransportType::AirPlay => kAudioDeviceTransportTypeAirPlay,
            TransportType::AVB => kAudioDeviceTransportTypeAVB,
            TransportType::Thunderbolt => kAudioDeviceTransportTypeThunderbolt,
            TransportType::ContinuityCapture => kAudioDeviceTransportTypeContinuityCapture,
            TransportType::ContinuityCaptureWired => kAudioDeviceTransportTypeContinuityCaptureWired,
            TransportType::ContinuityCaptureWireless => kAudioDeviceTransportTypeContinuityCaptureWireless,
            TransportType::Unknown(id) => id,
        }
    }
}

#[allow(non_upper_case_globals)]
impl From<TerminalType> for u32 {
    fn from(value: TerminalType) -> Self {
        match value {
            TerminalType::Line => kAudioStreamTerminalTypeLine,
            TerminalType::DigitalAudioInterface => kAudioStreamTerminalTypeDigitalAudioInterface,
            TerminalType::Speaker => kAudioStreamTerminalTypeSpeaker,
            TerminalType::Headphones => kAudioStreamTerminalTypeHeadphones,
            TerminalType::LFESpeaker => kAudioStreamTerminalTypeLFESpeaker,
            TerminalType::ReceiverSpeaker => kAudioStreamTerminalTypeReceiverSpeaker,
            TerminalType::Microphone => kAudioStreamTerminalTypeMicrophone,
            TerminalType::HeadsetMicrophone => kAudioStreamTerminalTypeHeadsetMicrophone,
            TerminalType::ReceiverMicrophone => kAudioStreamTerminalTypeReceiverMicrophone,
            TerminalType::TTY => kAudioStreamTerminalTypeTTY,
            TerminalType::HDMI => kAudioStreamTerminalTypeHDMI,
            TerminalType::DisplayPort => kAudioStreamTerminalTypeDisplayPort,
            TerminalType::Unknown(id) => id,
        }
    }
}
