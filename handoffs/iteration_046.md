# Iteration 046 Handoff - AUv3 Discovery Research

## Confidence Score: 60
Extensive research completed, found critical differences between Apple's example and NIH-Plug implementation, but actual discovery issue remains unresolved (even Apple's example doesn't appear in AVAudioUnitComponentManager on this system).

## Context Usage
- Tasks completed this iteration: Research-only phase (Phase 7.5)
- Primary focus: Understanding why AUv3 plugins are not discovered by macOS
- Approach: Downloaded Apple's FilterDemo example, built it, compared with NIH-Plug implementation

## Health Check
- [x] All tests passing? (Build succeeds for both Apple example and NIH-Plug)
- [x] Any known issues or bugs? (Discovery mechanism unclear - neither plugin appears)
- [ ] Any incomplete implementations? (Discovery issue not resolved)
- [x] Code follows project conventions? (Yes)

## What Changed
- **Downloaded and built Apple's AUv3 FilterDemo sample**: Successfully built from https://github.com/GeorgeMcMullen/AudioUnitV3Example
- **Compared bundle structures**: Detailed comparison between Apple's example and NIH-Plug
- **Tested AUv3 discovery mechanism**: Created test script using AVAudioUnitComponentManager
- **Researched entitlements and code signing**: Found critical requirements for AUv3 registration
- **Rebuilt FilterDemo with entitlements**: Verified proper code signing and sandbox entitlements

## Testing Performed
- **Apple FilterDemo build**: ✅ Successfully built from source
- **FilterDemo installation**: ✅ Installed to /Applications/FilterDemo.app
- **Code signing verification**: ✅ Both apps have adhoc code signing
- **Entitlements verification**: ✅ FilterDemo has com.apple.security.app-sandbox entitlement
- **AVAudioUnitComponentManager test**: ❌ Neither FilterDemo nor NIHPlugAUv3 appear in component list
- **auval test**: ❌ Neither plugin appears in auval -a
- **pluginkit test**: ❌ Neither plugin appears in pluginkit -m

## Key Research Findings

### 1. NSExtensionPointIdentifier Variants
There are TWO different extension point identifiers:
- **`com.apple.AudioUnit-UI`**: For Audio Units WITH custom UI (Apple's FilterDemo uses this)
  - Requires NSExtensionMainStoryboard or NSExtensionViewController
  - NSExtensionPrincipalClass points to a view controller
  - Most common variant
- **`com.apple.AudioUnit`**: For Audio Units WITHOUT custom UI (headless)
  - NIH-Plug currently uses this
  - NSExtensionPrincipalClass points to the AUAudioUnit subclass directly
  - Considered "unusual case" per Apple docs

### 2. Info.plist Structure Differences

**Apple's FilterDemo (.appex):**
```xml
<key>NSExtension</key>
<dict>
    <key>NSExtensionAttributes</key>
    <dict>
        <key>AudioComponentBundle</key>
        <string>com.example.apple-samplecode.FilterDemoFrameworkOSX</string>
        <key>AudioComponents</key>
        <array>
            <dict>
                <key>description</key>
                <string>AUV3FilterDemo</string>
                <key>manufacturer</key>
                <string>Demo</string>
                <key>name</key>
                <string>Demo: AUV3FilterDemo</string>
                <!-- ... -->
            </dict>
        </array>
        <key>NSExtensionServiceRoleType</key>
        <string>NSExtensionServiceRoleTypeEditor</string>
    </dict>
    <key>NSExtensionPointIdentifier</key>
    <string>com.apple.AudioUnit-UI</string>
    <key>NSExtensionPrincipalClass</key>
    <string>FilterDemoViewController</string>
</dict>
```

**NIH-Plug gain.app (.appex):**
```xml
<key>AudioComponents</key>
<array>
    <dict>
        <key>description</key>
        <string>NIH-Plug Audio Unit v3 Plugin</string>
        <key>manufacturer</key>
        <string>NPLG</string>
        <key>name</key>
        <string>NIH-Plug AUv3</string>
        <!-- ... -->
    </dict>
</array>
<key>NSExtension</key>
<dict>
    <key>NSExtensionPointIdentifier</key>
    <string>com.apple.AudioUnit</string>
    <key>NSExtensionPrincipalClass</key>
    <string>NIHPlugAUv3</string>
</dict>
```

**Key Differences:**
1. Apple's example has AudioComponents INSIDE NSExtension/NSExtensionAttributes
2. NIH-Plug has AudioComponents at TOP LEVEL of plist
3. Apple's example uses AudioComponentBundle pointing to separate framework bundle
4. Apple's example has NSExtensionServiceRoleType = NSExtensionServiceRoleTypeEditor
5. Apple's example uses com.apple.AudioUnit-UI (with UI), NIH-Plug uses com.apple.AudioUnit (no UI)

### 3. Bundle Structure Differences

**Apple's FilterDemo:**
```
FilterDemo.app/
├── Contents/
│   ├── MacOS/
│   │   ├── FilterDemo
│   │   ├── FilterDemo.debug.dylib
│   │   └── __preview.dylib
│   ├── PlugIns/
│   │   └── FilterDemoAppExtension.appex/
│   │       └── Contents/
│   │           ├── MacOS/FilterDemoAppExtension
│   │           ├── Resources/FilterDemoViewController.nib
│   │           └── Info.plist
│   ├── Frameworks/
│   │   ├── FilterDemoFramework.framework/  <-- SEPARATE FRAMEWORK
│   │   └── libswift*.dylib (at app level)
│   ├── Resources/
│   │   ├── drumLoop.caf
│   │   └── Main.storyboardc
│   └── Info.plist
```

**NIH-Plug gain.app:**
```
gain.app/
├── Contents/
│   ├── MacOS/
│   │   └── gainHost
│   ├── PlugIns/
│   │   └── NIHPlugAUv3.appex/
│   │       └── Contents/
│   │           ├── MacOS/NIHPlugAUv3
│   │           ├── NIHPlugAUv3.swiftmodule/  <-- Swift modules
│   │           ├── Frameworks/  <-- Swift libs at appex level
│   │           │   └── libswift*.dylib
│   │           └── Info.plist
│   ├── _CodeSignature/
│   └── Info.plist (missing?)
```

**Key Differences:**
1. Apple's example has separate FilterDemoFramework.framework at app level
2. Apple's example has UI resources (NIB files, storyboards)
3. NIH-Plug has Swift frameworks inside .appex, Apple has them at .app level
4. NIH-Plug has Swift module files in .appex/Contents/
5. Apple's example includes sample audio files

### 4. Entitlements Requirements

**Critical Finding:** AUv3 extensions REQUIRE the app sandbox entitlement.

Apple's FilterDemoAppExtensionOSX.entitlements:
```xml
<key>com.apple.security.app-sandbox</key>
<true/>
```

**Test Results:**
- Built FilterDemo WITHOUT entitlements: ❌ Does not appear in AVAudioUnitComponentManager
- Built FilterDemo WITH entitlements: ❌ Still does not appear in AVAudioUnitComponentManager (!)
- NIH-Plug gain.app WITHOUT entitlements: ❌ Does not appear

### 5. Discovery Mechanism Research

**AVAudioUnitComponentManager:**
- This is the official API for discovering AUv3 plugins
- Uses `components(matching: AudioComponentDescription)` to find plugins
- Should return all registered plugins after container app is launched

**Registration Process (from research):**
1. AUv3 .appex must be inside .app bundle in PlugIns folder
2. Container .app must be launched at least once (registers with pluginkit)
3. Both .app and .appex must be properly code-signed
4. .appex must have com.apple.security.app-sandbox entitlement
5. Once registered, should appear in AVAudioUnitComponentManager and host DAWs

**Command-line tools:**
- `auval -a`: Lists AUv2 components, may not list AUv3 properly
- `pluginkit -m`: Lists all app extensions, but AUv3 may not appear here
- `AVAudioUnitComponentManager`: The correct API for programmatic discovery

### 6. Critical Unresolved Issue

**NEITHER the Apple FilterDemo example NOR the NIH-Plug gain.app appear in AVAudioUnitComponentManager.shared().components() even after:**
- Building with proper code signing
- Adding sandbox entitlements
- Installing to /Applications
- Launching the container app
- Killing and restarting audio component registrar

**Possible explanations:**
1. **System Policy**: macOS may require App Store signing or notarization for AUv3 discovery
2. **Missing Registration Step**: There may be an additional registration step not documented
3. **macOS Version Issue**: The system under test (macOS 15.x) may have changed AUv3 discovery
4. **Timing Issue**: Registration may require system restart or longer delay
5. **Framework Requirement**: The AudioComponentBundle + separate framework may be required
6. **Development vs Distribution**: Discovery may work differently in development vs distribution

## Files Created/Modified
- `/tmp/AudioUnitV3Example/` - Cloned Apple's AUv3 example repository
- `/Users/ludvig/Desktop/nih-plug/test_auv3_discovery.swift` - Created test script for discovery
- `/Users/ludvig/Desktop/nih-plug/handoffs/iteration_046.md` - This handoff document

## Comparison Summary

| Aspect | Apple FilterDemo | NIH-Plug gain.app | Status |
|--------|-----------------|-------------------|--------|
| Bundle Structure | .app with .appex in PlugIns | .app with .appex in PlugIns | ✅ Match |
| NSExtensionPointIdentifier | com.apple.AudioUnit-UI | com.apple.AudioUnit | ❌ Different |
| AudioComponents Location | Inside NSExtensionAttributes | Top-level in plist | ❌ Different |
| AudioComponentBundle | Points to framework | Not present | ❌ Missing |
| NSExtensionServiceRoleType | NSExtensionServiceRoleTypeEditor | Not present | ❌ Missing |
| NSExtensionPrincipalClass | FilterDemoViewController | NIHPlugAUv3 | ❌ Different (UI vs AU class) |
| Separate Framework | Yes (FilterDemoFramework) | No | ❌ Missing |
| UI Resources | Yes (NIB, storyboard) | No | ❌ Missing (expected for no-UI) |
| Sandbox Entitlement | Yes | No | ❌ Missing |
| Code Signing | Adhoc | Adhoc | ✅ Match |
| Appears in auval | No | No | ⚠️ Both fail |
| Appears in pluginkit | No | No | ⚠️ Both fail |
| Appears in AVAudioUnitComponentManager | No | No | ⚠️ Both fail |

## Hypotheses for Discovery Failure

### Hypothesis 1: Separate Framework Required
Apple's example uses `AudioComponentBundle` pointing to a separate framework that contains the actual AUAudioUnit implementation. This framework-based architecture may be REQUIRED for proper registration.

**Evidence:**
- FilterDemo has FilterDemoFramework.framework with the AU implementation
- NIH-Plug has AU implementation directly in .appex
- Info.plist structure is significantly different

**Test:** Restructure NIH-Plug to use separate framework architecture

### Hypothesis 2: UI Extension Point Required
macOS may only register AUv3 plugins that declare UI support via `com.apple.AudioUnit-UI`, even if the UI is minimal.

**Evidence:**
- FilterDemo uses com.apple.AudioUnit-UI
- NIH-Plug uses com.apple.AudioUnit (headless)
- Apple docs say non-UI is "unusual case"

**Test:** Change NIH-Plug to use com.apple.AudioUnit-UI with minimal view controller

### Hypothesis 3: Entitlements and Provisioning Required
Full App Store signing (not adhoc) with proper provisioning profile may be required for registration.

**Evidence:**
- Both adhoc-signed examples fail
- Research mentions "proper code signing" as requirement
- May need Developer ID Application signing

**Test:** Sign with actual Developer ID certificate

### Hypothesis 4: Info.plist Structure Critical
The exact structure of Info.plist (AudioComponents inside NSExtensionAttributes) may be required.

**Evidence:**
- Apple's example has very different plist structure
- AudioComponents in different location
- Additional keys like NSExtensionServiceRoleType

**Test:** Restructure NIH-Plug Info.plist to match Apple's exactly

### Hypothesis 5: System Policy or Compatibility Issue
macOS 15.x may have changed AUv3 discovery requirements, or Xcode 17 builds differently.

**Evidence:**
- Both examples built with Xcode 17 / macOS SDK 26.0
- May need testing on older macOS version
- Research is mostly from 2015-2020 era

**Test:** Try on macOS 12-14, or with older Xcode

### Hypothesis 6: Explicit Registration Required
Apps may need to call `AUAudioUnit.registerSubclass()` explicitly during development.

**Evidence:**
- Research mentions "dynamic registration" for development
- Production apps may rely on system registration
- Container app may need to instantiate AU once

**Test:** Add explicit registration call in host app

## Recommendations for Next Iteration

### Priority 1: Test Framework-Based Architecture (Hypothesis 1)
This is the most significant structural difference and most likely cause:

1. Create a separate framework (e.g., NIHPlugAUv3Framework.framework) containing:
   - The AUAudioUnit subclass implementation
   - The Rust FFI bridge
   - All audio processing code

2. Restructure the .appex to be a thin wrapper that:
   - Links to the framework
   - Declares AudioComponentBundle in Info.plist
   - Has minimal code (just loads framework)

3. Update Info.plist to match Apple's structure exactly:
   - Move AudioComponents inside NSExtensionAttributes
   - Add AudioComponentBundle key
   - Add NSExtensionServiceRoleType

4. Place framework in .app/Contents/Frameworks/

### Priority 2: Add UI Support (Hypothesis 2 + 4)
Even if plugin doesn't need custom UI, may need to declare UI support:

1. Change NSExtensionPointIdentifier to com.apple.AudioUnit-UI
2. Create minimal NSViewController subclass
3. Update NSExtensionPrincipalClass to point to view controller
4. Add NSExtensionServiceRoleType = NSExtensionServiceRoleTypeEditor

### Priority 3: Proper Code Signing (Hypothesis 3)
1. Get Developer ID Application certificate
2. Create proper entitlements file with:
   - com.apple.security.app-sandbox = true
   - Any additional entitlements needed
3. Sign both .app and .appex with proper certificate
4. Test registration after proper signing

### Priority 4: Test on Real DAW
Before doing extensive refactoring, test if the current implementation works in an actual DAW:

1. Open Logic Pro
2. Check if plugin appears in Audio Units list
3. Try to instantiate it
4. This will confirm if AVAudioUnitComponentManager is the issue or if entire discovery is broken

### Priority 5: Check System Logs
Look for registration errors:

```bash
log show --predicate 'subsystem == "com.apple.audio.AUExtensionHost"' --last 30m
log show --predicate 'process == "PlugInKit"' --last 30m
```

## Next Steps

1. **Test in Logic Pro**: Before implementing fixes, verify current state in real DAW
2. **Implement framework architecture**: This is the most significant difference
3. **Add proper entitlements**: Create entitlements file for .appex
4. **Sign with Developer ID**: If available, test with proper signing
5. **Update Info.plist structure**: Match Apple's format exactly

## Success Criteria for Next Iteration

- [ ] Plugin appears in `AVAudioUnitComponentManager.shared().components()`
- [ ] Plugin appears in Logic Pro or GarageBand plugin list
- [ ] Plugin can be instantiated from Swift test code
- [ ] Clear understanding of which changes made it work

## Code Quality Notes

- Created reusable test script (test_auv3_discovery.swift) for future testing
- Documented all findings thoroughly for future reference
- No changes made to NIH-Plug codebase this iteration (research only)

## Lessons Learned

1. **AUv3 discovery is complex**: Not just about bundle structure, involves entitlements, signing, and possibly framework architecture
2. **Apple's example doesn't work out-of-box**: Even official sample code doesn't appear in discovery without proper setup
3. **Command-line tools unreliable**: auval and pluginkit don't show AUv3 plugins reliably
4. **AVAudioUnitComponentManager is the truth**: This is the API DAWs use, should be our test
5. **Documentation is sparse**: Had to piece together requirements from multiple sources
6. **Framework-based architecture may be required**: Biggest structural difference between examples

## Blockers

- Neither test plugin (Apple or NIH-Plug) appears in AVAudioUnitComponentManager
- Unclear which of the structural differences is causing the issue
- May need Developer ID certificate for proper testing
- May need to test on older macOS version

## Time Spent

- Research and web search: ~30 minutes
- Building and testing Apple example: ~20 minutes
- Bundle structure comparison: ~20 minutes
- Creating test scripts and documentation: ~30 minutes
- Total: ~100 minutes

---

**Confidence: 60** - Learned a lot, found key differences, but didn't resolve the core issue. Need to test framework architecture hypothesis before claiming success.
