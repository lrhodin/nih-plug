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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_info_default() {
        let info = TrackInfo::default();
        assert_eq!(info.name, None);
        assert_eq!(info.color, None);
        assert_eq!(info.index, None);
        assert_eq!(info.uid, None);
        assert!(!info.track_type.is_master);
        assert!(!info.track_type.is_bus);
        assert!(!info.track_type.is_return);
    }

    #[test]
    fn track_info_equality() {
        let mut info1 = TrackInfo::default();
        let mut info2 = TrackInfo::default();

        info1.name = Some("Track 1".to_string());
        info2.name = Some("Track 1".to_string());

        info1.color = Some((255, 0, 0, 255));
        info2.color = Some((255, 0, 0, 255));

        assert_eq!(info1, info2);
    }

    #[test]
    fn track_info_inequality() {
        let mut info1 = TrackInfo::default();
        let mut info2 = TrackInfo::default();

        info1.name = Some("Track 1".to_string());
        info2.name = Some("Track 2".to_string());

        assert_ne!(info1, info2);
    }

    #[test]
    fn track_type_flags() {
        let mut track_type = TrackType::default();
        assert!(!track_type.is_master);

        track_type.is_master = true;
        track_type.is_bus = true;

        assert!(track_type.is_master);
        assert!(track_type.is_bus);
        assert!(!track_type.is_return);
    }

    #[test]
    fn track_type_equality() {
        let mut type1 = TrackType::default();
        let mut type2 = TrackType::default();

        type1.is_master = true;
        type2.is_master = true;

        assert_eq!(type1, type2);
    }

    #[test]
    fn track_info_with_all_fields() {
        let info = TrackInfo {
            name: Some("Lead Vocal".to_string()),
            color: Some((120, 200, 80, 255)),
            index: Some(5),
            uid: Some("track-uuid-12345".to_string()),
            track_type: TrackType {
                is_master: false,
                is_bus: true,
                is_return: false,
            },
        };

        assert_eq!(info.name.as_deref(), Some("Lead Vocal"));
        assert_eq!(info.color, Some((120, 200, 80, 255)));
        assert_eq!(info.index, Some(5));
        assert_eq!(info.uid.as_deref(), Some("track-uuid-12345"));
        assert!(!info.track_type.is_master);
        assert!(info.track_type.is_bus);
        assert!(!info.track_type.is_return);
    }
}
