//
//  NIHPlugAUv3-Bridging-Header.h
//  NIH-Plug AUv3
//
//  Created by NIH-Plug on $(DATE).
//  Copyright © $(YEAR) NIH-Plug. All rights reserved.
//

#ifndef NIHPlugAUv3_Bridging_Header_h
#define NIHPlugAUv3_Bridging_Header_h

#import <Foundation/Foundation.h>
#import <AudioToolbox/AudioToolbox.h>
#import <AVFoundation/AVFoundation.h>

// FFI Function Declarations
// These functions are implemented in the Rust static library

/// Create a new plugin instance.
/// @return Plugin handle or NULL on failure
void* plugin_create(void);

/// Destroy a plugin instance.
/// @param handle Plugin handle to destroy
/// @return 0 on success, negative error code on failure
int32_t plugin_destroy(void* handle);

/// Initialize a plugin with the given configuration.
/// @param handle Plugin handle
/// @param sample_rate Sample rate in Hz
/// @param max_block_size Maximum block size
/// @param input_channels Number of input channels
/// @param output_channels Number of output channels
/// @return 0 on success, negative error code on failure
int32_t plugin_initialize(void* handle, float sample_rate, uint32_t max_block_size, uint32_t input_channels, uint32_t output_channels);

/// Process audio with the given buffers.
/// @param handle Plugin handle
/// @param input_buffers Array of input buffer pointers
/// @param output_buffers Array of output buffer pointers
/// @param num_channels Number of channels
/// @param num_frames Number of frames to process
/// @return 0 on success, negative error code on failure
int32_t plugin_process(void* handle, const float* const* input_buffers, float** output_buffers, uint32_t num_channels, uint32_t num_frames);

/// Get the number of parameters.
/// @param handle Plugin handle
/// @return Number of parameters
uint32_t plugin_get_parameter_count(void* handle);

/// Get parameter information.
/// @param handle Plugin handle
/// @param param_id Parameter ID
/// @param info Output parameter info structure
/// @return 0 on success, negative error code on failure
int32_t plugin_get_parameter_info(void* handle, uint32_t param_id, struct ParameterInfo* info);

/// Set a parameter value.
/// @param handle Plugin handle
/// @param param_id Parameter ID
/// @param normalized_value Normalized parameter value (0.0 to 1.0)
/// @return 0 on success, negative error code on failure
int32_t plugin_set_parameter(void* handle, uint32_t param_id, float normalized_value);

/// Get a parameter value.
/// @param handle Plugin handle
/// @param param_id Parameter ID
/// @param value Output parameter value
/// @return 0 on success, negative error code on failure
int32_t plugin_get_parameter(void* handle, uint32_t param_id, float* value);

/// Save plugin state.
/// @param handle Plugin handle
/// @param data Output state data pointer
/// @param size Output state data size
/// @return 0 on success, negative error code on failure
int32_t plugin_save_state(void* handle, uint8_t** data, uint32_t* size);

/// Load plugin state.
/// @param handle Plugin handle
/// @param data State data pointer
/// @param size State data size
/// @return 0 on success, negative error code on failure
int32_t plugin_load_state(void* handle, const uint8_t* data, uint32_t size);

/// Free memory allocated by the FFI layer.
/// @param ptr Pointer to free
void plugin_free(void* ptr);

// MARK: - C Structure Definitions

/// Parameter information structure for FFI.
struct ParameterInfo {
    uint32_t id;
    const char* name;
    const char* unit;
    float min_value;
    float max_value;
    float default_value;
};

// MARK: - Error Codes

/// FFI error codes
typedef enum {
    FFI_SUCCESS = 0,
    FFI_INVALID_ARGUMENT = -1,
    FFI_INVALID_HANDLE = -2,
    FFI_INITIALIZATION_FAILED = -3,
    FFI_PROCESSING_FAILED = -4,
    FFI_PARAMETER_ERROR = -5,
    FFI_STATE_ERROR = -6,
    FFI_MEMORY_ERROR = -7,
    FFI_NOT_INITIALIZED = -8
} FFIError;

#endif /* NIHPlugAUv3_Bridging_Header_h */