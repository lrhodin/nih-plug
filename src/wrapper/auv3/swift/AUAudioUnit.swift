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
@objc public class NIHPlugAUv3: AUAudioUnit {
    
    // MARK: - Properties
    
    /// The Rust plugin handle for FFI operations.
    private var pluginHandle: UnsafeMutableRawPointer?
    
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
            var paramInfo = ParameterInfo()
            paramInfo.id = 0
            paramInfo.name = nil
            paramInfo.unit = nil
            paramInfo.minValue = 0
            paramInfo.maxValue = 1
            paramInfo.defaultValue = 0
            let result = plugin_get_parameter_info(handle, UInt32(i), &paramInfo)
            
            if result == 0 { // Success
                let paramName = paramInfo.name != nil ? String(cString: paramInfo.name!) : "Parameter \(i)"
                let paramUnit = paramInfo.unit != nil ? String(cString: paramInfo.unit!) : nil
                
                let param = AUParameter.createParameter(
                    withIdentifier: "param_\(i)",
                    name: paramName,
                    address: UInt64(i),
                    min: paramInfo.minValue,
                    max: paramInfo.maxValue,
                    unit: .generic,
                    unitName: paramUnit,
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

// These functions are declared in the bridging header and will be linked from the Rust static library
// The bridging header provides the C function declarations that Swift can import