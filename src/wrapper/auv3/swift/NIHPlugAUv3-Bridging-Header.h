//
//  NIHPlugAUv3-Bridging-Header.h
//  NIH-Plug AUv3 Minimal Test
//
//  Created by Implementation Ralph on $(DATE).
//  Copyright © $(YEAR) NIH-Plug. All rights reserved.
//

#ifndef NIHPlugAUv3_Bridging_Header_h
#define NIHPlugAUv3_Bridging_Header_h

#import <Foundation/Foundation.h>
#import <AudioToolbox/AudioToolbox.h>
#import <AVFoundation/AVFoundation.h>

// FFI function declarations for Rust plugin integration
// These functions are implemented in the Rust static library

// Plugin handle type
typedef void* PluginHandle;

// Error codes
typedef enum {
    FFIErrorSuccess = 0,
    FFIErrorInvalidArgument = -1,
    FFIErrorInvalidHandle = -2,
    FFIErrorInitializationFailed = -3,
    FFIErrorProcessingFailed = -4,
    FFIErrorParameterError = -5,
    FFIErrorStateError = -6,
    FFIErrorMemoryError = -7,
    FFIErrorNotInitialized = -8
} FFIError;

// Parameter information structure
typedef struct {
    uint32_t id;
    char* name;
    char* unit;
    float min_value;
    float max_value;
    float default_value;
} ParameterInfo;

// Core plugin functions
PluginHandle plugin_create(void);
int32_t plugin_destroy(PluginHandle handle);
int32_t plugin_initialize(PluginHandle handle, float sample_rate, uint32_t max_block_size, uint32_t input_channels, uint32_t output_channels);
int32_t plugin_process(PluginHandle handle, const float** input_buffers, float** output_buffers, uint32_t num_channels, uint32_t num_frames);

// Parameter functions
uint32_t plugin_get_parameter_count(PluginHandle handle);
int32_t plugin_get_parameter_info(PluginHandle handle, uint32_t param_id, ParameterInfo* info);
int32_t plugin_set_parameter(PluginHandle handle, uint32_t param_id, float normalized_value);
int32_t plugin_get_parameter(PluginHandle handle, uint32_t param_id, float* value);

// State management functions
int32_t plugin_save_state(PluginHandle handle, uint8_t** data, uint32_t* size);
int32_t plugin_load_state(PluginHandle handle, const uint8_t* data, uint32_t size);

// Metadata functions
int32_t plugin_get_metadata(PluginHandle handle, char** name, char** vendor, char** version, char** url, char** email);
int32_t plugin_get_au_codes(PluginHandle handle, uint32_t* au_type, uint32_t* au_subtype, uint32_t* au_manufacturer);

// Memory management
void plugin_free(void* ptr);

#endif /* NIHPlugAUv3_Bridging_Header_h */