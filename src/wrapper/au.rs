//! Audio Unit v2 (AUv2) plugin wrapper for macOS.
//!
//! This wrapper allows NIH-plug plugins to be exported as Audio Unit v2 plugins on macOS.
//! The implementation follows similar patterns to the VST3 and CLAP wrappers.

#[macro_use]
mod util;

pub mod bindings;
mod callbacks;
mod context;
pub mod factory;
mod properties;
pub mod selectors;
mod wrapper;

/// Re-export for the macro
pub use self::wrapper::Wrapper;

/// Export one or more Audio Unit plugins from this library using the provided plugin types.
///
/// # Example
///
/// ```ignore
/// use nih_plug::prelude::*;
///
/// struct MyPlugin;
/// impl Plugin for MyPlugin {
///     const NAME: &'static str = "My Plugin";
///     const VENDOR: &'static str = "My Company";
///     // ... other required constants
/// }
///
/// impl ClapPlugin for MyPlugin {
///     const CLAP_ID: &'static str = "com.mycompany.myplugin";
/// }
///
/// // The factory function name should match what's in your Info.plist
/// nih_export_au!(MyPlugin);
/// ```
///
/// # Bundle Structure
///
/// Audio Unit plugins require a .component bundle with the following structure:
/// ```text
/// MyPlugin.component/
///   Contents/
///     Info.plist         - Plugin metadata and factory function reference
///     MacOS/
///       MyPlugin         - The compiled dylib
///     Resources/         - Optional resources
/// ```
///
/// The Info.plist must specify the factory function name. For a plugin called "MyPlugin",
/// the factory function will be named "MyPluginFactory".
///
/// # Platform Support
///
/// This macro only works on macOS. On other platforms, it will generate an empty implementation.
/// Audio Units are Apple's native plugin format for macOS and iOS.
#[macro_export]
macro_rules! nih_export_au {
    ($plugin_ty:ty) => {
        // Audio Units are only supported on macOS
        #[cfg(target_os = "macos")]
        mod au {
            use $crate::wrapper::au::factory::AudioComponentPlugInInstance;
            use $crate::wrapper::au::bindings::AudioComponentDescription;
            use super::*;

            // Generate the factory function
            // The name should match what's specified in your Info.plist under
            // AudioComponents -> factory
            $crate::au_factory_function!($plugin_ty, AU_FACTORY);
        }

        #[cfg(not(target_os = "macos"))]
        mod au {
            // Empty implementation for non-macOS platforms
        }
    };
}
