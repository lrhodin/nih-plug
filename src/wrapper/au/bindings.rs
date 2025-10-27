//! Rust bindings for the Audio Unit C API.
//!
//! This module provides the necessary types and constants for implementing an Audio Unit plugin.
//! While we use coreaudio-sys for some functionality, we define critical AU types here directly
//! to have full control over the plugin interface.

use std::os::raw::{c_char, c_void};

/// Four-character code type used throughout the Audio Unit API
pub type FourCharCode = u32;

/// Unique identifier for an Audio Unit component
pub type AudioComponentInstance = *mut c_void;

/// Audio Unit component description
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AudioComponentDescription {
    /// Type of component (e.g., kAudioUnitType_Effect, kAudioUnitType_MusicDevice)
    pub component_type: FourCharCode,
    /// Subtype identifier (unique to your plugin)
    pub component_sub_type: FourCharCode,
    /// Manufacturer identifier (your company/developer ID)
    pub component_manufacturer: FourCharCode,
    /// Component flags
    pub component_flags: u32,
    /// Component flags mask
    pub component_flags_mask: u32,
}

/// Audio Unit component types
pub mod component_types {
    use super::FourCharCode;

    /// Audio effect unit (processes audio in-place or with side-chain)
    pub const K_AUDIO_UNIT_TYPE_EFFECT: FourCharCode = u32::from_be_bytes(*b"aufx");

    /// Music device unit (software synthesizer)
    pub const K_AUDIO_UNIT_TYPE_MUSIC_DEVICE: FourCharCode = u32::from_be_bytes(*b"aumu");

    /// Music effect unit (processes MIDI and audio)
    pub const K_AUDIO_UNIT_TYPE_MUSIC_EFFECT: FourCharCode = u32::from_be_bytes(*b"aumf");

    /// Generator unit (generates audio without input)
    pub const K_AUDIO_UNIT_TYPE_GENERATOR: FourCharCode = u32::from_be_bytes(*b"augn");

    /// Output unit (renders audio to hardware)
    pub const K_AUDIO_UNIT_TYPE_OUTPUT: FourCharCode = u32::from_be_bytes(*b"auou");
}

/// Audio Unit component flags
pub mod component_flags {
    /// Marks the component as sandbox-safe
    pub const K_AUDIO_COMPONENT_FLAG_SANDBOX_SAFE: u32 = 1;
}

/// Helper function to create a FourCharCode from a byte string
pub const fn make_four_char_code(bytes: &[u8; 4]) -> FourCharCode {
    u32::from_be_bytes(*bytes)
}

/// Audio Unit property IDs
pub mod property_ids {
    /// Class info property
    pub const K_AUDIO_UNIT_PROPERTY_CLASS_INFO: u32 = 0;
    /// Make connection property
    pub const K_AUDIO_UNIT_PROPERTY_MAKE_CONNECTION: u32 = 1;
    /// Sample rate property
    pub const K_AUDIO_UNIT_PROPERTY_SAMPLE_RATE: u32 = 2;
    /// Parameter list property
    pub const K_AUDIO_UNIT_PROPERTY_PARAMETER_LIST: u32 = 3;
    /// Stream format property
    pub const K_AUDIO_UNIT_PROPERTY_STREAM_FORMAT: u32 = 8;
    /// Maximum frames per slice property
    pub const K_AUDIO_UNIT_PROPERTY_MAXIMUM_FRAMES_PER_SLICE: u32 = 14;
    /// Latency property
    pub const K_AUDIO_UNIT_PROPERTY_LATENCY: u32 = 35;
}

/// Audio Unit scope identifiers
pub mod scopes {
    /// Global scope (entire audio unit)
    pub const K_AUDIO_UNIT_SCOPE_GLOBAL: u32 = 0;
    /// Input scope
    pub const K_AUDIO_UNIT_SCOPE_INPUT: u32 = 1;
    /// Output scope
    pub const K_AUDIO_UNIT_SCOPE_OUTPUT: u32 = 2;
}

/// Audio Unit error codes (OSStatus)
pub mod errors {
    pub const NO_ERR: i32 = 0;
    pub const K_AUDIO_UNIT_ERR_INVALID_PROPERTY: i32 = -10879;
    pub const K_AUDIO_UNIT_ERR_UNINITIALIZED: i32 = -10849;
    pub const K_AUDIO_UNIT_ERR_INVALID_PARAMETER: i32 = -50;
}

/// The factory function signature for creating Audio Unit instances
///
/// This function is called by the Audio Component system to create new instances
/// of your audio unit. It should return a pointer to the AudioComponentPlugInInstance
/// structure for your plugin.
pub type AudioComponentFactoryFunction = unsafe extern "C" fn(
    desc: *const AudioComponentDescription,
) -> *mut c_void;
