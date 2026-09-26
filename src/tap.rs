//! Process taps: the audio a set of processes output, captured before it's
//! mixed into any device (macOS 14.2+).
//!
//! A tap is read through a private aggregate device
//! ([`AggregateDevice::with_tap`](crate::AggregateDevice::with_tap)), which
//! turns it into an ordinary input device.
//!
//! Tapping needs the user's System Audio Recording permission (Privacy &
//! Security). Without it a tap is still created, but delivers silence — and a
//! muting tap still mutes. Check the permission before tapping.

#![allow(unsafe_code)]

// ---- Imports ------------
use crate::{errors::{CoreAudioError, OSStatusCheck}, object::{AudioObject, Process, Tap}, property::TAP_UID};
use coreaudio_sys::AudioObjectID;
use objc2::AllocAnyThread;
use objc2_core_audio::{AudioHardwareCreateProcessTap, AudioHardwareDestroyProcessTap, CATapDescription, CATapMuteBehavior};
use objc2_foundation::{NSArray, NSNumber, NSString};

// ---- Enums ------------

/// What happens to a tapped process's own output.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TapMute {
    /// It keeps playing as normal; the tap gets a copy.
    #[default]
    Unmuted,
    /// It's silenced for as long as the tap exists.
    Muted,
    /// It's silenced only while the tap is being read.
    MutedWhenTapped,
}

impl From<TapMute> for CATapMuteBehavior {
    fn from(value: TapMute) -> Self {
        match value {
            TapMute::Unmuted => CATapMuteBehavior::Unmuted,
            TapMute::Muted => CATapMuteBehavior::Muted,
            TapMute::MutedWhenTapped => CATapMuteBehavior::MutedWhenTapped,
        }
    }
}

// ---- Structs ------------

/// A process tap, destroyed when dropped.
pub struct ProcessTap {
    tap: AudioObject<Tap>,
    uid: String,
}

impl ProcessTap {
    /// Taps `processes` mixed down to stereo, named `name`, with `mute`
    /// applied to their own output. The tap is private to this process.
    ///
    /// The mixdown folds every channel of the processes' output into two, so
    /// on a wide device (many channels, most of them silent) it comes out far
    /// quieter than the processes actually play. Use
    /// [`device_stream`](Self::device_stream) to capture them as they are.
    pub fn stereo_mixdown(
        processes: &[AudioObject<Process>],
        name: &str,
        mute: TapMute,
    ) -> Result<Self, CoreAudioError> {
        let ids = process_ids(processes);
        let description = unsafe {
            CATapDescription::initStereoMixdownOfProcesses(CATapDescription::alloc(), &ids)
        };
        Self::create(&description, name, mute)
    }

    /// Taps what `processes` play into stream `stream` of the device with UID
    /// `device_uid`, channel for channel and at full level: the tap has that
    /// stream's channels and runs at the device's rate. Named `name`, with
    /// `mute` applied to their own output; private to this process.
    pub fn device_stream(
        processes: &[AudioObject<Process>],
        device_uid: &str,
        stream: usize,
        name: &str,
        mute: TapMute,
    ) -> Result<Self, CoreAudioError> {
        let ids = process_ids(processes);
        let description = unsafe {
            CATapDescription::initWithProcesses_andDeviceUID_withStream(
                CATapDescription::alloc(),
                &ids,
                &NSString::from_str(device_uid),
                stream as _,
            )
        };
        Self::create(&description, name, mute)
    }

    fn create(description: &CATapDescription, name: &str, mute: TapMute) -> Result<Self, CoreAudioError> {
        let mut id: AudioObjectID = 0;
        unsafe {
            description.setName(&NSString::from_str(name));
            description.setPrivate(true);
            description.setMuteBehavior(mute.into());
            AudioHardwareCreateProcessTap(Some(description), &mut id).check()?;
        }

        let tap = AudioObject::<Tap>::from(id);
        let uid = match tap.get_property(TAP_UID) {
            Ok(uid) => uid,
            Err(error) => {
                unsafe {
                    AudioHardwareDestroyProcessTap(id);
                }
                return Err(error);
            }
        };
        Ok(Self { tap, uid })
    }

    /// The tap object, to read its properties.
    pub fn tap(&self) -> AudioObject<Tap> {
        self.tap
    }

    /// The tap's UID, for [`AggregateDevice::with_tap`](crate::AggregateDevice::with_tap).
    pub fn uid(&self) -> &str {
        &self.uid
    }
}

/// The process object IDs as the `NSArray<NSNumber>` a tap description takes.
fn process_ids(processes: &[AudioObject<Process>]) -> objc2::rc::Retained<NSArray<NSNumber>> {
    let ids: Vec<_> = processes.iter().map(|process| NSNumber::new_u32(process.id())).collect();
    NSArray::from_retained_slice(&ids)
}

impl Drop for ProcessTap {
    fn drop(&mut self) {
        unsafe {
            AudioHardwareDestroyProcessTap(self.tap.id());
        }
    }
}
