# AUv3 Comprehensive Architecture Summary for NIH-plug Integration

## Executive Summary

AUv3 represents a complete architectural shift from AUv2, requiring a Swift/Objective-C wrapper layer to bridge between the modern AVAudioUnit framework and NIH-plug's Rust core. This summary integrates all research findings into a cohesive implementation strategy.

## Architecture Overview

### Core Components
1. **Rust FFI Layer** - C-compatible exports for plugin functionality
2. **Swift AUAudioUnit Subclass** - Main plugin wrapper implementing AUv3 API
3. **Parameter Management System** - AUParameterTree integration with NIH-plug parameters
4. **Audio Processing Pipeline** - internalRenderBlock bridging to NIH-plug Buffer system
5. **State Management** - fullState serialization/deserialization
6. **Bundle Generation** - .appex bundle creation and Xcode project generation

## Key Architectural Differences from AUv2

| Aspect | AUv2 (Deprecated) | AUv3 (Modern) |
|--------|-------------------|---------------|
| **Bundle Format** | `.component` | `.appex` (app extension) |
| **API Framework** | C AudioComponent API | Swift AVAudioUnit framework |
| **Parameter System** | Manual callbacks | AUParameterTree with automation |
| **Audio Processing** | Direct buffer manipulation | internalRenderBlock with AVAudioPCMBuffer |
| **State Management** | Custom serialization | fullState property |
| **Platform Support** | macOS only | macOS 10.11+ and iOS 9+ |

## Implementation Strategy

### Phase 1: FFI Foundation
- **Rust Exports**: C-compatible functions for plugin lifecycle, parameters, audio processing, and state
- **C Headers**: Generated headers for Swift integration
- **Memory Management**: Safe ownership transfer between Rust and Swift
- **Error Handling**: Comprehensive error codes and validation

### Phase 2: Swift Wrapper
- **AUAudioUnit Subclass**: Main plugin class implementing AUv3 API
- **Parameter Mapping**: Convert NIH-plug parameters to AUParameterTree
- **Audio Processing**: internalRenderBlock bridging to Rust audio processing
- **State Management**: fullState integration with Rust state system

### Phase 3: Bundle Generation
- **Xcode Project**: Automated generation from plugin metadata
- **Info.plist**: App extension configuration with AudioComponents
- **Code Signing**: Development and distribution certificate handling
- **Installation**: Bundle placement and discovery

## Technical Implementation Details

### FFI Interface Design
```rust
// Core plugin lifecycle
pub extern "C" fn plugin_create() -> PluginHandle
pub extern "C" fn plugin_destroy(handle: PluginHandle)

// Parameter management
pub extern "C" fn plugin_get_parameter_count(handle: PluginHandle) -> u32
pub extern "C" fn plugin_set_parameter(handle: PluginHandle, id: u32, value: f32) -> i32
pub extern "C" fn plugin_get_parameter(handle: PluginHandle, id: u32) -> f32

// Audio processing
pub extern "C" fn plugin_process(handle: PluginHandle, input: *const f32, output: *mut f32, frames: u32, channels: u32) -> ProcessResult

// State management
pub extern "C" fn plugin_save_state(handle: PluginHandle, data: *mut u8, max_size: u32, actual_size: *mut u32) -> i32
pub extern "C" fn plugin_load_state(handle: PluginHandle, data: *const u8, size: u32) -> i32
```

### Swift Integration Pattern
```swift
class NIHPlugAUAudioUnit: AUAudioUnit {
    private var rustPlugin: PluginHandle?
    private var parameterMap: [AUParameterAddress: UInt32] = [:]
    
    override var internalRenderBlock: AUInternalRenderBlock {
        return { [weak self] (actionFlags, timestamp, frameCount, outputBusNumber, outputData, realtimeEventListHead, pullInputBlock) in
            // Convert AVAudioPCMBuffer to raw buffers
            // Call Rust audio processing via FFI
            // Handle results and errors
        }
    }
}
```

### Parameter Management Integration
- **AUParameterTree**: Automatic parameter automation and host integration
- **Parameter Mapping**: Bidirectional mapping between AUParameter and NIH-plug parameters
- **Value Conversion**: String conversion, unit mapping, and validation
- **Automation Support**: Sample-accurate parameter changes

### Audio Processing Pipeline
- **Buffer Conversion**: AVAudioPCMBuffer ↔ NIH-plug Buffer system
- **Multi-channel Support**: Proper handling of input/output channel layouts
- **Real-time Constraints**: Non-blocking, non-allocating audio processing
- **Error Handling**: Graceful fallback and error recovery

## Bundle Structure and Build Process

### .appex Bundle Layout
```
MyPlugin.appex/
├── Contents/
│   ├── MacOS/MyPlugin          # Swift binary + Rust static library
│   ├── Info.plist              # App extension metadata
│   └── Resources/              # Plugin resources
```

### Build Integration
1. **Rust Compilation**: Static library generation with FFI exports
2. **Xcode Project**: Automated generation from plugin metadata
3. **Swift Compilation**: AUAudioUnit subclass with Rust integration
4. **Bundle Creation**: .appex bundle assembly and code signing
5. **Installation**: Bundle placement in Audio Unit directories

## Integration with Existing NIH-plug Architecture

### Reusable Components
- **Parameter System**: NIH-plug parameter traits and validation
- **Buffer Management**: Existing Buffer and BufferConfig systems
- **State Serialization**: NIH-plug state management patterns
- **Error Handling**: NIH-plug error handling conventions
- **Logging**: nih_log! macro integration

### New Requirements
- **FFI Layer**: C-compatible exports for Swift integration
- **Swift Wrapper**: AUAudioUnit subclass implementation
- **Bundle Generation**: Xcode project and .appex bundle creation
- **Platform Support**: macOS 10.11+ and iOS 9+ compatibility

## Testing and Validation Strategy

### Unit Testing
- **FFI Functions**: Test all C-compatible exports
- **Parameter Mapping**: Verify parameter conversion accuracy
- **Audio Processing**: Test buffer conversion and processing
- **State Management**: Test serialization/deserialization

### Integration Testing
- **AU Lab**: Basic functionality and parameter access
- **Logic Pro**: Real-world DAW integration
- **GarageBand**: iOS compatibility testing
- **Performance**: Real-time audio processing validation

## Success Metrics

### Phase 1 Success (FFI Foundation)
- [ ] All FFI functions implemented and tested
- [ ] C headers generated and validated
- [ ] Memory management working correctly
- [ ] Error handling comprehensive

### Phase 2 Success (Swift Wrapper)
- [ ] AUAudioUnit subclass compiles and loads
- [ ] Parameters visible and controllable in DAW
- [ ] Audio processing functional
- [ ] State save/load working

### Phase 3 Success (Bundle Generation)
- [ ] .appex bundle builds and installs
- [ ] Plugin discoverable in DAWs
- [ ] All features working in real DAWs
- [ ] Ready for distribution

## Next Steps

1. **Implement FFI Layer**: Create Rust exports for all plugin functionality
2. **Create Swift Wrapper**: Implement AUAudioUnit subclass with FFI integration
3. **Build Bundle System**: Generate Xcode projects and .appex bundles
4. **Test Integration**: Validate functionality in real DAWs
5. **Optimize Performance**: Ensure real-time audio processing requirements

This architecture provides a complete path from NIH-plug's Rust-first approach to modern AUv3 plugin support, maintaining the existing codebase patterns while adding the necessary Swift/Objective-C integration layer.