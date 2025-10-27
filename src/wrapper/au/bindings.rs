//! Rust bindings for the Audio Unit C API.
//!
//! This module provides the necessary types and constants for implementing an Audio Unit plugin.
//! While we use coreaudio-sys for some functionality, we define critical AU types here directly
//! to have full control over the plugin interface.

use std::os::raw::c_void;

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

/// MIDI event structure for Audio Units
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MIDIEvent {
    /// The MIDI status byte (note on, note off, etc.)
    pub status: u8,
    /// The MIDI data byte 1 (note number, controller number, etc.)
    pub data1: u8,
    /// The MIDI data byte 2 (velocity, controller value, etc.)
    pub data2: u8,
    /// Reserved field
    pub reserved: u8,
    /// The sample offset within the current buffer
    pub sample_offset: u32,
}

/// MIDI event list structure
#[repr(C)]
#[derive(Debug)]
pub struct MIDIEventList {
    /// Number of MIDI events in the list
    pub num_events: u32,
    /// Array of MIDI events
    pub events: [MIDIEvent; 0],
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
    /// Preset property - for loading/saving presets
    pub const K_AUDIO_UNIT_PROPERTY_PRESET: u32 = 4;
    /// Current preset property - for getting current preset info
    pub const K_AUDIO_UNIT_PROPERTY_CURRENT_PRESET: u32 = 5;
    /// Factory presets property - for getting available presets
    pub const K_AUDIO_UNIT_PROPERTY_FACTORY_PRESETS: u32 = 6;
    /// MIDI input callback property
    pub const K_AUDIO_UNIT_PROPERTY_MIDI_INPUT_CALLBACK: u32 = 7;
    /// MIDI output callback property
    pub const K_AUDIO_UNIT_PROPERTY_MIDI_OUTPUT_CALLBACK: u32 = 8;
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
    pub const K_AUDIO_UNIT_ERR_INVALID_PROPERTY_VALUE: i32 = -10851;
    pub const K_AUDIO_UNIT_ERR_PROPERTY_NOT_WRITABLE: i32 = -10865;
    pub const K_AUDIO_UNIT_ERR_CANT_DO_IN_CURRENT_CONTEXT: i32 = -10863;
}

/// Audio stream basic description - describes the format of audio data
///
/// This struct is the AU equivalent of describing audio format parameters like
/// sample rate, channel count, bit depth, etc.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AudioStreamBasicDescription {
    /// Sample rate in Hz (e.g., 44100.0, 48000.0)
    pub sample_rate: f64,
    /// Four-character code for format ID (e.g., 'lpcm' for Linear PCM)
    pub format_id: FourCharCode,
    /// Format-specific flags
    pub format_flags: u32,
    /// Number of bytes per packet
    pub bytes_per_packet: u32,
    /// Number of frames per packet (usually 1 for PCM)
    pub frames_per_packet: u32,
    /// Number of bytes per frame
    pub bytes_per_frame: u32,
    /// Number of channels per frame
    pub channels_per_frame: u32,
    /// Number of bits per channel
    pub bits_per_channel: u32,
    /// Reserved, must be 0
    pub reserved: u32,
}

/// Audio format constants
pub mod format_constants {
    use super::FourCharCode;

    /// Linear PCM format ID
    pub const K_AUDIO_FORMAT_LINEAR_PCM: FourCharCode = u32::from_be_bytes(*b"lpcm");

    /// Format flags for Linear PCM
    pub const K_AUDIO_FORMAT_FLAG_IS_FLOAT: u32 = 1 << 0;
    pub const K_AUDIO_FORMAT_FLAG_IS_BIG_ENDIAN: u32 = 1 << 1;
    pub const K_AUDIO_FORMAT_FLAG_IS_SIGNED_INTEGER: u32 = 1 << 2;
    pub const K_AUDIO_FORMAT_FLAG_IS_PACKED: u32 = 1 << 3;
    pub const K_AUDIO_FORMAT_FLAG_IS_ALIGNED_HIGH: u32 = 1 << 4;
    pub const K_AUDIO_FORMAT_FLAG_IS_NON_INTERLEAVED: u32 = 1 << 5;
    pub const K_AUDIO_FORMAT_FLAG_IS_NON_MIXABLE: u32 = 1 << 6;

    /// Standard flags for 32-bit float non-interleaved (most common for AU)
    #[cfg(target_endian = "little")]
    pub const K_AUDIO_FORMAT_FLAGS_NATIVE_FLOAT_PACKED: u32 =
        K_AUDIO_FORMAT_FLAG_IS_FLOAT | K_AUDIO_FORMAT_FLAG_IS_PACKED;

    #[cfg(target_endian = "big")]
    pub const K_AUDIO_FORMAT_FLAGS_NATIVE_FLOAT_PACKED: u32 =
        K_AUDIO_FORMAT_FLAG_IS_FLOAT | K_AUDIO_FORMAT_FLAG_IS_PACKED | K_AUDIO_FORMAT_FLAG_IS_BIG_ENDIAN;
}

impl AudioStreamBasicDescription {
    /// Create a canonical non-interleaved 32-bit float PCM description
    ///
    /// This is the standard format used by most Audio Units for audio processing.
    pub fn canonical_float(sample_rate: f64, channels: u32) -> Self {
        Self {
            sample_rate,
            format_id: format_constants::K_AUDIO_FORMAT_LINEAR_PCM,
            format_flags: format_constants::K_AUDIO_FORMAT_FLAGS_NATIVE_FLOAT_PACKED
                | format_constants::K_AUDIO_FORMAT_FLAG_IS_NON_INTERLEAVED,
            bytes_per_packet: 4,  // 4 bytes per float
            frames_per_packet: 1,
            bytes_per_frame: 4,   // 4 bytes per float
            channels_per_frame: channels,
            bits_per_channel: 32,
            reserved: 0,
        }
    }
}

/// Audio buffer structure used in AudioBufferList
///
/// Represents a single channel or interleaved buffer of audio data.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AudioBuffer {
    /// Number of interleaved channels in the buffer
    pub number_channels: u32,
    /// Size of the buffer in bytes
    pub data_bytes_size: u32,
    /// Pointer to the audio data
    pub data: *mut c_void,
}

/// Audio buffer list structure
///
/// A variable-length structure that contains multiple audio buffers.
/// For non-interleaved audio, each buffer contains one channel.
#[repr(C)]
pub struct AudioBufferList {
    /// Number of buffers in the list
    pub number_buffers: u32,
    /// Array of buffers (variable length)
    pub buffers: [AudioBuffer; 1],
}

impl AudioBufferList {
    /// Get a slice of the buffers in this list.
    ///
    /// # Safety
    /// This is unsafe because AudioBufferList is a variable-length structure.
    /// The caller must ensure that number_buffers is correct.
    pub unsafe fn buffers(&self) -> &[AudioBuffer] {
        std::slice::from_raw_parts(self.buffers.as_ptr(), self.number_buffers as usize)
    }

    /// Get a mutable slice of the buffers in this list.
    ///
    /// # Safety
    /// This is unsafe because AudioBufferList is a variable-length structure.
    /// The caller must ensure that number_buffers is correct.
    pub unsafe fn buffers_mut(&mut self) -> &mut [AudioBuffer] {
        std::slice::from_raw_parts_mut(self.buffers.as_mut_ptr(), self.number_buffers as usize)
    }
}

/// Audio timestamp structure
///
/// Contains timing information for audio rendering.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AudioTimeStamp {
    /// Sample time
    pub sample_time: f64,
    /// Host time (mach_absolute_time)
    pub host_time: u64,
    /// Rate scalar for adjusting timing
    pub rate_scalar: f64,
    /// Word clock time
    pub word_clock_time: u64,
    /// SMPTE time
    pub smpte_time: [u32; 8],
    /// Flags indicating which fields are valid
    pub flags: u32,
    /// Reserved
    pub reserved: u32,
}

/// Render action flags
pub mod render_flags {
    /// Indicates that rendering is complete
    pub const K_AUDIO_UNIT_RENDER_ACTION_OUTPUT_IS_SILENCE: u32 = 1 << 4;
}

/// The factory function signature for creating Audio Unit instances
///
/// This function is called by the Audio Component system to create new instances
/// of your audio unit. It should return a pointer to the AudioComponentPlugInInstance
/// structure for your plugin.
pub type AudioComponentFactoryFunction = unsafe extern "C" fn(
    desc: *const AudioComponentDescription,
) -> *mut c_void;

/// Audio Unit preset data structure
///
/// This structure is used to represent a preset in the Audio Unit system.
/// It contains the preset name and data (serialized state).
#[repr(C)]
#[derive(Debug, Clone)]
pub struct AUPreset {
    /// The preset number (0-based index for factory presets, -1 for user presets)
    pub preset_number: i32,
    /// The preset name (null-terminated C string)
    pub preset_name: *mut i8,
}

/// Audio Unit class info data structure
///
/// This structure contains information about the Audio Unit class.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct AUClassInfo {
    /// The class name (null-terminated C string)
    pub class_name: *mut i8,
    /// The class version
    pub class_version: u32,
    /// The class description (null-terminated C string)
    pub class_description: *mut i8,
}
