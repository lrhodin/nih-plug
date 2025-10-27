use nih_plug::prelude::*;
use std::sync::Arc;

/// A simple gain plugin for testing AU functionality
struct GainAuTest {
    params: Arc<GainAuTestParams>,
}

#[derive(Params)]
struct GainAuTestParams {
    /// The gain parameter
    #[id = "gain"]
    pub gain: FloatParam,
}

impl Default for GainAuTest {
    fn default() -> Self {
        Self {
            params: Arc::new(GainAuTestParams::default()),
        }
    }
}

impl Default for GainAuTestParams {
    fn default() -> Self {
        Self {
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
        }
    }
}

impl Plugin for GainAuTest {
    const NAME: &'static str = "Gain AU Test";
    const VENDOR: &'static str = "NIH-Plug AU Test";
    const URL: &'static str = "https://github.com/robbert-vdh/nih-plug";
    const EMAIL: &'static str = "info@example.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            aux_input_ports: &[],
            aux_output_ports: &[],
            names: PortNames::const_default(),
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples() {
            let gain = self.params.gain.smoothed.next();

            for sample in channel_samples {
                *sample *= gain;
            }
        }

        ProcessStatus::Normal
    }

    fn deactivate(&mut self) {}
}

impl ClapPlugin for GainAuTest {
    const CLAP_ID: &'static str = "com.nih-plug.gain-au-test";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A simple gain plugin for testing AU functionality");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for GainAuTest {
    const VST3_CLASS_ID: [u8; 16] = *b"GainAuTestPlug00";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

// Export the plugin for all supported formats
nih_export_clap!(GainAuTest);
nih_export_vst3!(GainAuTest);
nih_export_au!(GainAuTest);