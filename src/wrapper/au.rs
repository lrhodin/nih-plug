//! Audio Unit v2 (AUv2) plugin wrapper for macOS.
//!
//! This wrapper allows NIH-plug plugins to be exported as Audio Unit v2 plugins on macOS.
//! The implementation follows similar patterns to the VST3 and CLAP wrappers.

#[macro_use]
mod util;

mod context;
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
///     // ...
/// }
///
/// nih_export_au!(MyPlugin);
/// ```
///
/// # Platform Support
///
/// This macro only works on macOS. On other platforms, it will generate an empty implementation.
/// Audio Units are Apple's native plugin format for macOS and iOS.
#[macro_export]
macro_rules! nih_export_au {
    ($($plugin_ty:ty),+) => {
        // Audio Units are only supported on macOS
        #[cfg(target_os = "macos")]
        mod au {
            use $crate::wrapper::au::Wrapper;
            use super::*;

            // TODO: Implement AU component registration and factory
            // This will follow the AudioComponent API pattern

            #[no_mangle]
            pub extern "C" fn AU_ENTRY_POINT() {
                // TODO: Initialize AU factory
                $crate::wrapper::setup_logger();
            }
        }

        #[cfg(not(target_os = "macos"))]
        mod au {
            // Empty implementation for non-macOS platforms
            pub extern "C" fn AU_ENTRY_POINT() {}
        }
    };
}
