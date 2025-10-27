//! Audio Unit property handlers.
//!
//! This module implements GetProperty and SetProperty callbacks which handle
//! configuration queries and changes from the AU host.

use std::ffi::c_void;
use std::ptr;

use crate::plugin::Plugin;

use super::bindings::{errors, property_ids, scopes, AudioStreamBasicDescription};
use super::factory::AudioComponentPlugInInstance;

/// Maximum number of frames per slice (buffer size).
///
/// This gets updated when the host calls SetProperty with kAudioUnitProperty_MaximumFramesPerSlice.
/// Default to 8192 which is a common maximum.
const DEFAULT_MAX_FRAMES_PER_SLICE: u32 = 8192;

impl<P: Plugin> AudioComponentPlugInInstance<P> {
    /// Get a property value from the plugin.
    ///
    /// This is called by the AU host to query various properties about the plugin's
    /// configuration and capabilities.
    ///
    /// # Safety
    /// This function is called from C and must handle null pointers and invalid parameters safely.
    pub unsafe extern "C" fn au_get_property(
        instance: *mut c_void,
        property_id: u32,
        scope: u32,
        element: u32,
        data: *mut c_void,
        data_size: *mut u32,
    ) -> i32 {
        if instance.is_null() || data_size.is_null() {
            return errors::K_AUDIO_UNIT_ERR_INVALID_PARAMETER;
        }

        let plugin_instance = &*(instance as *const Self);

        match property_id {
            property_ids::K_AUDIO_UNIT_PROPERTY_STREAM_FORMAT => {
                Self::get_stream_format(plugin_instance, scope, element, data, data_size)
            }
            property_ids::K_AUDIO_UNIT_PROPERTY_MAXIMUM_FRAMES_PER_SLICE => {
                Self::get_maximum_frames_per_slice(plugin_instance, data, data_size)
            }
            property_ids::K_AUDIO_UNIT_PROPERTY_SAMPLE_RATE => {
                Self::get_sample_rate(plugin_instance, scope, data, data_size)
            }
            property_ids::K_AUDIO_UNIT_PROPERTY_LATENCY => {
                Self::get_latency(plugin_instance, data, data_size)
            }
            _ => {
                nih_debug_assert!(
                    false,
                    "GetProperty called for unimplemented property: {}",
                    property_id
                );
                errors::K_AUDIO_UNIT_ERR_INVALID_PROPERTY
            }
        }
    }

    /// Set a property value on the plugin.
    ///
    /// This is called by the AU host to configure various aspects of the plugin.
    ///
    /// # Safety
    /// This function is called from C and must handle null pointers and invalid parameters safely.
    pub unsafe extern "C" fn au_set_property(
        instance: *mut c_void,
        property_id: u32,
        scope: u32,
        element: u32,
        data: *const c_void,
        data_size: u32,
    ) -> i32 {
        if instance.is_null() || data.is_null() {
            return errors::K_AUDIO_UNIT_ERR_INVALID_PARAMETER;
        }

        let plugin_instance = &mut *(instance as *mut Self);

        match property_id {
            property_ids::K_AUDIO_UNIT_PROPERTY_STREAM_FORMAT => {
                Self::set_stream_format(plugin_instance, scope, element, data, data_size)
            }
            property_ids::K_AUDIO_UNIT_PROPERTY_MAXIMUM_FRAMES_PER_SLICE => {
                Self::set_maximum_frames_per_slice(plugin_instance, data, data_size)
            }
            property_ids::K_AUDIO_UNIT_PROPERTY_SAMPLE_RATE => {
                Self::set_sample_rate(plugin_instance, scope, data, data_size)
            }
            _ => {
                nih_debug_assert!(
                    false,
                    "SetProperty called for unimplemented or read-only property: {}",
                    property_id
                );
                errors::K_AUDIO_UNIT_ERR_PROPERTY_NOT_WRITABLE
            }
        }
    }

    /// Get the stream format for input or output.
    unsafe fn get_stream_format(
        instance: &Self,
        scope: u32,
        _element: u32,
        data: *mut c_void,
        data_size: *mut u32,
    ) -> i32 {
        let required_size = std::mem::size_of::<AudioStreamBasicDescription>() as u32;

        // If data is null, just return the required size
        if data.is_null() {
            *data_size = required_size;
            return errors::NO_ERR;
        }

        // Check if the provided buffer is large enough
        if *data_size < required_size {
            return errors::K_AUDIO_UNIT_ERR_INVALID_PARAMETER;
        }

        // Get the current buffer configuration
        let buffer_config = instance.wrapper.buffer_config().read();
        let audio_io_layout = instance.wrapper.audio_io_layout().read();

        // Determine sample rate and channel count
        let sample_rate = buffer_config
            .as_ref()
            .map(|cfg| cfg.sample_rate as f64)
            .unwrap_or(44100.0);

        let channels = match scope {
            scopes::K_AUDIO_UNIT_SCOPE_INPUT => audio_io_layout
                .as_ref()
                .and_then(|layout| layout.main_input_channels)
                .map(|ch| ch.get())
                .unwrap_or(2),
            scopes::K_AUDIO_UNIT_SCOPE_OUTPUT => audio_io_layout
                .as_ref()
                .and_then(|layout| layout.main_output_channels)
                .map(|ch| ch.get())
                .unwrap_or(2),
            _ => return errors::K_AUDIO_UNIT_ERR_INVALID_PARAMETER,
        };

        // Create the stream description
        let format = AudioStreamBasicDescription::canonical_float(sample_rate, channels);

        // Copy to output buffer
        ptr::write(data as *mut AudioStreamBasicDescription, format);
        *data_size = required_size;

        errors::NO_ERR
    }

    /// Set the stream format for input or output.
    unsafe fn set_stream_format(
        instance: &mut Self,
        scope: u32,
        _element: u32,
        data: *const c_void,
        data_size: u32,
    ) -> i32 {
        let required_size = std::mem::size_of::<AudioStreamBasicDescription>() as u32;

        if data_size < required_size {
            return errors::K_AUDIO_UNIT_ERR_INVALID_PARAMETER;
        }

        let format = &*(data as *const AudioStreamBasicDescription);

        nih_log!(
            "SetProperty: StreamFormat on scope {} - rate={}, channels={}",
            scope,
            format.sample_rate,
            format.channels_per_frame
        );

        // Validate the format
        if format.format_id != super::bindings::format_constants::K_AUDIO_FORMAT_LINEAR_PCM {
            nih_log!("Unsupported audio format ID: {:08x}", format.format_id);
            return errors::K_AUDIO_UNIT_ERR_INVALID_PROPERTY_VALUE;
        }

        // We only support 32-bit float non-interleaved
        let expected_flags = super::bindings::format_constants::K_AUDIO_FORMAT_FLAGS_NATIVE_FLOAT_PACKED
            | super::bindings::format_constants::K_AUDIO_FORMAT_FLAG_IS_NON_INTERLEAVED;

        if format.format_flags != expected_flags {
            nih_log!(
                "Unsupported format flags: {:08x} (expected {:08x})",
                format.format_flags,
                expected_flags
            );
            return errors::K_AUDIO_UNIT_ERR_INVALID_PROPERTY_VALUE;
        }

        // Update the stored format based on scope
        // The actual initialization happens in au_initialize
        // Here we just validate and acknowledge the format

        errors::NO_ERR
    }

    /// Get the maximum frames per slice (buffer size).
    unsafe fn get_maximum_frames_per_slice(
        instance: &Self,
        data: *mut c_void,
        data_size: *mut u32,
    ) -> i32 {
        let required_size = std::mem::size_of::<u32>() as u32;

        if data.is_null() {
            *data_size = required_size;
            return errors::NO_ERR;
        }

        if *data_size < required_size {
            return errors::K_AUDIO_UNIT_ERR_INVALID_PARAMETER;
        }

        let max_frames = instance
            .wrapper
            .buffer_config()
            .read()
            .as_ref()
            .map(|cfg| cfg.max_buffer_size)
            .unwrap_or(DEFAULT_MAX_FRAMES_PER_SLICE);

        ptr::write(data as *mut u32, max_frames);
        *data_size = required_size;

        errors::NO_ERR
    }

    /// Set the maximum frames per slice (buffer size).
    #[allow(unused_variables)]
    unsafe fn set_maximum_frames_per_slice(
        instance: &mut Self,
        data: *const c_void,
        data_size: u32,
    ) -> i32 {
        let required_size = std::mem::size_of::<u32>() as u32;

        if data_size < required_size {
            return errors::K_AUDIO_UNIT_ERR_INVALID_PARAMETER;
        }

        let max_frames = *(data as *const u32);

        nih_log!("SetProperty: MaximumFramesPerSlice = {}", max_frames);

        // The max buffer size will be used during initialization
        // We don't need to store it separately as the wrapper handles this

        errors::NO_ERR
    }

    /// Get the sample rate.
    unsafe fn get_sample_rate(
        instance: &Self,
        _scope: u32,
        data: *mut c_void,
        data_size: *mut u32,
    ) -> i32 {
        let required_size = std::mem::size_of::<f64>() as u32;

        if data.is_null() {
            *data_size = required_size;
            return errors::NO_ERR;
        }

        if *data_size < required_size {
            return errors::K_AUDIO_UNIT_ERR_INVALID_PARAMETER;
        }

        let sample_rate = instance
            .wrapper
            .buffer_config()
            .read()
            .as_ref()
            .map(|cfg| cfg.sample_rate as f64)
            .unwrap_or(44100.0);

        ptr::write(data as *mut f64, sample_rate);
        *data_size = required_size;

        errors::NO_ERR
    }

    /// Set the sample rate.
    unsafe fn set_sample_rate(
        _instance: &mut Self,
        _scope: u32,
        data: *const c_void,
        data_size: u32,
    ) -> i32 {
        let required_size = std::mem::size_of::<f64>() as u32;

        if data_size < required_size {
            return errors::K_AUDIO_UNIT_ERR_INVALID_PARAMETER;
        }

        let sample_rate = *(data as *const f64);

        nih_log!("SetProperty: SampleRate = {}", sample_rate);

        // The sample rate will be used during initialization
        // We acknowledge it here but the actual configuration happens in au_initialize

        errors::NO_ERR
    }

    /// Get the plugin latency in samples.
    unsafe fn get_latency(instance: &Self, data: *mut c_void, data_size: *mut u32) -> i32 {
        let required_size = std::mem::size_of::<f64>() as u32;

        if data.is_null() {
            *data_size = required_size;
            return errors::NO_ERR;
        }

        if *data_size < required_size {
            return errors::K_AUDIO_UNIT_ERR_INVALID_PARAMETER;
        }

        // TODO: Track latency changes like VST3 wrapper does with AtomicU32
        // For now, return 0 latency. This will be implemented when we add
        // latency tracking infrastructure to the wrapper.
        let latency = 0.0;

        ptr::write(data as *mut f64, latency);
        *data_size = required_size;

        errors::NO_ERR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_float_format() {
        let format = AudioStreamBasicDescription::canonical_float(48000.0, 2);

        assert_eq!(format.sample_rate, 48000.0);
        assert_eq!(format.channels_per_frame, 2);
        assert_eq!(format.bits_per_channel, 32);
        assert_eq!(format.format_id, super::super::bindings::format_constants::K_AUDIO_FORMAT_LINEAR_PCM);
        assert!(format.format_flags & super::super::bindings::format_constants::K_AUDIO_FORMAT_FLAG_IS_FLOAT != 0);
        assert!(format.format_flags & super::super::bindings::format_constants::K_AUDIO_FORMAT_FLAG_IS_NON_INTERLEAVED != 0);
    }

    #[test]
    fn test_stream_format_size() {
        // Ensure AudioStreamBasicDescription has the expected size for C interop
        assert_eq!(
            std::mem::size_of::<AudioStreamBasicDescription>(),
            9 * std::mem::size_of::<u32>()
        );
    }
}
