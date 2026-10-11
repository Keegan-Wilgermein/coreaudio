//! Value ranges and channel pairs

use objc2_core_audio_types::AudioValueRange;
use std::ops::RangeInclusive;

/// The range of buffer frame sizes a device supports
///
/// Both ends of the range are inclusive
/// Use [`valid_sizes`](Self::valid_sizes) to enumerate only power-of-two sizes
/// within the range, as required by most CoreAudio devices
#[derive(Debug)]
pub struct BufferFrameSizeRange {
    /// Smallest supported buffer size in frames
    min: u32,
    /// Largest supported buffer size in frames
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

impl From<BufferFrameSizeRange> for AudioValueRange {
    fn from(value: BufferFrameSizeRange) -> Self {
        Self {
            mMinimum: value.min as f64,
            mMaximum: value.max as f64,
        }
    }
}

impl BufferFrameSizeRange {
    /// Returns all power-of-two values between `min` and `max` inclusive
    ///
    /// CoreAudio devices typically only accept power-of-two buffer sizes
    /// The returned list is sorted in ascending order
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

/// A continuous range of sample rates in Hz
///
/// Some devices report discrete rates as ranges where `min == max`; others
/// report a true continuous range
/// Use [`as_range`](Self::as_range) to get a standard Rust
/// `RangeInclusive<f64>`
#[derive(Debug, Clone, Copy)]
pub struct SampleRateRange {
    /// Lowest supported sample rate in Hz
    min: f64,
    /// Highest supported sample rate in Hz
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

impl From<SampleRateRange> for AudioValueRange {
    fn from(value: SampleRateRange) -> Self {
        Self {
            mMinimum: value.min,
            mMaximum: value.max,
        }
    }
}

impl SampleRateRange {
    /// Gets the minimum supported sample rate
    pub fn get_min(&self) -> f64 {
        self.min
    }

    /// Gets the maximum supported sample rate
    pub fn get_max(&self) -> f64 {
        self.max
    }
}

/// A continuous range of volume values in dBFS
///
/// Used by `DEVICE_VOLUME_RANGE_DECIBELS` and related properties to describe
/// the hardware's supported volume range on a given channel
#[derive(Debug, Clone, Copy)]
pub struct DBRange {
    /// Minimum volume in dBFS (typically a large negative number)
    min: f64,
    /// Maximum volume in dBFS (typically `0.0`)
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

impl From<DBRange> for AudioValueRange {
    fn from(value: DBRange) -> Self {
        Self {
            mMinimum: value.min,
            mMaximum: value.max,
        }
    }
}

impl DBRange {
    /// Minimum volume in dBFS
    pub fn min(&self) -> f64 {
        self.min
    }

    /// Maximum volume in dBFS
    pub fn max(&self) -> f64 {
        self.max
    }

    /// Returns the range as a standard `RangeInclusive<f64>`
    pub fn as_range(&self) -> RangeInclusive<f64> {
        self.min..=self.max
    }
}

/// A pair of channel indices representing a left/right stereo assignment
///
/// Used by `DEVICE_PREFERRED_CHANNELS_FOR_STEREO` and
/// `DEVICE_STEREO_PAN_CHANNELS` to identify which device channels carry the
/// left and right signals
#[derive(Debug, Clone, Copy)]
pub struct ChannelPair {
    /// One-based device channel index for the left signal
    left: u32,
    /// One-based device channel index for the right signal
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
    /// One-based device channel index for the left signal
    pub fn left(&self) -> u32 {
        self.left
    }

    /// One-based device channel index for the right signal
    pub fn right(&self) -> u32 {
        self.right
    }

    /// Returns the pair as a two-element array `[left, right]`
    pub fn as_array(&self) -> [u32; 2] {
        [self.left, self.right]
    }
}
