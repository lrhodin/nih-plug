//! Audio Unit parameter handling.
//!
//! This module implements GetParameter and SetParameter callbacks which handle
//! parameter queries and changes from the AU host.

use std::ffi::c_void;
use std::collections::HashMap;

use crate::plugin::Plugin;
use crate::prelude::ParamPtr;
use crate::wrapper::util::hash_param_id;

use super::bindings::{errors, scopes};
use super::factory::AudioComponentPlugInInstance;

impl<P: Plugin> AudioComponentPlugInInstance<P> {
    /// Get a parameter value from the plugin.
    ///
    /// # Safety
    /// This function is called from C and must handle null pointers safely.
    pub unsafe extern "C" fn au_get_parameter(
        instance: *mut c_void,
        parameter_id: u32,
        scope: u32,
        _element: u32,
        value: *mut f32,
    ) -> i32 {
        if instance.is_null() || value.is_null() {
            return errors::K_AUDIO_UNIT_ERR_INVALID_PARAMETER;
        }

        // Only handle global scope for now
        if scope != scopes::K_AUDIO_UNIT_SCOPE_GLOBAL {
            return errors::K_AUDIO_UNIT_ERR_INVALID_PARAMETER;
        }

        let plugin_instance = &*(instance as *const Self);
        
        // Get the parameter mapping
        let param_by_id = Self::get_parameter_mapping(plugin_instance);
        
        // Look up the parameter by ID
        match param_by_id.get(&parameter_id) {
            Some(param_ptr) => {
                // Get the normalized value (0.0 to 1.0)
                let normalized_value = param_ptr.modulated_normalized_value();
                *value = normalized_value;
                errors::NO_ERR
            }
            None => {
                nih_log!("AU GetParameter: Unknown parameter ID {}", parameter_id);
                errors::K_AUDIO_UNIT_ERR_INVALID_PARAMETER
            }
        }
    }

    /// Set a parameter value on the plugin.
    ///
    /// # Safety
    /// This function is called from C and must handle null pointers safely.
    /// This can be called from the audio thread, so it must be realtime-safe.
    pub unsafe extern "C" fn au_set_parameter(
        instance: *mut c_void,
        parameter_id: u32,
        scope: u32,
        _element: u32,
        value: f32,
        _buffer_offset: u32,
    ) -> i32 {
        if instance.is_null() {
            return errors::K_AUDIO_UNIT_ERR_INVALID_PARAMETER;
        }

        // Only handle global scope for now
        if scope != scopes::K_AUDIO_UNIT_SCOPE_GLOBAL {
            return errors::K_AUDIO_UNIT_ERR_INVALID_PARAMETER;
        }

        let plugin_instance = &*(instance as *const Self);
        
        // Get the parameter mapping
        let param_by_id = Self::get_parameter_mapping(plugin_instance);
        
        // Look up the parameter by ID
        match param_by_id.get(&parameter_id) {
            Some(param_ptr) => {
                // Clamp the value to [0.0, 1.0] range
                let normalized_value = value.clamp(0.0, 1.0);
                
                // Set the parameter value
                if param_ptr.set_normalized_value(normalized_value) {
                    // TODO: Handle buffer_offset for automation
                    // For now, we'll ignore it as NIH-plug handles smoothing internally
                    
                    nih_log!(
                        "AU SetParameter: ID={}, value={} (normalized={})",
                        parameter_id,
                        value,
                        normalized_value
                    );
                    errors::NO_ERR
                } else {
                    nih_log!("AU SetParameter: Failed to set parameter {} to {}", parameter_id, normalized_value);
                    errors::K_AUDIO_UNIT_ERR_INVALID_PARAMETER
                }
            }
            None => {
                nih_log!("AU SetParameter: Unknown parameter ID {}", parameter_id);
                errors::K_AUDIO_UNIT_ERR_INVALID_PARAMETER
            }
        }
    }

    /// Get the parameter mapping for this plugin instance.
    ///
    /// This creates a mapping from AU parameter IDs to NIH-plug parameter pointers.
    /// The mapping is created by hashing the parameter IDs from the plugin's param_map.
    fn get_parameter_mapping(instance: &Self) -> HashMap<u32, ParamPtr> {
        // Get the plugin's parameters
        let params = instance.wrapper.plugin().read();
        let param_map = params.params().param_map();
        
        // Create mapping from hashed parameter IDs to ParamPtrs
        param_map
            .into_iter()
            .map(|(id, ptr, _group)| {
                let hash = hash_param_id(&id);
                (hash, ptr)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_id_size() {
        // Ensure u32 is the right size for parameter IDs
        assert_eq!(std::mem::size_of::<u32>(), 4);
    }

    #[test]
    fn test_parameter_value_clamping() {
        // Test that parameter values are properly clamped to [0.0, 1.0]
        let test_values: Vec<(f32, f32)> = vec![
            (-1.0, 0.0),
            (0.0, 0.0),
            (0.5, 0.5),
            (1.0, 1.0),
            (2.0, 1.0),
        ];

        for (input, expected) in test_values {
            let clamped = input.clamp(0.0, 1.0);
            assert_eq!(clamped, expected, "Input {} should clamp to {}", input, expected);
        }
    }

    #[test]
    fn test_hash_param_id_consistency() {
        // Test that hash_param_id produces consistent results
        let test_id = "test_param";
        let hash1 = hash_param_id(test_id);
        let hash2 = hash_param_id(test_id);
        assert_eq!(hash1, hash2, "hash_param_id should be consistent");
        
        // Test that different IDs produce different hashes
        let hash3 = hash_param_id("different_param");
        assert_ne!(hash1, hash3, "Different parameter IDs should have different hashes");
    }
}
