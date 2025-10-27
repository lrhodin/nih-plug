#!/usr/bin/env swift

import Foundation
import AudioToolbox

// Test if the NIHPlugAUv3 plugin can be found and loaded
print("Testing NIHPlugAUv3 plugin loading...")

// Search for Audio Units with our plugin's identifiers
var componentDescription = AudioComponentDescription()
componentDescription.componentType = OSType("aufx".fourCharCode)
componentDescription.componentSubType = OSType("Nplg".fourCharCode)
componentDescription.componentManufacturer = OSType("NPLG".fourCharCode)
componentDescription.componentFlags = 0
componentDescription.componentFlagsMask = 0

print("Searching for Audio Unit with:")
print("  Type: aufx (\(componentDescription.componentType))")
print("  Subtype: Nplg (\(componentDescription.componentSubType))")
print("  Manufacturer: NPLG (\(componentDescription.componentManufacturer))")

var component: AudioComponent? = nil
component = AudioComponentFindNext(component, &componentDescription)

if let foundComponent = component {
    print("✅ Plugin found!")
    
    // Try to get component info
    var componentInfo = AudioComponentDescription()
    let status = AudioComponentGetDescription(foundComponent, &componentInfo)
    
    if status == noErr {
        print("✅ Component description retrieved successfully")
        print("  Type: \(componentInfo.componentType)")
        print("  Subtype: \(componentInfo.componentSubType)")
        print("  Manufacturer: \(componentInfo.componentManufacturer)")
    } else {
        print("❌ Failed to get component description: \(status)")
    }
    
    // Try to get component name
    var componentName: Unmanaged<CFString>? = nil
    let nameStatus = AudioComponentCopyName(foundComponent, &componentName)
    
    if nameStatus == noErr, let name = componentName?.takeRetainedValue() {
        print("✅ Component name: \(name)")
    } else {
        print("❌ Failed to get component name: \(nameStatus)")
    }
    
} else {
    print("❌ Plugin not found!")
    
    // List all available Audio Units to help debug
    print("\nAvailable Audio Units:")
    var searchComponent: AudioComponent? = nil
    var count = 0
    var emptyDescription = AudioComponentDescription()
    while let foundComponent = AudioComponentFindNext(searchComponent, &emptyDescription) {
        var componentInfo = AudioComponentDescription()
        let status = AudioComponentGetDescription(foundComponent, &componentInfo)
        
        if status == noErr {
            let typeString = String(bytes: withUnsafeBytes(of: &componentInfo.componentType) { Data($0) }, encoding: .ascii) ?? "????"
            let subtypeString = String(bytes: withUnsafeBytes(of: &componentInfo.componentSubType) { Data($0) }, encoding: .ascii) ?? "????"
            let manufacturerString = String(bytes: withUnsafeBytes(of: &componentInfo.componentManufacturer) { Data($0) }, encoding: .ascii) ?? "????"
            
            print("  \(typeString) - \(subtypeString) - \(manufacturerString)")
            count += 1
            if count > 20 { // Limit output
                print("  ... (showing first 20)")
                break
            }
        }
        searchComponent = foundComponent
    }
}

// Extension to convert string to FourCharCode
extension String {
    var fourCharCode: UInt32 {
        let chars = Array(self.utf8)
        return UInt32(chars[0]) << 24 | UInt32(chars[1]) << 16 | UInt32(chars[2]) << 8 | UInt32(chars[3])
    }
}