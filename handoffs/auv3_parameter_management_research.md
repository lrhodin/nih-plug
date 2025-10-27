# AUParameter and AUParameterTree for Parameter Management

## Overview

AUv3 uses a sophisticated parameter management system built around `AUParameter` and `AUParameterTree` classes. This system provides automatic parameter automation, value persistence, and host integration for Audio Unit plugins.

## Core Classes

### 1. AUParameter

`AUParameter` represents an individual parameter in the plugin. It provides value storage, validation, and change notification.

#### Key Properties

```swift
class AUParameter: AUParameterNode {
    // Basic Properties
    var address: AUParameterAddress { get }
    var identifier: String { get }
    var displayName: String { get set }
    var unit: AudioUnitParameterUnit { get }
    var unitName: String? { get set }
    
    // Value Properties
    var value: Float { get set }
    var minValue: Float { get }
    var maxValue: Float { get }
    var defaultValue: Float { get }
    
    // Flags
    var flags: AudioUnitParameterOptions { get }
    
    // Value Strings (for discrete parameters)
    var valueStrings: [String]? { get set }
    
    // Dependent Parameters
    var dependentParameters: [NSNumber]? { get set }
}
```

#### Parameter Units

```swift
enum AudioUnitParameterUnit: UInt32 {
    case generic = 0
    case indexed = 1
    case boolean = 2
    case percent = 3
    case seconds = 4
    case sampleFrames = 5
    case phase = 6
    case rate = 7
    case hertz = 8
    case cents = 9
    case relativeSemiTones = 10
    case midiNoteNumber = 11
    case midiController = 12
    case decibels = 13
    case linearGain = 14
    case degrees = 15
    case equalPowerCrossfade = 16
    case mixerFaderCurve1 = 17
    case pan = 18
    case meters = 19
    case absoluteCents = 20
    case octaves = 21
    case BPM = 22
    case beats = 23
    case milliseconds = 24
    case ratio = 25
    case customUnit = 26
}
```

#### Parameter Flags

```swift
struct AudioUnitParameterOptions: OptionSet {
    static let flag_IsReadable = AudioUnitParameterOptions(rawValue: 1 << 0)
    static let flag_IsWritable = AudioUnitParameterOptions(rawValue: 1 << 1)
    static let flag_IsHighResolution = AudioUnitParameterOptions(rawValue: 1 << 2)
    static let flag_IsNonRealTime = AudioUnitParameterOptions(rawValue: 1 << 3)
    static let flag_IsGlobal = AudioUnitParameterOptions(rawValue: 1 << 4)
    static let flag_IsElement = AudioUnitParameterOptions(rawValue: 1 << 5)
    static let flag_IsReadOnly = AudioUnitParameterOptions(rawValue: 1 << 6)
    static let flag_IsHidden = AudioUnitParameterOptions(rawValue: 1 << 7)
    static let flag_IsDisplayLogarithmic = AudioUnitParameterOptions(rawValue: 1 << 8)
    static let flag_IsDisplaySquareRoot = AudioUnitParameterOptions(rawValue: 1 << 9)
    static let flag_IsDisplayExponential = AudioUnitParameterOptions(rawValue: 1 << 10)
    static let flag_IsDisplaySymmetric = AudioUnitParameterOptions(rawValue: 1 << 11)
    static let flag_IsDisplaySigned = AudioUnitParameterOptions(rawValue: 1 << 12)
    static let flag_IsDisplayBoolean = AudioUnitParameterOptions(rawValue: 1 << 13)
    static let flag_IsDisplayInteger = AudioUnitParameterOptions(rawValue: 1 << 14)
    static let flag_IsDisplaySquareRoot = AudioUnitParameterOptions(rawValue: 1 << 15)
    static let flag_IsDisplayCubic = AudioUnitParameterOptions(rawValue: 1 << 16)
    static let flag_IsDisplayCubicRoot = AudioUnitParameterOptions(rawValue: 1 << 17)
    static let flag_IsDisplayExponential = AudioUnitParameterOptions(rawValue: 1 << 18)
    static let flag_IsDisplayExponentialRoot = AudioUnitParameterOptions(rawValue: 1 << 19)
    static let flag_IsDisplayExponentialRoot2 = AudioUnitParameterOptions(rawValue: 1 << 20)
    static let flag_IsDisplayExponentialRoot3 = AudioUnitParameterOptions(rawValue: 1 << 21)
    static let flag_IsDisplayExponentialRoot4 = AudioUnitParameterOptions(rawValue: 1 << 22)
    static let flag_IsDisplayExponentialRoot5 = AudioUnitParameterOptions(rawValue: 1 << 23)
    static let flag_IsDisplayExponentialRoot6 = AudioUnitParameterOptions(rawValue: 1 << 24)
    static let flag_IsDisplayExponentialRoot7 = AudioUnitParameterOptions(rawValue: 1 << 25)
    static let flag_IsDisplayExponentialRoot8 = AudioUnitParameterOptions(rawValue: 1 << 26)
    static let flag_IsDisplayExponentialRoot9 = AudioUnitParameterOptions(rawValue: 1 << 27)
    static let flag_IsDisplayExponentialRoot10 = AudioUnitParameterOptions(rawValue: 1 << 28)
    static let flag_IsDisplayExponentialRoot11 = AudioUnitParameterOptions(rawValue: 1 << 29)
    static let flag_IsDisplayExponentialRoot12 = AudioUnitParameterOptions(rawValue: 1 << 30)
    static let flag_IsDisplayExponentialRoot13 = AudioUnitParameterOptions(rawValue: 1 << 31)
}
```

### 2. AUParameterTree

`AUParameterTree` manages a collection of parameters and provides automation support.

#### Key Properties

```swift
class AUParameterTree: AUParameterNode {
    // Tree Structure
    var children: [AUParameterNode] { get set }
    
    // Parameter Access
    var allParameters: [AUParameter] { get }
    func parameter(withAddress address: AUParameterAddress) -> AUParameter?
    func parameter(withID identifier: String) -> AUParameter?
    
    // Value Management
    var implementorValueObserver: ((AUParameter, Float) -> Void)?
    var implementorValueProvider: ((AUParameter) -> Float)?
    var implementorStringFromValueCallback: ((AUParameter, UnsafeMutablePointer<Float>) -> String?)?
    var implementorValueFromStringCallback: ((AUParameter, String) -> Float)?
    
    // Automation
    var implementorValueObserverWithToken: ((AUParameter, AUParameterObserverToken, Float) -> Void)?
    var implementorValueProviderWithToken: ((AUParameter, AUParameterObserverToken) -> Float)?
    
    // State
    var implementorStateFromValueCallback: ((AUParameter, Float) -> [String: Any]?)?
    var implementorValueFromStateCallback: ((AUParameter, [String: Any]) -> Float)?
}
```

## Integration with NIH-plug

### 1. Parameter Mapping

Based on the existing AU wrapper patterns, we need to map NIH-plug parameters to AUParameter instances:

```swift
class ParameterMapper {
    private var parameterMap: [AUParameterAddress: UInt32] = [:]
    private var rustPlugin: PluginHandle?
    
    func createParameterTree(from rustPlugin: PluginHandle) -> AUParameterTree {
        self.rustPlugin = rustPlugin
        
        let paramCount = plugin_get_parameter_count(rustPlugin)
        var parameters: [AUParameter] = []
        
        for i in 0..<paramCount {
            var paramInfo = ParameterInfo()
            if plugin_get_parameter_info(rustPlugin, i, &paramInfo) == PLUGIN_OK {
                let parameter = AUParameter(
                    identifier: String(cString: paramInfo.name),
                    name: String(cString: paramInfo.name),
                    address: AUParameterAddress(i),
                    min: paramInfo.min_value,
                    max: paramInfo.max_value,
                    unit: mapUnit(paramInfo.unit),
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
        setupParameterObservers(parameterTree)
        
        return parameterTree
    }
    
    private func setupParameterObservers(_ parameterTree: AUParameterTree) {
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
    }
    
    private func mapUnit(_ unit: UInt32) -> AudioUnitParameterUnit {
        switch unit {
        case 0: return .generic
        case 1: return .indexed
        case 2: return .boolean
        case 3: return .percent
        case 4: return .seconds
        case 5: return .sampleFrames
        case 6: return .phase
        case 7: return .rate
        case 8: return .hertz
        case 9: return .cents
        case 10: return .relativeSemiTones
        case 11: return .midiNoteNumber
        case 12: return .midiController
        case 13: return .decibels
        case 14: return .linearGain
        case 15: return .degrees
        case 16: return .equalPowerCrossfade
        case 17: return .mixerFaderCurve1
        case 18: return .pan
        case 19: return .meters
        case 20: return .absoluteCents
        case 21: return .octaves
        case 22: return .BPM
        case 23: return .beats
        case 24: return .milliseconds
        case 25: return .ratio
        case 26: return .customUnit
        default: return .generic
        }
    }
}
```

### 2. Parameter Groups

AUv3 supports parameter grouping through `AUParameterGroup`:

```swift
class ParameterGroupMapper {
    func createParameterGroups(from rustPlugin: PluginHandle) -> [AUParameterGroup] {
        // Get parameter groups from Rust plugin
        let groupCount = plugin_get_parameter_group_count(rustPlugin)
        var groups: [AUParameterGroup] = []
        
        for i in 0..<groupCount {
            var groupInfo = ParameterGroupInfo()
            if plugin_get_parameter_group_info(rustPlugin, i, &groupInfo) == PLUGIN_OK {
                let group = AUParameterGroup(
                    identifier: String(cString: groupInfo.name),
                    name: String(cString: groupInfo.name),
                    children: [] // Will be populated with parameters
                )
                groups.append(group)
            }
        }
        
        return groups
    }
}
```

### 3. Parameter Automation

AUv3 provides built-in parameter automation support:

```swift
class ParameterAutomation {
    private var parameterTree: AUParameterTree?
    private var rustPlugin: PluginHandle?
    
    func setupAutomation(for parameterTree: AUParameterTree, rustPlugin: PluginHandle) {
        self.parameterTree = parameterTree
        self.rustPlugin = rustPlugin
        
        // Set up automation observers
        parameterTree.implementorValueObserverWithToken = { [weak self] parameter, token, value in
            guard let self = self, let plugin = self.rustPlugin else { return }
            if let paramId = self.parameterMap[parameter.address] {
                // Schedule parameter change for sample-accurate automation
                self.scheduleParameterChange(paramId: paramId, value: value, token: token)
            }
        }
    }
    
    private func scheduleParameterChange(paramId: UInt32, value: Float, token: AUParameterObserverToken) {
        // This would integrate with the audio processing pipeline
        // to apply parameter changes at the correct sample offset
    }
}
```

## Parameter Types and Validation

### 1. Value Validation

```swift
extension AUParameter {
    func validateValue(_ value: Float) -> Float {
        return max(minValue, min(maxValue, value))
    }
    
    func setValue(_ value: Float) {
        self.value = validateValue(value)
    }
}
```

### 2. String Conversion

```swift
class ParameterStringConverter {
    func setupStringConversion(for parameterTree: AUParameterTree) {
        parameterTree.implementorStringFromValueCallback = { parameter, value in
            switch parameter.unit {
            case .indexed:
                if let valueStrings = parameter.valueStrings {
                    let index = Int(value.rounded())
                    return valueStrings[safe: index] ?? "\(index)"
                }
                return "\(Int(value.rounded()))"
            case .boolean:
                return value > 0.5 ? "On" : "Off"
            case .percent:
                return String(format: "%.1f%%", value * 100)
            case .decibels:
                return String(format: "%.1f dB", value)
            case .hertz:
                return String(format: "%.1f Hz", value)
            default:
                return String(format: "%.3f", value)
            }
        }
        
        parameterTree.implementorValueFromStringCallback = { parameter, string in
            switch parameter.unit {
            case .indexed:
                if let valueStrings = parameter.valueStrings,
                   let index = valueStrings.firstIndex(of: string) {
                    return Float(index)
                }
                return Float(string) ?? parameter.defaultValue
            case .boolean:
                return string.lowercased() == "on" ? 1.0 : 0.0
            case .percent:
                if let value = Float(string.replacingOccurrences(of: "%", with: "")) {
                    return value / 100.0
                }
                return parameter.defaultValue
            case .decibels:
                if let value = Float(string.replacingOccurrences(of: " dB", with: "")) {
                    return value
                }
                return parameter.defaultValue
            case .hertz:
                if let value = Float(string.replacingOccurrences(of: " Hz", with: "")) {
                    return value
                }
                return parameter.defaultValue
            default:
                return Float(string) ?? parameter.defaultValue
            }
        }
    }
}
```

## State Management

### 1. Parameter State Serialization

```swift
extension AUParameterTree {
    func serializeState() -> [String: Any] {
        var state: [String: Any] = [:]
        
        for parameter in allParameters {
            state[parameter.identifier] = parameter.value
        }
        
        return state
    }
    
    func deserializeState(_ state: [String: Any]) {
        for parameter in allParameters {
            if let value = state[parameter.identifier] as? Float {
                parameter.value = value
            }
        }
    }
}
```

### 2. Integration with fullState

```swift
class StateManager {
    private var parameterTree: AUParameterTree?
    private var rustPlugin: PluginHandle?
    
    func setupStateManagement(for parameterTree: AUParameterTree, rustPlugin: PluginHandle) {
        self.parameterTree = parameterTree
        self.rustPlugin = rustPlugin
    }
    
    func saveState() -> [String: Any]? {
        guard let plugin = rustPlugin else { return nil }
        
        // Get state from Rust plugin
        let maxSize: UInt32 = 1024 * 1024
        let buffer = UnsafeMutablePointer<CChar>.allocate(capacity: Int(maxSize))
        defer { buffer.deallocate() }
        
        var actualSize: UInt32 = 0
        if pluginSaveState(plugin, buffer, maxSize, &actualSize) == PLUGIN_OK {
            let data = Data(bytes: buffer, count: Int(actualSize))
            return ["state": data]
        }
        
        return nil
    }
    
    func loadState(_ state: [String: Any]) {
        guard let plugin = rustPlugin, let stateData = state["state"] as? Data else { return }
        
        stateData.withUnsafeBytes { bytes in
            _ = pluginLoadState(plugin, bytes.bindMemory(to: CChar.self).baseAddress!, UInt32(stateData.count))
        }
    }
}
```

## Best Practices

### 1. Parameter Design
- Use meaningful identifiers and display names
- Choose appropriate units for each parameter
- Set reasonable min/max values
- Use value strings for discrete parameters

### 2. Performance
- Minimize parameter tree updates
- Use batch operations when possible
- Cache parameter values when appropriate
- Avoid expensive operations in parameter observers

### 3. Error Handling
- Validate parameter values before setting
- Handle missing parameters gracefully
- Provide fallback values for invalid inputs
- Log parameter errors for debugging

### 4. Testing
- Test parameter value ranges
- Verify string conversion accuracy
- Test parameter automation
- Validate state serialization/deserialization

## Next Steps

1. Research internalRenderBlock audio processing
2. Create comprehensive architecture summary
3. Implement parameter management in FFI layer
4. Create Swift parameter wrapper
5. Integrate with existing NIH-plug parameter system