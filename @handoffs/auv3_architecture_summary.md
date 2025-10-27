# AUv3 Architecture Summary for NIH-plug Integration

## Executive Summary

This document provides a comprehensive architecture overview for integrating NIH-plug with Audio Unit v3 (AUv3). The integration requires a Swift/Objective-C wrapper layer that bridges between Apple's AVAudioUnit framework and NIH-plug's Rust-based plugin system.

## Architecture Overview

### 1. High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Host DAW (Logic Pro, etc.)              │
└─────────────────────┬───────────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────────┐
│                AUv3 App Extension (.appex)                 │
│  ┌─────────────────────────────────────────────────────────┐│
│  │              Swift AUAudioUnit Subclass                 ││
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌──────────┐││
│  │  │   Parameter     │  │   Audio         │  │  State   │││
│  │  │   Management    │  │   Processing    │  │  Mgmt    │││
│  │  └─────────────────┘  └─────────────────┘  └──────────┘││
│  └─────────────────────┬─────────────────────────────────┘│
│                        │                                   │
│  ┌─────────────────────▼─────────────────────────────────┐│
│  │              FFI Layer (C Headers)                   ││
│  └─────────────────────┬─────────────────────────────────┘│
└────────────────────────┼───────────────────────────────────┘
                         │
┌────────────────────────▼───────────────────────────────────┐
│                Rust Plugin Core                            │
│  ┌─────────────────────────────────────────────────────────┐│
│  │              NIH-plug Plugin Trait                     ││
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐││
│  │  │ Parameters  │ │ Audio      │ │ State               │││
│  │  │             │ │ Processing │ │ Serialization       │││
│  │  └─────────────┘ └─────────────┘ └─────────────────────┘││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

### 2. Component Breakdown

#### Swift Layer (AUv3 App Extension)
- **AUAudioUnit Subclass**: Main entry point for AUv3
- **Parameter Management**: AUParameterTree integration
- **Audio Processing**: internalRenderBlock implementation
- **State Management**: fullState property handling
- **Bundle Structure**: .appex bundle with Info.plist

#### FFI Layer (C Headers)
- **Plugin Lifecycle**: create/destroy functions
- **Parameter Interface**: get/set parameter functions
- **Audio Processing**: process audio function
- **State Interface**: save/load state functions
- **Error Handling**: error codes and validation

#### Rust Layer (NIH-plug Core)
- **Plugin Trait**: Existing NIH-plug plugin interface
- **Parameter System**: Existing parameter management
- **Audio Processing**: Existing audio processing pipeline
- **State System**: Existing state serialization

## Key Integration Points

### 1. Parameter Management

**Swift Side:**
```swift
class ParameterManager {
    private var parameterTree: AUParameterTree?
    private var rustPlugin: PluginHandle?
    
    func setupParameters() {
        // Create AUParameter instances from Rust plugin
        // Set up parameter observers and providers
        // Handle parameter automation
    }
}
```

**Rust Side:**
```rust
// FFI functions for parameter management
#[no_mangle]
pub extern "C" fn plugin_get_parameter_count(handle: PluginHandle) -> u32

#[no_mangle]
pub extern "C" fn plugin_set_parameter(handle: PluginHandle, param_id: u32, value: f32) -> i32
```

### 2. Audio Processing

**Swift Side:**
```swift
override var internalRenderBlock: AUInternalRenderBlock {
    return { [weak self] (actionFlags, timestamp, frameCount, outputBusNumber, outputData, realtimeEventListHead, pullInputBlock) in
        // Convert AVAudioPCMBuffer to raw floats
        // Call Rust audio processing
        // Handle results and errors
    }
}
```

**Rust Side:**
```rust
#[no_mangle]
pub extern "C" fn plugin_process(
    handle: PluginHandle,
    input: *const f32,
    output: *mut f32,
    num_frames: u32,
    num_channels: u32
) -> ProcessResult
```

### 3. State Management

**Swift Side:**
```swift
override var fullState: [String : Any]? {
    get {
        // Serialize state from Rust plugin
        // Return as dictionary
    }
    set {
        // Deserialize state to Rust plugin
        // Handle errors gracefully
    }
}
```

**Rust Side:**
```rust
#[no_mangle]
pub extern "C" fn plugin_save_state(handle: PluginHandle, data: *mut u8, max_size: u32, actual_size: *mut u32) -> i32

#[no_mangle]
pub extern "C" fn plugin_load_state(handle: PluginHandle, data: *const u8, size: u32) -> i32
```

## Implementation Strategy

### Phase 1: FFI Foundation
1. Create C-compatible function signatures
2. Implement basic plugin lifecycle functions
3. Add parameter get/set functions
4. Implement audio processing function
5. Add state serialization functions

### Phase 2: Swift Wrapper
1. Create AUAudioUnit subclass
2. Implement parameter management
3. Implement audio processing
4. Implement state management
5. Add error handling and validation

### Phase 3: Bundle Creation
1. Create .appex bundle structure
2. Generate Info.plist from plugin metadata
3. Integrate with existing bundler system
4. Add code signing support
5. Test bundle installation

### Phase 4: Testing and Validation
1. Test in AU Lab
2. Test in Logic Pro
3. Test parameter automation
4. Test state save/load
5. Performance profiling

## Technical Requirements

### 1. Build System
- **Rust**: Compile as static library (.a)
- **Swift**: Create Xcode project for app extension
- **Bundler**: Extend nih_plug_xtask for AUv3 support
- **Code Signing**: Support for development and distribution

### 2. Platform Support
- **macOS**: 10.11+ (AUv3 requirement)
- **iOS**: 9+ (if needed)
- **Architecture**: x86_64 and arm64 support

### 3. Performance Requirements
- **Real-time**: Non-blocking audio processing
- **Latency**: Minimal processing overhead
- **Memory**: Efficient buffer management
- **Threading**: Thread-safe parameter updates

## Key Differences from AUv2

| Aspect | AUv2 | AUv3 |
|--------|------|------|
| **Architecture** | C AudioComponent API | Swift AVAudioUnit framework |
| **Bundle Type** | .component | .appex |
| **Parameter System** | Manual callbacks | AUParameterTree |
| **Audio Processing** | Direct buffer manipulation | internalRenderBlock |
| **State Management** | Custom serialization | fullState property |
| **Platform Support** | macOS only | macOS + iOS |

## Integration Benefits

### 1. Modern Architecture
- Uses Apple's modern AVAudioUnit framework
- Supports both macOS and iOS
- Better integration with host DAWs
- Improved parameter automation

### 2. NIH-plug Compatibility
- Reuses existing NIH-plug abstractions
- Maintains parameter system consistency
- Preserves audio processing pipeline
- Keeps state management patterns

### 3. Developer Experience
- Swift provides better error handling
- AUParameterTree simplifies parameter management
- internalRenderBlock is more intuitive
- Better debugging and profiling tools

## Risk Mitigation

### 1. Technical Risks
- **FFI Complexity**: Use well-tested patterns
- **Performance**: Profile and optimize
- **Threading**: Use lock-free data structures
- **Memory Management**: Careful ownership handling

### 2. Platform Risks
- **macOS Version**: Target 10.11+ minimum
- **iOS Compatibility**: Test on multiple devices
- **Code Signing**: Handle development and distribution
- **Bundle Validation**: Test in multiple DAWs

## Success Metrics

### 1. Functionality
- [ ] Plugin loads in AU Lab
- [ ] Plugin loads in Logic Pro
- [ ] Parameters respond correctly
- [ ] Audio processes without artifacts
- [ ] State saves and loads correctly

### 2. Performance
- [ ] Real-time audio processing
- [ ] Low latency (< 10ms)
- [ ] Minimal CPU usage
- [ ] No memory leaks
- [ ] Thread-safe operation

### 3. Quality
- [ ] No crashes or hangs
- [ ] Graceful error handling
- [ ] Proper resource cleanup
- [ ] Consistent behavior
- [ ] Production-ready code

## Next Steps

1. **Implement FFI Layer**: Create C-compatible functions
2. **Create Swift Wrapper**: Implement AUAudioUnit subclass
3. **Extend Bundler**: Add AUv3 support to nih_plug_xtask
4. **Test Integration**: Validate in real DAWs
5. **Performance Optimization**: Profile and optimize
6. **Documentation**: Create user guides and examples

## Conclusion

The AUv3 integration represents a significant architectural shift from AUv2, requiring a Swift wrapper layer and FFI bridge to NIH-plug's Rust core. However, this approach provides modern platform support, better host integration, and maintains compatibility with NIH-plug's existing abstractions. The implementation strategy outlined above provides a clear path to successful integration while mitigating technical risks and ensuring production-quality results.