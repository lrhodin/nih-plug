//! Audio Unit initialization and lifecycle callbacks.
//!
//! This module implements the Initialize and Uninitialize callbacks which handle
//! plugin lifecycle management.

use std::ffi::c_void;
use std::num::NonZeroU32;

use crate::plugin::Plugin;
use crate::prelude::{AudioIOLayout, BufferConfig, PortNames, ProcessMode};

use super::bindings::errors;
use super::factory::AudioComponentPlugInInstance;

/// Default sample rate to use if none is specified
const DEFAULT_SAMPLE_RATE: f32 = 44100.0;

/// Default max buffer size to use if none is specified
const DEFAULT_MAX_BUFFER_SIZE: u32 = 8192;

impl<P: Plugin> AudioComponentPlugInInstance<P> {
    /// Initialize the Audio Unit.
    ///
    /// This is called by the host when the plugin should prepare for processing.
    /// We need to:
    /// 1. Determine the audio configuration (sample rate, channels, buffer size)
    /// 2. Select an appropriate AudioIOLayout for the plugin
    /// 3. Call the plugin's initialize method with the configuration
    ///
    /// # Safety
    /// This function is called from C and must handle null pointers safely.
    pub unsafe extern "C" fn au_initialize(instance: *mut c_void) -> i32 {
        if instance.is_null() {
            return errors::K_AUDIO_UNIT_ERR_INVALID_PARAMETER;
        }

        let plugin_instance = &*(instance as *const Self);

        // Get current buffer config if it was set via properties
        // Otherwise, use defaults
        let (sample_rate, max_buffer_size) = {
            let config = plugin_instance.wrapper.buffer_config().read();
            match config.as_ref() {
                Some(cfg) => (cfg.sample_rate, cfg.max_buffer_size),
                None => (DEFAULT_SAMPLE_RATE, DEFAULT_MAX_BUFFER_SIZE),
            }
        };

        // Get the audio I/O layout
        // For now, we'll use a simple stereo in/out configuration
        // TODO: Query the plugin's supported layouts and choose the best one
        let audio_io_layout = {
            let existing_layout = plugin_instance.wrapper.audio_io_layout().read();
            match existing_layout.as_ref() {
                Some(layout) => layout.clone(),
                None => {
                    // Default to stereo in/out
                    AudioIOLayout {
                        main_input_channels: NonZeroU32::new(2),
                        main_output_channels: NonZeroU32::new(2),
                        aux_input_ports: &[],
                        aux_output_ports: &[],
                        names: PortNames::const_default(),
                    }
                }
            }
        };

        // Create the buffer configuration
        let buffer_config = BufferConfig {
            sample_rate,
            min_buffer_size: None,
            max_buffer_size,
            process_mode: ProcessMode::Realtime,
        };

        nih_log!(
            "AU Initialize: sample_rate={}, max_buffer_size={}, layout={:?}",
            sample_rate,
            max_buffer_size,
            audio_io_layout
        );

        // Call the wrapper's initialize method
        if !plugin_instance
            .wrapper
            .initialize(buffer_config, audio_io_layout)
        {
            nih_log!("AU Initialize failed");
            return errors::K_AUDIO_UNIT_ERR_UNINITIALIZED;
        }

        nih_log!("AU Initialize succeeded");
        errors::NO_ERR
    }

    /// Uninitialize the Audio Unit.
    ///
    /// This is called by the host when the plugin should clean up and stop processing.
    ///
    /// # Safety
    /// This function is called from C and must handle null pointers safely.
    pub unsafe extern "C" fn au_uninitialize(instance: *mut c_void) -> i32 {
        if instance.is_null() {
            return errors::K_AUDIO_UNIT_ERR_INVALID_PARAMETER;
        }

        let plugin_instance = &*(instance as *const Self);

        nih_log!("AU Uninitialize");

        // Deactivate the plugin
        plugin_instance.wrapper.deactivate();

        errors::NO_ERR
    }

    /// Reset the Audio Unit.
    ///
    /// This is called by the host when the plugin should clear its internal state
    /// (e.g., delay lines, reverb tails, etc.) without being deactivated.
    ///
    /// # Safety
    /// This function is called from C and must handle null pointers safely.
    pub unsafe extern "C" fn au_reset(instance: *mut c_void) -> i32 {
        if instance.is_null() {
            return errors::K_AUDIO_UNIT_ERR_INVALID_PARAMETER;
        }

        let plugin_instance = &*(instance as *const Self);

        nih_log!("AU Reset");

        // Call the plugin's reset method
        {
            let mut plugin = plugin_instance.wrapper.plugin().write();
            (*plugin).reset();
        }

        errors::NO_ERR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_config_defaults() {
        // Ensure our defaults are reasonable
        assert!(DEFAULT_SAMPLE_RATE > 0.0);
        assert!(DEFAULT_MAX_BUFFER_SIZE > 0);
        assert!(DEFAULT_MAX_BUFFER_SIZE <= 16384); // Should be reasonable
    }
}
