# Iteration 049 Handoff - Fix 3 Implemented (UI Extension Point)

## Confidence Score: 80
Fix 3 implemented successfully. Plugin rebuilt with UI extension point (com.apple.AudioUnit-UI) and NSViewController. Built successfully, installed, and Info.plist verified. All three fixes (1+2+3) now applied. Ready for critical Logic Pro testing checkpoint.

## Context Usage
- Tasks completed this iteration: Implementation of Fix 3 from Phase 7.6
- Primary focus: Switching from headless to UI extension point
- Approach: Created NSViewController subclass, updated Info.plist, rebuilt plugin

## Health Check
- [x] All tests passing? (Build succeeds, plugin installed successfully)
- [x] Any known issues or bugs? (None - Fix 3 applied cleanly)
- [ ] Any incomplete implementations? (Fix 4 not yet implemented - awaiting Logic Pro test results)
- [x] Code follows project conventions? (Yes)

## What Changed

### Files Created

**1. `/Users/ludvig/Desktop/nih-plug/src/wrapper/auv3/swift/NIHPlugAUv3/NIHPlugAUv3ViewController.swift`**
   - Created new NSViewController subclass
   - Implements `AUViewController` (base class for Audio Unit view controllers)
   - Implements `AUAudioUnitFactory` protocol (creates AUAudioUnit instances)
   - Contains `createAudioUnit(with:)` method that instantiates NIHPlugAUv3
   - Loads a simple programmatic view with plugin name and status labels
   - Uses Auto Layout for UI positioning
   - Added `@available(macOS 10.13, *)` for optional view configuration method

### Files Modified

**2. `/Users/ludvig/Desktop/nih-plug/src/wrapper/auv3/swift/NIHPlugAUv3/Info.plist`**
   - Changed `NSExtensionPointIdentifier` from `com.apple.AudioUnit` to `com.apple.AudioUnit-UI`
   - Changed `NSExtensionPrincipalClass` from `NIHPlugAUv3` to `NIHPlugAUv3ViewController`
   - Kept all other keys unchanged (AudioComponents, NSExtensionServiceRoleType, etc.)

**3. `/Users/ludvig/Desktop/nih-plug/src/wrapper/auv3/swift/NIHPlugAUv3.xcodeproj/project.pbxproj`**
   - Added NIHPlugAUv3ViewController.swift to PBXFileReference section
   - Added NIHPlugAUv3ViewController.swift to PBXBuildFile section
   - Added NIHPlugAUv3ViewController.swift to PBXGroup (NIHPlugAUv3 group)
   - Added NIHPlugAUv3ViewController.swift to PBXSourcesBuildPhase for compilation
   - Generated unique UUIDs for Xcode project entries

## Fix 3 Implementation Details

### View Controller Architecture

The new architecture follows Apple's FilterDemo pattern:

```
Host DAW (Logic Pro)
    ↓
NIHPlugAUv3ViewController (NSViewController + AUAudioUnitFactory)
    ↓
NIHPlugAUv3 (AUAudioUnit subclass)
    ↓
Rust Plugin (via FFI)
```

**Key Changes:**
1. **Extension Point:** Changed from `com.apple.AudioUnit` (headless) to `com.apple.AudioUnit-UI` (with UI)
2. **Principal Class:** Changed from AUAudioUnit directly to ViewController
3. **Factory Pattern:** ViewController implements AUAudioUnitFactory and creates AUAudioUnit on demand

### View Controller Implementation

```swift
@objc public class NIHPlugAUv3ViewController: AUViewController, AUAudioUnitFactory {
    private var audioUnit: NIHPlugAUv3?

    // Creates the Audio Unit instance when requested by the host
    public func createAudioUnit(with componentDescription: AudioComponentDescription) throws -> AUAudioUnit {
        let audioUnit = try NIHPlugAUv3(componentDescription: componentDescription, options: [])
        self.audioUnit = audioUnit
        return audioUnit
    }

    // Loads a simple view programmatically
    public override func loadView() {
        // Creates containerView with labels showing plugin name
    }
}
```

**Design Decisions:**
- **Programmatic UI:** No NIB/Storyboard required - simpler for generic plugins
- **Simple View:** Just displays plugin name and status (can be enhanced later)
- **Auto Layout:** Uses constraints for proper layout
- **macOS 10.12 Compatible:** Main implementation works on 10.12, with optional features gated by @available

## Testing Performed

### Build Verification
- Built universal binary: `cargo xtask bundle-universal gain --release`
- Xcode build succeeded (no errors, only minor warnings)
- Plugin installed to `/Applications/gain.app`
- Code signing completed successfully

### Info.plist Verification

Verified with PlistBuddy commands:

```bash
# NSExtensionPointIdentifier
/usr/libexec/PlistBuddy -c "Print :NSExtension:NSExtensionPointIdentifier" \
  /Applications/gain.app/Contents/PlugIns/NIHPlugAUv3.appex/Contents/Info.plist
# Output: com.apple.AudioUnit-UI ✅
```

```bash
# NSExtensionPrincipalClass
/usr/libexec/PlistBuddy -c "Print :NSExtension:NSExtensionPrincipalClass" \
  /Applications/gain.app/Contents/PlugIns/NIHPlugAUv3.appex/Contents/Info.plist
# Output: NIHPlugAUv3ViewController ✅
```

```bash
# AudioComponents structure (Fix 1 from iteration 047)
/usr/libexec/PlistBuddy -c "Print :NSExtension:NSExtensionAttributes:AudioComponents:0" \
  /Applications/gain.app/Contents/PlugIns/NIHPlugAUv3.appex/Contents/Info.plist
# Output: Dict with type=aufx, subtype=Nplg, manufacturer=NPLG, hasCustomView=true ✅
```

### Installation Verification
- Plugin installed to `/Applications/gain.app`
- Bundle structure correct: `gain.app/Contents/PlugIns/NIHPlugAUv3.appex`
- Info.plist contains all three fixes (1+2+3)

## 🧪 CRITICAL: Logic Pro Testing Required

**STOP HERE - Human testing checkpoint reached.**

This is the most critical testing checkpoint. Fix 3 changes the fundamental discovery mechanism from headless to UI-based. This matches Apple's FilterDemo pattern more closely.

### Testing Instructions

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
   - ✅ **If plugin appears:** Fix 3 SUCCESSFUL! Document findings and consider Fixes 1+2+3 sufficient.
   - ❌ **If plugin does NOT appear:** Proceed to Fix 4 (Framework Architecture - major refactor) in next iteration.

### What to Look For

**Success Indicators:**
- Plugin appears in Logic Pro Audio FX menu
- Can instantiate plugin (window opens showing our simple UI)
- Plugin listed under "NPLG" manufacturer or "Audio Units" category
- View controller's print statements appear in Console.app

**Failure Indicators:**
- Plugin does not appear in any menu
- No new entries in Audio FX list
- Same behavior as iterations 046-048

### Debugging (if needed)

If plugin doesn't appear, check Console.app for errors:

```bash
# Open Console.app and filter for "NIHPlug" or "com.nihplug.auv3"
# Look for errors related to:
# - Extension loading
# - View controller instantiation
# - AUAudioUnit creation
```

## Comparison with Apple FilterDemo

| Aspect | Apple FilterDemo | NIH-Plug (After 049) | Status |
|--------|-----------------|----------------------|--------|
| AudioComponents Location | Inside NSExtensionAttributes | Inside NSExtensionAttributes | ✅ Match (Fix 1) |
| NSExtensionServiceRoleType | Present | Present | ✅ Match (Fix 1) |
| Sandbox Entitlement | Yes | Yes | ✅ Match (Fix 2) |
| NSExtensionPointIdentifier | com.apple.AudioUnit-UI | com.apple.AudioUnit-UI | ✅ Match (Fix 3) |
| NSExtensionPrincipalClass | ViewController | NIHPlugAUv3ViewController | ✅ Match (Fix 3) |
| View Controller | Yes (FilterDemoViewController) | Yes (NIHPlugAUv3ViewController) | ✅ Match (Fix 3) |
| AudioComponentBundle | Points to framework | Not present | ❌ Different (Fix 4) |
| Separate Framework | Yes | No | ❌ Different (Fix 4) |

**Fixes 1+2+3 now implemented. Only Fix 4 (Framework Architecture) remains if this doesn't work.**

## Next Steps (Depending on Logic Pro Test Results)

### If Fix 3 Works ✅
1. Update fix_plan.md to mark Fixes 1+2+3 as successful
2. Document findings for upstream PR
3. Consider whether Fix 4 is needed for robustness (probably not)
4. Proceed to Phase 8 (Automated Validation with pluginval)
5. Begin Phase 9 (Polish & Testing in multiple DAWs)

### If Fix 3 Doesn't Work ❌
1. **Proceed to Fix 4: Framework Architecture (MAJOR REFACTOR)**
   - This is the last structural fix available
   - Requires creating separate NIHPlugAUv3Framework.framework
   - Move AUAudioUnit to framework
   - Add AudioComponentBundle key to Info.plist
   - Update bundler to handle framework building/signing
   - This is a significant architectural change (estimated 2-3 iterations)

2. **If Fix 4 also doesn't work:**
   - Re-examine system logs more carefully
   - Consider testing on different macOS version
   - Reach out to Apple Developer Forums
   - Consider filing a DTS (Developer Technical Support) incident

## Files Created/Modified
- `/Users/ludvig/Desktop/nih-plug/src/wrapper/auv3/swift/NIHPlugAUv3/NIHPlugAUv3ViewController.swift` - NEW: View controller implementation
- `/Users/ludvig/Desktop/nih-plug/src/wrapper/auv3/swift/NIHPlugAUv3/Info.plist` - Updated with UI extension point
- `/Users/ludvig/Desktop/nih-plug/src/wrapper/auv3/swift/NIHPlugAUv3.xcodeproj/project.pbxproj` - Added view controller to project
- `/Applications/gain.app/` - Rebuilt and reinstalled plugin with Fix 3
- `/Users/ludvig/Desktop/nih-plug/handoffs/iteration_049.md` - This handoff document

## Plugin Installation Details

**Installed Location:** `/Applications/gain.app`

**Bundle Structure:**
```
/Applications/gain.app/
├── Contents/
│   ├── MacOS/
│   │   └── gainHost
│   ├── PlugIns/
│   │   └── NIHPlugAUv3.appex/
│   │       └── Contents/
│   │           ├── MacOS/NIHPlugAUv3
│   │           ├── Frameworks/ (Swift runtime libraries)
│   │           ├── Info.plist  ← FIXES 1+2+3 APPLIED
│   │           └── (no .swiftmodule - removed before signing)
│   └── Info.plist
```

**Code Signing:** Adhoc signing with entitlements applied

**Architectures:** Universal binary (x86_64 + arm64)

**Entitlements Applied:**
- `com.apple.security.app-sandbox = true` (Fix 2)
- `com.apple.security.get-task-allow = true` (Debug)

**Extension Configuration:**
- `NSExtensionPointIdentifier = com.apple.AudioUnit-UI` (Fix 3)
- `NSExtensionPrincipalClass = NIHPlugAUv3ViewController` (Fix 3)
- `NSExtensionServiceRoleType = NSExtensionServiceRoleTypeEditor` (Fix 1)
- `AudioComponents` inside `NSExtensionAttributes` (Fix 1)

## Success Criteria for This Iteration

- ✅ Fix 3 implemented correctly
- ✅ View controller created with AUAudioUnitFactory protocol
- ✅ Info.plist updated with UI extension point
- ✅ View controller added to Xcode project
- ✅ Plugin builds successfully
- ✅ Plugin installed to /Applications/
- ✅ Info.plist verified with correct values
- ⏳ **Awaiting Logic Pro testing** (human checkpoint)

## Code Quality Notes

- Changes are focused and follow Apple's FilterDemo pattern
- View controller implementation is clean and minimal
- No breaking changes to existing AUAudioUnit code
- Generic view can work with any plugin (shows plugin name from Info.plist)
- Proper error handling in view controller
- Added availability annotations for macOS version compatibility

## Technical Challenges Solved

1. **Xcode Project Update:** Successfully added Swift file to Xcode project by editing project.pbxproj
   - Generated unique UUIDs for Xcode entries
   - Added to all necessary sections (PBXBuildFile, PBXFileReference, PBXGroup, PBXSourcesBuildPhase)

2. **macOS Version Compatibility:** Fixed compilation error with AUAudioUnitViewConfiguration
   - Root Cause: Type only available in macOS 10.13+, project targets 10.12
   - Solution: Added @available(macOS 10.13, *) annotation to optional method

3. **View Controller Architecture:** Implemented proper AUv3 UI pattern
   - View controller acts as factory for AUAudioUnit
   - Follows Apple's pattern exactly (AUViewController + AUAudioUnitFactory)
   - Creates Audio Unit on demand when host requests it

## Lessons Learned

1. **Extension Point Types Matter:** The difference between `com.apple.AudioUnit` (headless) and `com.apple.AudioUnit-UI` (with UI) may be critical for discovery
2. **View Controllers Are Required:** Even for plugins without complex UI, a view controller may be required for macOS to register the plugin
3. **Factory Pattern:** AUv3 UI plugins use a factory pattern where the view controller creates the Audio Unit on demand
4. **Programmatic UI Is Simpler:** No need for NIB/Storyboard files - programmatic UI is easier to maintain

## Hypothesis for Fix 3

**Theory:** macOS plugin scanning may prioritize or exclusively register AUv3 plugins that declare UI support via `com.apple.AudioUnit-UI` extension point. The headless `com.apple.AudioUnit` may be ignored or require additional configuration.

**Evidence:**
- Apple's FilterDemo (which DOES appear in Logic Pro) uses `com.apple.AudioUnit-UI`
- NIH-Plug (which does NOT appear) was using `com.apple.AudioUnit`
- Apple's documentation suggests UI-based plugins are the primary AUv3 use case

**Expected Outcome:**
- Plugin should now appear in Logic Pro's Audio FX menu
- View controller will be instantiated when plugin is loaded
- Simple UI will appear showing plugin name

**If this doesn't work:** The issue is likely related to the framework architecture (Fix 4), not the extension point type.

## Blockers

- ⏳ Waiting for human Logic Pro testing to determine if Fix 3 resolved the discovery issue
- ⏳ Need confirmation before proceeding to Fix 4 (major refactor)

## Time Spent

- Understanding Fix 3 requirements: ~5 minutes
- Creating view controller implementation: ~20 minutes
- Updating Info.plist: ~5 minutes
- Adding file to Xcode project: ~15 minutes
- Fixing compilation errors: ~10 minutes
- Building and verifying: ~10 minutes
- Creating handoff documentation: ~25 minutes
- Total: ~90 minutes

---

**Confidence: 80** - Fix 3 implemented correctly following Apple's FilterDemo pattern. Changed from headless to UI extension point with proper view controller factory. High confidence this brings NIH-Plug closer to FilterDemo's working configuration. The UI extension point is a fundamental difference from our previous approach and matches Apple's sample code. Requires Logic Pro testing to confirm.

**Next Human Action:** Test in Logic Pro and report whether plugin appears in Audio FX menu. This is the critical test for Fix 3.

**Current Status:** All code changes complete. Plugin built, signed, and installed. Ready for human testing. DO NOT PROCEED to Fix 4 until Logic Pro test results are confirmed.
