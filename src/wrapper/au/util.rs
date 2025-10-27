//! Utility functions and macros for the Audio Unit wrapper.

use std::ffi::CStr;
use std::os::raw::c_char;

/// Convert a C string to a Rust string slice, returning an empty string if the pointer is null.
pub fn c_str_to_str(c_str: *const c_char) -> &'static str {
    if c_str.is_null() {
        return "";
    }

    unsafe {
        CStr::from_ptr(c_str)
            .to_str()
            .unwrap_or("")
    }
}

/// Create a fixed-size buffer from a string, suitable for AU string fields.
pub fn str_to_fixed_buffer<const N: usize>(s: &str) -> [u8; N] {
    let mut buffer = [0u8; N];
    let bytes = s.as_bytes();
    let len = bytes.len().min(N - 1); // Leave room for null terminator
    buffer[..len].copy_from_slice(&bytes[..len]);
    buffer
}

/// Macro for checking null pointers in AU callbacks.
macro_rules! check_null_ptr {
    ($ptr:expr) => {
        if $ptr.is_null() {
            return $crate::wrapper::au::util::AU_INVALID_PARAMETER;
        }
    };
    ($($ptr:expr),+) => {
        $(check_null_ptr!($ptr);)+
    };
}

/// Audio Unit result codes
pub const AU_NO_ERROR: i32 = 0;
pub const AU_INVALID_PARAMETER: i32 = -50; // kAudio_ParamError
pub const AU_UNIMPLEMENTED: i32 = -4; // kAudio_UnimplementedError
