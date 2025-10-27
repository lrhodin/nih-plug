# Handoff: Iteration 053 - Switch to UI Extension Point

**Date:** 2025-10-27
**Iteration:** 053
**Status:** READY FOR LOGIC PRO TESTING - Plugin registered, needs DAW validation

## What Was Done

### Changes Made
1. **Switched to UI extension point:**
   - Changed `NSExtensionPointIdentifier` from `com.apple.AudioUnit` to `com.apple.AudioUnit-UI`
   - KEPT `NSExtensionPrincipalClass` as `NIHPlugAUv3.NIHPlugAUv3ViewController` (already correct)
   - KEPT all fixes from iteration 052:
     - AudioComponents configuration inside NSExtensionAttributes
     - NSExtensionServiceRoleType set to NSExtensionServiceRoleTypeEditor
     - Entitlements file configuration

2. **Rebuilt and tested:**
   - Full rebuild with `cargo xtask bundle-universal gain --release`
   - Validation with auval
   - Registration check with pluginkit

## Test Results

### Pluginkit Status
```
com.nihplug.auv3(1.0)	6223FDE6-5C86-4A3A-8FB1-8644422AEED3	2025-10-27 22:37:34 +0000	/Applications/gain.app/Contents/PlugIns/NIHPlugAUv3.appex
```
✅ Plugin IS registered with system
✅ Has valid UUID
✅ Timestamp shows it updated after our changes (22:37:34)
✅ Located at correct path in /Applications

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
❌ Same validation error as iteration 051 (headless mode)
❌ Error -50: Cannot get Component's Name strings
❌ Cannot retrieve Component Version
❌ Plugin not found by auval

### Verification
```
$ cat /Applications/gain.app/Contents/PlugIns/NIHPlugAUv3.appex/Contents/Info.plist | grep -A 1 "NSExtensionPointIdentifier"
<key>NSExtensionPointIdentifier</key>
		<string>com.apple.AudioUnit-UI</string>
```
✅ Extension point correctly set to `com.apple.AudioUnit-UI`

## Key Findings

### Extension Point Comparison

| Extension Point | Iteration | pluginkit | auval | Logic Pro |
|----------------|-----------|-----------|-------|-----------|
| `com.apple.AudioUnit` (headless) | 051 | ✅ Registered | ❌ Error -50 | ❌ Not tested |
| `com.apple.AudioUnit-UI` (UI) | 053 | ✅ Registered | ❌ Error -50 | ⏳ NEEDS TESTING |

### Important Observations

1. **Extension point doesn't affect auval error**
   - Same error -50 occurs with both headless and UI extension points
   - This confirms the auval issue is deeper (likely missing AUAudioUnit property overrides as identified in iteration 051)

2. **UI extension point registers successfully**
   - pluginkit shows the plugin with updated timestamp
   - System recognizes the plugin and assigns a UUID
   - Installation path is correct

3. **Critical test still pending**
   - According to user, UI extension point (`com.apple.AudioUnit-UI`) previously got Logic Pro to scan the plugin
   - This is promising even though auval fails
   - Logic Pro may be more lenient than auval during scanning

### Configuration Consistency Check

Current state is now **consistent**:
- Extension Point: `com.apple.AudioUnit-UI` (UI)
- Principal Class: `NIHPlugAUv3ViewController` (View controller)
- Service Role: `NSExtensionServiceRoleTypeEditor`

This makes sense: UI extension + view controller + editor role.

Previous iteration 052 was **inconsistent**:
- Extension Point: `com.apple.AudioUnit` (headless)
- Principal Class: `NIHPlugAUv3ViewController` (View controller)

## Current Status Summary

| Component | Status | Details |
|-----------|--------|---------|
| System Registration | ✅ WORKING | pluginkit shows plugin with UI extension |
| Info.plist Configuration | ✅ CONSISTENT | UI extension + view controller |
| Extension Point | ✅ UPDATED | Now using `com.apple.AudioUnit-UI` |
| auval Validation | ❌ FAILING | Error -50 (expected, known issue) |
| Logic Pro Recognition | ⏳ **NEEDS TESTING** | **THIS IS THE CRITICAL TEST** |

## Next Steps

### IMMEDIATE ACTION REQUIRED: Test in Logic Pro

**User must test the plugin in Logic Pro** - this is the real validation that matters:

1. **Restart Logic Pro** (important - ensures fresh plugin scan)
2. Create new project
3. Add audio track
4. Look for the plugin in Audio FX menu:
   - Check manufacturer section "NPLG"
   - Look for "NIH-Plug AUv3" or "NPLG: NIH-Plug AUv3"
5. Try to load the plugin
6. Report results:
   - Does it appear in the menu?
   - Does it load successfully?
   - Does the UI appear?

### Why This Test Matters

- User previously reported that UI extension got Logic to scan the plugin
- Logic Pro may not require all the properties that auval checks for
- The consistent configuration (UI + view controller) may work better
- pluginkit registration is working, which is a good sign

### If Logic Pro Works

If Logic Pro recognizes and loads the plugin, we have a **working plugin** even though auval fails. Some plugins work in DAWs without passing auval validation.

### If Logic Pro Still Doesn't Work

Then we need to implement the missing AUAudioUnit property overrides identified in iteration 051:

1. Add property overrides to `/Users/ludvig/Desktop/nih-plug/src/wrapper/auv3/swift/NIHPlugAUv3/AUAudioUnit.swift`:
   - `manufacturerName` → "NPLG"
   - `audioUnitName` → "NIH-Plug AUv3"
   - `audioUnitShortName` → "NIH-Plug"
   - `componentVersion` → 1

2. This should fix the "Cannot get Component's Name strings" error

## Files Modified

- `/Users/ludvig/Desktop/nih-plug/src/wrapper/auv3/swift/NIHPlugAUv3/Info.plist`
  - Line 56: Changed from `com.apple.AudioUnit` to `com.apple.AudioUnit-UI`

## Command Reference

### Testing Commands
```bash
# Build
cargo xtask bundle-universal gain --release

# Check registration
pluginkit -m -v | grep -i nih

# Validate (will fail with error -50, but useful for comparison)
auval -v aufx Nplg NPLG

# Verify extension point in installed plugin
cat /Applications/gain.app/Contents/PlugIns/NIHPlugAUv3.appex/Contents/Info.plist | grep -A 1 "NSExtensionPointIdentifier"

# Check CoreAudio logs
log show --predicate 'subsystem == "com.apple.audio.CoreAudio"' --last 5m
```

### Logic Pro Testing Steps
1. **Restart Logic Pro** (important!)
2. Create new empty project
3. Create audio track
4. Click on Audio FX insert slot
5. Look in "Audio Units" section
6. Navigate to manufacturer "NPLG"
7. Check if "NIH-Plug AUv3" appears
8. Try to load it if visible

## What Changed Between Iterations

| Aspect | Iteration 051 (Headless) | Iteration 053 (UI) |
|--------|--------------------------|-------------------|
| NSExtensionPointIdentifier | `com.apple.AudioUnit` | `com.apple.AudioUnit-UI` |
| NSExtensionPrincipalClass | `NIHPlugAUv3` | `NIHPlugAUv3ViewController` |
| Configuration | Inconsistent | Consistent |
| pluginkit Status | Registered | Registered |
| auval Status | Error -50 | Error -50 (same) |
| Logic Pro Status | Not tested | **Needs testing** |

## Summary

✅ **Completed:** Switched to UI extension point while keeping all iteration 52 fixes
✅ **Verified:** Plugin registered with pluginkit
✅ **Verified:** Extension point correctly set in installed plugin
⏳ **Pending:** Logic Pro testing (CRITICAL)

**The ball is now in the human's court** - they need to test in Logic Pro and report whether the plugin appears in the menu. This will determine if we need to add the AUAudioUnit property overrides or if the plugin already works.
