//! The main Audio Unit wrapper implementation.
//!
//! This wrapper implements the Audio Component API and bridges between the AU host
//! and the NIH-plug Plugin trait.

use std::sync::Arc;
use parking_lot::RwLock;

use crate::plugin::Plugin;
use crate::prelude::{AudioIOLayout, BufferConfig, Params};

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
}

impl<P: Plugin> Drop for Wrapper<P> {
    fn drop(&mut self) {
        self.deactivate();
    }
}

// Audio Unit requires Send + Sync
unsafe impl<P: Plugin> Send for Wrapper<P> {}
unsafe impl<P: Plugin> Sync for Wrapper<P> {}
