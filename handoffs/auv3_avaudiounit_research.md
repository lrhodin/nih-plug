# AVAudioUnit Class Hierarchy and Core Methods

## Overview

AVAudioUnit is the base class for Audio Unit v3 plugins in the AVAudioUnit framework. It provides the foundation for creating audio processing plugins that integrate with macOS and iOS audio systems.

## Class Hierarchy

```
NSObject
  └── AVAudioNode
      └── AVAudioUnit
          ├── AUAudioUnit (Abstract base class)
          │   ├── AUAudioUnitV2Bridge (Legacy support)
          │   └── Custom AUAudioUnit subclasses (Our implementation)
          └── AVAudioUnitEffect (Built-in effects)
```

## Key Classes for AUv3 Implementation

### 1. AUAudioUnit (Primary Class)

**Purpose**: The main class that must be subclassed for custom AUv3 plugins.

**Key Properties**:
- `inputBusses: [AUAudioUnitBus]` - Input audio busses
- `outputBusses: [AUAudioUnitBus]` - Output audio busses
- `parameterTree: AUParameterTree?` - Parameter management
- `fullState: [String: Any]?` - State serialization
- `isRenderedOffline: Bool` - Offline rendering flag
- `latency: TimeInterval` - Plugin latency
- `tailTime: TimeInterval` - Plugin tail time

**Key Methods**:
- `init(componentDescription: AudioComponentDescription)` - Initialization
- `allocateRenderResources() throws` - Allocate audio resources
- `deallocateRenderResources()` - Deallocate audio resources
- `internalRenderBlock: AUInternalRenderBlock` - Audio processing block

### 2. AUAudioUnitBus

**Purpose**: Represents audio input/output channels.

**Key Properties**:
- `format: AVAudioFormat` - Audio format (sample rate, channels)
- `name: String` - Bus name
- `enabled: Bool` - Whether bus is active

### 3. AUParameterTree

**Purpose**: Manages plugin parameters and automation.

**Key Methods**:
- `parameter(withAddress: AUParameterAddress) -> AUParameter?` - Get parameter
- `parameter(withID: AUParameterNodeID) -> AUParameter?` - Get parameter by ID
- `allParameters: [AUParameter]` - All parameters
- `value(forParameter: AUParameter) -> Float` - Get parameter value
- `setValue(_:forParameter:originator:)` - Set parameter value

### 4. AUParameter

**Purpose**: Individual parameter representation.

**Key Properties**:
- `address: AUParameterAddress` - Unique parameter address
- `identifier: String` - Parameter identifier
- `displayName: String` - Display name
- `unit: AudioUnitParameterUnit` - Parameter unit
- `minValue: Float` - Minimum value
- `maxValue: Float` - Maximum value
- `defaultValue: Float` - Default value
- `value: Float` - Current value

## Core Integration Methods

### Initialization Flow

```swift
class MyAUAudioUnit: AUAudioUnit {
    override init(componentDescription: AudioComponentDescription) throws {
        // 1. Initialize base class
        try super.init(componentDescription: componentDescription)
        
        // 2. Set up audio busses
        setupAudioBusses()
        
        // 3. Set up parameters
        setupParameters()
        
        // 4. Initialize Rust plugin via FFI
        initializeRustPlugin()
    }
}
```

### Audio Processing Flow

```swift
override var internalRenderBlock: AUInternalRenderBlock {
    return { [weak self] (actionFlags, timestamp, frameCount, outputBusNumber, outputData, realtimeEventListHead, pullInputBlock) in
        // 1. Get input audio from pullInputBlock
        // 2. Convert to Rust-compatible format
        // 3. Call Rust audio processing via FFI
        // 4. Convert output back to AU format
        // 5. Return processed audio
    }
}
```

### Parameter Management Flow

```swift
private func setupParameters() {
    // 1. Create parameter tree
    let parameterTree = AUParameterTree.createParameterTree(withChildren: parameters)
    
    // 2. Set up parameter observers
    parameterTree.implementorValueObserver = { [weak self] parameter, value in
        // Call Rust parameter setter via FFI
        self?.setRustParameter(parameter: parameter, value: value)
    }
    
    parameterTree.implementorValueProvider = { [weak self] parameter in
        // Call Rust parameter getter via FFI
        return self?.getRustParameter(parameter: parameter) ?? 0.0
    }
    
    self.parameterTree = parameterTree
}
```

## Integration with NIH-plug

### FFI Interface Requirements

Based on the VST3 and CLAP wrapper patterns, we need:

1. **Plugin Initialization**:
   - `plugin_init() -> *mut c_void` - Initialize plugin instance
   - `plugin_deinit(instance: *mut c_void)` - Cleanup plugin instance

2. **Parameter Management**:
   - `plugin_get_parameter_count(instance: *mut c_void) -> u32`
   - `plugin_get_parameter_info(instance: *mut c_void, index: u32) -> ParameterInfo`
   - `plugin_set_parameter(instance: *mut c_void, id: u32, value: f32)`
   - `plugin_get_parameter(instance: *mut c_void, id: u32) -> f32`

3. **Audio Processing**:
   - `plugin_process(instance: *mut c_void, input: *const f32, output: *mut f32, frames: u32) -> i32`

4. **State Management**:
   - `plugin_save_state(instance: *mut c_void, data: *mut u8, size: *mut usize) -> i32`
   - `plugin_load_state(instance: *mut c_void, data: *const u8, size: usize) -> i32`

### Swift Wrapper Structure

```swift
class NIHPlugAUAudioUnit: AUAudioUnit {
    private var rustPlugin: UnsafeMutableRawPointer?
    private var parameterMap: [AUParameterAddress: UInt32] = [:]
    
    // FFI function declarations
    private let pluginInit: @convention(c) () -> UnsafeMutableRawPointer?
    private let pluginDeinit: @convention(c) (UnsafeMutableRawPointer?) -> Void
    private let pluginProcess: @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<Float>, UnsafeMutablePointer<Float>, UInt32) -> Int32
    // ... more FFI functions
}
```

## Key Differences from VST3/CLAP

### 1. Object-Oriented vs Functional
- **VST3/CLAP**: C-style function callbacks
- **AUv3**: Swift/Objective-C class methods

### 2. Parameter Management
- **VST3/CLAP**: Manual parameter mapping and callbacks
- **AUv3**: Built-in AUParameterTree with automatic management

### 3. Audio Processing
- **VST3/CLAP**: Direct buffer manipulation
- **AUv3**: AVAudioPCMBuffer objects with internalRenderBlock

### 4. State Management
- **VST3/CLAP**: Custom serialization
- **AUv3**: Built-in fullState property with automatic serialization

## Implementation Strategy

1. **Create FFI Layer**: C-compatible functions for Rust integration
2. **Implement Swift Wrapper**: AUAudioUnit subclass with FFI calls
3. **Parameter Mapping**: Convert NIH-plug parameters to AUParameterTree
4. **Audio Processing**: Bridge between AVAudioPCMBuffer and Rust buffers
5. **State Management**: Serialize/deserialize through FFI layer

## Next Steps

1. Research AUv3 app extension bundle structure
2. Study Swift-to-Rust FFI patterns
3. Research AUParameter and AUParameterTree implementation
4. Research internalRenderBlock audio processing
5. Create comprehensive architecture summary