//! Different contexts the plugin can use to make callbacks to the host in different...contexts.

use std::fmt::Display;

pub mod gui;
pub mod init;
pub mod process;

// Contexts for more plugin-API specific features
pub mod remote_controls;

/// Information about the track/channel this plugin is inserted on
#[derive(Debug, Clone, PartialEq)]
pub struct TrackInfo {
    /// Track name from the DAW
    pub name: Option<String>,
    /// Track color as RGBA (0-255 per channel)
    pub color: Option<(u8, u8, u8, u8)>,
    /// Channel index within its namespace
    pub index: Option<i32>,
    /// Unique identifier for the track
    pub uid: Option<String>,
    /// Track type flags (master, bus, return)
    pub track_type: TrackType,
}

impl Default for TrackInfo {
    fn default() -> Self {
        Self {
            name: None,
            color: None,
            index: None,
            uid: None,
            track_type: TrackType::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TrackType {
    pub is_master: bool,
    pub is_bus: bool,
    pub is_return: bool,
}

/// The currently active plugin API. This may be useful to display in an about screen in the
/// plugin's GUI for debugging purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginApi {
    Clap,
    Standalone,
    Vst3,
}

impl Display for PluginApi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginApi::Clap => write!(f, "CLAP"),
            PluginApi::Standalone => write!(f, "standalone"),
            PluginApi::Vst3 => write!(f, "VST3"),
        }
    }
}
