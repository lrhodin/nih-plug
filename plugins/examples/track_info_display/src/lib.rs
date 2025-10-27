use atomic_refcell::AtomicRefCell;
use nih_plug::prelude::*;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// A plugin that displays track information from the host
struct TrackInfoDisplay {
    params: Arc<TrackInfoParams>,
    /// Track info data for GUI display
    track_name: Arc<AtomicRefCell<String>>,
    track_color: Arc<AtomicRefCell<String>>,
    is_master: Arc<AtomicBool>,
    is_bus: Arc<AtomicBool>,
    is_return: Arc<AtomicBool>,
    /// Counter to log track info periodically
    process_counter: usize,
}

#[derive(Params)]
struct TrackInfoParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,
}

#[derive(Lens)]
struct Data {
    params: Arc<TrackInfoParams>,
    track_name: Arc<AtomicRefCell<String>>,
    track_color: Arc<AtomicRefCell<String>>,
    is_master: Arc<AtomicBool>,
    is_bus: Arc<AtomicBool>,
    is_return: Arc<AtomicBool>,
}

impl Model for Data {}

impl Default for TrackInfoDisplay {
    fn default() -> Self {
        Self {
            params: Arc::new(TrackInfoParams::default()),
            track_name: Arc::new(AtomicRefCell::new(String::from("No track info"))),
            track_color: Arc::new(AtomicRefCell::new(String::from("Unknown"))),
            is_master: Arc::new(AtomicBool::new(false)),
            is_bus: Arc::new(AtomicBool::new(false)),
            is_return: Arc::new(AtomicBool::new(false)),
            process_counter: 0,
        }
    }
}

impl Default for TrackInfoParams {
    fn default() -> Self {
        Self {
            editor_state: ViziaState::new(|| (400, 350)),
        }
    }
}

impl Plugin for TrackInfoDisplay {
    const NAME: &'static str = "Track Info Display";
    const VENDOR: &'static str = "NIH-plug";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "info@example.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let track_name = self.track_name.clone();
        let track_color = self.track_color.clone();
        let is_master = self.is_master.clone();
        let is_bus = self.is_bus.clone();
        let is_return = self.is_return.clone();

        create_vizia_editor(
            self.params.editor_state.clone(),
            ViziaTheming::Custom,
            move |cx, _| {
                assets::register_noto_sans_light(cx);
                assets::register_noto_sans_thin(cx);

                Data {
                    params: Default::default(),
                    track_name: track_name.clone(),
                    track_color: track_color.clone(),
                    is_master: is_master.clone(),
                    is_bus: is_bus.clone(),
                    is_return: is_return.clone(),
                }
                .build(cx);

                VStack::new(cx, |cx| {
                    Label::new(cx, "Track Information Display")
                        .font_family(vec![FamilyOwned::Name(String::from(
                            assets::NOTO_SANS,
                        ))])
                        .font_weight(FontWeightKeyword::Thin)
                        .font_size(30.0)
                        .height(Pixels(50.0))
                        .child_top(Stretch(1.0))
                        .child_bottom(Pixels(0.0));

                    Label::new(cx, "This plugin demonstrates the track info API")
                        .font_size(14.0)
                        .height(Pixels(20.0))
                        .color(Color::rgb(180, 180, 180));

                    // Track name display
                    VStack::new(cx, |cx| {
                        Label::new(cx, "Track Name:")
                            .font_size(16.0)
                            .font_weight(FontWeightKeyword::Bold)
                            .height(Pixels(25.0));

                        let track_name_clone = track_name.clone();
                        Label::new(
                            cx,
                            Data::track_name.map(move |_| {
                                track_name_clone
                                    .try_borrow()
                                    .map(|guard| guard.clone())
                                    .unwrap_or_else(|_| String::from("..."))
                            }),
                        )
                        .font_size(20.0)
                        .height(Pixels(35.0))
                        .color(Color::rgb(100, 200, 255));
                    })
                    .row_between(Pixels(5.0))
                    .top(Pixels(20.0));

                    // Track color display
                    VStack::new(cx, |cx| {
                        Label::new(cx, "Track Color:")
                            .font_size(16.0)
                            .font_weight(FontWeightKeyword::Bold)
                            .height(Pixels(25.0));

                        let track_color_clone = track_color.clone();
                        Label::new(
                            cx,
                            Data::track_color.map(move |_| {
                                track_color_clone
                                    .try_borrow()
                                    .map(|guard| guard.clone())
                                    .unwrap_or_else(|_| String::from("..."))
                            }),
                        )
                        .font_size(16.0)
                        .height(Pixels(25.0));
                    })
                    .row_between(Pixels(5.0))
                    .top(Pixels(15.0));

                    // Track type badges
                    VStack::new(cx, |cx| {
                        Label::new(cx, "Track Type:")
                            .font_size(16.0)
                            .font_weight(FontWeightKeyword::Bold)
                            .height(Pixels(25.0));

                        HStack::new(cx, |cx| {
                            Label::new(
                                cx,
                                Data::is_master.map(|v| {
                                    if v.load(Ordering::Relaxed) {
                                        "MASTER ✓"
                                    } else {
                                        "MASTER"
                                    }
                                }),
                            )
                            .font_size(12.0)
                            .width(Pixels(80.0))
                            .height(Pixels(25.0))
                            .background_color(Color::rgb(80, 80, 80));

                            Label::new(
                                cx,
                                Data::is_bus.map(|v| {
                                    if v.load(Ordering::Relaxed) {
                                        "BUS ✓"
                                    } else {
                                        "BUS"
                                    }
                                }),
                            )
                            .font_size(12.0)
                            .width(Pixels(80.0))
                            .height(Pixels(25.0))
                            .background_color(Color::rgb(80, 80, 80));

                            Label::new(
                                cx,
                                Data::is_return.map(|v| {
                                    if v.load(Ordering::Relaxed) {
                                        "RETURN ✓"
                                    } else {
                                        "RETURN"
                                    }
                                }),
                            )
                            .font_size(12.0)
                            .width(Pixels(80.0))
                            .height(Pixels(25.0))
                            .background_color(Color::rgb(80, 80, 80));
                        })
                        .col_between(Pixels(10.0));
                    })
                    .row_between(Pixels(5.0))
                    .top(Pixels(15.0));

                    Label::new(cx, "Check console for detailed track info logs")
                        .font_size(12.0)
                        .top(Pixels(20.0))
                        .color(Color::rgb(150, 150, 150));
                })
                .row_between(Pixels(0.0))
                .child_left(Stretch(1.0))
                .child_right(Stretch(1.0));

                ResizeHandle::new(cx);
            },
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        true
    }

    fn reset(&mut self) {}

    fn process(
        &mut self,
        _buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Update track info from context
        if let Some(track_info) = context.track_info() {
            // Update track name
            if let Some(name) = &track_info.name {
                // Use try_borrow for reading to avoid panic if GUI thread is also accessing
                if let Ok(current_name_guard) = self.track_name.try_borrow() {
                    if *current_name_guard != *name {
                        drop(current_name_guard); // Release read lock before trying write lock
                        // Use try_borrow_mut to avoid panic if GUI thread is reading
                        if let Ok(mut guard) = self.track_name.try_borrow_mut() {
                            *guard = name.clone();
                        }
                    }
                }
            }

            // Update track color
            if let Some((r, g, b, a)) = track_info.color {
                let color_str = format!("RGBA({}, {}, {}, {})", r, g, b, a);
                // Use try_borrow_mut to avoid panic if GUI thread is reading
                if let Ok(mut guard) = self.track_color.try_borrow_mut() {
                    *guard = color_str;
                }
            } else {
                // Use try_borrow_mut to avoid panic if GUI thread is reading
                if let Ok(mut guard) = self.track_color.try_borrow_mut() {
                    *guard = String::from("No color info");
                }
            }

            // Update track type flags
            self.is_master
                .store(track_info.track_type.is_master, Ordering::Relaxed);
            self.is_bus
                .store(track_info.track_type.is_bus, Ordering::Relaxed);
            self.is_return
                .store(track_info.track_type.is_return, Ordering::Relaxed);

            // Log track info periodically (every 2000 process calls, ~few seconds)
            self.process_counter += 1;
            if self.process_counter >= 2000 {
                self.process_counter = 0;

                nih_log!("=== TRACK INFO RECEIVED ===");
                if let Some(name) = &track_info.name {
                    nih_log!("  Track Name: {}", name);
                }
                if let Some((r, g, b, a)) = track_info.color {
                    nih_log!("  Track Color: RGBA({}, {}, {}, {})", r, g, b, a);
                }
                if let Some(index) = track_info.index {
                    nih_log!("  Track Index: {}", index);
                }
                if let Some(uid) = &track_info.uid {
                    nih_log!("  Track UID: {}", uid);
                }
                nih_log!(
                    "  Track Type: master={}, bus={}, return={}",
                    track_info.track_type.is_master,
                    track_info.track_type.is_bus,
                    track_info.track_type.is_return
                );
                nih_log!("================================");
            }
        } else {
            // Reset to default when no track info available
            // Use try_borrow for reading to avoid panic if GUI thread is accessing
            if let Ok(name_guard) = self.track_name.try_borrow() {
                if name_guard.as_str() != "No track info" {
                    drop(name_guard); // Release read lock
                    // Use try_borrow_mut to avoid panic if GUI thread is reading
                    if let Ok(mut guard) = self.track_name.try_borrow_mut() {
                        *guard = String::from("No track info");
                    }
                    if let Ok(mut guard) = self.track_color.try_borrow_mut() {
                        *guard = String::from("Unknown");
                    }
                    self.is_master.store(false, Ordering::Relaxed);
                    self.is_bus.store(false, Ordering::Relaxed);
                    self.is_return.store(false, Ordering::Relaxed);
                }
            }
        }

        // Pass through audio
        ProcessStatus::Normal
    }
}

impl ClapPlugin for TrackInfoDisplay {
    const CLAP_ID: &'static str = "com.nih-plug.track-info-display";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Example plugin demonstrating track info API");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] =
        &[ClapFeature::AudioEffect, ClapFeature::Utility];
}

impl Vst3Plugin for TrackInfoDisplay {
    const VST3_CLASS_ID: [u8; 16] = *b"TrackInfoDisplay";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Tools];
}

nih_export_clap!(TrackInfoDisplay);
nih_export_vst3!(TrackInfoDisplay);
