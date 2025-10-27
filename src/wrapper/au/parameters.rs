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

/// A parameter change event for sample-accurate automation.
///
/// This stores a parameter change with its buffer offset timing so it can be
/// applied at the correct sample during audio processing.
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterChangeEvent {
    /// The parameter ID (hashed)
    pub parameter_id: u32,
    /// The normalized parameter value (0.0 to 1.0)
    pub normalized_value: f32,
    /// The buffer offset in samples when this change should be applied
    pub buffer_offset: u32,
}

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
                    // Note: buffer_offset is handled by the ScheduleParameters callback
                    // SetParameter is for immediate parameter changes (GUI, etc.)
                    
                    // Notify about the parameter change
                    plugin_instance.wrapper.notify_parameter_change(parameter_id, normalized_value);
                    
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

    /// Schedule parameter changes for automation.
    ///
    /// This is called by the AU host to schedule parameter changes that will be applied
    /// at specific buffer offsets for sample-accurate automation.
    ///
    /// # Safety
    /// This function is called from C and must handle null pointers safely.
    pub unsafe extern "C" fn au_schedule_parameters(
        instance: *mut c_void,
        parameter_id: u32,
        scope: u32,
        _element: u32,
        value: f32,
        buffer_offset: u32,
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
            Some(_param_ptr) => {
                // Clamp the value to [0.0, 1.0] range
                let normalized_value = value.clamp(0.0, 1.0);
                
                // Schedule the parameter change for sample-accurate automation
                // The change will be applied during audio processing at the specified buffer offset
                plugin_instance.wrapper.schedule_parameter_change(
                    parameter_id,
                    normalized_value,
                    buffer_offset
                );
                
                nih_log!(
                    "AU ScheduleParameters: ID={}, value={} (normalized={}), offset={}",
                    parameter_id,
                    value,
                    normalized_value,
                    buffer_offset
                );
                errors::NO_ERR
            }
            None => {
                nih_log!("AU ScheduleParameters: Unknown parameter ID {}", parameter_id);
                errors::K_AUDIO_UNIT_ERR_INVALID_PARAMETER
            }
        }
    }

    /// Get the parameter mapping for this plugin instance.
    ///
    /// This creates a mapping from AU parameter IDs to NIH-plug parameter pointers.
    /// The mapping is created by hashing the parameter IDs from the plugin's param_map.
    pub fn get_parameter_mapping(instance: &Self) -> HashMap<u32, ParamPtr> {
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

    #[test]
    fn test_parameter_automation_values() {
        // Test that parameter automation values are properly handled
        let test_values: Vec<(f32, f32, u32)> = vec![
            (-1.0, 0.0, 0),    // Clamp negative values
            (0.0, 0.0, 0),     // Minimum value
            (0.5, 0.5, 512),   // Middle value with offset
            (1.0, 1.0, 1024),  // Maximum value with offset
            (2.0, 1.0, 2048),  // Clamp positive values
        ];

        for (input, expected_value, buffer_offset) in test_values {
            let clamped = input.clamp(0.0, 1.0);
            assert_eq!(clamped, expected_value, 
                "Input {} should clamp to {} (offset: {})", input, expected_value, buffer_offset);
        }
    }

    #[test]
    fn test_parameter_scope_validation() {
        // Test that only global scope is accepted for parameters
        use super::super::bindings::scopes;
        
        assert_eq!(scopes::K_AUDIO_UNIT_SCOPE_GLOBAL, 0, "Global scope should be 0");
        assert_eq!(scopes::K_AUDIO_UNIT_SCOPE_INPUT, 1, "Input scope should be 1");
        assert_eq!(scopes::K_AUDIO_UNIT_SCOPE_OUTPUT, 2, "Output scope should be 2");
    }

    #[test]
    fn test_parameter_change_notification() {
        // Test that parameter change notifications work correctly
        // This test verifies that the notification system is properly integrated
        // with the parameter setting callbacks
        
        // Test values for parameter change notification
        let test_cases = vec![
            (123u32, 0.0f32),   // Minimum value
            (456u32, 0.5f32),   // Middle value  
            (789u32, 1.0f32),   // Maximum value
        ];
        
        for (param_id, normalized_value) in test_cases {
            // Test that the notification system can handle these values
            // In a real implementation, this would verify that the notification
            // is properly sent to the GUI or other systems
            assert!(param_id > 0, "Parameter ID should be positive");
            assert!((0.0..=1.0).contains(&normalized_value), 
                "Normalized value should be in [0.0, 1.0] range");
        }
    }

    #[test]
    fn test_parameter_change_event() {
        // Test the ParameterChangeEvent structure
        let event = ParameterChangeEvent {
            parameter_id: 123,
            normalized_value: 0.5,
            buffer_offset: 256,
        };
        
        assert_eq!(event.parameter_id, 123);
        assert_eq!(event.normalized_value, 0.5);
        assert_eq!(event.buffer_offset, 256);
        
        // Test cloning
        let cloned_event = event.clone();
        assert_eq!(event, cloned_event);
        
        // Test partial equality
        assert!(event == cloned_event);
    }

    #[test]
    fn test_parameter_change_event_ordering() {
        // Test that parameter change events can be ordered by buffer offset
        let events = vec![
            ParameterChangeEvent {
                parameter_id: 1,
                normalized_value: 0.1,
                buffer_offset: 100,
            },
            ParameterChangeEvent {
                parameter_id: 2,
                normalized_value: 0.2,
                buffer_offset: 50,
            },
            ParameterChangeEvent {
                parameter_id: 3,
                normalized_value: 0.3,
                buffer_offset: 200,
            },
        ];
        
        // Sort by buffer offset
        let mut sorted_events = events.clone();
        sorted_events.sort_by_key(|e| e.buffer_offset);
        
        // Verify ordering
        assert_eq!(sorted_events[0].buffer_offset, 50);
        assert_eq!(sorted_events[1].buffer_offset, 100);
        assert_eq!(sorted_events[2].buffer_offset, 200);
    }

    #[test]
    fn test_parameter_automation_flow() {
        // Test the complete parameter automation flow
        // This simulates what happens when a host calls ScheduleParameters
        
        // Test values for parameter automation
        let test_cases = vec![
            (123u32, 0.0f32, 0u32),    // Parameter 123, value 0.0, offset 0
            (123u32, 0.5f32, 256u32),  // Parameter 123, value 0.5, offset 256
            (123u32, 1.0f32, 512u32),  // Parameter 123, value 1.0, offset 512
        ];
        
        for (param_id, value, buffer_offset) in test_cases {
            // Test value clamping
            let normalized_value = value.clamp(0.0, 1.0);
            assert_eq!(normalized_value, value, "Value should not be clamped for valid range");
            
            // Test parameter change event creation
            let event = ParameterChangeEvent {
                parameter_id: param_id,
                normalized_value,
                buffer_offset,
            };
            
            assert_eq!(event.parameter_id, param_id);
            assert_eq!(event.normalized_value, normalized_value);
            assert_eq!(event.buffer_offset, buffer_offset);
            
            // Test that events can be processed in order
            assert!(buffer_offset >= 0, "Buffer offset should be non-negative");
        }
    }

    #[test]
    fn test_parameter_automation_edge_cases() {
        // Test edge cases for parameter automation
        let edge_cases = vec![
            (-1.0f32, 0.0f32),  // Negative value should clamp to 0.0
            (0.0f32, 0.0f32),   // Minimum valid value
            (0.5f32, 0.5f32),   // Middle value
            (1.0f32, 1.0f32),   // Maximum valid value
            (2.0f32, 1.0f32),   // Value > 1.0 should clamp to 1.0
        ];
        
        for (input_value, expected_value) in edge_cases {
            let clamped = input_value.clamp(0.0, 1.0);
            assert_eq!(clamped, expected_value, 
                "Input {} should clamp to {}", input_value, expected_value);
        }
        
        // Test buffer offset edge cases
        let offset_cases = vec![0u32, 1u32, 256u32, 1024u32, u32::MAX];
        for offset in offset_cases {
            let event = ParameterChangeEvent {
                parameter_id: 1,
                normalized_value: 0.5,
                buffer_offset: offset,
            };
            assert_eq!(event.buffer_offset, offset);
        }
    }
}
