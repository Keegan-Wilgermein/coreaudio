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
    pub fn stereo_mixdown(
        processes: &[AudioObject<Process>],
        name: &str,
        mute: TapMute,
    ) -> Result<Self, CoreAudioError> {
        let ids: Vec<_> = processes.iter().map(|process| NSNumber::new_u32(process.id())).collect();
        let ids = NSArray::from_retained_slice(&ids);

        let mut id: AudioObjectID = 0;
        unsafe {
            let description = CATapDescription::initStereoMixdownOfProcesses(
                CATapDescription::alloc(),
                &ids,
            );
            description.setName(&NSString::from_str(name));
            description.setPrivate(true);
            description.setMuteBehavior(mute.into());
            AudioHardwareCreateProcessTap(Some(&description), &mut id).check()?;
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

impl Drop for ProcessTap {
    fn drop(&mut self) {
        unsafe {
            AudioHardwareDestroyProcessTap(self.tap.id());
        }
    }
}
