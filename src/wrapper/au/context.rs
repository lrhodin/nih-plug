//! Audio Unit plugin context implementation.
//!
//! This module provides the `InitContext`, `ProcessContext`, and other context implementations
//! specific to Audio Units.

use crate::context::init::InitContext;
use crate::context::process::{ProcessContext, Transport};
use crate::context::PluginApi;
use crate::midi::NoteEvent;
use crate::plugin::Plugin;
use crate::prelude::{AudioIOLayout, BufferConfig};

/// A [`InitContext`] implementation for the Audio Unit wrapper.
pub(crate) struct WrapperInitContext<'a, P: Plugin> {
    pub(super) _phantom: std::marker::PhantomData<&'a P>,
    pub(super) buffer_config: &'a BufferConfig,
    pub(super) audio_io_layout: &'a AudioIOLayout,
}

impl<P: Plugin> InitContext<P> for WrapperInitContext<'_, P> {
    fn plugin_api(&self) -> PluginApi {
        PluginApi::AudioUnit
    }

    fn execute(&self, task: P::BackgroundTask) {
        // Execute the task directly on this thread during initialization
        // TODO: Add proper task executor integration
        nih_debug_assert_failure!("Background task execution not yet implemented for AU");
        drop(task);
    }

    fn set_latency_samples(&self, _samples: u32) {
        // TODO: Implement latency reporting for AU
        nih_debug_assert_failure!("Latency reporting not yet implemented for AU");
    }

    fn set_current_voice_capacity(&self, _capacity: u32) {
        // TODO: Implement voice capacity for AU (polyphonic plugins)
        nih_debug_assert_failure!("Voice capacity not yet implemented for AU");
    }
}

/// A [`ProcessContext`] implementation for the Audio Unit wrapper.
pub(crate) struct WrapperProcessContext<'a, P: Plugin> {
    pub(super) _phantom: std::marker::PhantomData<&'a P>,
    pub(super) sample_rate: f32,
    pub(super) transport: Transport,
    pub(super) wrapper: &'a super::wrapper::Wrapper<P>,
}

impl<'a, P: Plugin> WrapperProcessContext<'a, P> {
    /// Create a new process context from an AudioComponentPlugInInstance.
    ///
    /// This extracts the necessary information (sample rate, transport) from the plugin instance.
    pub(crate) fn new(
        instance: &'a super::factory::AudioComponentPlugInInstance<P>,
    ) -> Self {
        let sample_rate = instance
            .wrapper
            .buffer_config()
            .read()
            .as_ref()
            .map(|cfg| cfg.sample_rate)
            .unwrap_or(44100.0);

        // TODO: Extract transport info from AU host
        let transport = Transport::new(sample_rate);

        Self {
            _phantom: std::marker::PhantomData,
            sample_rate,
            transport,
            wrapper: &instance.wrapper,
        }
    }
}

impl<P: Plugin> ProcessContext<P> for WrapperProcessContext<'_, P> {
    fn plugin_api(&self) -> PluginApi {
        PluginApi::AudioUnit
    }

    fn execute_background(&self, _task: P::BackgroundTask) {
        // TODO: Implement background task execution for AU
        nih_debug_assert_failure!("Background tasks not yet implemented for AU");
    }

    fn execute_gui(&self, _task: P::BackgroundTask) {
        // TODO: Implement GUI task execution for AU
        nih_debug_assert_failure!("GUI tasks not yet implemented for AU");
    }

    fn transport(&self) -> &Transport {
        // TODO: Implement transport info from AU host
        &self.transport
    }

    fn next_event(&mut self) -> Option<NoteEvent<P::SysExMessage>> {
        // Get the next MIDI event from the wrapper's queue
        self.wrapper.get_next_midi_event()
    }

    fn send_event(&mut self, _event: NoteEvent<P::SysExMessage>) {
        // TODO: Implement sending events to AU host
        nih_debug_assert_failure!("Sending events not yet implemented for AU");
    }

    fn set_latency_samples(&self, _samples: u32) {
        // TODO: Implement runtime latency changes for AU
        nih_debug_assert_failure!("Runtime latency changes not yet implemented for AU");
    }

    fn set_current_voice_capacity(&self, _capacity: u32) {
        // TODO: Implement runtime voice capacity changes for AU
        nih_debug_assert_failure!("Runtime voice capacity not yet implemented for AU");
    }
}
