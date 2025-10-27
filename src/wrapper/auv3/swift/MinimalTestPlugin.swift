//
//  MinimalTestPlugin.swift
//  NIH-Plug AUv3 Minimal Test
//
//  Created by Implementation Ralph on $(DATE).
//  Copyright © $(YEAR) NIH-Plug. All rights reserved.
//

import AVFoundation
import AudioToolbox

/// Minimal AUv3 test plugin to isolate recognition issues.
/// This plugin implements only the absolute basics required for AUv3 recognition.
@objc public class MinimalTestPlugin: AUAudioUnit {
    
    // MARK: - Audio Unit Factory
    
    @objc public static func createAudioUnit(componentDescription: AudioComponentDescription) -> AUAudioUnit? {
        print("MinimalTestPlugin: createAudioUnit called")
        do {
            let plugin = try MinimalTestPlugin(componentDescription: componentDescription)
            print("MinimalTestPlugin: Successfully created plugin instance")
            return plugin
        } catch {
            print("MinimalTestPlugin: Failed to create plugin instance: \(error)")
            return nil
        }
    }
    
    // MARK: - Properties
    
    /// The parameter tree for the plugin.
    private var _parameterTree: AUParameterTree?
    
    /// The input busses for the plugin.
    private var _inputBusses: AUAudioUnitBusArray?
    
    /// The output busses for the plugin.
    private var _outputBusses: AUAudioUnitBusArray?
    
    // MARK: - Initialization
    
    public override init(componentDescription: AudioComponentDescription, options: AudioComponentInstantiationOptions = []) throws {
        print("MinimalTestPlugin: Starting initialization...")
        print("MinimalTestPlugin: Component description - type: \(componentDescription.componentType), subtype: \(componentDescription.componentSubType), manufacturer: \(componentDescription.componentManufacturer)")
        
        try super.init(componentDescription: componentDescription, options: options)
        
        print("MinimalTestPlugin: Super init completed")
        
        // Set up audio unit properties
        setupAudioUnit()
        
        print("MinimalTestPlugin: Initialization completed successfully")
    }
    
    deinit {
        print("MinimalTestPlugin: Deinitializing...")
    }
    
    // MARK: - Audio Unit Setup
    
    private func setupAudioUnit() {
        print("MinimalTestPlugin: Setting up audio unit...")
        
        // Set up input and output busses
        // For now, we'll use stereo input/output
        let inputBus = try! AUAudioUnitBus(format: AVAudioFormat(standardFormatWithSampleRate: 44100, channels: 2)!)
        let outputBus = try! AUAudioUnitBus(format: AVAudioFormat(standardFormatWithSampleRate: 44100, channels: 2)!)
        
        _inputBusses = AUAudioUnitBusArray(audioUnit: self, busType: .input, busses: [inputBus])
        _outputBusses = AUAudioUnitBusArray(audioUnit: self, busType: .output, busses: [outputBus])
        
        print("MinimalTestPlugin: Input busses: \(_inputBusses?.count ?? 0), Output busses: \(_outputBusses?.count ?? 0)")
        
        // Set up parameter tree
        setupParameterTree()
        
        print("MinimalTestPlugin: Audio unit setup completed")
    }
    
    private func setupParameterTree() {
        print("MinimalTestPlugin: Setting up parameter tree...")
        // Create an empty parameter tree
        _parameterTree = AUParameterTree.createTree(withChildren: [])
        print("MinimalTestPlugin: Parameter tree created")
    }
    
    // MARK: - AVAudioUnit Overrides
    
    public override var parameterTree: AUParameterTree? {
        get { 
            print("MinimalTestPlugin: parameterTree getter called")
            return _parameterTree 
        }
        set { 
            print("MinimalTestPlugin: parameterTree setter called")
            _parameterTree = newValue 
        }
    }
    
    public override var inputBusses: AUAudioUnitBusArray {
        print("MinimalTestPlugin: inputBusses getter called")
        return _inputBusses ?? AUAudioUnitBusArray(audioUnit: self, busType: .input, busses: [])
    }
    
    public override var outputBusses: AUAudioUnitBusArray {
        print("MinimalTestPlugin: outputBusses getter called")
        return _outputBusses ?? AUAudioUnitBusArray(audioUnit: self, busType: .output, busses: [])
    }
    
    // MARK: - Required Audio Unit Properties
    
    /// Indicates whether the audio unit can process audio in-place.
    public override var canProcessInPlace: Bool {
        print("MinimalTestPlugin: canProcessInPlace getter called")
        return true
    }
    
    /// Indicates whether the audio unit should allocate input bus.
    public override var shouldAllocateInputBus: Bool {
        print("MinimalTestPlugin: shouldAllocateInputBus getter called")
        return true
    }
    
    /// Maximum number of frames the audio unit can render in a single call.
    public override var maximumFramesToRender: AUAudioFrameCount {
        print("MinimalTestPlugin: maximumFramesToRender getter called")
        return 512
    }
    
    // MARK: - Required Audio Unit Interface Methods
    
    public override func allocateRenderResources() throws {
        print("MinimalTestPlugin: allocateRenderResources called")
        try super.allocateRenderResources()
        print("MinimalTestPlugin: Render resources allocated successfully")
    }
    
    public override func deallocateRenderResources() {
        print("MinimalTestPlugin: deallocateRenderResources called")
        super.deallocateRenderResources()
        print("MinimalTestPlugin: Render resources deallocated")
    }
    
    public override func reset() {
        print("MinimalTestPlugin: reset called")
        super.reset()
        print("MinimalTestPlugin: Audio unit reset")
    }
    
    // MARK: - Audio Processing
    
    public override var internalRenderBlock: AUInternalRenderBlock {
        print("MinimalTestPlugin: internalRenderBlock getter called")
        return { (actionFlags, timestamp, frameCount, outputBusNumber, outputData, renderEvent, pullInputBlock) in
            print("MinimalTestPlugin: Audio processing block called - frameCount: \(frameCount)")
            
            // For now, just pass through the audio (or silence if no input)
            if let pullInputBlock = pullInputBlock {
                // Pass through input to output
                return pullInputBlock(actionFlags, timestamp, frameCount, 0, outputData)
            } else {
                // Generate silence
                let outputBufferList = outputData
                let outputBuffers = UnsafeMutableAudioBufferListPointer(outputBufferList)
                
                for i in 0..<outputBuffers.count {
                    let buffer = outputBuffers[i]
                    if let data = buffer.mData {
                        let samples = data.assumingMemoryBound(to: Float.self)
                        for j in 0..<Int(frameCount) {
                            samples[j] = 0.0
                        }
                    }
                }
                
                return noErr
            }
        }
    }
}

// MARK: - Audio Unit Registration Entry Point

/// Main entry point for Audio Unit registration.
@objc public func registerMinimalTestPlugin() {
    print("MinimalTestPlugin: registerMinimalTestPlugin called")
    // AUv3 registration is handled automatically by the NSExtension framework
    // through the Info.plist configuration. No manual registration needed.
    print("MinimalTestPlugin: Audio Unit registration handled by NSExtension framework")
}

/// Alternative registration method that might be needed for some systems
@objc public static func registerAudioUnit() {
    print("MinimalTestPlugin: Static registration method called")
    // This method might be called by the system for registration
}