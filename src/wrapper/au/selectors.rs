//! Audio Unit selector constants and dispatch system.
//!
//! This module defines the selector codes used by the Audio Unit API to dispatch
//! different operations on the plugin instance.

/// Audio Unit selector codes
///
/// These values are used by the Component Manager and Audio Component API to dispatch
/// different method calls to the audio unit.
#[repr(i16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioUnitSelector {
    /// Range identifier
    Range = 0x0000,
    /// Initialize the audio unit
    Initialize = 0x0001,
    /// Uninitialize and cleanup
    Uninitialize = 0x0002,
    /// Get property info (size, writable)
    GetPropertyInfo = 0x0003,
    /// Get a property value
    GetProperty = 0x0004,
    /// Set a property value
    SetProperty = 0x0005,
    /// Get a parameter value
    GetParameter = 0x0006,
    /// Set a parameter value
    SetParameter = 0x0007,
    /// Reset the audio unit state
    Reset = 0x0009,
    /// Add a property change listener
    AddPropertyListener = 0x000A,
    /// Remove a property change listener
    RemovePropertyListener = 0x000B,
    /// Render audio (main processing)
    Render = 0x000E,
    /// Add a render notification callback
    AddRenderNotify = 0x000F,
    /// Remove a render notification callback
    RemoveRenderNotify = 0x0010,
    /// Schedule parameter changes
    ScheduleParameters = 0x0011,
    /// Remove property listener with user data
    RemovePropertyListenerWithUserData = 0x0012,
    /// Complex render (with timestamp)
    ComplexRender = 0x0013,
    /// Process audio (offline)
    Process = 0x0014,
    /// Process multiple buffers
    ProcessMultiple = 0x0015,
}

impl AudioUnitSelector {
    /// Convert an i16 selector code to an AudioUnitSelector enum
    pub fn from_i16(value: i16) -> Option<Self> {
        match value {
            0x0000 => Some(Self::Range),
            0x0001 => Some(Self::Initialize),
            0x0002 => Some(Self::Uninitialize),
            0x0003 => Some(Self::GetPropertyInfo),
            0x0004 => Some(Self::GetProperty),
            0x0005 => Some(Self::SetProperty),
            0x0006 => Some(Self::GetParameter),
            0x0007 => Some(Self::SetParameter),
            0x0009 => Some(Self::Reset),
            0x000A => Some(Self::AddPropertyListener),
            0x000B => Some(Self::RemovePropertyListener),
            0x000E => Some(Self::Render),
            0x000F => Some(Self::AddRenderNotify),
            0x0010 => Some(Self::RemoveRenderNotify),
            0x0011 => Some(Self::ScheduleParameters),
            0x0012 => Some(Self::RemovePropertyListenerWithUserData),
            0x0013 => Some(Self::ComplexRender),
            0x0014 => Some(Self::Process),
            0x0015 => Some(Self::ProcessMultiple),
            _ => None,
        }
    }

    /// Get a human-readable name for the selector
    pub fn name(&self) -> &'static str {
        match self {
            Self::Range => "Range",
            Self::Initialize => "Initialize",
            Self::Uninitialize => "Uninitialize",
            Self::GetPropertyInfo => "GetPropertyInfo",
            Self::GetProperty => "GetProperty",
            Self::SetProperty => "SetProperty",
            Self::GetParameter => "GetParameter",
            Self::SetParameter => "SetParameter",
            Self::Reset => "Reset",
            Self::AddPropertyListener => "AddPropertyListener",
            Self::RemovePropertyListener => "RemovePropertyListener",
            Self::Render => "Render",
            Self::AddRenderNotify => "AddRenderNotify",
            Self::RemoveRenderNotify => "RemoveRenderNotify",
            Self::ScheduleParameters => "ScheduleParameters",
            Self::RemovePropertyListenerWithUserData => "RemovePropertyListenerWithUserData",
            Self::ComplexRender => "ComplexRender",
            Self::Process => "Process",
            Self::ProcessMultiple => "ProcessMultiple",
        }
    }

    /// Check if this selector is required for a minimal audio effect plugin
    pub fn is_required_for_effect(&self) -> bool {
        matches!(
            self,
            Self::Initialize
                | Self::Uninitialize
                | Self::GetProperty
                | Self::SetProperty
                | Self::GetParameter
                | Self::SetParameter
                | Self::ScheduleParameters
                | Self::Render
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selector_conversion() {
        assert_eq!(
            AudioUnitSelector::from_i16(0x0001),
            Some(AudioUnitSelector::Initialize)
        );
        assert_eq!(
            AudioUnitSelector::from_i16(0x000E),
            Some(AudioUnitSelector::Render)
        );
        assert_eq!(AudioUnitSelector::from_i16(0xFFFFu16 as i16), None);
    }

    #[test]
    fn test_required_selectors() {
        assert!(AudioUnitSelector::Initialize.is_required_for_effect());
        assert!(AudioUnitSelector::Render.is_required_for_effect());
        assert!(!AudioUnitSelector::ComplexRender.is_required_for_effect());
    }
}
