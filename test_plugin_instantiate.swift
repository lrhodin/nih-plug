#!/usr/bin/env swift

import AVFoundation
import AudioToolbox

print("Testing NIH-Plug AUv3 instantiation...")

// Component description for our plugin
var componentDesc = AudioComponentDescription(
    componentType: fourCharCodeFrom("aufx"),
    componentSubType: fourCharCodeFrom("Nplg"),
    componentManufacturer: fourCharCodeFrom("NPLG"),
    componentFlags: 0,
    componentFlagsMask: 0
)

print("Looking for component with description:")
print("  Type: aufx (\(componentDesc.componentType))")
print("  Subtype: Nplg (\(componentDesc.componentSubType))")
print("  Manufacturer: NPLG (\(componentDesc.componentManufacturer))")

// Try to find the component
if let component = AudioComponentFindNext(nil, &componentDesc) {
    print("✅ Found audio component!")

    // Get component info
    var name: Unmanaged<CFString>?
    var version: UInt32 = 0
    var manufacturer: Unmanaged<CFString>?

    AudioComponentCopyName(component, &name)
    AudioComponentGetVersion(component, &version)

    if let componentName = name?.takeRetainedValue() as String? {
        print("  Name: \(componentName)")
    }
    print("  Version: \(version)")

    // Try to instantiate
    print("\nAttempting to instantiate audio unit...")

    AUAudioUnit.instantiate(with: componentDesc, options: []) { audioUnit, error in
        if let error = error {
            print("❌ Failed to instantiate: \(error.localizedDescription)")
            print("   Error code: \(error._code)")
            print("   Domain: \(error._domain)")
        } else if let audioUnit = audioUnit {
            print("✅ Successfully instantiated audio unit!")
            print("   Class: \(type(of: audioUnit))")
            print("   ComponentDescription: \(audioUnit.componentDescription)")

            // Try to allocate resources
            do {
                try audioUnit.allocateRenderResources()
                print("✅ Successfully allocated render resources!")

                // Clean up
                audioUnit.deallocateRenderResources()
                print("✅ Successfully deallocated render resources!")
            } catch {
                print("❌ Failed to allocate resources: \(error)")
            }
        }
    }

    // Wait for async completion
    RunLoop.main.run(until: Date(timeIntervalSinceNow: 5))

} else {
    print("❌ Could not find audio component!")
    print("\nSearching for all audio effects...")

    var searchDesc = AudioComponentDescription(
        componentType: fourCharCodeFrom("aufx"),
        componentSubType: 0,
        componentManufacturer: 0,
        componentFlags: 0,
        componentFlagsMask: 0
    )

    var comp: AudioComponent? = AudioComponentFindNext(nil, &searchDesc)
    var count = 0
    while comp != nil {
        var name: Unmanaged<CFString>?
        AudioComponentCopyName(comp!, &name)
        if let componentName = name?.takeRetainedValue() as String? {
            print("  Found: \(componentName)")
            count += 1
        }
        comp = AudioComponentFindNext(comp, &searchDesc)
    }
    print("Total audio effects found: \(count)")
}

// Helper function to convert string to FourCharCode
func fourCharCodeFrom(_ string: String) -> FourCharCode {
    let chars = Array(string.utf8)
    guard chars.count == 4 else { return 0 }
    return FourCharCode(chars[0]) << 24 | FourCharCode(chars[1]) << 16 | FourCharCode(chars[2]) << 8 | FourCharCode(chars[3])
}

print("\nTest complete.")
