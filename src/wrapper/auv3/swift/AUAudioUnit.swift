//
//  AUAudioUnit.swift
//  NIH-Plug AUv3
//
//  Created by NIH-Plug on $(DATE).
//  Copyright © $(YEAR) NIH-Plug. All rights reserved.
//

import AVFoundation
import AudioToolbox

/// Main AUAudioUnit subclass for NIH-Plug AUv3 integration.
///
/// This class provides the bridge between the AUv3 host and the Rust FFI layer.
/// It handles parameter management, audio processing, and state management.
@objc public class AUAudioUnit: AVAudioUnit {
    
    // MARK: - Properties
    
    /// The Rust plugin handle for FFI operations.
    private var pluginHandle: OpaquePointer?
    
    /// The parameter tree for the plugin.
    private var _parameterTree: AUParameterTree?
    
    /// The input busses for the plugin.
    private var _inputBusses: AUAudioUnitBusArray?
    
    /// The output busses for the plugin.
    private var _outputBusses: AUAudioUnitBusArray?
    
    /// Whether the plugin is initialized.
    private var isInitialized = false
    
    // MARK: - Initialization
    
    public override init(audioComponentDescription: AudioComponentDescription) throws {
        try super.init(audioComponentDescription: audioComponentDescription)
        
        // Initialize the plugin handle
        pluginHandle = plugin_create()
        if pluginHandle == nil {
            throw NSError(domain: "NIHPlugAUv3", code: -1, userInfo: [NSLocalizedDescriptionKey: "Failed to create plugin instance"])
        }
        
        // Set up audio unit properties
        setupAudioUnit()
    }
    
    deinit {
        // Clean up the plugin handle
        if let handle = pluginHandle {
            plugin_destroy(handle)
        }
    }
    
    // MARK: - Audio Unit Setup
    
    private func setupAudioUnit() {
        // Set up input and output busses
        // For now, we'll use stereo input/output
        let inputBus = try! AUAudioUnitBus(format: AVAudioFormat(standardFormatWithSampleRate: 44100, channels: 2)!)
        let outputBus = try! AUAudioUnitBus(format: AVAudioFormat(standardFormatWithSampleRate: 44100, channels: 2)!)
        
        _inputBusses = AUAudioUnitBusArray(audioUnit: self, busType: .input, busses: [inputBus])
        _outputBusses = AUAudioUnitBusArray(audioUnit: self, busType: .output, busses: [outputBus])
        
        // Set up parameter tree
        setupParameterTree()
    }
    
    private func setupParameterTree() {
        guard let handle = pluginHandle else { return }
        
        // Query the Rust plugin for parameter count
        let paramCount = plugin_get_parameter_count(handle)
        
        if paramCount == 0 {
            // No parameters, create empty tree
            _parameterTree = AUParameterTree.createTree(withChildren: [])
            return
        }
        
        // Create parameters for each plugin parameter
        var parameters: [AUParameter] = []
        
        for i in 0..<paramCount {
            var paramInfo = ParameterInfo(id: 0, name: nil, unit: nil, minValue: 0, maxValue: 1, defaultValue: 0)
            let result = plugin_get_parameter_info(handle, UInt32(i), &paramInfo)
            
            if result == 0 { // Success
                let param = AUParameter.createParameter(
                    withIdentifier: "param_\(i)",
                    name: String(cString: paramInfo.name),
                    address: UInt64(i),
                    min: paramInfo.minValue,
                    max: paramInfo.maxValue,
                    unit: .generic,
                    unitName: paramInfo.unit != nil ? String(cString: paramInfo.unit) : nil,
                    flags: [],
                    valueStrings: nil,
                    dependentParameters: nil
                )
                
                // Set up parameter value change handling
                param.value = paramInfo.defaultValue
                parameters.append(param)
            }
        }
        
        // Create the parameter tree with all parameters
        _parameterTree = AUParameterTree.createTree(withChildren: parameters)
        
        // Set up parameter value change observers
        setupParameterObservers()
    }
    
    private func setupParameterObservers() {
        guard let parameterTree = _parameterTree else { return }
        
        // Set up parameter value change handling
        parameterTree.implementorValueObserver = { [weak self] parameter, value in
            self?.updateParameter(parameter, value: value)
        }
        
        parameterTree.implementorValueProvider = { [weak self] parameter in
            return self?.getParameterValue(parameter) ?? 0.0
        }
    }
    
    // MARK: - AVAudioUnit Overrides
    
    public override var parameterTree: AUParameterTree? {
        return _parameterTree
    }
    
    public override var inputBusses: AUAudioUnitBusArray? {
        return _inputBusses
    }
    
    public override var outputBusses: AUAudioUnitBusArray? {
        return _outputBusses
    }
    
    // MARK: - Audio Processing
    
    public override var internalRenderBlock: AUInternalRenderBlock? {
        return { [weak self] (actionFlags, timestamp, frameCount, outputBusNumber, outputData, renderEvent, pullInputBlock) in
            guard let self = self else { return noErr }
            
            // Ensure we have a valid plugin handle
            guard let handle = self.pluginHandle else {
                return noErr
            }
            
            // Get the output buffer list
            guard let outputBufferList = outputData else {
                return noErr
            }
            
            let outputBuffers = UnsafeMutableAudioBufferListPointer(outputBufferList)
            let numChannels = Int(outputBuffers.count)
            let numFrames = Int(frameCount)
            
            if numChannels == 0 || numFrames == 0 {
                return noErr
            }
            
            // Prepare input and output buffer arrays for FFI
            var inputChannelPointers: [UnsafePointer<Float>?] = []
            var outputChannelPointers: [UnsafeMutablePointer<Float>?] = []
            
            // Handle input if we have a pull input block
            if let pullInputBlock = pullInputBlock {
                // Create a temporary input buffer
                let inputBufferList = AudioBufferList.allocate(maximumBuffers: numChannels)
                defer { inputBufferList.deallocate() }
                
                // Pull input audio
                let inputStatus = pullInputBlock(actionFlags, timestamp, frameCount, 0, inputBufferList.unsafeMutablePointer)
                if inputStatus != noErr {
                    return inputStatus
                }
                
                // Convert input buffers to pointers
                let inputBuffers = UnsafeMutableAudioBufferListPointer(inputBufferList.unsafeMutablePointer)
                for i in 0..<numChannels {
                    if i < inputBuffers.count {
                        let buffer = inputBuffers[i]
                        if let data = buffer.mData {
                            inputChannelPointers.append(data.assumingMemoryBound(to: Float.self))
                        } else {
                            inputChannelPointers.append(nil)
                        }
                    } else {
                        inputChannelPointers.append(nil)
                    }
                }
            }
            
            // Convert output buffers to pointers
            for i in 0..<numChannels {
                if i < outputBuffers.count {
                    let buffer = outputBuffers[i]
                    if let data = buffer.mData {
                        outputChannelPointers.append(data.assumingMemoryBound(to: Float.self))
                    } else {
                        outputChannelPointers.append(nil)
                    }
                } else {
                    outputChannelPointers.append(nil)
                }
            }
            
            // Call the Rust audio processing function
            let result = inputChannelPointers.withUnsafeBufferPointer { inputPtr in
                outputChannelPointers.withUnsafeMutableBufferPointer { outputPtr in
                    plugin_process(
                        handle,
                        inputChannelPointers.isEmpty ? nil : inputPtr.baseAddress,
                        outputPtr.baseAddress,
                        UInt32(numChannels),
                        UInt32(numFrames)
                    )
                }
            }
            
            if result != 0 {
                print("Audio processing failed with error: \(result)")
                return noErr
            }
            
            return noErr
        }
    }
    
    // MARK: - State Management
    
    public override var fullState: [String : Any]? {
        get {
            guard let handle = pluginHandle else { return nil }
            
            var data: UnsafeMutablePointer<UInt8>?
            var size: UInt32 = 0
            
            let result = plugin_save_state(handle, &data, &size)
            if result == 0 && data != nil {
                let stateData = Data(bytes: data!, count: Int(size))
                plugin_free(data!)
                return ["state": stateData]
            }
            
            return nil
        }
        set {
            guard let handle = pluginHandle,
                  let state = newValue?["state"] as? Data else { return }
            
            state.withUnsafeBytes { bytes in
                plugin_load_state(handle, bytes.bindMemory(to: UInt8.self).baseAddress!, UInt32(state.count))
            }
        }
    }
    
    // MARK: - Parameter Management
    
    private func updateParameter(_ parameter: AUParameter, value: Float) {
        guard let handle = pluginHandle else { return }
        
        let paramId = UInt32(parameter.address)
        let result = plugin_set_parameter(handle, paramId, value)
        
        if result != 0 {
            print("Failed to set parameter \(paramId) to value \(value), error: \(result)")
        }
    }
    
    private func getParameterValue(_ parameter: AUParameter) -> Float {
        guard let handle = pluginHandle else { return 0.0 }
        
        let paramId = UInt32(parameter.address)
        var value: Float = 0.0
        let result = plugin_get_parameter(handle, paramId, &value)
        
        if result != 0 {
            print("Failed to get parameter \(paramId), error: \(result)")
            return 0.0
        }
        
        return value
    }
}

// MARK: - FFI Function Declarations

// These functions are declared here and will be linked from the Rust static library
@_silgen_name("plugin_create")
func plugin_create() -> OpaquePointer?

@_silgen_name("plugin_destroy")
func plugin_destroy(_ handle: OpaquePointer?) -> Int32

@_silgen_name("plugin_initialize")
func plugin_initialize(_ handle: OpaquePointer?, _ sampleRate: Float, _ maxBlockSize: UInt32, _ inputChannels: UInt32, _ outputChannels: UInt32) -> Int32

@_silgen_name("plugin_process")
func plugin_process(_ handle: OpaquePointer?, _ inputBuffers: UnsafePointer<UnsafePointer<Float>?>, _ outputBuffers: UnsafeMutablePointer<UnsafeMutablePointer<Float>?>, _ numChannels: UInt32, _ numFrames: UInt32) -> Int32

@_silgen_name("plugin_get_parameter_count")
func plugin_get_parameter_count(_ handle: OpaquePointer?) -> UInt32

@_silgen_name("plugin_get_parameter_info")
func plugin_get_parameter_info(_ handle: OpaquePointer?, _ paramId: UInt32, _ info: UnsafeMutablePointer<ParameterInfo>) -> Int32

@_silgen_name("plugin_set_parameter")
func plugin_set_parameter(_ handle: OpaquePointer?, _ paramId: UInt32, _ normalizedValue: Float) -> Int32

@_silgen_name("plugin_get_parameter")
func plugin_get_parameter(_ handle: OpaquePointer?, _ paramId: UInt32, _ value: UnsafeMutablePointer<Float>) -> Int32

@_silgen_name("plugin_save_state")
func plugin_save_state(_ handle: OpaquePointer?, _ data: UnsafeMutablePointer<UnsafeMutablePointer<UInt8>?>, _ size: UnsafeMutablePointer<UInt32>) -> Int32

@_silgen_name("plugin_load_state")
func plugin_load_state(_ handle: OpaquePointer?, _ data: UnsafePointer<UInt8>, _ size: UInt32) -> Int32

@_silgen_name("plugin_free")
func plugin_free(_ ptr: UnsafeMutableRawPointer?)

// MARK: - C Structure Definitions

/// Parameter information structure for FFI.
struct ParameterInfo {
    let id: UInt32
    let name: UnsafePointer<CChar>
    let unit: UnsafePointer<CChar>
    let minValue: Float
    let maxValue: Float
    let defaultValue: Float
}