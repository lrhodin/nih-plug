#!/usr/bin/env swift

import Foundation
import AVFoundation
import AudioToolbox

// Test if AUv3 plugins can be found using AVAudioUnitComponentManager
print("Testing AUv3 plugin discovery...")
print("================================================================================\n")

// Create an AVAudioUnitComponentManager to search for AUv3 plugins
let componentManager = AVAudioUnitComponentManager.shared()

// Test 1: Search for NIHPlugAUv3
print("Test 1: Searching for NIHPlugAUv3 plugin...")
let nihplugDescription = AudioComponentDescription(
    componentType: OSType("aufx".fourCharCode),
    componentSubType: OSType("Nplg".fourCharCode),
    componentManufacturer: OSType("NPLG".fourCharCode),
    componentFlags: 0,
    componentFlagsMask: 0
)

let nihplugComponents = componentManager.components(matching: nihplugDescription)
print("  Found \(nihplugComponents.count) NIHPlugAUv3 components")
for component in nihplugComponents {
    print("    - \(component.name) (\(component.manufacturerName))")
}
print()

// Test 2: Search for Apple's FilterDemo
print("Test 2: Searching for FilterDemo plugin...")
let filterDescription = AudioComponentDescription(
    componentType: OSType("aufx".fourCharCode),
    componentSubType: OSType("f1tR".fourCharCode),
    componentManufacturer: OSType("Demo".fourCharCode),
    componentFlags: 0,
    componentFlagsMask: 0
)

let filterComponents = componentManager.components(matching: filterDescription)
print("  Found \(filterComponents.count) FilterDemo components")
for component in filterComponents {
    print("    - \(component.name) (\(component.manufacturerName))")
}
print()

// Test 3: List ALL Audio Units to see what's available
print("Test 3: Listing all Audio Units (first 30)...")
let allDescription = AudioComponentDescription(
    componentType: 0,
    componentSubType: 0,
    componentManufacturer: 0,
    componentFlags: 0,
    componentFlagsMask: 0
)

let allComponents = componentManager.components(matching: allDescription)
print("  Total components found: \(allComponents.count)")
var count = 0
for component in allComponents {
    if count >= 30 {
        print("  ... (showing first 30 of \(allComponents.count))")
        break
    }
    print("    - \(component.name) (\(component.manufacturerName)) - \(component.typeName)")
    count += 1
}

// Extension to convert string to FourCharCode
extension String {
    var fourCharCode: UInt32 {
        let chars = Array(self.utf8)
        return UInt32(chars[0]) << 24 | UInt32(chars[1]) << 16 | UInt32(chars[2]) << 8 | UInt32(chars[3])
    }
}
