use nih_plug::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

struct TrackInfoTest {
    params: Arc<TrackInfoParams>,
    /// Counter to throttle logging (log every N process calls)
    process_counter: AtomicU32,
}

#[derive(Params)]
struct TrackInfoParams {}

impl Default for TrackInfoTest {
    fn default() -> Self {
        Self {
            params: Arc::new(TrackInfoParams {}),
            process_counter: AtomicU32::new(0),
        }
    }
}

impl Default for TrackInfoParams {
    fn default() -> Self {
        Self {}
    }
}

impl Plugin for TrackInfoTest {
    const NAME: &'static str = "Track Info Test";
    const VENDOR: &'static str = "NIH-plug";
    const URL: &'static str = "https://github.com/robbert-vdh/nih-plug";
    const EMAIL: &'static str = "info@example.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
    ];

    const SAMPLE_ACCURATE_AUTOMATION: bool = false;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn process(
        &mut self,
        _buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Log track info every 2000 process calls to avoid spam
        let counter = self.process_counter.fetch_add(1, Ordering::Relaxed);
        if counter % 2000 == 0 {
            if let Some(track_info) = context.track_info() {
                nih_log!(
                    "=== CLAP TRACK INFO RECEIVED ==="
                );
                nih_log!(
                    "  Track Name: {}",
                    track_info.name.as_ref().map(|s| s.as_str()).unwrap_or("<none>")
                );
                if let Some((r, g, b, a)) = track_info.color {
                    nih_log!("  Track Color: RGBA({}, {}, {}, {})", r, g, b, a);
                } else {
                    nih_log!("  Track Color: <none>");
                }
                nih_log!(
                    "  Track Type: master={}, bus={}, return={}",
                    track_info.track_type.is_master,
                    track_info.track_type.is_bus,
                    track_info.track_type.is_return
                );
                if let Some(index) = track_info.index {
                    nih_log!("  Track Index: {}", index);
                }
                if let Some(uid) = &track_info.uid {
                    nih_log!("  Track UID: {}", uid);
                }
                nih_log!("================================");
            } else {
                nih_log!("CLAP TRACK INFO: None available from host");
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for TrackInfoTest {
    const CLAP_ID: &'static str = "com.nih-plug.track-info-test";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Test plugin for verifying CLAP track-info extension");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Utility,
    ];
}

nih_export_clap!(TrackInfoTest);
