# Handoff: Iteration 051 - Revert to Headless Mode

**Date:** 2025-10-27
**Iteration:** 051
**Status:** VALIDATION STILL FAILING - Same error persists

## What Was Done

### Changes Made
1. **Reverted Info.plist to headless mode:**
   - Changed `NSExtensionPointIdentifier` from `com.apple.AudioUnit-UI` back to `com.apple.AudioUnit`
   - Changed `NSExtensionPrincipalClass` from `NIHPlugAUv3ViewController` back to `NIHPlugAUv3`
   - KEPT AudioComponents inside NSExtensionAttributes (Fix 1)
   - KEPT NSExtensionServiceRoleType (Fix 2)

2. **Kept all other working fixes:**
   - Entitlements file unchanged (Fix 2 from iteration 049)
   - AudioComponents structure preserved (Fix 1 from iteration 049)

3. **Rebuilt and tested:**
   - Full rebuild with `cargo xtask bundle-universal gain --release`
   - Validation with auval
   - Registration check with pluginkit

## Test Results

### Pluginkit Status
```
com.nihplug.auv3(1.0)	359FA0A5-81D7-4E61-BBD9-C5395558BCE6	2025-10-27 22:09:38 +0000	/Applications/gain.app/Contents/PlugIns/NIHPlugAUv3.appex
```
✅ Plugin IS registered with system
✅ Has valid UUID
✅ Timestamp shows it updated after our changes

### Auval Validation
```
AU Validation Tool
    Version: 1.10.0
    Copyright 2003-2019, Apple Inc. All Rights Reserved.

--------------------------------------------------
VALIDATING AUDIO UNIT: 'aufx' - 'Nplg' - 'NPLG'
--------------------------------------------------
ERROR: Cannot get Component's Name strings
ERROR: Error from retrieving Component Version: -50

* * FAIL
--------------------------------------------------
TESTING OPEN TIMES:
FATAL ERROR: didn't find the component
```
❌ Same validation error as before
❌ Error -50: Cannot get Component's Name strings
❌ Cannot retrieve Component Version

## Key Findings

### What This Test Proved
1. **The UI extension point was NOT the problem**
   - Reverting from `com.apple.AudioUnit-UI` to `com.apple.AudioUnit` didn't fix validation
   - Same error -50 persists
   - Plugin still can't be found by auval

2. **Fixes 1 + 2 are working correctly**
   - Plugin continues to be discovered by pluginkit
   - Registration and UUID are valid
   - The problem is happening AFTER discovery, during validation

3. **The root cause is deeper**
   - The "Cannot get Component's Name strings" error suggests an issue with how the AUAudioUnit is returning metadata
   - This is likely in the Swift/Rust bridge or in how the component properties are exposed

## Current Status Summary

| Component | Status | Details |
|-----------|--------|---------|
| System Registration | ✅ WORKING | pluginkit shows plugin |
| Plugin Discovery | ✅ WORKING | Logic Pro scans at startup |
| Component Name Strings | ❌ FAILING | Error -50 during validation |
| Component Version | ❌ FAILING | Cannot retrieve |
| Plugin Menu Appearance | ❌ FAILING | Doesn't show in Logic Pro |

## Next Steps to Consider

### Hypothesis: The AUAudioUnit implementation isn't properly returning component information

**Potential fixes to investigate:**

1. **Check AUAudioUnit.swift implementation:**
   - Look at how `manufacturerName`, `componentVersion`, `audioUnitName` are implemented
   - Verify these properties are being overridden correctly
   - Check if the Rust FFI bridge is returning the right values

2. **Inspect the component registration:**
   - The Info.plist has the name as `NPLG:NIH-Plug AUv3`
   - Check if auval is looking for different name strings
   - Consider if the format of the name matters

3. **Test with a minimal Swift AUv3:**
   - Create a minimal Swift-only AUv3 that doesn't use Rust
   - See if it validates successfully
   - Compare its implementation with ours

4. **Check AudioComponentDescription:**
   - Verify the type, subtype, manufacturer codes match between Info.plist and Swift code
   - Look at how the component description is created in NIHPlugAUv3.swift

5. **Enable verbose logging:**
   - Add logging to the AUAudioUnit initialization
   - Check Console.app for any error messages during validation
   - Look for CoreAudio framework errors

## Files Modified

- `/Users/ludvig/Desktop/nih-plug/src/wrapper/auv3/swift/NIHPlugAUv3/Info.plist`
  - Lines 53-56: Reverted extension point from UI to headless

## Command Reference

### Testing Commands
```bash
# Build
cargo xtask bundle-universal gain --release

# Check registration
pluginkit -m -v | grep -i nih

# Validate
auval -v aufx Nplg NPLG

# Check logs
log show --predicate 'subsystem == "com.apple.audio.CoreAudio"' --last 5m
```

### Logic Pro Testing
1. Open Logic Pro (or restart if already open)
2. Create new track
3. Check Audio FX menu
4. Look for "NIH-Plug AUv3" or "NPLG" in manufacturer section

## CRITICAL DISCOVERY: Missing AUAudioUnit Properties

### Root Cause Identified

After examining `/Users/ludvig/Desktop/nih-plug/src/wrapper/auv3/swift/NIHPlugAUv3/AUAudioUnit.swift`, I found that the class is **missing the required property overrides** that auval needs:

**Missing Properties:**
- `manufacturerName` - NOT overridden
- `audioUnitName` (or `name`) - NOT overridden
- `componentVersion` - NOT overridden
- `audioUnitShortName` - NOT overridden

These are the "component name strings" that auval is complaining about with error -50!

The Info.plist has these values:
- name: `NPLG:NIH-Plug AUv3`
- description: `NIH-Plug Audio Unit v3 Plugin`
- version: `1`
- manufacturer: `NPLG`

But they need to be exposed through the AUAudioUnit class properties as well. The system can't retrieve them because the Swift class doesn't provide them.

### What AUAudioUnit Should Override

According to Apple's AUAudioUnit documentation, these properties should be overridden:

```swift
public override var manufacturerName: String {
    return "NPLG"  // Must match Info.plist manufacturer
}

public override var audioUnitName: String? {
    return "NIH-Plug AUv3"  // Must match Info.plist name
}

public override var audioUnitShortName: String? {
    return "NIH-Plug"  // Short name for UI
}

public override var componentVersion: UInt32 {
    return 1  // Must match Info.plist version
}
```

## Recommendation for Next Iteration

**HIGH PRIORITY FIX:** Add the missing property overrides to `AUAudioUnit.swift`

The validation issue is NOT related to the extension point (headless vs UI). The problem is that the AUAudioUnit class isn't exposing the component metadata properties that auval requires.

### Exact Fix Needed

Add these property overrides to the `NIHPlugAUv3` class in `AUAudioUnit.swift`:

1. Add after the `// MARK: - Required Audio Unit Properties` section (around line 125):

```swift
// MARK: - Component Information Properties

public override var manufacturerName: String {
    return "NPLG"
}

public override var audioUnitName: String? {
    return "NIH-Plug AUv3"
}

public override var audioUnitShortName: String? {
    return "NIH-Plug"
}

public override var componentVersion: UInt32 {
    return 1
}
```

2. Rebuild and test with auval

This should fix the "Cannot get Component's Name strings" error and allow validation to proceed.

The fact that pluginkit sees it but auval can't retrieve name strings confirms the problem is in the AudioUnit API implementation, not in the registration/discovery mechanism.
