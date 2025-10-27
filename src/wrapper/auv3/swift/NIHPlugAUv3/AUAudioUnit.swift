//
//  AUAudioUnit.swift
//  NIH-Plug AUv3 Minimal Test
//
//  Created by Implementation Ralph on $(DATE).
//  Copyright © $(YEAR) NIH-Plug. All rights reserved.
//

import AVFoundation
import AudioToolbox

/// Minimal AUv3 test plugin to isolate recognition issues.
/// This plugin implements only the absolute basics required for AUv3 recognition.
@objc public class NIHPlugAUv3: AUAudioUnit {
    
    // MARK: - Audio Unit Factory
    
    @objc public static func createAudioUnit(componentDescription: AudioComponentDescription) -> AUAudioUnit? {
        print("NIHPlugAUv3: createAudioUnit called")
        do {
            let plugin = try NIHPlugAUv3(componentDescription: componentDescription)
            print("NIHPlugAUv3: Successfully created plugin instance")
            return plugin
        } catch {
            print("NIHPlugAUv3: Failed to create plugin instance: \(error)")
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
        print("NIHPlugAUv3: Starting initialization...")
        print("NIHPlugAUv3: Component description - type: \(componentDescription.componentType), subtype: \(componentDescription.componentSubType), manufacturer: \(componentDescription.componentManufacturer)")
        
        try super.init(componentDescription: componentDescription, options: options)
        
        print("NIHPlugAUv3: Super init completed")
        
        // Set up audio unit properties
        setupAudioUnit()
        
        print("NIHPlugAUv3: Initialization completed successfully")
    }
    
    deinit {
        print("NIHPlugAUv3: Deinitializing...")
    }
    
    // MARK: - Audio Unit Setup
    
    private func setupAudioUnit() {
        print("NIHPlugAUv3: Setting up audio unit...")

        // DISABLED FFI TEST CODE - Testing if this prevents initialization
        // let testHandle = plugin_create()
        // if testHandle != nil {
        //     print("NIHPlugAUv3: FFI plugin_create() succeeded - handle: \(testHandle!)")
        //     let paramCount = plugin_get_parameter_count(testHandle!)
        //     print("NIHPlugAUv3: Plugin has \(paramCount) parameters")
        //     let destroyResult = plugin_destroy(testHandle!)
        //     print("NIHPlugAUv3: Plugin destroy result: \(destroyResult)")
        // } else {
        //     print("NIHPlugAUv3: FFI plugin_create() failed")
        // }

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
        print("NIHPlugAUv3: Setting up parameter tree...")
        // Create an empty parameter tree
        _parameterTree = AUParameterTree.createTree(withChildren: [])
        print("NIHPlugAUv3: Parameter tree created")
    }
    
    // MARK: - AVAudioUnit Overrides
    
    public override var parameterTree: AUParameterTree? {
        get { 
            print("NIHPlugAUv3: parameterTree getter called")
            return _parameterTree 
        }
        set { 
            print("NIHPlugAUv3: parameterTree setter called")
            _parameterTree = newValue 
        }
    }
    
    public override var inputBusses: AUAudioUnitBusArray {
        print("NIHPlugAUv3: inputBusses getter called")
        return _inputBusses ?? AUAudioUnitBusArray(audioUnit: self, busType: .input, busses: [])
    }
    
    public override var outputBusses: AUAudioUnitBusArray {
        print("NIHPlugAUv3: outputBusses getter called")
        return _outputBusses ?? AUAudioUnitBusArray(audioUnit: self, busType: .output, busses: [])
    }
    
    // MARK: - Required Audio Unit Properties

    /// Indicates whether the audio unit can process audio in-place.
    public override var canProcessInPlace: Bool {
        print("NIHPlugAUv3: canProcessInPlace getter called")
        return true
    }

    /// Maximum number of frames the audio unit can render in a single call.
    public override var maximumFramesToRender: AUAudioFrameCount {
        get {
            print("NIHPlugAUv3: maximumFramesToRender getter called")
            return 512
        }
        set {
            print("NIHPlugAUv3: maximumFramesToRender setter called")
            // Do nothing - we don't support changing this
        }
    }

    // COMMENTED OUT FOR TESTING - THIS WAS CAUSING CRASHES
    // /// Channel capabilities - declares support for stereo input/output.
    // /// Format: [inputChannels, outputChannels]
    // /// (2, 2) means 2 input channels -> 2 output channels (stereo)
    // public override var channelCapabilities: [NSNumber]? {
    //     print("NIHPlugAUv3: channelCapabilities getter called")
    //     return [2, 2]
    // }

    // MARK: - Required Audio Unit Interface Methods
    
    public override func allocateRenderResources() throws {
        print("NIHPlugAUv3: allocateRenderResources called")
        try super.allocateRenderResources()
        print("NIHPlugAUv3: Render resources allocated successfully")
    }
    
    public override func deallocateRenderResources() {
        print("NIHPlugAUv3: deallocateRenderResources called")
        super.deallocateRenderResources()
        print("NIHPlugAUv3: Render resources deallocated")
    }
    
    public override func reset() {
        print("NIHPlugAUv3: reset called")
        super.reset()
        print("NIHPlugAUv3: Audio unit reset")
    }
    
    // MARK: - Audio Processing
    
    public override var internalRenderBlock: AUInternalRenderBlock {
        print("NIHPlugAUv3: internalRenderBlock getter called")
        return { (actionFlags, timestamp, frameCount, outputBusNumber, outputData, renderEvent, pullInputBlock) in
            print("NIHPlugAUv3: Audio processing block called - frameCount: \(frameCount)")
            
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
    
    // MARK: - Audio Unit Registration Entry Point
    
    /// Alternative registration method that might be needed for some systems
    @objc public class func registerAudioUnit() {
        print("NIHPlugAUv3: Static registration method called")
        // This method might be called by the system for registration
    }
}