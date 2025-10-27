---
date: 2025-10-27T14:58:55-0700
researcher: Implementation Ralph
git_commit: fddacbae6bbe885306dbdaac5acb2a3c2aec7f40
branch: master
repository: nih-plug
topic: "AUv3 Validation Error Debugging"
tags: [auv3, validation, debugging, audio-unit, swift]
status: in_progress
last_updated: 2025-10-27
last_updated_by: Implementation Ralph
type: debugging
---

# Handoff: Iteration 050 - AUv3 Validation Error Debugging

## Task(s)

**Status: IN PROGRESS**

The plugin is successfully being SCANNED by Logic Pro (Fixes 1+2+3 from previous iterations solved the discovery issue), but it is being REJECTED during validation with these errors:

```
ERROR: Cannot get Component's Name strings
ERROR: Error from retrieving Component Version: -50
```

**Current Task:** Fix validation errors by ensuring proper component identification in the AUv3 extension.

**What I Tried:**
1. Initially attempted to override `componentName`, `componentVersion`, and `manufacturerName` properties in the `NIHPlugAUv3` class
2. Research revealed these are READ-ONLY properties that are automatically derived from Info.plist
3. Removed the property overrides and updated Info.plist to use the proper "Manufacturer:Unit Name" format
4. Changed component name from "NIH-Plug AUv3" to "NPLG:NIH-Plug AUv3" in Info.plist
5. Rebuilt and reinstalled the plugin
6. Validation errors persist despite correct Info.plist configuration

## Critical References

- Apple Developer Documentation on AUAudioUnit component properties (read-only, derived from Info.plist)
- Info.plist AudioComponents array configuration in `/Users/ludvig/Desktop/nih-plug/src/wrapper/auv3/swift/NIHPlugAUv3/Info.plist`
- AUv3 extension point: `com.apple.AudioUnit-UI` (used for plugins with custom UI)

## Recent Changes

**Modified Files:**
- `/Users/ludvig/Desktop/nih-plug/src/wrapper/auv3/swift/NIHPlugAUv3/AUAudioUnit.swift:145-161` - Initially added property overrides (REVERTED)
- `/Users/ludvig/Desktop/nih-plug/src/wrapper/auv3/swift/NIHPlugAUv3/Info.plist:39` - Changed name from "NIH-Plug AUv3" to "NPLG:NIH-Plug AUv3"
- `/Users/ludvig/Desktop/nih-plug/.ralph/current_iteration.txt:1` - Updated to iteration 050

**Build Status:**
- Plugin builds successfully with no compilation errors
- Plugin installs to `/Applications/gain.app/Contents/PlugIns/NIHPlugAUv3.appex/`
- Extension is registered with pluginkit: `com.nihplug.auv3(1.0)`

## Learnings

### Key Discovery: Read-Only Properties
The `componentName`, `componentVersion`, and `manufacturerName` properties in AUAudioUnit are **read-only** and cannot be overridden in Swift. These values are automatically derived from the Info.plist configuration.

### Proper Naming Convention
According to Apple's documentation, the component name in Info.plist should follow the format: `"Manufacturer:Unit Name"`. The system parses this to extract both the manufacturer name and unit name. Example: `"NPLG:NIH-Plug AUv3"`

### Extension Registration
- AUv3 extensions are registered with `pluginkit` when the parent application launches
- Use `pluginkit -m -v` to verify extension registration
- Registration shows: `com.nihplug.auv3(1.0)` at the correct path
- The extension IS being discovered by the system

### Error -50
Error code -50 is a macOS error meaning "parameter error" or "invalid parameter". In the context of Audio Units, this typically indicates a mismatch between what the system expects and what the plugin provides.

### Extension Point Identifier
Using `com.apple.AudioUnit-UI` as the NSExtensionPointIdentifier is correct for plugins with a custom view controller factory. This differs from `com.apple.AudioUnit` which is for plugins without UI.

## Artifacts

**Configuration Files:**
- `/Users/ludvig/Desktop/nih-plug/src/wrapper/auv3/swift/NIHPlugAUv3/Info.plist` - Updated with proper naming convention
- `/Users/ludvig/Desktop/nih-plug/src/wrapper/auv3/swift/NIHPlugAUv3/AUAudioUnit.swift` - Reverted to not override read-only properties

**Build Output:**
- `/Users/ludvig/Desktop/nih-plug/target/bundled/gainHost.app/` - Built host application
- `/Applications/gain.app/` - Installed plugin location

**Key Info.plist Configuration:**
```xml
<key>name</key>
<string>NPLG:NIH-Plug AUv3</string>
<key>manufacturer</key>
<string>NPLG</string>
<key>version</key>
<integer>1</integer>
```

## Action Items & Next Steps

### 1. Investigate Extension Point Identifier Issue
The plugin uses `com.apple.AudioUnit-UI` as the NSExtensionPointIdentifier. While this is correct for plugins with UI, it may require additional configuration beyond just having a view controller.

**Action:** Research if `com.apple.AudioUnit-UI` requires explicit property implementations that differ from the standard `com.apple.AudioUnit` extension point.

### 2. Test with Standard Extension Point
Try changing the NSExtensionPointIdentifier from `com.apple.AudioUnit-UI` to `com.apple.AudioUnit` to see if the validation error is specific to the UI extension point.

**File:** `/Users/ludvig/Desktop/nih-plug/src/wrapper/auv3/swift/NIHPlugAUv3/Info.plist:54`

### 3. Verify View Controller Factory Implementation
Since we're using `com.apple.AudioUnit-UI`, ensure the `NIHPlugAUv3ViewController` properly implements the `AUAudioUnitFactory` protocol.

**File:** `/Users/ludvig/Desktop/nih-plug/src/wrapper/auv3/swift/NIHPlugAUv3/NIHPlugAUv3ViewController.swift:14`

### 4. Check for Missing Protocol Methods
Verify that all required methods from AUAudioUnit are properly implemented. The error "Cannot get Component's Name strings" suggests the system cannot retrieve these values through the expected API.

**Action:** Add explicit logging in the `NIHPlugAUv3` class to verify when/if component properties are being accessed.

### 5. Test with Logic Pro
Since pluginkit shows the plugin is registered and Logic Pro is scanning it, try actually opening Logic Pro to see if it can load the plugin despite the auval errors.

**Note:** auval is a strict validator and may flag issues that don't prevent actual functionality.

### 6. Consider Alternative AUv3 Architecture
If the UI extension point continues to cause issues, consider whether the plugin needs a custom UI at all for initial validation. A minimal AUv3 without UI might pass validation more easily.

## Other Notes

### Current Plugin State
- ✅ Plugin discovered by system (pluginkit shows it)
- ✅ Plugin scanned by Logic Pro
- ❌ Plugin rejected during validation (auval fails)
- ❓ Unknown if plugin actually loads in Logic Pro despite auval failure

### Validation Commands Used
```bash
# Check plugin registration
pluginkit -m -v | grep -i "nih\|gain"

# Validate with auval
auval -v aufx Nplg NPLG

# Reinstall plugin
rm -rf /Applications/gain.app && cp -R /Users/ludvig/Desktop/nih-plug/target/bundled/gainHost.app /Applications/gain.app

# Force re-registration
killall -9 AudioComponentRegistrar
pluginkit -a /Applications/gain.app/Contents/PlugIns/NIHPlugAUv3.appex
```

### Build System
The plugin uses a custom `cargo xtask bundle-universal gain --release` command that:
1. Builds Rust static library for both x86_64 and arm64
2. Creates universal binary
3. Builds Swift Xcode project
4. Creates host app with embedded .appex extension
5. Installs to `/Applications/`

### Related Code Locations
- Swift AUAudioUnit implementation: `/Users/ludvig/Desktop/nih-plug/src/wrapper/auv3/swift/NIHPlugAUv3/AUAudioUnit.swift`
- View controller: `/Users/ludvig/Desktop/nih-plug/src/wrapper/auv3/swift/NIHPlugAUv3/NIHPlugAUv3ViewController.swift`
- Info.plist: `/Users/ludvig/Desktop/nih-plug/src/wrapper/auv3/swift/NIHPlugAUv3/Info.plist`
- Build system: `/Users/ludvig/Desktop/nih-plug/nih_plug_xtask/src/lib.rs` (search for "xcode_build_auv3")

### Research Findings
From web searches:
- These validation errors have been reported since ~2013-2016 with AUv2/AUv3
- Common causes: Info.plist misconfiguration, missing properties, registration issues
- The `-50` error code specifically indicates "parameter error"
- Some plugins pass validation after simply launching the host app first
- auval is strict and may report errors that don't prevent actual functionality in DAWs
