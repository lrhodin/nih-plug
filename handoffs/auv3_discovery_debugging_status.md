# AUv3 Discovery Debugging - Current Status Research

**Date**: 2025-10-27T21:11:46Z
**Researcher**: Claude (Sonnet 4.5)
**Git Commit**: fddacbae6bbe885306dbdaac5acb2a3c2aec7f40
**Branch**: master
**Repository**: nih-plug (lrhodin/nih-plug)

## Research Question

Why is the NIH-Plug AUv3 plugin not appearing in Logic Pro, despite implementing all three documented fixes (Info.plist structure, app sandbox entitlement, and UI extension point)?

## Summary

**CRITICAL FINDING: The plugin has NOT been tested in Logic Pro yet!**

All handoff documents (iterations 047-049) state "awaiting Logic Pro testing" as the next step. The three fixes have been successfully implemented and the plugin is properly installed, but **the actual test in Logic Pro has not been performed**.

**Current State:**
- ✅ Plugin installed at `/Applications/gain.app`
- ✅ Fix 1 applied: AudioComponents inside NSExtensionAttributes
- ✅ Fix 2 applied: App sandbox entitlement (`com.apple.security.app-sandbox`)
- ✅ Fix 3 applied: UI extension point (`com.apple.AudioUnit-UI`) with ViewController
- ✅ Plugin registered with pluginkit
- ❌ Plugin NOT appearing in AVAudioUnitComponentManager (programmatic test)
- ❓ **Unknown: Plugin status in Logic Pro** (test not yet performed)

**Key Insight from Iteration 046:**
Apple's FilterDemo sample ALSO did not appear in AVAudioUnitComponentManager during programmatic tests, but it DID appear in Logic Pro when tested manually. This suggests that **programmatic discovery tests may not be representative of how Logic Pro actually discovers plugins**.

**RECOMMENDED IMMEDIATE ACTION:**
Test the plugin in Logic Pro before implementing any additional fixes. The fixes may already be working, but this hasn't been validated yet.

## Detailed Findings

### 1. Plugin Installation Verification

**Installation Location**: `/Applications/gain.app`

**Bundle Structure Verified:**
```
/Applications/gain.app/
├── Contents/
│   ├── MacOS/
│   │   └── gainHost
│   └── PlugIns/
│       └── NIHPlugAUv3.appex/
│           └── Contents/
│               ├── Info.plist  ← All fixes applied here
│               ├── MacOS/NIHPlugAUv3
│               └── Frameworks/ (Swift runtime libraries)
```

**Verification Commands Executed:**
```bash
# Plugin exists
ls -la /Applications/gain.app/Contents/PlugIns/NIHPlugAUv3.appex/Contents/Info.plist
# Output: -rwxr-xr-x@ 1 ludvig admin 2203 Oct 27 12:29 ...

# Extension point identifier
/usr/libexec/PlistBuddy -c "Print :NSExtension:NSExtensionPointIdentifier" ...
# Output: com.apple.AudioUnit-UI ✅

# Principal class
/usr/libexec/PlistBuddy -c "Print :NSExtension:NSExtensionPrincipalClass" ...
# Output: NIHPlugAUv3ViewController ✅

# Entitlements
codesign -d --entitlements :- /Applications/gain.app/Contents/PlugIns/NIHPlugAUv3.appex
# Output: com.apple.security.app-sandbox = true ✅
#         com.apple.security.get-task-allow = true ✅
```

### 2. Fix Implementation Status

#### Fix 1: Info.plist Structure (Iteration 047)
**Status**: ✅ IMPLEMENTED

**Changes Made** (src/wrapper/auv3/swift/NIHPlugAUv3/Info.plist:25-52):
- AudioComponents array moved inside `NSExtension/NSExtensionAttributes` (was previously at top level)
- Added `NSExtensionServiceRoleType = "NSExtensionServiceRoleTypeEditor"`
- All AudioComponents keys preserved (type, subtype, manufacturer, name, description, version, sandboxSafe, hasCustomView)

**Verification**: Info.plist structure now matches Apple's FilterDemo example.

#### Fix 2: App Sandbox Entitlement (Iteration 048)
**Status**: ✅ IMPLEMENTED

**Changes Made**:
- Created entitlements file: `src/wrapper/auv3/swift/NIHPlugAUv3/NIHPlugAUv3.entitlements`
- Added `com.apple.security.app-sandbox = true`
- Added `com.apple.security.get-task-allow = true` (for debugging)
- Updated bundler to sign with entitlements (nih_plug_xtask/src/lib.rs:1111-1174)
- Fixed .swiftmodule removal issue that was preventing signing

**Verification**: Entitlements are embedded in both the .appex binary and bundle.

#### Fix 3: UI Extension Point (Iteration 049)
**Status**: ✅ IMPLEMENTED

**Changes Made**:
- Created `NIHPlugAUv3ViewController.swift` (AUViewController + AUAudioUnitFactory)
- Changed `NSExtensionPointIdentifier` from `com.apple.AudioUnit` to `com.apple.AudioUnit-UI`
- Changed `NSExtensionPrincipalClass` from `NIHPlugAUv3` to `NIHPlugAUv3ViewController`
- View controller creates AUAudioUnit on demand via factory pattern

**Verification**: Info.plist now uses UI extension point and view controller as principal class.

### 3. PlugInKit Registration Status

**Command**: `pluginkit -m -v`

**Result**: ✅ Plugin IS registered
```
com.nihplug.auv3(1.0)	9D471691-8F4D-4231-A89C-F4BFF61C12D0	2025-10-27 19:29:26 +0000	/Applications/gain.app/Contents/PlugIns/NIHPlugAUv3.appex
```

**Analysis**: The plugin is successfully registered as an app extension with PlugInKit. The timestamp (19:29:26) indicates it was registered shortly after the most recent build (Info.plist modified at 12:29). This is a positive sign.

### 4. AVAudioUnitComponentManager Discovery Test

**Test Script**: `test_auv3_discovery.swift`

**Result**: ❌ Plugin NOT found (0 components)

**Test Output**:
```
Test 1: Searching for NIHPlugAUv3 plugin...
  Found 0 NIHPlugAUv3 components

Test 3: Listing all Audio Units (first 30)...
  Total components found: 57
  [Only Apple-provided Audio Units listed]
```

**Analysis**: The plugin does not appear in AVAudioUnitComponentManager's component list, even though:
- It's properly installed
- It's registered with pluginkit
- All three fixes are applied
- The bundle structure is correct

**CRITICAL CONTEXT from Iteration 046**:
During the discovery research phase, **Apple's FilterDemo sample also failed to appear in AVAudioUnitComponentManager** during programmatic tests. However, when tested manually in Logic Pro, FilterDemo DID appear and worked correctly.

This strongly suggests that:
1. AVAudioUnitComponentManager may not be the actual discovery mechanism used by Logic Pro
2. Logic Pro may use additional criteria or a different API for plugin discovery
3. **Programmatic tests may give false negatives**

### 5. System Logs Analysis

**Command**: `log show --predicate 'subsystem == "com.apple.audio" OR process == "AudioComponentRegistrar" OR process CONTAINS "Logic"' --last 10m`

**Result**: No relevant errors found

**Analysis**: The absence of errors in system logs is actually a positive sign - it means:
- No registration failures
- No code signing errors
- No extension loading errors
- The system appears to be accepting the plugin without issues

### 6. Architecture and Implementation Review

#### View Controller Implementation
**File**: src/wrapper/auv3/swift/NIHPlugAUv3/NIHPlugAUv3ViewController.swift:14-99

**Key Components**:
```swift
@objc public class NIHPlugAUv3ViewController: AUViewController, AUAudioUnitFactory {
    // Factory method for creating Audio Unit
    public func createAudioUnit(with componentDescription: AudioComponentDescription) throws -> AUAudioUnit

    // View lifecycle
    public override func loadView()
    public override func viewDidLoad()
}
```

**Pattern**: Factory pattern matching Apple's FilterDemo exactly.

**Logging**: Comprehensive print statements added for debugging:
- "NIHPlugAUv3ViewController: loadView called"
- "NIHPlugAUv3ViewController: createAudioUnit called"
- Component description details

**Note**: These log statements will appear in Console.app when/if Logic Pro attempts to load the plugin.

#### Audio Unit Implementation
**File**: src/wrapper/auv3/swift/NIHPlugAUv3/AUAudioUnit.swift:14-203

**Key Features**:
- Proper AUAudioUnit subclass
- Implements required properties (parameterTree, inputBusses, outputBusses)
- Implements required methods (allocateRenderResources, deallocateRenderResources, reset)
- FFI integration for Rust plugin (plugin_create, plugin_get_parameter_count, plugin_destroy)
- Comprehensive logging throughout

**FFI Verification**: Plugin tests FFI during initialization:
```swift
let testHandle = plugin_create()
if testHandle != nil {
    print("NIHPlugAUv3: FFI plugin_create() succeeded - handle: \(testHandle!)")
    let paramCount = plugin_get_parameter_count(testHandle!)
    print("NIHPlugAUv3: Plugin has \(paramCount) parameters")
}
```

### 7. Comparison with Apple FilterDemo

**From Iteration 046 Research:**

| Aspect | Apple FilterDemo | NIH-Plug (Current) | Status |
|--------|-----------------|---------------------|--------|
| AudioComponents Location | Inside NSExtensionAttributes | Inside NSExtensionAttributes | ✅ Match |
| NSExtensionServiceRoleType | Present | Present | ✅ Match |
| Sandbox Entitlement | Yes | Yes | ✅ Match |
| NSExtensionPointIdentifier | com.apple.AudioUnit-UI | com.apple.AudioUnit-UI | ✅ Match |
| NSExtensionPrincipalClass | ViewController | NIHPlugAUv3ViewController | ✅ Match |
| View Controller | Yes | Yes | ✅ Match |
| AudioComponentBundle | Points to framework | Not present | ❌ Different |
| Separate Framework | Yes | No | ❌ Different |

**Analysis**: The only remaining differences are the framework architecture (Fix 4). However, **Fix 4 should only be attempted if testing in Logic Pro confirms that Fixes 1-3 are insufficient**.

### 8. Testing Workflow Documentation

**From Iteration 049 handoff (lines 140-185):**

The documented testing workflow is:

1. **Quit and Restart Logic Pro** (if open)
   ```bash
   killall "Logic Pro" 2>/dev/null
   ```

2. **Optional: Reset PlugInKit cache** (if plugin was previously registered)
   ```bash
   pluginkit -v -m -D -i com.nihplug.auv3.NIHPlugAUv3
   ```

3. **Open Logic Pro**

4. **Create or open a project**

5. **Check if plugin appears:**
   - Go to Audio FX menu
   - Look for "NIH-Plug AUv3" under Audio Units
   - Look in manufacturer category "NPLG"
   - Look for any new entries that weren't there before

6. **Report Results:**
   - ✅ **If plugin appears**: Fix 3 SUCCESSFUL! Document findings
   - ❌ **If plugin does NOT appear**: Proceed to Fix 4 (Framework Architecture)

**Success Indicators:**
- Plugin appears in Logic Pro Audio FX menu
- Can instantiate plugin (window opens showing simple UI)
- Plugin listed under "NPLG" manufacturer or "Audio Units" category
- View controller's print statements appear in Console.app

**Failure Indicators:**
- Plugin does not appear in any menu
- No new entries in Audio FX list
- Same behavior as iterations 046-048

### 9. Available Testing Scripts

The repository contains several testing approaches:

**1. Discovery Test** (test_auv3_discovery.swift)
- Uses AVAudioUnitComponentManager API
- Current result: Plugin not found
- **Note**: May not be representative of Logic Pro's discovery mechanism

**2. Plugin Loading Test** (test_auv3_plugin_loading.swift)
- Attempts instantiation with AVAudioUnit
- Not tested yet (requires plugin to be discoverable first)

**3. Console.app Monitoring**
```bash
# Filter for NIHPlug or com.nihplug.auv3
# Look for view controller logs:
# - "NIHPlugAUv3ViewController: loadView called"
# - "NIHPlugAUv3ViewController: createAudioUnit called"
# - "NIHPlugAUv3: Starting initialization..."
```

### 10. Build and Installation Process

**Current Build Command**: `cargo xtask bundle-universal gain --release`

**Build Flow** (from bundler research):
1. Rust library build (x86_64 + arm64) → universal binary
2. Xcode build of Swift app extension
3. Bundle assembly (host app + .appex)
4. Pre-signing cleanup (.swiftmodule removal)
5. Code signing with entitlements (binary → .appex → .app)
6. Installation to `/Applications/`

**Last Build**: October 27, 2025 at 12:29 (based on Info.plist timestamp)

**Verification**: All build artifacts present and properly signed.

## Historical Context

### Iteration 046: Discovery Research
- Downloaded and tested Apple's FilterDemo
- Identified 5 structural differences
- **Key Finding**: FilterDemo appeared in Logic Pro, NIH-Plug did not
- Created prioritized fix plan

### Iteration 047: Fix 1 Implementation
- Corrected Info.plist structure
- Moved AudioComponents inside NSExtensionAttributes
- Added NSExtensionServiceRoleType
- **Status**: Awaiting Logic Pro testing

### Iteration 048: Fix 2 Implementation
- Added app sandbox entitlement
- Updated bundler to sign with entitlements
- Fixed .swiftmodule removal issue
- **Status**: Awaiting Logic Pro testing

### Iteration 049: Fix 3 Implementation
- Created view controller (NIHPlugAUv3ViewController)
- Switched to UI extension point (com.apple.AudioUnit-UI)
- Implemented AUAudioUnitFactory protocol
- **Status**: Awaiting Logic Pro testing

### Current Iteration: 048 (per .ralph/current_iteration.txt)
**Note**: There's a discrepancy - the current_iteration.txt file shows iteration 048, but iteration 049 handoff exists and describes completed work. The plugin appears to be at the state described in iteration 049.

## Technical Implementation Details

### Info.plist Configuration (Current)

**Extension Point**: com.apple.AudioUnit-UI
**Principal Class**: NIHPlugAUv3ViewController

**AudioComponents Configuration**:
```xml
<key>NSExtensionAttributes</key>
<dict>
    <key>AudioComponents</key>
    <array>
        <dict>
            <key>type</key>
            <string>aufx</string>  <!-- Audio Effect -->
            <key>subtype</key>
            <string>Nplg</string>  <!-- NIH-Plug identifier -->
            <key>manufacturer</key>
            <string>NPLG</string>  <!-- NIH-Plug manufacturer -->
            <key>name</key>
            <string>NIH-Plug AUv3</string>
            <key>description</key>
            <string>NIH-Plug Audio Unit v3 Plugin</string>
            <key>version</key>
            <integer>1</integer>
            <key>sandboxSafe</key>
            <true/>
            <key>hasCustomView</key>
            <true/>
        </dict>
    </array>
    <key>NSExtensionServiceRoleType</key>
    <string>NSExtensionServiceRoleTypeEditor</string>
</dict>
```

**Four-Character Codes**:
- Type: `aufx` (0x61756678) = Audio Effect
- Subtype: `Nplg` (0x4E706C67) = NIH-Plug
- Manufacturer: `NPLG` (0x4E504C47) = NIH-Plug

### Entitlements Configuration

**File**: src/wrapper/auv3/swift/NIHPlugAUv3/NIHPlugAUv3.entitlements

```xml
<key>com.apple.security.app-sandbox</key>
<true/>
<key>com.apple.security.get-task-allow</key>
<true/>
```

**Applied To**:
1. Binary: `.appex/Contents/MacOS/NIHPlugAUv3`
2. Bundle: `.appex`
3. Host app: `.app`

### Code Signing Status

**Signing Type**: Ad-hoc (no developer certificate)
**Entitlements**: Properly embedded
**Architectures**: Universal binary (x86_64 + arm64)

## Open Questions

1. **Has the plugin been tested in Logic Pro?**
   - Current evidence: NO
   - All handoffs say "awaiting Logic Pro testing"
   - This is the critical missing step

2. **Why doesn't AVAudioUnitComponentManager find the plugin?**
   - Possible answer: This API may not be what Logic Pro uses
   - Evidence: FilterDemo also failed this test but worked in Logic Pro
   - Conclusion: Programmatic test may be unreliable

3. **Is Fix 4 (Framework Architecture) necessary?**
   - Unknown until Logic Pro testing is performed
   - Should only implement if Fixes 1-3 are confirmed insufficient

4. **Are there any Console.app errors when Logic Pro scans for plugins?**
   - Unknown - Console.app should be monitored during Logic Pro plugin scan
   - View controller logs would confirm if Logic Pro attempts to load the plugin

## Recommendations

### IMMEDIATE ACTION (Highest Priority)

**1. Test in Logic Pro NOW**
- Launch Logic Pro
- Open or create a project
- Navigate to Audio FX menu
- Look for "NIH-Plug AUv3" or entries under "NPLG" manufacturer
- **DO NOT IMPLEMENT ANY MORE FIXES until this test is performed**

**2. Monitor Console.app During Test**
```bash
# Open Console.app
# Filter by: "nihplug" OR "nplug" OR "NIHPlug"
# Watch for view controller log messages
# Look for any error messages during plugin scan
```

**3. Document Results**
- If plugin appears: **SUCCESS** - Document which fixes worked
- If plugin doesn't appear: Document exact behavior and any Console.app errors

### If Plugin Appears in Logic Pro

**Actions**:
1. Mark Fixes 1-3 as successful
2. Test plugin functionality (can it be instantiated? does UI appear? does audio process?)
3. Update fix_plan.md
4. Consider whether Fix 4 (framework architecture) is needed for robustness
5. Proceed to Phase 8 (Automated Validation with pluginval)
6. Begin Phase 9 (Polish & Testing in multiple DAWs)

### If Plugin Does NOT Appear in Logic Pro

**Before implementing Fix 4**:
1. Check Console.app for ANY errors or warnings
2. Verify plugin shows in System Preferences → Extensions
3. Try resetting PlugInKit cache: `pluginkit -r`
4. Try rebooting the system (PlugInKit cache may need refresh)
5. Check if Logic Pro has its own plugin blacklist or cache

**If still not appearing, implement Fix 4**:
- Create separate framework architecture (NIHPlugAUv3Framework.framework)
- Add AudioComponentBundle key to Info.plist
- This is a major refactor (estimated 2-3 iterations)
- See fix_plan.md lines 209-238 for detailed steps

### Additional Debugging Steps

**1. Check System Preferences**
```bash
# Open System Preferences → Extensions
# Look for NIHPlugAUv3 in the list
# Verify it's enabled
```

**2. Reset Audio Component Cache**
```bash
# Kill audio component registrar
killall -9 AudioComponentRegistrar

# Reset pluginkit cache
pluginkit -r

# Restart CoreAudio (requires sudo)
sudo killall coreaudiod
```

**3. Check for Gatekeeper Issues**
```bash
# Check if app is quarantined
xattr -l /Applications/gain.app

# Remove quarantine if present
sudo xattr -d com.apple.quarantine /Applications/gain.app
```

**4. Verbose PlugInKit Info**
```bash
# Get detailed info about the plugin
pluginkit -v -m -D -i com.nihplug.auv3
```

## Code References

### Key Implementation Files

- `src/wrapper/auv3/swift/NIHPlugAUv3/Info.plist:25-57` - Extension configuration with all three fixes
- `src/wrapper/auv3/swift/NIHPlugAUv3/NIHPlugAUv3ViewController.swift:14-99` - View controller factory
- `src/wrapper/auv3/swift/NIHPlugAUv3/AUAudioUnit.swift:14-203` - Audio Unit implementation
- `src/wrapper/auv3/swift/NIHPlugAUv3/NIHPlugAUv3.entitlements:1-9` - Sandbox entitlements
- `nih_plug_xtask/src/lib.rs:1111-1174` - Code signing with entitlements
- `nih_plug_xtask/src/lib.rs:1406-1489` - AUv3 build process

### Testing and Verification

- `test_auv3_discovery.swift:1-77` - AVAudioUnitComponentManager test
- `test_auv3_plugin_loading.swift:1-73` - Plugin loading test
- `handoffs/iteration_049.md:140-185` - Testing workflow documentation

## Related Documentation

### Handoff Documents
- `handoffs/iteration_046.md` - Discovery research (4,700+ words)
- `handoffs/iteration_047.md` - Fix 1 implementation
- `handoffs/iteration_048.md` - Fix 2 implementation
- `handoffs/iteration_049.md` - Fix 3 implementation

### Plans and Specifications
- `fix_plan.md` - Complete fix plan with all phases
- `AUV3_DEPLOYMENT_ISSUE.md` - Deployment issue documentation
- `src/wrapper/auv3/swift/README.md` - AUv3 wrapper documentation

### Research Documents
- `handoffs/auv3_bundle_structure_research.md` - Bundle structure requirements
- `handoffs/auv3_architecture_summary.md` - Architecture overview

## Conclusion

**The plugin is ready for Logic Pro testing.** All three documented fixes have been successfully implemented:
1. ✅ Info.plist structure matches Apple's FilterDemo
2. ✅ App sandbox entitlement is applied
3. ✅ UI extension point with view controller is implemented

The fact that AVAudioUnitComponentManager can't find the plugin is **not necessarily a failure indicator**. Research shows that Apple's own FilterDemo sample had the same behavior in programmatic tests but worked correctly in Logic Pro.

**CRITICAL NEXT STEP**: Test the plugin in Logic Pro. This is the only way to determine if the fixes are sufficient or if Fix 4 (framework architecture) is needed.

**DO NOT IMPLEMENT FIX 4** until Logic Pro testing confirms it's necessary. The current implementation may already be working - it just hasn't been tested in the actual target environment.

## Success Criteria

- [ ] Plugin tested in Logic Pro (IMMEDIATE PRIORITY)
- [ ] Plugin appearance in Logic Pro documented (success or failure)
- [ ] Console.app logs analyzed during plugin scan
- [ ] If successful: Document which fixes were sufficient
- [ ] If unsuccessful: Document exact failure mode and proceed to Fix 4

## Timeline and Next Actions

**Immediate** (5 minutes):
1. Open Logic Pro
2. Navigate to Audio FX menu
3. Search for "NIH-Plug AUv3" or "NPLG"
4. Document whether plugin appears

**If Successful** (30 minutes):
1. Test plugin functionality
2. Update iteration handoff
3. Update fix_plan.md
4. Plan next phase (validation)

**If Unsuccessful** (2-3 iterations):
1. Analyze Console.app logs
2. Try debugging steps above
3. If still failing, implement Fix 4 (framework architecture)
4. This requires significant refactoring of build system

---

**Bottom Line**: You're stuck because you haven't tested in Logic Pro yet. All the code is implemented. The next step is manual testing in the actual DAW, not more programmatic tests or more fixes.
