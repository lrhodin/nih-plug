#!/usr/bin/env swift

import Foundation
import AudioToolbox
import AVFoundation

// Test programmatic loading of the NIH-Plug AUv3 plugin
print("Testing NIH-Plug AUv3 plugin loading...")

// Create AudioComponentDescription for our plugin
var desc = AudioComponentDescription(
    componentType: 0x61756678,  // aufx (Audio Unit Effect)
    componentSubType: 0x4E706C67,  // Nplg (NIH-Plug)
    componentManufacturer: 0x4E504C47,  // NPLG (NIH-Plug)
    componentFlags: 0,
    componentFlagsMask: 0
)

print("Looking for Audio Unit with description:")
print("  Type: 0x\(String(desc.componentType, radix: 16))")
print("  Subtype: 0x\(String(desc.componentSubType, radix: 16))")
print("  Manufacturer: 0x\(String(desc.componentManufacturer, radix: 16))")

// Try to find the component
var component: AudioComponent? = nil
let status = AudioComponentFindNext(nil, &desc, &component)

if status == noErr, let foundComponent = component {
    print("✅ Found Audio Unit component!")
    
    // Try to instantiate it
    var audioUnit: AudioUnit? = nil
    let instantiateStatus = AudioComponentInstanceNew(foundComponent, &audioUnit)
    
    if instantiateStatus == noErr, let unit = audioUnit {
        print("✅ Successfully instantiated Audio Unit!")
        
        // Clean up
        AudioComponentInstanceDispose(unit)
    } else {
        print("❌ Failed to instantiate Audio Unit: \(instantiateStatus)")
    }
} else {
    print("❌ Failed to find Audio Unit component: \(status)")
    
    // List all available Audio Units to see what's registered
    print("\nAvailable Audio Units:")
    var searchDesc = AudioComponentDescription()
    searchDesc.componentType = 0x61756678  // aufx
    searchDesc.componentSubType = 0
    searchDesc.componentManufacturer = 0
    searchDesc.componentFlags = 0
    searchDesc.componentFlagsMask = 0
    
    var currentComponent: AudioComponent? = nil
    while true {
        let findStatus = AudioComponentFindNext(currentComponent, &searchDesc, &currentComponent)
        if findStatus != noErr {
            break
        }
        
        if let comp = currentComponent {
            var compDesc = AudioComponentDescription()
            let nameStatus = AudioComponentGetDescription(comp, &compDesc)
            if nameStatus == noErr {
                print("  Found: Type=0x\(String(compDesc.componentType, radix: 16)), Subtype=0x\(String(compDesc.componentSubType, radix: 16)), Manufacturer=0x\(String(compDesc.componentManufacturer, radix: 16))")
            }
        }
    }
}