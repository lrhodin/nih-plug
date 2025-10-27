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
    
    // MARK: - Audio Unit Registration
    
    // Note: AUv3 registration is handled through Info.plist configuration
    // The NSExtensionPrincipalClass points to this class, and the AudioComponents
    // section in Info.plist defines the Audio Unit metadata
    
    // MARK: - Audio Unit Factory
    
    @objc public static func createAudioUnit(componentDescription: AudioComponentDescription) -> AUAudioUnit? {
        do {
            return try NIHPlugAUv3(componentDescription: componentDescription)
        } catch {
            print("Failed to create NIHPlugAUv3: \(error)")
            return nil
        }
    }
    
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
    
    public override init(componentDescription: AudioComponentDescription, options: AudioComponentInstantiationOptions = []) throws {
        print("NIHPlugAUv3: Starting initialization...")
        print("NIHPlugAUv3: Component description - type: \(componentDescription.componentType), subtype: \(componentDescription.componentSubType), manufacturer: \(componentDescription.componentManufacturer)")
        
        try super.init(componentDescription: componentDescription, options: options)
        
        // Initialize the plugin handle
        print("NIHPlugAUv3: Creating plugin instance...")
        pluginHandle = plugin_create()
        if pluginHandle == nil {
            print("NIHPlugAUv3: Failed to create plugin instance")
            throw NSError(domain: "NIHPlugAUv3", code: -1, userInfo: [NSLocalizedDescriptionKey: "Failed to create plugin instance"])
        }
        print("NIHPlugAUv3: Plugin instance created successfully")
        
        // Set up audio unit properties
        setupAudioUnit()
        
        print("NIHPlugAUv3: Initialization completed successfully")
    }
    
    deinit {
        // Clean up the plugin handle
        if let handle = pluginHandle {
            plugin_destroy(handle)
        }
    }
    
    // MARK: - Audio Unit Setup
    
    private func setupAudioUnit() {
        print("NIHPlugAUv3: Setting up audio unit...")
        
        // Set up input and output busses
        // For now, we'll use stereo input/output
        let inputBus = try! AUAudioUnitBus(format: AVAudioFormat(standardFormatWithSampleRate: 44100, channels: 2)!)
        let outputBus = try! AUAudioUnitBus(format: AVAudioFormat(standardFormatWithSampleRate: 44100, channels: 2)!)
        
        _inputBusses = AUAudioUnitBusArray(audioUnit: self, busType: .input, busses: [inputBus])
        _outputBusses = AUAudioUnitBusArray(audioUnit: self, busType: .output, busses: [outputBus])
        
        print("NIHPlugAUv3: Input busses: \(_inputBusses?.count ?? 0), Output busses: \(_outputBusses?.count ?? 0)")
        
        // Set up parameter tree
        setupParameterTree()
        
        print("NIHPlugAUv3: Audio unit setup completed")
    }
    
    private func setupParameterTree() {
        // For now, create an empty parameter tree
        // This will be implemented properly once we have the correct AUParameter API
        _parameterTree = AUParameterTree.createTree(withChildren: [])
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
        get { return _parameterTree }
        set { _parameterTree = newValue }
    }
    
    public override var inputBusses: AUAudioUnitBusArray {
        return _inputBusses ?? AUAudioUnitBusArray(audioUnit: self, busType: .input, busses: [])
    }
    
    public override var outputBusses: AUAudioUnitBusArray {
        return _outputBusses ?? AUAudioUnitBusArray(audioUnit: self, busType: .output, busses: [])
    }
    
    // MARK: - Required Audio Unit Properties
    
    /// Indicates whether the audio unit can process audio in-place.
    /// For most effect plugins, this should be true.
    public override var canProcessInPlace: Bool {
        return true
    }
    
    /// Indicates whether the audio unit should allocate input bus.
    /// For effect plugins, this should be true.
    public override var shouldAllocateInputBus: Bool {
        return true
    }
    
    /// Maximum number of frames the audio unit can render in a single call.
    /// This is important for buffer management and performance.
    public override var maximumFramesToRender: AUAudioFrameCount {
        return 512 // Use a reasonable default for most plugins
    }
    
    // MARK: - Required Audio Unit Interface Methods
    
    public override func allocateRenderResources() throws {
        try super.allocateRenderResources()
        
        guard let handle = pluginHandle else {
            throw NSError(domain: "NIHPlugAUv3", code: -1, userInfo: [NSLocalizedDescriptionKey: "Plugin handle is null"])
        }
        
        // Initialize the plugin with the current audio format
        let sampleRate = Float(outputBusses[0].format.sampleRate)
        let maxBlockSize = UInt32(512) // Use a reasonable default
        let inputChannels = UInt32(inputBusses.count)
        let outputChannels = UInt32(outputBusses.count)
        
        let result = plugin_initialize(handle, sampleRate, maxBlockSize, inputChannels, outputChannels)
        if result != 0 {
            throw NSError(domain: "NIHPlugAUv3", code: Int(result), userInfo: [NSLocalizedDescriptionKey: "Plugin initialization failed"])
        }
        
        isInitialized = true
        print("NIHPlugAUv3: Render resources allocated successfully")
    }
    
    public override func deallocateRenderResources() {
        super.deallocateRenderResources()
        isInitialized = false
        print("NIHPlugAUv3: Render resources deallocated")
    }
    
    public override func reset() {
        super.reset()
        print("NIHPlugAUv3: Audio unit reset")
    }
    
    // MARK: - Audio Processing
    
    public override var internalRenderBlock: AUInternalRenderBlock {
        return { [weak self] (actionFlags, timestamp, frameCount, outputBusNumber, outputData, renderEvent, pullInputBlock) in
            guard let self = self else { return noErr }
            
            // Ensure we have a valid plugin handle and are initialized
            guard let handle = self.pluginHandle, self.isInitialized else {
                return noErr
            }
            
            // Get the output buffer list
            let outputBufferList = outputData
            
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
                defer { free(inputBufferList.unsafeMutablePointer) }
                
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
            
            let _ = state.withUnsafeBytes { bytes in
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

// MARK: - Audio Unit Registration Entry Point

/// Main entry point for Audio Unit registration.
/// This function is called when the extension is loaded.
/// For AUv3, registration is handled automatically by the NSExtension framework.
@objc public func registerNIHPlugAudioUnit() {
    // AUv3 registration is handled automatically by the NSExtension framework
    // through the Info.plist configuration. No manual registration needed.
    print("NIHPlugAUv3: Audio Unit registration handled by NSExtension framework")
}

/// Alternative registration method that might be needed for some systems
@objc public static func registerAudioUnit() {
    print("NIHPlugAUv3: Static registration method called")
    // This method might be called by the system for registration
}

// MARK: - FFI Function Declarations

// These functions are declared in the bridging header and will be linked from the Rust static library
// The bridging header provides the C function declarations that Swift can import