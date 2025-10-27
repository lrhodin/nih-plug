# AUv3 Architecture Summary for NIH-plug Integration

## Executive Summary

AUv3 represents a complete architectural shift from AUv2, requiring a Swift/Objective-C wrapper layer to bridge between the modern AVAudioUnit framework and NIH-plug's Rust core.

## Key Architectural Changes

### 1. **Bundle Format**
- **AUv2**: `.component` bundles with C AudioComponent API
- **AUv3**: `.appex` app extension bundles with Swift AVAudioUnit framework

### 2. **Integration Approach**
- **AUv2**: Direct C integration with function pointers
- **AUv3**: Swift AUAudioUnit subclass + FFI layer to Rust

### 3. **Parameter Management**
- **AUv2**: Manual parameter callbacks and ID mapping
- **AUv3**: AUParameter/AUParameterTree with automatic management

### 4. **Audio Processing**
- **AUv2**: Direct buffer manipulation with C callbacks
- **AUv3**: AVAudioPCMBuffer objects with internalRenderBlock

## NIH-plug Integration Strategy

### What We Can Reuse
- NIH-plug integration patterns from AUv2 implementation
- Parameter mapping and state serialization concepts
- Buffer handling logic (adapted for AVAudioPCMBuffer)
- Error handling patterns

### What Must Be Built New
- Swift AUAudioUnit subclass wrapper
- FFI layer between Swift and Rust
- Xcode project generation system
- App extension bundle creation
- AVAudioUnit framework integration

## Implementation Phases

1. **FFI Foundation**: Create Rust exports callable from Swift
2. **Swift Wrapper**: Implement AUAudioUnit subclass
3. **Parameter System**: Map NIH-plug parameters to AUParameterTree
4. **Audio Processing**: Bridge AVAudioPCMBuffer to NIH-plug Buffer
5. **State Management**: Implement fullState serialization
6. **Build Automation**: Generate Xcode projects and bundles

## Technical Requirements

- **Platform**: macOS 10.11+ and iOS 9+
- **Languages**: Swift/Objective-C + Rust FFI
- **Frameworks**: AVAudioUnit, AudioToolbox
- **Build System**: Xcode project generation + Rust compilation
- **Bundle Type**: App extension (.appex)

## Next Steps

1. Create FFI layer with basic plugin exports
2. Implement minimal Swift AUAudioUnit subclass
3. Test basic plugin loading and parameter access
4. Build complete audio processing pipeline
5. Add state management and preset support

This architecture allows NIH-plug to support modern AUv3 plugins while maintaining the existing Rust-first approach through a clean FFI interface.