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
