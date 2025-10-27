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
        // For now, create an empty parameter tree
        // In a full implementation, this would query the Rust plugin for parameters
        _parameterTree = AUParameterTree.createParameter(withIdentifier: "master", name: "Master", address: 0, min: 0, max: 1, unit: .generic, unitName: nil, flags: [], valueStrings: nil, dependentParameters: nil)
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
            
            // For now, just pass through the audio
            // In a full implementation, this would call the Rust plugin's process function
            if let pullInputBlock = pullInputBlock {
                let status = pullInputBlock(actionFlags, timestamp, frameCount, 0, outputData)
                if status != noErr {
                    return status
                }
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
        plugin_set_parameter(handle, paramId, value)
    }
    
    private func getParameterValue(_ parameter: AUParameter) -> Float {
        guard let handle = pluginHandle else { return 0.0 }
        
        let paramId = UInt32(parameter.address)
        var value: Float = 0.0
        plugin_get_parameter(handle, paramId, &value)
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