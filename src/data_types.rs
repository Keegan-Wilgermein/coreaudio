//! # Data types

// ---- Imports ------------
use std::ops::RangeInclusive;

// ---- Enums ------------
/// Input or output device selector
pub enum Scope {
    /// Input devices
    Input,
    /// Output devices
    Output,
}

// ----- Structs ------------
/// A range of sizes supported by a device to be used as a buffer size
pub struct BufferFrameSizeRange {
    min: u32,
    max: u32,
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

pub struct SampleRateRange {
    min: f64,
    max: f64,
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
