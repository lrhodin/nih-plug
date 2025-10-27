//! Audio Unit v3 (AUv3) plugin wrapper for macOS and iOS.
//!
//! This wrapper allows NIH-plug plugins to be exported as Audio Unit v3 plugins.
//! AUv3 is the modern Audio Unit format that uses Swift AUAudioUnit subclasses
//! and requires an FFI layer to bridge between Swift and Rust.

pub mod ffi;

/// Re-export the FFI module for external use.
pub use ffi::*;