# AUv2 vs AUv3 Architectural Differences

## Overview

Audio Unit v3 (AUv3) represents a fundamental architectural shift from AUv2, moving from a C-based component system to a modern Swift/Objective-C app extension framework.

## Key Architectural Differences

### 1. **Component vs App Extension**

**AUv2 (Deprecated):**
- Uses `.component` bundles
- C-based AudioComponent API
- Loaded as system components
- Direct C function callbacks

**AUv3 (Modern):**
- Uses `.appex` app extension bundles
- Swift/Objective-C AVAudioUnit framework
- Loaded as app extensions
- Object-oriented subclassing approach

### 2. **API Framework**

**AUv2:**
- AudioToolbox framework
- C AudioComponent API
- Direct function pointers for callbacks
- Manual memory management

**AUv3:**
- AVAudioUnit framework
- Swift/Objective-C class hierarchy
- Method-based callbacks
- Automatic reference counting (ARC)

### 3. **Parameter Management**

**AUv2:**
- Direct parameter get/set callbacks
- Manual parameter ID mapping
- Custom parameter change scheduling

**AUv3:**
- AUParameter and AUParameterTree classes
- Automatic parameter tree management
- Built-in parameter automation support
- Observer pattern for parameter changes

### 4. **Audio Processing**

**AUv2:**
- Direct buffer manipulation
- Manual buffer list handling
- C-style audio processing callbacks

**AUv3:**
- AVAudioPCMBuffer objects
- internalRenderBlock for processing
- Automatic buffer management
- Higher-level audio processing interface

### 5. **State Management**

**AUv2:**
- Custom state serialization
- Manual preset handling
- Direct property access

**AUv3:**
- fullState property for serialization
- Built-in preset management
- AUAudioUnitPreset support
- Automatic state restoration

### 6. **Bundle Structure**

**AUv2:**
```
MyPlugin.component/
  Contents/
    MacOS/MyPlugin          # Mach-O component
    Info.plist              # Component metadata
    Resources/
```

**AUv3:**
```
MyPlugin.appex/
  Contents/
    MacOS/MyPlugin          # Mach-O app extension
    Info.plist              # Extension metadata
    Resources/
```

### 7. **Platform Support**

**AUv2:**
- macOS only
- Deprecated on newer macOS versions
- No iOS support

**AUv3:**
- macOS 10.11+ and iOS 9+
- Modern, actively supported
- Cross-platform compatibility

### 8. **Integration Approach**

**AUv2:**
- Direct C integration
- Static linking possible
- Simple FFI requirements

**AUv3:**
- Swift/Objective-C wrapper required
- FFI layer between Swift and Rust
- More complex build process
- Requires Xcode project generation

## Implications for NIH-plug Integration

### What We Can Reuse from AUv2
- NIH-plug integration patterns
- Parameter mapping logic
- Buffer handling concepts
- State serialization approach
- Error handling patterns

### What Must Be Completely Rewritten
- All C AudioComponent API code
- Parameter callback system
- Audio processing callbacks
- Bundle creation process
- Build system integration

### New Requirements for AUv3
- Swift AUAudioUnit subclass
- FFI layer for Rust integration
- Xcode project generation
- App extension bundle creation
- AVAudioUnit framework integration

## Migration Strategy

1. **Study AUv2 patterns** - Understand NIH-plug integration approaches
2. **Create FFI layer** - Bridge between Swift and Rust
3. **Implement Swift wrapper** - AUAudioUnit subclass
4. **Generate app extension** - Bundle creation and Xcode integration
5. **Test in real DAWs** - Verify functionality across platforms

## Conclusion

AUv3 represents a complete architectural overhaul from AUv2. While the underlying NIH-plug integration concepts remain similar, the implementation approach is fundamentally different, requiring a Swift/Objective-C wrapper layer and modern app extension architecture.