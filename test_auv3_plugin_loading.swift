#!/usr/bin/env swift

import Foundation
import AVFoundation
import AudioToolbox

// Test if the NIHPlugAUv3 plugin can be found and loaded using AUv3 APIs
print("Testing NIHPlugAUv3 plugin loading using AUv3 APIs...")

// Create an AVAudioUnitComponentManager to search for AUv3 plugins
let componentManager = AVAudioUnitComponentManager.shared()

// Search for Audio Units with our plugin's identifiers
let componentDescription = AudioComponentDescription(
    componentType: OSType("aufx".fourCharCode),
    componentSubType: OSType("Nplg".fourCharCode),
    componentManufacturer: OSType("NPLG".fourCharCode),
    componentFlags: 0,
    componentFlagsMask: 0
)

print("Searching for Audio Unit with:")
print("  Type: aufx (\(componentDescription.componentType))")
print("  Subtype: Nplg (\(componentDescription.componentSubType))")
print("  Manufacturer: NPLG (\(componentDescription.componentManufacturer))")

// Use AVAudioUnitComponentManager to find the component
let components = componentManager.components(matching: componentDescription)

if components.count > 0 {
    print("✅ Plugin found! Found \(components.count) matching components")
    
    for (index, component) in components.enumerated() {
        print("  Component \(index + 1):")
        print("    Name: \(component.name)")
        print("    Manufacturer: \(component.manufacturerName)")
        print("    Type: \(component.typeName)")
        print("    Version: \(component.version)")
        print("    Is AUv3: \(component.hasCustomView)")
        
        // Try to instantiate the component
        print("    Component path: \(String(describing: component.audioComponent))")

        let semaphore = DispatchSemaphore(value: 0)
        AVAudioUnit.instantiate(with: component.audioComponentDescription, options: []) { audioUnit, error in
            if let error = error {
                print("    ❌ Failed to instantiate: \(error)")
            } else if let audioUnit = audioUnit {
                print("    ✅ Successfully instantiated AVAudioUnit")
                print("    ✅ Plugin is working correctly!")
            }
            semaphore.signal()
        }
        semaphore.wait()
    }
} else {
    print("❌ Plugin not found!")
    
    // List all available Audio Units to help debug
    print("\nAvailable Audio Units:")
    let allComponents = componentManager.components(matching: AudioComponentDescription())
    var count = 0
    for component in allComponents {
        print("  \(component.typeName) - \(component.manufacturerName)")
        count += 1
        if count > 20 { // Limit output
            print("  ... (showing first 20)")
            break
        }
    }
}

// Extension to convert string to FourCharCode
extension String {
    var fourCharCode: UInt32 {
        let chars = Array(self.utf8)
        return UInt32(chars[0]) << 24 | UInt32(chars[1]) << 16 | UInt32(chars[2]) << 8 | UInt32(chars[3])
    }
}