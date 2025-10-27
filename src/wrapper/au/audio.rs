//! Audio Unit audio rendering callbacks.
//!
//! This module implements the Render callback which is called by the host
//! to process audio buffers.

use std::ffi::c_void;

use crate::buffer::Buffer;
use crate::plugin::Plugin;
use crate::prelude::AuxiliaryBuffers;

use super::bindings::{errors, AudioBufferList, AudioTimeStamp};
use super::context::WrapperProcessContext;
use super::factory::AudioComponentPlugInInstance;

impl<P: Plugin> AudioComponentPlugInInstance<P> {
    /// Render audio for the Audio Unit.
    ///
    /// This is the main audio processing callback. It's called by the host
    /// with input audio and expects output audio in return.
    ///
    /// # Safety
    /// This function is called from C in a realtime audio context.
    /// It must not allocate, lock, or perform any blocking operations.
    #[allow(unused_variables)]
    pub unsafe extern "C" fn au_render(
        instance: *mut c_void,
        io_action_flags: *mut u32,
        in_time_stamp: *const AudioTimeStamp,
        in_bus_number: u32,
        in_number_frames: u32,
        io_data: *mut AudioBufferList,
    ) -> i32 {
        if instance.is_null() || io_data.is_null() {
            return errors::K_AUDIO_UNIT_ERR_INVALID_PARAMETER;
        }

        let plugin_instance = &*(instance as *const Self);

        // Get the buffer list
        let buffer_list = &mut *io_data;
        let buffers = buffer_list.buffers_mut();

        // Check if we have any buffers
        if buffers.is_empty() {
            return errors::K_AUDIO_UNIT_ERR_INVALID_PARAMETER;
        }

        // Get the number of channels from the buffer list
        // For non-interleaved audio, each buffer is one channel
        let num_channels = buffers.len();
        let num_frames = in_number_frames as usize;

        // Validate buffer sizes
        for buffer in buffers.iter() {
            let expected_size = (num_frames * std::mem::size_of::<f32>()) as u32;
            if buffer.data_bytes_size < expected_size {
                nih_log!(
                    "AU Render: buffer size mismatch, expected at least {}, got {}",
                    expected_size,
                    buffer.data_bytes_size
                );
                return errors::K_AUDIO_UNIT_ERR_INVALID_PARAMETER;
            }
        }

        // Create a NIH-plug Buffer from the AU buffers
        // We need to convert raw pointers to slices and then set up the Buffer
        let mut output_slices: Vec<&mut [f32]> = buffers
            .iter_mut()
            .map(|buf| {
                std::slice::from_raw_parts_mut(
                    buf.data as *mut f32,
                    num_frames
                )
            })
            .collect();

        // Create the buffer using set_slices
        let mut buffer_storage = Buffer::default();
        buffer_storage.set_slices(num_frames, |slices| {
            slices.clear();
            slices.extend(output_slices.iter_mut().map(|s| &mut s[..]));
        });

        let mut buffer = buffer_storage;

        // Create empty auxiliary buffers
        // TODO: Handle auxiliary inputs/outputs when needed
        let mut aux = AuxiliaryBuffers {
            inputs: &mut [],
            outputs: &mut [],
        };

        // Create the process context
        // TODO: Extract timing information from in_time_stamp
        // TODO: Handle transport state
        let mut context = WrapperProcessContext::new(plugin_instance);

        // Process scheduled parameter changes for sample-accurate automation
        // This needs to be done before audio processing
        Self::process_scheduled_parameter_changes(plugin_instance, num_frames);

        // Process MIDI events for the current buffer
        // This needs to be done before audio processing
        Self::process_midi_events(plugin_instance, num_frames);

        // Process the audio
        let result = {
            let mut plugin = plugin_instance.wrapper.plugin().write();
            (*plugin).process(&mut buffer, &mut aux, &mut context)
        };

        // Handle the processing result
        use crate::plugin::ProcessStatus;
        match result {
            ProcessStatus::Normal => errors::NO_ERR,
            ProcessStatus::Tail(_) => {
                // TODO: Handle tail length reporting
                errors::NO_ERR
            }
            ProcessStatus::KeepAlive => errors::NO_ERR,
            ProcessStatus::Error(msg) => {
                nih_log!("AU Render: Plugin returned error: {}", msg);
                // Return success anyway to avoid crashing the host
                // The plugin should handle errors gracefully
                errors::NO_ERR
            }
        }
    }

    /// Process scheduled parameter changes for sample-accurate automation.
    ///
    /// This method applies any parameter changes that are scheduled for the current
    /// audio buffer. It processes changes in order of their buffer offsets.
    fn process_scheduled_parameter_changes(plugin_instance: &Self, num_frames: usize) {
        // Get the parameter mapping
        let param_by_id = Self::get_parameter_mapping(plugin_instance);
        
        // Process parameter changes for the entire buffer
        // In a full implementation, this would be done in smaller chunks
        // for true sample-accurate automation
        for frame_offset in 0..num_frames {
            if let Some(event) = plugin_instance.wrapper.get_next_parameter_change(frame_offset as u32) {
                // Apply the parameter change
                if let Some(param_ptr) = param_by_id.get(&event.parameter_id) {
                    if unsafe { param_ptr.set_normalized_value(event.normalized_value) } {
                        nih_log!(
                            "AU Applied scheduled parameter change: ID={}, value={}, offset={}",
                            event.parameter_id,
                            event.normalized_value,
                            event.buffer_offset
                        );
                    } else {
                        nih_log!(
                            "AU Failed to apply scheduled parameter change: ID={}, value={}",
                            event.parameter_id,
                            event.normalized_value
                        );
                    }
                } else {
                    nih_log!(
                        "AU Unknown parameter ID in scheduled change: {}",
                        event.parameter_id
                    );
                }
            }
        }
    }

    /// Process MIDI events for the current audio buffer.
    ///
    /// This method processes any MIDI events that are queued for the current
    /// audio buffer. In a full implementation, this would handle MIDI events
    /// from the AU host.
    fn process_midi_events(plugin_instance: &Self, _num_frames: usize) {
        // For now, we don't have a way to receive MIDI events from the AU host
        // This would need to be implemented through the AU MIDI system
        // which typically requires the plugin to be registered as a MusicDevice
        // or MusicEffect component type
        
        // TODO: Implement MIDI event processing from AU host
        // This would involve:
        // 1. Registering the plugin as a MusicDevice/MusicEffect component
        // 2. Implementing MIDI event callbacks from the host
        // 3. Converting AU MIDI events to NIH-plug NoteEvent format
        // 4. Adding events to the wrapper's MIDI event queue
        
        // For now, we just clear any existing MIDI events
        plugin_instance.wrapper.clear_midi_events();
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_size_calculation() {
        // Test that our buffer size calculation is correct
        let num_frames = 512;
        let expected_bytes = (num_frames * std::mem::size_of::<f32>()) as u32;
        assert_eq!(expected_bytes, 2048); // 512 frames * 4 bytes per f32
    }
}
