# Swift-to-Rust FFI Examples and Patterns

## Overview

This document outlines the patterns and examples for creating a Foreign Function Interface (FFI) between Swift and Rust for AUv3 plugin development. The FFI layer allows Swift AUAudioUnit subclasses to call into Rust plugin implementations.

## Basic FFI Architecture

### 1. Rust Side (C-compatible exports)

```rust
// src/wrapper/auv3/ffi.rs
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_float, c_int, c_uint, c_void};

/// Plugin instance handle (opaque pointer)
pub type PluginHandle = *mut c_void;

/// Error codes
pub const PLUGIN_OK: c_int = 0;
pub const PLUGIN_ERROR: c_int = -1;
pub const PLUGIN_INVALID_PARAM: c_int = -2;

/// Parameter information structure
#[repr(C)]
pub struct ParameterInfo {
    pub id: c_uint,
    pub name: *const c_char,
    pub min_value: c_float,
    pub max_value: c_float,
    pub default_value: c_float,
    pub unit: c_uint,
}

/// Audio processing result
#[repr(C)]
pub struct ProcessResult {
    pub status: c_int,
    pub latency: c_uint,
    pub tail_time: c_uint,
}

// Plugin lifecycle functions
#[no_mangle]
pub extern "C" fn plugin_create() -> PluginHandle {
    // Create plugin instance and return opaque handle
    Box::into_raw(Box::new(PluginInstance::new())) as PluginHandle
}

#[no_mangle]
pub extern "C" fn plugin_destroy(handle: PluginHandle) {
    if !handle.is_null() {
        unsafe {
            let _ = Box::from_raw(handle as *mut PluginInstance);
        }
    }
}

// Parameter management
#[no_mangle]
pub extern "C" fn plugin_get_parameter_count(handle: PluginHandle) -> c_uint {
    if handle.is_null() {
        return 0;
    }
    
    unsafe {
        let plugin = &*(handle as *const PluginInstance);
        plugin.get_parameter_count()
    }
}

#[no_mangle]
pub extern "C" fn plugin_get_parameter_info(
    handle: PluginHandle,
    index: c_uint,
    info: *mut ParameterInfo,
) -> c_int {
    if handle.is_null() || info.is_null() {
        return PLUGIN_INVALID_PARAM;
    }
    
    unsafe {
        let plugin = &*(handle as *const PluginInstance);
        if let Some(param_info) = plugin.get_parameter_info(index) {
            *info = param_info;
            PLUGIN_OK
        } else {
            PLUGIN_ERROR
        }
    }
}

#[no_mangle]
pub extern "C" fn plugin_set_parameter(
    handle: PluginHandle,
    param_id: c_uint,
    value: c_float,
) -> c_int {
    if handle.is_null() {
        return PLUGIN_INVALID_PARAM;
    }
    
    unsafe {
        let plugin = &mut *(handle as *mut PluginInstance);
        if plugin.set_parameter(param_id, value) {
            PLUGIN_OK
        } else {
            PLUGIN_ERROR
        }
    }
}

#[no_mangle]
pub extern "C" fn plugin_get_parameter(
    handle: PluginHandle,
    param_id: c_uint,
) -> c_float {
    if handle.is_null() {
        return 0.0;
    }
    
    unsafe {
        let plugin = &*(handle as *const PluginInstance);
        plugin.get_parameter(param_id)
    }
}

// Audio processing
#[no_mangle]
pub extern "C" fn plugin_process(
    handle: PluginHandle,
    input: *const c_float,
    output: *mut c_float,
    num_frames: c_uint,
    num_channels: c_uint,
) -> ProcessResult {
    if handle.is_null() || input.is_null() || output.is_null() {
        return ProcessResult {
            status: PLUGIN_INVALID_PARAM,
            latency: 0,
            tail_time: 0,
        };
    }
    
    unsafe {
        let plugin = &mut *(handle as *mut PluginInstance);
        let input_slice = std::slice::from_raw_parts(input, (num_frames * num_channels) as usize);
        let output_slice = std::slice::from_raw_parts_mut(output, (num_frames * num_channels) as usize);
        
        match plugin.process(input_slice, output_slice) {
            Ok(result) => ProcessResult {
                status: PLUGIN_OK,
                latency: result.latency,
                tail_time: result.tail_time,
            },
            Err(_) => ProcessResult {
                status: PLUGIN_ERROR,
                latency: 0,
                tail_time: 0,
            },
        }
    }
}

// State management
#[no_mangle]
pub extern "C" fn plugin_save_state(
    handle: PluginHandle,
    data: *mut c_char,
    max_size: c_uint,
    actual_size: *mut c_uint,
) -> c_int {
    if handle.is_null() || data.is_null() || actual_size.is_null() {
        return PLUGIN_INVALID_PARAM;
    }
    
    unsafe {
        let plugin = &*(handle as *const PluginInstance);
        if let Ok(state_data) = plugin.serialize_state() {
            let state_bytes = state_data.as_bytes();
            if state_bytes.len() <= max_size as usize {
                std::ptr::copy_nonoverlapping(
                    state_bytes.as_ptr(),
                    data as *mut u8,
                    state_bytes.len(),
                );
                *actual_size = state_bytes.len() as c_uint;
                PLUGIN_OK
            } else {
                PLUGIN_ERROR
            }
        } else {
            PLUGIN_ERROR
        }
    }
}

#[no_mangle]
pub extern "C" fn plugin_load_state(
    handle: PluginHandle,
    data: *const c_char,
    size: c_uint,
) -> c_int {
    if handle.is_null() || data.is_null() {
        return PLUGIN_INVALID_PARAM;
    }
    
    unsafe {
        let plugin = &mut *(handle as *mut PluginInstance);
        let state_bytes = std::slice::from_raw_parts(data as *const u8, size as usize);
        if plugin.deserialize_state(state_bytes).is_ok() {
            PLUGIN_OK
        } else {
            PLUGIN_ERROR
        }
    }
}
```

### 2. C Header File

```c
// include/plugin_ffi.h
#ifndef PLUGIN_FFI_H
#define PLUGIN_FFI_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque handle type
typedef void* PluginHandle;

// Error codes
#define PLUGIN_OK 0
#define PLUGIN_ERROR -1
#define PLUGIN_INVALID_PARAM -2

// Parameter information structure
typedef struct {
    uint32_t id;
    const char* name;
    float min_value;
    float max_value;
    float default_value;
    uint32_t unit;
} ParameterInfo;

// Audio processing result
typedef struct {
    int32_t status;
    uint32_t latency;
    uint32_t tail_time;
} ProcessResult;

// Plugin lifecycle
PluginHandle plugin_create(void);
void plugin_destroy(PluginHandle handle);

// Parameter management
uint32_t plugin_get_parameter_count(PluginHandle handle);
int32_t plugin_get_parameter_info(PluginHandle handle, uint32_t index, ParameterInfo* info);
int32_t plugin_set_parameter(PluginHandle handle, uint32_t param_id, float value);
float plugin_get_parameter(PluginHandle handle, uint32_t param_id);

// Audio processing
ProcessResult plugin_process(
    PluginHandle handle,
    const float* input,
    float* output,
    uint32_t num_frames,
    uint32_t num_channels
);

// State management
int32_t plugin_save_state(PluginHandle handle, char* data, uint32_t max_size, uint32_t* actual_size);
int32_t plugin_load_state(PluginHandle handle, const char* data, uint32_t size);

#ifdef __cplusplus
}
#endif

#endif // PLUGIN_FFI_H
```

### 3. Swift Side (AUAudioUnit subclass)

```swift
// MyPluginAUAudioUnit.swift
import AudioToolbox
import AVFoundation

@objc(MyPluginAUAudioUnit)
class MyPluginAUAudioUnit: AUAudioUnit {
    private var rustPlugin: PluginHandle?
    private var parameterMap: [AUParameterAddress: UInt32] = [:]
    
    // FFI function declarations
    private let pluginCreate: @convention(c) () -> PluginHandle
    private let pluginDestroy: @convention(c) (PluginHandle?) -> Void
    private let pluginGetParameterCount: @convention(c) (PluginHandle?) -> UInt32
    private let pluginGetParameterInfo: @convention(c) (PluginHandle?, UInt32, UnsafeMutablePointer<ParameterInfo>) -> Int32
    private let pluginSetParameter: @convention(c) (PluginHandle?, UInt32, Float) -> Int32
    private let pluginGetParameter: @convention(c) (PluginHandle?, UInt32) -> Float
    private let pluginProcess: @convention(c) (PluginHandle?, UnsafePointer<Float>, UnsafeMutablePointer<Float>, UInt32, UInt32) -> ProcessResult
    private let pluginSaveState: @convention(c) (PluginHandle?, UnsafeMutablePointer<CChar>, UInt32, UnsafeMutablePointer<UInt32>) -> Int32
    private let pluginLoadState: @convention(c) (PluginHandle?, UnsafePointer<CChar>, UInt32) -> Int32
    
    override init(componentDescription: AudioComponentDescription) throws {
        // Load the Rust library
        guard let library = dlopen("libmy_plugin_rust.a", RTLD_NOW) else {
            throw NSError(domain: "MyPlugin", code: -1, userInfo: [NSLocalizedDescriptionKey: "Failed to load Rust library"])
        }
        
        // Get function pointers
        pluginCreate = dlsym(library, "plugin_create")
        pluginDestroy = dlsym(library, "plugin_destroy")
        pluginGetParameterCount = dlsym(library, "plugin_get_parameter_count")
        pluginGetParameterInfo = dlsym(library, "plugin_get_parameter_info")
        pluginSetParameter = dlsym(library, "plugin_set_parameter")
        pluginGetParameter = dlsym(library, "plugin_get_parameter")
        pluginProcess = dlsym(library, "plugin_process")
        pluginSaveState = dlsym(library, "plugin_save_state")
        pluginLoadState = dlsym(library, "plugin_load_state")
        
        try super.init(componentDescription: componentDescription)
        
        // Initialize Rust plugin
        rustPlugin = pluginCreate()
        setupAudioBusses()
        setupParameters()
    }
    
    deinit {
        if let plugin = rustPlugin {
            pluginDestroy(plugin)
        }
    }
    
    private func setupAudioBusses() {
        // Set up input and output busses based on plugin requirements
        let inputBus = try! AUAudioUnitBus(format: AVAudioFormat(standardFormatWithSampleRate: 44100, channels: 2)!)
        let outputBus = try! AUAudioUnitBus(format: AVAudioFormat(standardFormatWithSampleRate: 44100, channels: 2)!)
        
        inputBusses = [inputBus]
        outputBusses = [outputBus]
    }
    
    private func setupParameters() {
        guard let plugin = rustPlugin else { return }
        
        let paramCount = pluginGetParameterCount(plugin)
        var parameters: [AUParameter] = []
        
        for i in 0..<paramCount {
            var paramInfo = ParameterInfo()
            if pluginGetParameterInfo(plugin, i, &paramInfo) == PLUGIN_OK {
                let parameter = AUParameter(
                    identifier: String(cString: paramInfo.name),
                    name: String(cString: paramInfo.name),
                    address: AUParameterAddress(i),
                    min: paramInfo.min_value,
                    max: paramInfo.max_value,
                    unit: .generic,
                    unitName: nil,
                    flags: [.flag_IsReadable, .flag_IsWritable],
                    valueStrings: nil,
                    dependentParameters: nil
                )
                
                parameters.append(parameter)
                parameterMap[AUParameterAddress(i)] = paramInfo.id
            }
        }
        
        let parameterTree = AUParameterTree.createParameterTree(withChildren: parameters)
        
        // Set up parameter observers
        parameterTree.implementorValueObserver = { [weak self] parameter, value in
            guard let self = self, let plugin = self.rustPlugin else { return }
            if let paramId = self.parameterMap[parameter.address] {
                _ = self.pluginSetParameter(plugin, paramId, value)
            }
        }
        
        parameterTree.implementorValueProvider = { [weak self] parameter in
            guard let self = self, let plugin = self.rustPlugin else { return 0.0 }
            if let paramId = self.parameterMap[parameter.address] {
                return self.pluginGetParameter(plugin, paramId)
            }
            return 0.0
        }
        
        self.parameterTree = parameterTree
    }
    
    override var internalRenderBlock: AUInternalRenderBlock {
        return { [weak self] (actionFlags, timestamp, frameCount, outputBusNumber, outputData, realtimeEventListHead, pullInputBlock) in
            guard let self = self, let plugin = self.rustPlugin else {
                return
            }
            
            // Get input audio
            var inputBuffer: AVAudioPCMBuffer?
            if let pullInputBlock = pullInputBlock {
                var inputTimeStamp = timestamp.pointee
                let status = pullInputBlock(actionFlags, &inputTimeStamp, frameCount, 0, &inputBuffer)
                guard status == noErr else { return }
            }
            
            // Convert to raw buffers
            let inputFloats = inputBuffer?.floatChannelData?[0] ?? UnsafePointer<Float>(bitPattern: 0)!
            let outputFloats = outputData.pointee.mBuffers.mData!.assumingMemoryBound(to: Float.self)
            
            // Process audio
            let result = self.pluginProcess(
                plugin,
                inputFloats,
                outputFloats,
                UInt32(frameCount),
                UInt32(outputData.pointee.mBuffers.mNumberChannels)
            )
            
            // Handle result
            if result.status != PLUGIN_OK {
                // Handle error
            }
        }
    }
    
    override var fullState: [String : Any]? {
        get {
            guard let plugin = rustPlugin else { return nil }
            
            let maxSize: UInt32 = 1024 * 1024 // 1MB max
            let buffer = UnsafeMutablePointer<CChar>.allocate(capacity: Int(maxSize))
            defer { buffer.deallocate() }
            
            var actualSize: UInt32 = 0
            if pluginSaveState(plugin, buffer, maxSize, &actualSize) == PLUGIN_OK {
                let data = Data(bytes: buffer, count: Int(actualSize))
                return ["state": data]
            }
            
            return nil
        }
        set {
            guard let plugin = rustPlugin, let state = newValue?["state"] as? Data else { return }
            
            state.withUnsafeBytes { bytes in
                _ = pluginLoadState(plugin, bytes.bindMemory(to: CChar.self).baseAddress!, UInt32(state.count))
            }
        }
    }
}
```

## Key FFI Patterns

### 1. Memory Management

**Rust Side:**
- Use `Box::into_raw()` to transfer ownership to C
- Use `Box::from_raw()` to reclaim ownership
- Always check for null pointers

**Swift Side:**
- Use `deinit` to clean up resources
- Use `weak self` in closures to avoid retain cycles
- Use `defer` for cleanup

### 2. Error Handling

**Rust Side:**
- Return error codes as integers
- Use `Result` types internally, convert to error codes at FFI boundary
- Provide detailed error information when possible

**Swift Side:**
- Check return codes from FFI calls
- Use `guard` statements for early returns
- Convert error codes to `NSError` when needed

### 3. String Handling

**Rust Side:**
- Use `CString` for owned strings
- Use `CStr` for borrowed strings
- Return `*const c_char` for string data

**Swift Side:**
- Use `String(cString:)` to convert from C strings
- Use `withCString` to convert Swift strings to C strings
- Be careful with string lifetimes

### 4. Array/Buffer Handling

**Rust Side:**
- Use `std::slice::from_raw_parts` for safe buffer access
- Validate buffer sizes before processing
- Use `unsafe` blocks only when necessary

**Swift Side:**
- Use `UnsafePointer` and `UnsafeMutablePointer` for raw data
- Use `withUnsafeBytes` for safe buffer access
- Validate buffer sizes before passing to Rust

## Build Integration

### 1. Cargo.toml

```toml
[lib]
crate-type = ["staticlib"]
name = "my_plugin_rust"

[dependencies]
# ... other dependencies
```

### 2. Xcode Project

```swift
// In Xcode project settings:
// - Add libmy_plugin_rust.a to "Link Binary With Libraries"
// - Add include/ directory to "Header Search Paths"
// - Add -lc++ to "Other Linker Flags"
```

### 3. Build Script

```bash
#!/bin/bash
# Build Rust library
cargo build --release --lib

# Copy to Xcode project
cp target/release/libmy_plugin_rust.a MyPlugin/
cp include/plugin_ffi.h MyPlugin/
```

## Testing and Debugging

### 1. Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_plugin_creation() {
        let handle = plugin_create();
        assert!(!handle.is_null());
        plugin_destroy(handle);
    }
    
    #[test]
    fn test_parameter_handling() {
        let handle = plugin_create();
        let count = plugin_get_parameter_count(handle);
        assert!(count > 0);
        plugin_destroy(handle);
    }
}
```

### 2. Swift Tests

```swift
class MyPluginTests: XCTestCase {
    func testPluginInitialization() {
        let plugin = try! MyPluginAUAudioUnit(componentDescription: componentDescription)
        XCTAssertNotNil(plugin.parameterTree)
    }
}
```

## Best Practices

### 1. Safety
- Always validate pointers before use
- Use `unsafe` blocks sparingly
- Implement proper error handling
- Test edge cases thoroughly

### 2. Performance
- Minimize FFI calls
- Use batch operations when possible
- Avoid allocations in audio processing paths
- Profile performance regularly

### 3. Maintainability
- Use clear naming conventions
- Document all FFI functions
- Keep FFI layer thin
- Separate concerns properly

## Next Steps

1. Research AUParameter and AUParameterTree implementation
2. Research internalRenderBlock audio processing
3. Create comprehensive architecture summary
4. Implement actual FFI layer
5. Create Swift wrapper implementation