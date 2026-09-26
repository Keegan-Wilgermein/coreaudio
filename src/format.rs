//! Channel layout queries answered by AudioToolbox's `AudioFormat` services.
//!
//! These don't touch any device: they expand predefined layouts and give the
//! display names macOS uses for layouts and speakers.

#![allow(unsafe_code)]

// ---- Imports ------------
use crate::{data_types::{ChannelDescription, ChannelLayout, ChannelLayoutTag}, errors::{CoreAudioError, OSStatusCheck}};
use std::{ffi::c_void, ptr::null_mut};
use core_foundation::{base::TCFType, string::{CFString, CFStringRef}};
use coreaudio_sys::{AudioFormatGetProperty, AudioFormatGetPropertyInfo, AudioFormatPropertyID, kAudioFormatProperty_ChannelLayoutForTag, kAudioFormatProperty_ChannelLayoutName, kAudioFormatProperty_ChannelLayoutSimpleName, kAudioFormatProperty_ChannelName, kAudioFormatProperty_ChannelShortName};

// ---- Functions -----------

/// Expands a predefined layout into one [`ChannelDescription`] per channel.
pub fn layout_for_tag(tag: ChannelLayoutTag) -> Result<ChannelLayout, CoreAudioError> {
    let specifier = tag.0.to_ne_bytes();
    let bytes = get_bytes(kAudioFormatProperty_ChannelLayoutForTag, &specifier)?;
    ChannelLayout::from_bytes(&bytes)
}

/// The full name of a layout, e.g. "5.1 (L R C LFE Ls Rs)".
pub fn layout_name(layout: &ChannelLayout) -> Result<String, CoreAudioError> {
    get_string(kAudioFormatProperty_ChannelLayoutName, &layout.to_bytes())
}

/// The short name of a layout, e.g. "5.1".
pub fn layout_simple_name(layout: &ChannelLayout) -> Result<String, CoreAudioError> {
    get_string(kAudioFormatProperty_ChannelLayoutSimpleName, &layout.to_bytes())
}

/// The name of the speaker a channel feeds, e.g. "Left Surround".
pub fn channel_name(description: &ChannelDescription) -> Result<String, CoreAudioError> {
    get_string(kAudioFormatProperty_ChannelName, &description.to_bytes())
}

/// The abbreviated name of the speaker a channel feeds, e.g. "Ls".
pub fn channel_short_name(description: &ChannelDescription) -> Result<String, CoreAudioError> {
    get_string(kAudioFormatProperty_ChannelShortName, &description.to_bytes())
}

/// Reads a variable-length `AudioFormat` property into a byte buffer.
fn get_bytes(property: AudioFormatPropertyID, specifier: &[u8]) -> Result<Vec<u8>, CoreAudioError> {
    unsafe {
        let mut size = 0u32;
        AudioFormatGetPropertyInfo(
            property,
            specifier.len() as u32,
            specifier.as_ptr() as *const c_void,
            &mut size,
        ).check()?;

        let mut buffer = vec![0u8; size as usize];
        AudioFormatGetProperty(
            property,
            specifier.len() as u32,
            specifier.as_ptr() as *const c_void,
            &mut size,
            buffer.as_mut_ptr() as *mut c_void,
        ).check()?;

        buffer.truncate(size as usize);
        Ok(buffer)
    }
}

/// Reads a `CFStringRef` `AudioFormat` property. The string is created for the
/// caller, so it's taken under the create rule and released on drop.
fn get_string(property: AudioFormatPropertyID, specifier: &[u8]) -> Result<String, CoreAudioError> {
    unsafe {
        let mut name: CFStringRef = null_mut::<c_void>() as CFStringRef;
        let mut size = size_of::<CFStringRef>() as u32;
        AudioFormatGetProperty(
            property,
            specifier.len() as u32,
            specifier.as_ptr() as *const c_void,
            &mut size,
            &mut name as *mut CFStringRef as *mut c_void,
        ).check()?;

        if name.is_null() {
            return Err(CoreAudioError::from_error_kind(crate::ErrorKind::CFStringConversion));
        }

        Ok(CFString::wrap_under_create_rule(name).to_string())
    }
}
