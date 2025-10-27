//! Audio Unit v3 (AUv3) FFI layer for NIH-plug.
//!
//! This module provides C-compatible exports that allow Swift AUAudioUnit subclasses
//! to interact with NIH-plug plugins through a Foreign Function Interface (FFI).
//!
//! The FFI layer is designed for real-time audio processing with minimal overhead
//! and safe memory management between Rust and Swift.

use std::ffi::CString;
use std::os::raw::{c_float, c_int, c_uint, c_void};
use std::sync::Arc;

use crate::prelude::{
    AudioIOLayout, BufferConfig, Params, Plugin, ProcessContext, ProcessStatus,
    FloatParam, FloatRange, SmoothingStyle, MidiConfig, AuxiliaryBuffers, Buffer,
    util, formatters
};
use crate::audio_setup::ProcessMode;
use crate::params::Param;
use std::num::NonZeroU32;

/// Error codes for FFI operations.
///
/// These codes are returned by FFI functions to indicate success or failure.
/// They are designed to be compatible with both Rust and C error handling.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FFIError {
    /// Operation completed successfully.
    Success = 0,
    /// Invalid argument passed to function.
    InvalidArgument = -1,
    /// Invalid plugin handle.
    InvalidHandle = -2,
    /// Plugin initialization failed.
    InitializationFailed = -3,
    /// Audio processing failed.
    ProcessingFailed = -4,
    /// Parameter operation failed.
    ParameterError = -5,
    /// State serialization/deserialization failed.
    StateError = -6,
    /// Memory allocation failed.
    MemoryError = -7,
    /// Plugin not initialized.
    NotInitialized = -8,
}

impl FFIError {
    /// Convert to i32 for C compatibility.
    pub fn as_i32(self) -> i32 {
        self as i32
    }

    /// Convert from i32, defaulting to InvalidArgument for unknown values.
    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => Self::Success,
            -1 => Self::InvalidArgument,
            -2 => Self::InvalidHandle,
            -3 => Self::InitializationFailed,
            -4 => Self::ProcessingFailed,
            -5 => Self::ParameterError,
            -6 => Self::StateError,
            -7 => Self::MemoryError,
            -8 => Self::NotInitialized,
            _ => Self::InvalidArgument,
        }
    }
}

/// Plugin handle type for FFI operations.
///
/// This is an opaque pointer that represents a plugin instance.
/// The actual plugin data is stored in a Box<PluginWrapper<P>>.
pub type PluginHandle = *mut c_void;

/// Plugin wrapper that contains the actual plugin instance and metadata.
///
/// This struct is what gets stored behind the PluginHandle pointer.
/// It provides a safe interface between the FFI layer and the NIH-plug Plugin trait.
pub struct PluginWrapper {
    /// The plugin instance.
    plugin: TestGainPlugin,
    /// The plugin's parameters.
    params: Arc<dyn Params>,
    /// The current buffer configuration.
    buffer_config: Option<BufferConfig>,
    /// The current audio I/O layout.
    audio_io_layout: Option<AudioIOLayout>,
    /// Whether the plugin is initialized.
    initialized: bool,
}

impl PluginWrapper {
    /// Create a new plugin wrapper.
    pub fn new(plugin: TestGainPlugin, params: Arc<dyn Params>) -> Self {
        Self {
            plugin,
            params,
            buffer_config: None,
            audio_io_layout: None,
            initialized: false,
        }
    }

    /// Initialize the plugin with the given configuration.
    pub fn initialize(&mut self, buffer_config: BufferConfig, audio_io_layout: AudioIOLayout) -> Result<(), FFIError> {
        self.buffer_config = Some(buffer_config);
        self.audio_io_layout = Some(audio_io_layout);
        self.initialized = true;
        Ok(())
    }

    /// Process audio with the given buffers.
    pub fn process(&mut self, input_buffers: &[&[f32]], output_buffers: &mut [&mut [f32]]) -> Result<(), FFIError> {
        if !self.initialized {
            return Err(FFIError::NotInitialized);
        }

        // For now, just copy input to output
        // In a full implementation, this would call the actual plugin process method
        let output_len = output_buffers.len();
        for (channel_idx, (input_channel, output_channel)) in input_buffers.iter().zip(output_buffers.iter_mut()).enumerate() {
            if channel_idx < output_len {
                output_channel.copy_from_slice(input_channel);
            }
        }

        Ok(())
    }

    /// Get the number of parameters.
    pub fn parameter_count(&self) -> u32 {
        self.params.param_map().len() as u32
    }

    /// Get parameter information.
    pub fn get_parameter_info(&self, param_id: u32) -> Option<ParameterInfo> {
        let param_map = self.params.param_map();
        if param_id >= param_map.len() as u32 {
            return None;
        }

        let (_, param_ptr, _) = &param_map[param_id as usize];
        
        Some(ParameterInfo {
            id: param_id,
            name: CString::new(unsafe { param_ptr.name() }).ok()?,
            unit: CString::new(unsafe { param_ptr.unit() }).ok()?,
            min_value: unsafe { param_ptr.preview_plain(0.0) },
            max_value: unsafe { param_ptr.preview_plain(1.0) },
            default_value: unsafe { param_ptr.default_normalized_value() },
        })
    }

    /// Set a parameter value.
    pub fn set_parameter(&mut self, param_id: u32, normalized_value: f32) -> Result<(), FFIError> {
        let param_map = self.params.param_map();
        if param_id >= param_map.len() as u32 {
            return Err(FFIError::InvalidArgument);
        }

        let (_, param_ptr, _) = &param_map[param_id as usize];
        unsafe { param_ptr.set_normalized_value(normalized_value) };
        Ok(())
    }

    /// Get a parameter value.
    pub fn get_parameter(&self, param_id: u32) -> Result<f32, FFIError> {
        let param_map = self.params.param_map();
        if param_id >= param_map.len() as u32 {
            return Err(FFIError::InvalidArgument);
        }

        let (_, param_ptr, _) = &param_map[param_id as usize];
        Ok(unsafe { param_ptr.modulated_normalized_value() })
    }

    /// Save plugin state to a byte array.
    pub fn save_state(&self) -> Result<Vec<u8>, FFIError> {
        // For now, return empty state
        // In a full implementation, this would serialize the plugin state
        Ok(Vec::new())
    }

    /// Load plugin state from a byte array.
    pub fn load_state(&mut self, _state_data: &[u8]) -> Result<(), FFIError> {
        // For now, do nothing
        // In a full implementation, this would deserialize the plugin state
        Ok(())
    }
}

/// Parameter information structure for FFI.
#[repr(C)]
pub struct ParameterInfo {
    /// Parameter ID.
    pub id: u32,
    /// Parameter name (null-terminated C string).
    pub name: CString,
    /// Parameter unit (null-terminated C string).
    pub unit: CString,
    /// Minimum parameter value.
    pub min_value: f32,
    /// Maximum parameter value.
    pub max_value: f32,
    /// Default parameter value (normalized).
    pub default_value: f32,
}

// FFI Functions

/// Create a new plugin instance.
///
/// # Safety
/// This function is unsafe because it deals with raw pointers and memory allocation.
/// The returned handle must be freed with `plugin_destroy()`.
#[no_mangle]
pub unsafe extern "C" fn plugin_create() -> PluginHandle {
    // For now, create a simple gain plugin for testing
    // In a full implementation, this would be configurable
    let plugin = TestGainPlugin::default();
    let params = plugin.params();
    let wrapper = Box::new(PluginWrapper::new(plugin, params));
    Box::into_raw(wrapper) as PluginHandle
}

/// Simple test gain plugin for AUv3 testing.
#[derive(Default)]
struct TestGainPlugin {
    params: Arc<TestGainParams>,
}

struct TestGainParams {
    pub gain: FloatParam,
}

impl Default for TestGainParams {
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

unsafe impl Params for TestGainParams {
    fn param_map(&self) -> Vec<(String, crate::params::internals::ParamPtr, String)> {
        vec![(
            "gain".to_string(),
            self.gain.as_ptr(),
            "Gain".to_string(),
        )]
    }
}

impl Plugin for TestGainPlugin {
    const NAME: &'static str = "Test Gain AUv3";
    const VENDOR: &'static str = "NIH-Plug";
    const URL: &'static str = "https://github.com/robbert-vdh/nih-plug";
    const EMAIL: &'static str = "info@example.com";
    const VERSION: &'static str = "1.0.0";

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
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
}

/// Destroy a plugin instance.
///
/// # Safety
/// This function is unsafe because it deals with raw pointers and memory deallocation.
/// The handle must be valid and not used after this call.
#[no_mangle]
pub unsafe extern "C" fn plugin_destroy(handle: PluginHandle) -> c_int {
    if handle.is_null() {
        return FFIError::InvalidHandle.as_i32();
    }

    let _wrapper = Box::from_raw(handle as *mut PluginWrapper);
    FFIError::Success.as_i32()
}

/// Initialize a plugin with the given configuration.
///
/// # Safety
/// This function is unsafe because it deals with raw pointers.
/// The handle must be valid and not null.
#[no_mangle]
pub unsafe extern "C" fn plugin_initialize(
    handle: PluginHandle,
    sample_rate: c_float,
    max_block_size: c_uint,
    input_channels: c_uint,
    output_channels: c_uint,
) -> c_int {
    if handle.is_null() {
        return FFIError::InvalidHandle.as_i32();
    }

    let wrapper = &mut *(handle as *mut PluginWrapper);
    
    let buffer_config = BufferConfig {
        sample_rate: sample_rate as f32,
        max_buffer_size: max_block_size as u32,
        min_buffer_size: Some(1),
        process_mode: ProcessMode::Realtime,
    };

    let audio_io_layout = AudioIOLayout {
        main_input_channels: std::num::NonZeroU32::new(input_channels as u32),
        main_output_channels: std::num::NonZeroU32::new(output_channels as u32),
        ..Default::default()
    };

    match wrapper.initialize(buffer_config, audio_io_layout) {
        Ok(()) => FFIError::Success.as_i32(),
        Err(err) => err.as_i32(),
    }
}

/// Process audio with the given buffers.
///
/// # Safety
/// This function is unsafe because it deals with raw pointers and assumes valid buffer layouts.
/// The handle must be valid and the buffer pointers must point to valid memory.
#[no_mangle]
pub unsafe extern "C" fn plugin_process(
    handle: PluginHandle,
    input_buffers: *const *const c_float,
    output_buffers: *mut *mut c_float,
    num_channels: c_uint,
    num_frames: c_uint,
) -> c_int {
    if handle.is_null() || input_buffers.is_null() || output_buffers.is_null() {
        return FFIError::InvalidArgument.as_i32();
    }

    let wrapper = &mut *(handle as *mut PluginWrapper);

    // Convert C arrays to Rust slices
    let input_slices: Vec<&[f32]> = (0..num_channels)
        .map(|i| {
            let ptr = *input_buffers.add(i as usize);
            std::slice::from_raw_parts(ptr, num_frames as usize)
        })
        .collect();

    let mut output_slices: Vec<&mut [f32]> = (0..num_channels)
        .map(|i| {
            let ptr = *output_buffers.add(i as usize);
            std::slice::from_raw_parts_mut(ptr, num_frames as usize)
        })
        .collect();

    match wrapper.process(&input_slices, &mut output_slices) {
        Ok(()) => FFIError::Success.as_i32(),
        Err(err) => err.as_i32(),
    }
}

/// Get the number of parameters.
///
/// # Safety
/// This function is unsafe because it deals with raw pointers.
/// The handle must be valid and not null.
#[no_mangle]
pub unsafe extern "C" fn plugin_get_parameter_count(handle: PluginHandle) -> c_uint {
    if handle.is_null() {
        return 0;
    }

    let wrapper = &*(handle as *const PluginWrapper);
    wrapper.parameter_count()
}

/// Get parameter information.
///
/// # Safety
/// This function is unsafe because it deals with raw pointers.
/// The handle must be valid and not null.
#[no_mangle]
pub unsafe extern "C" fn plugin_get_parameter_info(
    handle: PluginHandle,
    param_id: c_uint,
    info: *mut ParameterInfo,
) -> c_int {
    if handle.is_null() || info.is_null() {
        return FFIError::InvalidArgument.as_i32();
    }

    let wrapper = &*(handle as *const PluginWrapper);
    
    match wrapper.get_parameter_info(param_id) {
        Some(param_info) => {
            *info = param_info;
            FFIError::Success.as_i32()
        }
        None => FFIError::ParameterError.as_i32(),
    }
}

/// Set a parameter value.
///
/// # Safety
/// This function is unsafe because it deals with raw pointers.
/// The handle must be valid and not null.
#[no_mangle]
pub unsafe extern "C" fn plugin_set_parameter(
    handle: PluginHandle,
    param_id: c_uint,
    normalized_value: c_float,
) -> c_int {
    if handle.is_null() {
        return FFIError::InvalidHandle.as_i32();
    }

    let wrapper = &mut *(handle as *mut PluginWrapper);
    
    match wrapper.set_parameter(param_id, normalized_value as f32) {
        Ok(()) => FFIError::Success.as_i32(),
        Err(err) => err.as_i32(),
    }
}

/// Get a parameter value.
///
/// # Safety
/// This function is unsafe because it deals with raw pointers.
/// The handle must be valid and not null.
#[no_mangle]
pub unsafe extern "C" fn plugin_get_parameter(
    handle: PluginHandle,
    param_id: c_uint,
    value: *mut c_float,
) -> c_int {
    if handle.is_null() || value.is_null() {
        return FFIError::InvalidArgument.as_i32();
    }

    let wrapper = &*(handle as *const PluginWrapper);
    
    match wrapper.get_parameter(param_id) {
        Ok(param_value) => {
            *value = param_value;
            FFIError::Success.as_i32()
        }
        Err(err) => err.as_i32(),
    }
}

/// Save plugin state.
///
/// # Safety
/// This function is unsafe because it deals with raw pointers and memory allocation.
/// The handle must be valid and not null. The returned data must be freed with `free()`.
#[no_mangle]
pub unsafe extern "C" fn plugin_save_state(
    handle: PluginHandle,
    data: *mut *mut u8,
    size: *mut c_uint,
) -> c_int {
    if handle.is_null() || data.is_null() || size.is_null() {
        return FFIError::InvalidArgument.as_i32();
    }

    let wrapper = &*(handle as *const PluginWrapper);
    
    match wrapper.save_state() {
        Ok(state_data) => {
            let state_size = state_data.len() as c_uint;
            let state_ptr = std::alloc::alloc(std::alloc::Layout::from_size_align(state_size as usize, 1).unwrap()) as *mut u8;
            
            if state_ptr.is_null() {
                return FFIError::MemoryError.as_i32();
            }
            
            std::ptr::copy_nonoverlapping(state_data.as_ptr(), state_ptr, state_size as usize);
            *data = state_ptr;
            *size = state_size;
            
            FFIError::Success.as_i32()
        }
        Err(err) => err.as_i32(),
    }
}

/// Load plugin state.
///
/// # Safety
/// This function is unsafe because it deals with raw pointers.
/// The handle must be valid and not null. The data pointer must point to valid state data.
#[no_mangle]
pub unsafe extern "C" fn plugin_load_state(
    handle: PluginHandle,
    data: *const u8,
    size: c_uint,
) -> c_int {
    if handle.is_null() || data.is_null() {
        return FFIError::InvalidArgument.as_i32();
    }

    let wrapper = &mut *(handle as *mut PluginWrapper);
    let state_data = std::slice::from_raw_parts(data, size as usize);
    
    match wrapper.load_state(state_data) {
        Ok(()) => FFIError::Success.as_i32(),
        Err(err) => err.as_i32(),
    }
}

/// Free memory allocated by the FFI layer.
///
/// # Safety
/// This function is unsafe because it deals with raw pointers and memory deallocation.
/// The ptr must be a valid pointer returned by a previous FFI call.
#[no_mangle]
pub unsafe extern "C" fn plugin_free(ptr: *mut c_void) {
    if !ptr.is_null() {
        std::alloc::dealloc(ptr as *mut u8, std::alloc::Layout::from_size_align(1, 1).unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation_and_parameters() {
        unsafe {
            // Test plugin creation
            let handle = plugin_create();
            assert!(!handle.is_null(), "Plugin creation should succeed");

            // Test parameter count
            let param_count = plugin_get_parameter_count(handle);
            assert_eq!(param_count, 1, "Should have exactly 1 parameter");

            // Test parameter info
            let mut param_info = ParameterInfo {
                id: 0,
                name: CString::new("").unwrap(),
                unit: CString::new("").unwrap(),
                min_value: 0.0,
                max_value: 1.0,
                default_value: 0.0,
            };

            let result = plugin_get_parameter_info(handle, 0, &mut param_info);
            assert_eq!(result, 0, "Parameter info query should succeed");
            assert_eq!(param_info.id, 0, "Parameter ID should be 0");
            assert_eq!(param_info.name.to_string_lossy(), "Gain", "Parameter name should be 'Gain'");
            assert_eq!(param_info.unit.to_string_lossy(), " dB", "Parameter unit should be ' dB'");

            // Test parameter get/set
            let mut value: f32 = 0.0;
            let get_result = plugin_get_parameter(handle, 0, &mut value);
            assert_eq!(get_result, 0, "Parameter get should succeed");
            // The default value should be 0.5 (normalized value for 0 dB gain)
            assert!((value - 0.5).abs() < 0.001, "Default parameter value should be 0.5 (normalized), got {}", value);

            let set_result = plugin_set_parameter(handle, 0, 0.5);
            assert_eq!(set_result, 0, "Parameter set should succeed");

            let get_result2 = plugin_get_parameter(handle, 0, &mut value);
            assert_eq!(get_result2, 0, "Parameter get after set should succeed");
            assert!((value - 0.5).abs() < 0.001, "Parameter value should be 0.5 after setting");

            // Test plugin destruction
            let destroy_result = plugin_destroy(handle);
            assert_eq!(destroy_result, 0, "Plugin destruction should succeed");
        }
    }

    #[test]
    fn test_parameter_info_structure() {
        let params = TestGainParams::default();
        let param_map = params.param_map();
        
        assert_eq!(param_map.len(), 1, "Should have exactly 1 parameter");
        
        let (id, param_ptr, name) = &param_map[0];
        assert_eq!(id, "gain", "Parameter ID should be 'gain'");
        assert_eq!(name, "Gain", "Parameter name should be 'Gain'");
        
        // Test parameter value conversion
        let normalized_value = unsafe { param_ptr.modulated_normalized_value() };
        // The default normalized value should be 0.5 (middle of the range for 0 dB gain)
        assert!((normalized_value - 0.5).abs() < 0.001, "Default normalized value should be 0.5");
        
        let plain_value = unsafe { param_ptr.preview_plain(0.5) };
        assert!(plain_value > 0.0, "Plain value should be positive for 0.5 normalized");
    }
}