//! The main Audio Unit wrapper implementation.
//!
//! This wrapper implements the Audio Component API and bridges between the AU host
//! and the NIH-plug Plugin trait.

use std::sync::Arc;
use std::collections::VecDeque;
use parking_lot::RwLock;

use crate::plugin::Plugin;
use crate::prelude::{AudioIOLayout, BufferConfig, Params};

use super::parameters::ParameterChangeEvent;

use super::context::WrapperInitContext;

/// The actual wrapper that interfaces between the AU host and the NIH-plug plugin.
///
/// This struct maintains the plugin state and handles all AU callbacks.
pub struct Wrapper<P: Plugin> {
    /// The plugin instance.
    plugin: RwLock<P>,

    /// The plugin's parameters. These are wrapped in an `Arc` so they can be shared
    /// with the GUI and other parts of the plugin.
    params: Arc<dyn Params>,

    /// The current buffer configuration, set during initialization.
    buffer_config: RwLock<Option<BufferConfig>>,

    /// The current audio I/O layout, set during initialization.
    audio_io_layout: RwLock<Option<AudioIOLayout>>,

    /// Queue of parameter change events for sample-accurate automation.
    /// These events are applied during audio processing at their specified buffer offsets.
    parameter_change_queue: RwLock<VecDeque<ParameterChangeEvent>>,
}

impl<P: Plugin> Wrapper<P> {
    /// Create a new wrapper instance.
    pub fn new() -> Arc<Self> {
        let plugin = RwLock::new(P::default());
        let params = {
            let plugin_guard = plugin.read();
            plugin_guard.params()
        };

        Arc::new(Self {
            plugin,
            params,
            buffer_config: RwLock::new(None),
            audio_io_layout: RwLock::new(None),
            parameter_change_queue: RwLock::new(VecDeque::new()),
        })
    }

    /// Initialize the plugin with the given buffer configuration and I/O layout.
    ///
    /// This is called by the AU host when the plugin is first loaded or when the
    /// audio configuration changes.
    pub fn initialize(&self, buffer_config: BufferConfig, audio_io_layout: AudioIOLayout) -> bool {
        *self.buffer_config.write() = Some(buffer_config.clone());
        *self.audio_io_layout.write() = Some(audio_io_layout.clone());

        let mut plugin = self.plugin.write();

        // Create the init context
        let mut context = WrapperInitContext {
            _phantom: std::marker::PhantomData,
            buffer_config: &buffer_config,
            audio_io_layout: &audio_io_layout,
        };

        // Call the plugin's initialize method
        if !plugin.initialize(&audio_io_layout, &buffer_config, &mut context) {
            nih_log!("Plugin initialization failed");
            return false;
        }

        // Call reset immediately after initialization
        plugin.reset();

        true
    }

    /// Process an audio buffer.
    ///
    /// This is called by the AU host for each audio buffer that needs to be processed.
    pub fn process(&self, _num_frames: usize) -> i32 {
        // TODO: Implement actual audio processing
        // This will:
        // 1. Get the plugin instance
        // 2. Create a Buffer from the AU buffers
        // 3. Create a ProcessContext
        // 4. Call plugin.process()
        // 5. Return the result

        super::util::AU_NO_ERROR
    }

    /// Deactivate the plugin.
    ///
    /// This is called when the AU host stops using the plugin.
    pub fn deactivate(&self) {
        let mut plugin = self.plugin.write();
        plugin.deactivate();
    }

    /// Notify about a parameter change.
    ///
    /// This is called when a parameter value changes to allow the plugin
    /// to respond to the change (e.g., update GUI, trigger recomputation).
    pub fn notify_parameter_change(&self, param_id: u32, normalized_value: f32) {
        // For now, we'll just log the parameter change
        // In a full implementation, this would notify the GUI or other systems
        nih_log!(
            "AU Parameter change notification: ID={}, value={}",
            param_id,
            normalized_value
        );
        
        // TODO: Implement proper parameter change notification system
        // This could involve:
        // 1. Notifying the GUI about the change
        // 2. Triggering plugin recomputation if needed
        // 3. Updating any cached values
    }

    /// Schedule a parameter change for sample-accurate automation.
    ///
    /// This adds a parameter change event to the queue with its buffer offset
    /// timing. The change will be applied during audio processing at the
    /// specified sample offset.
    pub fn schedule_parameter_change(&self, param_id: u32, normalized_value: f32, buffer_offset: u32) {
        let event = ParameterChangeEvent {
            parameter_id: param_id,
            normalized_value,
            buffer_offset,
        };
        
        let mut queue = self.parameter_change_queue.write();
        queue.push_back(event);
        
        nih_log!(
            "AU Scheduled parameter change: ID={}, value={}, offset={}",
            param_id,
            normalized_value,
            buffer_offset
        );
    }

    /// Get the next parameter change event for the current buffer offset.
    ///
    /// This returns the next parameter change event that should be applied
    /// at or before the given buffer offset. The event is removed from the queue.
    pub fn get_next_parameter_change(&self, current_offset: u32) -> Option<ParameterChangeEvent> {
        let mut queue = self.parameter_change_queue.write();
        
        // Find the first event that should be applied at or before current_offset
        if let Some(pos) = queue.iter().position(|event| event.buffer_offset <= current_offset) {
            queue.remove(pos)
        } else {
            None
        }
    }

    /// Clear all pending parameter change events.
    ///
    /// This is called when the plugin is reset or deactivated.
    pub fn clear_parameter_changes(&self) {
        let mut queue = self.parameter_change_queue.write();
        queue.clear();
        nih_log!("AU Cleared all pending parameter changes");
    }

    /// Get a reference to the buffer configuration.
    pub(crate) fn buffer_config(&self) -> &RwLock<Option<BufferConfig>> {
        &self.buffer_config
    }

    /// Get a reference to the audio I/O layout.
    pub(crate) fn audio_io_layout(&self) -> &RwLock<Option<AudioIOLayout>> {
        &self.audio_io_layout
    }

    /// Get a reference to the plugin instance.
    pub(crate) fn plugin(&self) -> &RwLock<P> {
        &self.plugin
    }

    /// Get a reference to the parameters.
    #[allow(dead_code)]
    pub(crate) fn params(&self) -> &Arc<dyn Params> {
        &self.params
    }
}

impl<P: Plugin> Drop for Wrapper<P> {
    fn drop(&mut self) {
        self.deactivate();
    }
}

// Audio Unit requires Send + Sync
unsafe impl<P: Plugin> Send for Wrapper<P> {}
unsafe impl<P: Plugin> Sync for Wrapper<P> {}
