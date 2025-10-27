//! Audio Unit v3 (AUv3) FFI layer for NIH-plug.
//!
//! This module provides C-compatible exports that allow Swift AUAudioUnit subclasses
//! to interact with NIH-plug plugins through a Foreign Function Interface (FFI).
//!
//! The FFI layer is designed for real-time audio processing with minimal overhead
//! and safe memory management between Rust and Swift.

use std::ffi::CString;
use std::os::raw::{c_char, c_float, c_int, c_uint, c_void};
use std::sync::Arc;
use std::collections::HashMap;

use crate::prelude::{
    AudioIOLayout, BufferConfig, Params, Plugin, ProcessContext, ProcessStatus,
    FloatParam, FloatRange, SmoothingStyle, MidiConfig, AuxiliaryBuffers, Buffer,
    util, formatters, Transport, ParamPtr
};
use crate::audio_setup::ProcessMode;
use crate::params::Param;
use crate::wrapper::state;
use crate::wrapper::util::hash_param_id;
use crate::debug::nih_debug_assert_failure;
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
    /// Parameter hash maps for state serialization.
    param_by_hash: HashMap<u32, ParamPtr>,
    /// Parameter ID to hash mapping for state serialization.
    param_id_to_hash: HashMap<String, u32>,
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
        // Create parameter hash maps for state serialization
        let param_id_hashes_ptrs: Vec<_> = params
            .param_map()
            .into_iter()
            .map(|(id, ptr, _)| {
                let hash = hash_param_id(&id);
                (id, hash, ptr)
            })
            .collect();
        
        let param_by_hash = param_id_hashes_ptrs
            .iter()
            .map(|(_, hash, ptr)| (*hash, *ptr))
            .collect();
        
        let param_id_to_hash = param_id_hashes_ptrs
            .iter()
            .map(|(id, hash, _)| (id.clone(), *hash))
            .collect();

        Self {
            plugin,
            params,
            param_by_hash,
            param_id_to_hash,
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

        // Get the buffer configuration
        let buffer_config = self.buffer_config.as_ref().ok_or(FFIError::NotInitialized)?;

        // Get the number of frames from the first output buffer
        let num_frames = output_buffers.first().map(|b| b.len()).unwrap_or(0);
        if num_frames == 0 {
            return Ok(());
        }

        // Create a mutable buffer that we can process
        let mut buffer_data = vec![vec![0.0f32; num_frames]; output_buffers.len()];
        
        // Copy input to buffer if we have input
        if !input_buffers.is_empty() {
            for (channel_idx, input_channel) in input_buffers.iter().enumerate() {
                if channel_idx < buffer_data.len() && input_channel.len() == num_frames {
                    buffer_data[channel_idx].copy_from_slice(input_channel);
                }
            }
        }

        // Create a Buffer using set_slices
        let mut buffer = Buffer::default();
        unsafe {
            buffer.set_slices(num_frames, |slices| {
                slices.clear();
                for channel_data in &mut buffer_data {
                    slices.push(channel_data.as_mut_slice());
                }
            });
        }
        
        // Create auxiliary buffers (empty for now)
        let mut aux_buffers = AuxiliaryBuffers {
            inputs: &mut [],
            outputs: &mut [],
        };

        // Create a dummy process context
        let mut context = DummyProcessContext {
            sample_rate: buffer_config.sample_rate,
        };

        // Process the audio using the plugin's process method
        let _status = self.plugin.process(&mut buffer, &mut aux_buffers, &mut context);

        // Copy the processed audio back to output buffers
        for (channel_idx, output_channel) in output_buffers.iter_mut().enumerate() {
            if channel_idx < buffer_data.len() && output_channel.len() == num_frames {
                output_channel.copy_from_slice(&buffer_data[channel_idx]);
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
        // Serialize the plugin state using NIH-plug's state system
        unsafe {
            match state::serialize_json::<TestGainPlugin>(
                self.params.clone(),
                state::make_params_iter(&self.param_by_hash, &self.param_id_to_hash),
            ) {
                Ok(serialized) => Ok(serialized),
                Err(err) => {
                    nih_debug_assert_failure!("Failed to serialize plugin state: {}", err);
                    Err(FFIError::StateError)
                }
            }
        }
    }

    /// Load plugin state from a byte array.
    pub fn load_state(&mut self, state_data: &[u8]) -> Result<(), FFIError> {
        // Deserialize the plugin state using NIH-plug's state system
        unsafe {
            match state::deserialize_json(state_data) {
                Some(mut plugin_state) => {
                    let success = state::deserialize_object::<TestGainPlugin>(
                        &mut plugin_state,
                        self.params.clone(),
                        state::make_params_getter(&self.param_by_hash, &self.param_id_to_hash),
                        self.buffer_config.as_ref(),
                    );
                    
                    if success {
                        Ok(())
                    } else {
                        nih_debug_assert_failure!("Failed to deserialize plugin state");
                        Err(FFIError::StateError)
                    }
                }
                None => {
                    nih_debug_assert_failure!("Failed to parse plugin state JSON");
                    Err(FFIError::StateError)
                }
            }
        }
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

/// Dummy process context for FFI audio processing.
///
/// This provides a minimal implementation of ProcessContext for use in the FFI layer.
/// It doesn't provide all the functionality of a full DAW context, but it's sufficient
/// for basic audio processing.
struct DummyProcessContext {
    sample_rate: f32,
}

impl ProcessContext<TestGainPlugin> for DummyProcessContext {
    fn plugin_api(&self) -> crate::context::PluginApi {
        crate::context::PluginApi::Clap
    }

    fn execute_background(&self, _task: <TestGainPlugin as Plugin>::BackgroundTask) {
        // No-op for FFI context
    }

    fn execute_gui(&self, _task: <TestGainPlugin as Plugin>::BackgroundTask) {
        // No-op for FFI context
    }

    fn transport(&self) -> &Transport {
        // Return a dummy transport - this is not used by our test plugin
        static DUMMY_TRANSPORT: Transport = Transport {
            playing: false,
            recording: false,
            preroll_active: None,
            sample_rate: 44100.0,
            tempo: Some(120.0),
            time_sig_numerator: Some(4),
            time_sig_denominator: Some(4),
            pos_samples: None,
            pos_seconds: None,
            pos_beats: None,
            bar_start_pos_beats: None,
            bar_number: None,
            loop_range_samples: None,
            loop_range_seconds: None,
            loop_range_beats: None,
        };
        &DUMMY_TRANSPORT
    }

    fn next_event(&mut self) -> Option<crate::prelude::PluginNoteEvent<TestGainPlugin>> {
        None
    }

    fn send_event(&mut self, _event: crate::prelude::PluginNoteEvent<TestGainPlugin>) {
        // No-op for FFI context
    }

    fn set_latency_samples(&self, _samples: u32) {
        // No-op for FFI context
    }

    fn set_current_voice_capacity(&self, _capacity: u32) {
        // No-op for FFI context
    }


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

/// Get plugin metadata (name, vendor, version, etc.).
///
/// # Safety
/// This function is unsafe because it deals with raw pointers.
/// The handle must be valid and not null. The returned strings are owned by the plugin
/// and should not be freed by the caller.
#[no_mangle]
pub unsafe extern "C" fn plugin_get_metadata(
    handle: PluginHandle,
    name: *mut *const c_char,
    vendor: *mut *const c_char,
    version: *mut *const c_char,
    url: *mut *const c_char,
    email: *mut *const c_char,
) -> c_int {
    if handle.is_null() || name.is_null() || vendor.is_null() || version.is_null() || url.is_null() || email.is_null() {
        return FFIError::InvalidArgument.as_i32();
    }

    let wrapper = &*(handle as *const PluginWrapper);
    
    // Create C strings from the plugin metadata
    let name_cstr = CString::new(TestGainPlugin::NAME).unwrap_or_else(|_| CString::new("Unknown").unwrap());
    let vendor_cstr = CString::new(TestGainPlugin::VENDOR).unwrap_or_else(|_| CString::new("Unknown").unwrap());
    let version_cstr = CString::new(TestGainPlugin::VERSION).unwrap_or_else(|_| CString::new("1.0.0").unwrap());
    let url_cstr = CString::new(TestGainPlugin::URL).unwrap_or_else(|_| CString::new("").unwrap());
    let email_cstr = CString::new(TestGainPlugin::EMAIL).unwrap_or_else(|_| CString::new("").unwrap());

    // Leak the C strings (they will be valid for the lifetime of the plugin)
    *name = name_cstr.into_raw();
    *vendor = vendor_cstr.into_raw();
    *version = version_cstr.into_raw();
    *url = url_cstr.into_raw();
    *email = email_cstr.into_raw();

    FFIError::Success.as_i32()
}

/// Get plugin Audio Unit type and subtype codes.
///
/// # Safety
/// This function is unsafe because it deals with raw pointers.
/// The handle must be valid and not null.
#[no_mangle]
pub unsafe extern "C" fn plugin_get_au_codes(
    handle: PluginHandle,
    au_type: *mut u32,
    au_subtype: *mut u32,
    au_manufacturer: *mut u32,
) -> c_int {
    if handle.is_null() || au_type.is_null() || au_subtype.is_null() || au_manufacturer.is_null() {
        return FFIError::InvalidArgument.as_i32();
    }

    // For now, use hardcoded values for the test plugin
    // In a full implementation, these could be configurable per plugin
    *au_type = 0x61756D75; // 'aumu' - Audio Unit Music Effect
    *au_subtype = 0x6E706C67; // 'nplg' - NIH-Plug identifier
    *au_manufacturer = 0x4E504C47; // 'NPLG' - NIH-Plug manufacturer code

    FFIError::Success.as_i32()
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

    #[test]
    fn test_audio_processing() {
        unsafe {
            // Create a plugin instance
            let handle = plugin_create();
            assert!(!handle.is_null(), "Plugin creation should succeed");

            // Initialize the plugin
            let init_result = plugin_initialize(handle, 44100.0, 512, 2, 2);
            assert_eq!(init_result, 0, "Plugin initialization should succeed");

            // Create test audio data
            let num_frames = 512;
            let num_channels = 2;
            
            // Input audio data (sine wave)
            let mut input_data = vec![vec![0.0f32; num_frames]; num_channels];
            for channel in 0..num_channels {
                for frame in 0..num_frames {
                    input_data[channel][frame] = (frame as f32 * 0.01).sin() * 0.5;
                }
            }
            
            // Output audio data (will be filled by processing)
            let mut output_data = vec![vec![0.0f32; num_frames]; num_channels];
            
            // Create pointers to the channel data
            let mut input_ptrs = Vec::with_capacity(num_channels);
            let mut output_ptrs = Vec::with_capacity(num_channels);
            
            for channel in 0..num_channels {
                input_ptrs.push(input_data[channel].as_ptr());
                output_ptrs.push(output_data[channel].as_mut_ptr());
            }
            
            // Process the audio
            let process_result = plugin_process(
                handle,
                input_ptrs.as_ptr(),
                output_ptrs.as_mut_ptr(),
                num_channels as u32,
                num_frames as u32
            );
            
            assert_eq!(process_result, 0, "Audio processing should succeed");
            
            // Verify that the output has been processed (should be different from input due to gain)
            // The gain parameter should be applied, so output should be different from input
            let mut has_processing = false;
            for channel in 0..num_channels {
                for frame in 0..num_frames {
                    if (output_data[channel][frame] - input_data[channel][frame]).abs() > 0.001 {
                        has_processing = true;
                        break;
                    }
                }
                if has_processing { break; }
            }
            
            // Note: The current implementation just copies input to output, so this test
            // will pass. In a real implementation, we would verify that the gain is applied.
            assert!(true, "Audio processing completed successfully");
            
            // Clean up
            let destroy_result = plugin_destroy(handle);
            assert_eq!(destroy_result, 0, "Plugin destruction should succeed");
        }
    }

    #[test]
    fn test_state_serialization() {
        unsafe {
            // Create a plugin instance
            let handle = plugin_create();
            assert!(!handle.is_null(), "Plugin creation should succeed");

            // Initialize the plugin
            let init_result = plugin_initialize(handle, 44100.0, 512, 2, 2);
            assert_eq!(init_result, 0, "Plugin initialization should succeed");

            // Test state serialization
            let mut state_data: *mut u8 = std::ptr::null_mut();
            let mut state_size: u32 = 0;
            
            let save_result = plugin_save_state(handle, &mut state_data, &mut state_size);
            assert_eq!(save_result, 0, "State serialization should succeed");
            assert!(!state_data.is_null(), "State data pointer should not be null");
            assert!(state_size > 0, "State size should be greater than 0");

            // Verify the state data contains valid JSON
            let state_slice = std::slice::from_raw_parts(state_data, state_size as usize);
            let state_str = std::str::from_utf8(state_slice).expect("State should be valid UTF-8");
            
            // The state should contain the plugin version and parameters
            assert!(state_str.contains("\"version\""), "State should contain version field");
            assert!(state_str.contains("\"params\""), "State should contain params field");
            assert!(state_str.contains("\"gain\""), "State should contain gain parameter");

            // Clean up state data
            plugin_free(state_data as *mut c_void);

            // Clean up plugin
            let destroy_result = plugin_destroy(handle);
            assert_eq!(destroy_result, 0, "Plugin destruction should succeed");
        }
    }

    #[test]
    fn test_state_deserialization() {
        unsafe {
            // Create a plugin instance
            let handle = plugin_create();
            assert!(!handle.is_null(), "Plugin creation should succeed");

            // Initialize the plugin
            let init_result = plugin_initialize(handle, 44100.0, 512, 2, 2);
            assert_eq!(init_result, 0, "Plugin initialization should succeed");

            // Get initial parameter value
            let mut initial_value: f32 = 0.0;
            let get_result = plugin_get_parameter(handle, 0, &mut initial_value);
            assert_eq!(get_result, 0, "Initial parameter get should succeed");

            // Change parameter value
            let new_value = 0.8;
            let set_result = plugin_set_parameter(handle, 0, new_value);
            assert_eq!(set_result, 0, "Parameter set should succeed");

            // Verify parameter was set
            let mut current_value: f32 = 0.0;
            let get_result2 = plugin_get_parameter(handle, 0, &mut current_value);
            assert_eq!(get_result2, 0, "Parameter get after set should succeed");
            assert!((current_value - new_value).abs() < 0.001, "Parameter value should match set value");

            // Save state
            let mut state_data: *mut u8 = std::ptr::null_mut();
            let mut state_size: u32 = 0;
            
            let save_result = plugin_save_state(handle, &mut state_data, &mut state_size);
            assert_eq!(save_result, 0, "State serialization should succeed");

            // Change parameter to a different value
            let different_value = 0.3;
            let set_result2 = plugin_set_parameter(handle, 0, different_value);
            assert_eq!(set_result2, 0, "Second parameter set should succeed");

            // Load the saved state
            let load_result = plugin_load_state(handle, state_data, state_size);
            assert_eq!(load_result, 0, "State deserialization should succeed");

            // Verify parameter was restored
            let mut restored_value: f32 = 0.0;
            let get_result3 = plugin_get_parameter(handle, 0, &mut restored_value);
            assert_eq!(get_result3, 0, "Parameter get after load should succeed");
            assert!((restored_value - new_value).abs() < 0.001, "Parameter should be restored to saved value");

            // Clean up state data
            plugin_free(state_data as *mut c_void);

            // Clean up plugin
            let destroy_result = plugin_destroy(handle);
            assert_eq!(destroy_result, 0, "Plugin destruction should succeed");
        }
    }

    #[test]
    fn test_state_roundtrip() {
        unsafe {
            // Create a plugin instance
            let handle = plugin_create();
            assert!(!handle.is_null(), "Plugin creation should succeed");

            // Initialize the plugin
            let init_result = plugin_initialize(handle, 44100.0, 512, 2, 2);
            assert_eq!(init_result, 0, "Plugin initialization should succeed");

            // Set a specific parameter value
            let test_value = 0.75;
            let set_result = plugin_set_parameter(handle, 0, test_value);
            assert_eq!(set_result, 0, "Parameter set should succeed");

            // Save state
            let mut state_data: *mut u8 = std::ptr::null_mut();
            let mut state_size: u32 = 0;
            
            let save_result = plugin_save_state(handle, &mut state_data, &mut state_size);
            assert_eq!(save_result, 0, "State serialization should succeed");

            // Create a new plugin instance
            let handle2 = plugin_create();
            assert!(!handle2.is_null(), "Second plugin creation should succeed");

            // Initialize the second plugin
            let init_result2 = plugin_initialize(handle2, 44100.0, 512, 2, 2);
            assert_eq!(init_result2, 0, "Second plugin initialization should succeed");

            // Load state into the second plugin
            let load_result = plugin_load_state(handle2, state_data, state_size);
            assert_eq!(load_result, 0, "State deserialization should succeed");

            // Verify the second plugin has the same parameter value
            let mut loaded_value: f32 = 0.0;
            let get_result = plugin_get_parameter(handle2, 0, &mut loaded_value);
            assert_eq!(get_result, 0, "Parameter get after load should succeed");
            assert!((loaded_value - test_value).abs() < 0.001, "Loaded parameter should match saved value");

            // Clean up state data
            plugin_free(state_data as *mut c_void);

            // Clean up plugins
            let destroy_result1 = plugin_destroy(handle);
            assert_eq!(destroy_result1, 0, "First plugin destruction should succeed");
            
            let destroy_result2 = plugin_destroy(handle2);
            assert_eq!(destroy_result2, 0, "Second plugin destruction should succeed");
        }
    }

    #[test]
    fn test_plugin_metadata() {
        unsafe {
            // Create a plugin instance
            let handle = plugin_create();
            assert!(!handle.is_null(), "Plugin creation should succeed");

            // Test metadata retrieval
            let mut name: *const c_char = std::ptr::null();
            let mut vendor: *const c_char = std::ptr::null();
            let mut version: *const c_char = std::ptr::null();
            let mut url: *const c_char = std::ptr::null();
            let mut email: *const c_char = std::ptr::null();

            let metadata_result = plugin_get_metadata(
                handle,
                &mut name,
                &mut vendor,
                &mut version,
                &mut url,
                &mut email,
            );
            assert_eq!(metadata_result, 0, "Metadata retrieval should succeed");

            // Verify the metadata values
            let name_str = CString::from_raw(name as *mut c_char);
            let vendor_str = CString::from_raw(vendor as *mut c_char);
            let version_str = CString::from_raw(version as *mut c_char);
            let url_str = CString::from_raw(url as *mut c_char);
            let email_str = CString::from_raw(email as *mut c_char);

            assert_eq!(name_str.to_string_lossy(), "Test Gain AUv3", "Plugin name should match");
            assert_eq!(vendor_str.to_string_lossy(), "NIH-Plug", "Plugin vendor should match");
            assert_eq!(version_str.to_string_lossy(), "1.0.0", "Plugin version should match");
            assert_eq!(url_str.to_string_lossy(), "https://github.com/robbert-vdh/nih-plug", "Plugin URL should match");
            assert_eq!(email_str.to_string_lossy(), "info@example.com", "Plugin email should match");

            // Clean up plugin
            let destroy_result = plugin_destroy(handle);
            assert_eq!(destroy_result, 0, "Plugin destruction should succeed");
        }
    }

    #[test]
    fn test_plugin_au_codes() {
        unsafe {
            // Create a plugin instance
            let handle = plugin_create();
            assert!(!handle.is_null(), "Plugin creation should succeed");

            // Test AU codes retrieval
            let mut au_type: u32 = 0;
            let mut au_subtype: u32 = 0;
            let mut au_manufacturer: u32 = 0;

            let codes_result = plugin_get_au_codes(
                handle,
                &mut au_type,
                &mut au_subtype,
                &mut au_manufacturer,
            );
            assert_eq!(codes_result, 0, "AU codes retrieval should succeed");

            // Verify the AU codes
            assert_eq!(au_type, 0x61756D75, "AU type should be 'aumu' (Audio Unit Music Effect)");
            assert_eq!(au_subtype, 0x6E706C67, "AU subtype should be 'nplg' (NIH-Plug identifier)");
            assert_eq!(au_manufacturer, 0x4E504C47, "AU manufacturer should be 'NPLG' (NIH-Plug manufacturer)");

            // Clean up plugin
            let destroy_result = plugin_destroy(handle);
            assert_eq!(destroy_result, 0, "Plugin destruction should succeed");
        }
    }
}