# AUv3 Audio Processing (internalRenderBlock)

## Overview

AUv3 audio processing is handled through the `internalRenderBlock` property of `AUAudioUnit`. This block is called by the host to process audio buffers and must be implemented to provide real-time audio processing capabilities.

## Core Concepts

### 1. internalRenderBlock

The `internalRenderBlock` is a closure that processes audio data. It's called by the host for each audio buffer that needs to be processed.

```swift
typealias AUInternalRenderBlock = (
    UnsafeMutablePointer<AudioUnitRenderActionFlags>,
    UnsafePointer<AudioTimeStamp>,
    AUAudioFrameCount,
    Int,
    UnsafeMutablePointer<AudioBufferList>,
    UnsafePointer<AURenderEvent>?,
    AURenderPullInputBlock?
) -> Void
```

### 2. Audio Buffer Management

AUv3 uses `AVAudioPCMBuffer` for audio data, which provides a high-level interface for audio buffer management.

```swift
class AVAudioPCMBuffer: AVAudioBuffer {
    var frameCapacity: AVAudioFrameCount { get }
    var frameLength: AVAudioFrameCount { get set }
    var stride: Int { get }
    var floatChannelData: UnsafeMutablePointer<UnsafeMutablePointer<Float>>? { get }
    var int16ChannelData: UnsafeMutablePointer<UnsafeMutablePointer<Int16>>? { get }
    var int32ChannelData: UnsafeMutablePointer<UnsafeMutablePointer<Int32>>? { get }
}
```

## Implementation Patterns

### 1. Basic Audio Processing

```swift
class MyPluginAUAudioUnit: AUAudioUnit {
    private var rustPlugin: PluginHandle?
    
    override var internalRenderBlock: AUInternalRenderBlock {
        return { [weak self] (actionFlags, timestamp, frameCount, outputBusNumber, outputData, realtimeEventListHead, pullInputBlock) in
            guard let self = self, let plugin = self.rustPlugin else {
                return
            }
            
            // Get input audio
            var inputBuffer: AVAudioPCMBuffer?
            if let pullInputBlock = pullInputBlock {
                var inputTimeStamp = timestamp.pointee
                let status = pullInputBlock(actionFlags, &inputTimeStamp, frameCount, 0, &inputBuffer)
                guard status == noErr else { return }
            }
            
            // Convert to raw buffers
            let inputFloats = inputBuffer?.floatChannelData?[0] ?? UnsafePointer<Float>(bitPattern: 0)!
            let outputFloats = outputData.pointee.mBuffers.mData!.assumingMemoryBound(to: Float.self)
            
            // Process audio
            let result = self.pluginProcess(
                plugin,
                inputFloats,
                outputFloats,
                UInt32(frameCount),
                UInt32(outputData.pointee.mBuffers.mNumberChannels)
            )
            
            // Handle result
            if result.status != PLUGIN_OK {
                // Handle error
            }
        }
    }
}
```

### 2. Multi-Channel Audio Processing

```swift
class MultiChannelAudioProcessor {
    private var rustPlugin: PluginHandle?
    
    func processMultiChannel(
        inputBuffer: AVAudioPCMBuffer?,
        outputBuffer: AVAudioPCMBuffer,
        frameCount: AVAudioFrameCount
    ) {
        guard let plugin = rustPlugin else { return }
        
        let inputChannels = inputBuffer?.format.channelCount ?? 0
        let outputChannels = outputBuffer.format.channelCount
        
        // Process each channel
        for channel in 0..<outputChannels {
            let inputChannel = channel < inputChannels ? channel : 0
            let inputFloats = inputBuffer?.floatChannelData?[inputChannel] ?? UnsafePointer<Float>(bitPattern: 0)!
            let outputFloats = outputBuffer.floatChannelData![channel]
            
            let result = pluginProcess(
                plugin,
                inputFloats,
                outputFloats,
                UInt32(frameCount),
                UInt32(outputChannels)
            )
            
            if result.status != PLUGIN_OK {
                // Handle error
            }
        }
    }
}
```

### 3. Sample-Accurate Parameter Automation

```swift
class ParameterAutomationProcessor {
    private var parameterTree: AUParameterTree?
    private var rustPlugin: PluginHandle?
    
    func processParameterAutomation(
        frameCount: AVAudioFrameCount,
        timestamp: AudioTimeStamp
    ) {
        guard let parameterTree = parameterTree, let plugin = rustPlugin else { return }
        
        // Process parameter changes for each frame
        for frame in 0..<frameCount {
            // Get parameter changes for this frame
            let parameterChanges = getParameterChangesForFrame(frame, timestamp: timestamp)
            
            for (paramId, value) in parameterChanges {
                // Apply parameter change
                _ = pluginSetParameter(plugin, paramId, value)
            }
        }
    }
    
    private func getParameterChangesForFrame(
        _ frame: AVAudioFrameCount,
        timestamp: AudioTimeStamp
    ) -> [UInt32: Float] {
        // Implementation would depend on the specific automation system
        // This is a simplified example
        return [:]
    }
}
```

## Buffer Format Handling

### 1. Format Conversion

```swift
class BufferFormatConverter {
    func convertToRustFormat(
        inputBuffer: AVAudioPCMBuffer?,
        outputBuffer: AVAudioPCMBuffer,
        frameCount: AVAudioFrameCount
    ) -> (input: UnsafePointer<Float>, output: UnsafeMutablePointer<Float>) {
        
        let inputFloats = inputBuffer?.floatChannelData?[0] ?? UnsafePointer<Float>(bitPattern: 0)!
        let outputFloats = outputBuffer.floatChannelData![0]
        
        return (input: inputFloats, output: outputFloats)
    }
    
    func convertFromRustFormat(
        outputBuffer: AVAudioPCMBuffer,
        frameCount: AVAudioFrameCount
    ) {
        // Post-processing if needed
        // For example, applying gain, limiting, etc.
    }
}
```

### 2. Interleaved vs Non-Interleaved

```swift
class BufferLayoutHandler {
    func processInterleaved(
        inputBuffer: AVAudioPCMBuffer?,
        outputBuffer: AVAudioPCMBuffer,
        frameCount: AVAudioFrameCount
    ) {
        // Handle interleaved audio format
        // Input: [L1, R1, L2, R2, ...]
        // Output: [L1, R1, L2, R2, ...]
    }
    
    func processNonInterleaved(
        inputBuffer: AVAudioPCMBuffer?,
        outputBuffer: AVAudioPCMBuffer,
        frameCount: AVAudioFrameCount
    ) {
        // Handle non-interleaved audio format
        // Input: [L1, L2, L3, ...], [R1, R2, R3, ...]
        // Output: [L1, L2, L3, ...], [R1, R2, R3, ...]
    }
}
```

## Real-Time Constraints

### 1. Performance Requirements

```swift
class RealtimeAudioProcessor {
    private var rustPlugin: PluginHandle?
    
    func processRealtime(
        inputBuffer: AVAudioPCMBuffer?,
        outputBuffer: AVAudioPCMBuffer,
        frameCount: AVAudioFrameCount
    ) {
        // Real-time audio processing must be:
        // 1. Non-blocking
        // 2. Non-allocating
        // 3. Deterministic
        // 4. Low-latency
        
        guard let plugin = rustPlugin else { return }
        
        // Pre-allocate buffers if needed
        // Use stack-allocated arrays for small data
        // Avoid dynamic memory allocation
        // Minimize function call overhead
        
        let result = pluginProcess(
            plugin,
            inputBuffer?.floatChannelData?[0] ?? UnsafePointer<Float>(bitPattern: 0)!,
            outputBuffer.floatChannelData![0],
            UInt32(frameCount),
            UInt32(outputBuffer.format.channelCount)
        )
        
        // Handle result without blocking
        if result.status != PLUGIN_OK {
            // Log error but don't block
            // Consider using a lock-free error reporting mechanism
        }
    }
}
```

### 2. Thread Safety

```swift
class ThreadSafeAudioProcessor {
    private var rustPlugin: PluginHandle?
    private let processingQueue = DispatchQueue(label: "audio.processing", qos: .userInteractive)
    
    func processThreadSafe(
        inputBuffer: AVAudioPCMBuffer?,
        outputBuffer: AVAudioPCMBuffer,
        frameCount: AVAudioFrameCount
    ) {
        // Audio processing runs on a dedicated high-priority thread
        // Must be thread-safe with parameter changes and state updates
        
        processingQueue.sync {
            guard let plugin = rustPlugin else { return }
            
            // Process audio
            let result = pluginProcess(
                plugin,
                inputBuffer?.floatChannelData?[0] ?? UnsafePointer<Float>(bitPattern: 0)!,
                outputBuffer.floatChannelData![0],
                UInt32(frameCount),
                UInt32(outputBuffer.format.channelCount)
            )
            
            // Handle result
            if result.status != PLUGIN_OK {
                // Handle error
            }
        }
    }
}
```

## Error Handling

### 1. Graceful Error Handling

```swift
class ErrorHandlingAudioProcessor {
    private var rustPlugin: PluginHandle?
    private var errorCount: Int = 0
    private let maxErrors = 10
    
    func processWithErrorHandling(
        inputBuffer: AVAudioPCMBuffer?,
        outputBuffer: AVAudioPCMBuffer,
        frameCount: AVAudioFrameCount
    ) {
        guard let plugin = rustPlugin else { return }
        
        do {
            let result = pluginProcess(
                plugin,
                inputBuffer?.floatChannelData?[0] ?? UnsafePointer<Float>(bitPattern: 0)!,
                outputBuffer.floatChannelData![0],
                UInt32(frameCount),
                UInt32(outputBuffer.format.channelCount)
            )
            
            if result.status != PLUGIN_OK {
                handleProcessingError(result.status)
            } else {
                errorCount = 0 // Reset error count on success
            }
        } catch {
            handleProcessingError(PLUGIN_ERROR)
        }
    }
    
    private func handleProcessingError(_ status: Int32) {
        errorCount += 1
        
        if errorCount > maxErrors {
            // Too many errors, disable processing
            // Consider notifying the host
            return
        }
        
        // Log error for debugging
        // Consider using a lock-free logging mechanism
    }
}
```

### 2. Fallback Processing

```swift
class FallbackAudioProcessor {
    private var rustPlugin: PluginHandle?
    private var fallbackMode: Bool = false
    
    func processWithFallback(
        inputBuffer: AVAudioPCMBuffer?,
        outputBuffer: AVAudioPCMBuffer,
        frameCount: AVAudioFrameCount
    ) {
        guard let plugin = rustPlugin else { return }
        
        if fallbackMode {
            // Use simplified processing
            processFallback(inputBuffer: inputBuffer, outputBuffer: outputBuffer, frameCount: frameCount)
        } else {
            // Use full processing
            let result = pluginProcess(
                plugin,
                inputBuffer?.floatChannelData?[0] ?? UnsafePointer<Float>(bitPattern: 0)!,
                outputBuffer.floatChannelData![0],
                UInt32(frameCount),
                UInt32(outputBuffer.format.channelCount)
            )
            
            if result.status != PLUGIN_OK {
                // Switch to fallback mode
                fallbackMode = true
                processFallback(inputBuffer: inputBuffer, outputBuffer: outputBuffer, frameCount: frameCount)
            }
        }
    }
    
    private func processFallback(
        inputBuffer: AVAudioPCMBuffer?,
        outputBuffer: AVAudioPCMBuffer,
        frameCount: AVAudioFrameCount
    ) {
        // Simple pass-through or basic processing
        if let input = inputBuffer {
            // Copy input to output
            outputBuffer.floatChannelData![0].assign(from: input.floatChannelData![0], count: Int(frameCount))
        } else {
            // Generate silence
            outputBuffer.floatChannelData![0].assign(repeating: 0.0, count: Int(frameCount))
        }
    }
}
```

## Integration with NIH-plug

### 1. Buffer Conversion

```swift
class NIHPlugBufferConverter {
    func convertToNIHPlugFormat(
        inputBuffer: AVAudioPCMBuffer?,
        outputBuffer: AVAudioPCMBuffer,
        frameCount: AVAudioFrameCount
    ) -> (input: [UnsafePointer<Float>], output: [UnsafeMutablePointer<Float>]) {
        
        var inputChannels: [UnsafePointer<Float>] = []
        var outputChannels: [UnsafeMutablePointer<Float>] = []
        
        // Convert input channels
        if let input = inputBuffer {
            for channel in 0..<input.format.channelCount {
                inputChannels.append(input.floatChannelData![Int(channel)])
            }
        }
        
        // Convert output channels
        for channel in 0..<outputBuffer.format.channelCount {
            outputChannels.append(outputBuffer.floatChannelData![Int(channel)])
        }
        
        return (input: inputChannels, output: outputChannels)
    }
}
```

### 2. Context Creation

```swift
class NIHPlugContextCreator {
    func createProcessContext(
        timestamp: AudioTimeStamp,
        frameCount: AVAudioFrameCount
    ) -> ProcessContext {
        // Create process context for NIH-plug
        // This would integrate with the existing NIH-plug context system
        return ProcessContext()
    }
}
```

## Best Practices

### 1. Performance
- Minimize allocations in audio processing
- Use stack-allocated arrays for small data
- Avoid dynamic memory allocation
- Use SIMD instructions when possible
- Profile performance regularly

### 2. Thread Safety
- Use lock-free data structures
- Minimize shared state
- Use atomic operations for counters
- Avoid blocking operations
- Use dedicated audio processing threads

### 3. Error Handling
- Handle errors gracefully
- Don't block on errors
- Use fallback processing when possible
- Log errors for debugging
- Consider using lock-free error reporting

### 4. Testing
- Test with various buffer sizes
- Test with different sample rates
- Test with different channel counts
- Test error conditions
- Test real-time performance

## Next Steps

1. Create comprehensive architecture summary
2. Implement audio processing in FFI layer
3. Create Swift audio processing wrapper
4. Integrate with existing NIH-plug audio system
5. Test with real DAWs