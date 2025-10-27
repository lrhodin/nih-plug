# Iteration 048 Handoff - Fix 2 Implemented (App Sandbox Entitlement)

## Confidence Score: 75
Fix 2 implemented successfully. Plugin rebuilt and installed with app sandbox entitlement applied to both .app and .appex bundles. Entitlements verified in signed binaries. Ready for Logic Pro testing to determine if Fixes 1+2 resolve the discovery issue.

## Context Usage
- Tasks completed this iteration: Implementation of Fix 2 from Phase 7.6
- Primary focus: Adding app sandbox entitlement to AUv3 bundles
- Approach: Modified bundler to sign with entitlements, fixed .swiftmodule signing issue

## Health Check
- [x] All tests passing? (Build succeeds, plugin installed successfully)
- [x] Any known issues or bugs? (None - Fixes 1+2 applied cleanly)
- [ ] Any incomplete implementations? (Fixes 3-4 not yet implemented - awaiting Logic Pro test results)
- [x] Code follows project conventions? (Yes)

## What Changed

### Files Modified

**1. `/Users/ludvig/Desktop/nih-plug/src/wrapper/auv3/swift/NIHPlugAUv3/NIHPlugAUv3.entitlements`**
   - Added `com.apple.security.app-sandbox = true` entitlement
   - Kept existing `com.apple.security.get-task-allow = true` (for debugging)

**2. `/Users/ludvig/Desktop/nih-plug/nih_plug_xtask/src/lib.rs`**
   - Added `maybe_codesign_with_entitlements()` function to support entitlements during code signing
   - Modified `maybe_codesign()` to call the new function with `None` for backward compatibility
   - Updated AUv3 bundler to sign both .app and .appex with entitlements file
   - Added fix to remove .swiftmodule directory before signing (prevents "bundle format unrecognized" errors)

## Entitlements Implementation Details

### Entitlements File Content
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>com.apple.security.app-sandbox</key>
	<true/>
	<key>com.apple.security.get-task-allow</key>
	<true/>
</dict>
</plist>
```

### Code Signing Flow
1. Xcode builds .appex with entitlements (configured in project settings)
2. Bundler copies .appex to host app bundle
3. **NEW:** Bundler removes .swiftmodule directory from .appex (prevents signing errors)
4. **NEW:** Bundler signs .appex binary with entitlements
5. **NEW:** Bundler signs .appex bundle with entitlements
6. **NEW:** Bundler signs host .app with entitlements

### Critical Fix: .swiftmodule Removal
The .swiftmodule directory was causing code signing to fail with "bundle format unrecognized, invalid, or unsuitable" errors. This directory is only needed at build time and causes issues when codesign tries to validate it as part of the bundle.

**Solution:** Remove the .swiftmodule directory before code signing. This matches Apple's approach (FilterDemo doesn't have .swiftmodule in the final bundle).

## Testing Performed

### Build Verification
- ✅ Built universal binary: `cargo xtask bundle-universal gain --release`
- ✅ Xcode build succeeded (no errors)
- ✅ Plugin installed to `/Applications/gain.app`
- ✅ Code signing completed without errors (no more "bundle format unrecognized" errors)

### Entitlements Verification
```bash
# Verified .appex entitlements
codesign -d --entitlements /tmp/appex_ent.xml /Applications/gain.app/Contents/PlugIns/NIHPlugAUv3.appex
```

**Output confirms:**
- `com.apple.security.app-sandbox = true` ✅
- `com.apple.security.get-task-allow = true` ✅

```bash
# Verified .app entitlements
codesign -d --entitlements /tmp/app_ent.xml /Applications/gain.app
```

**Output confirms:**
- `com.apple.security.app-sandbox = true` ✅
- `com.apple.security.get-task-allow = true` ✅

### Installation Verification
- ✅ Plugin installed to `/Applications/gain.app`
- ✅ Bundle structure correct: `gain.app/Contents/PlugIns/NIHPlugAUv3.appex`
- ✅ Both bundles signed with entitlements

## 🧪 CRITICAL: Logic Pro Testing Required

**STOP HERE - Human testing checkpoint reached.**

### Testing Instructions

1. **Quit and Restart Logic Pro** (if open)
   ```bash
   killall "Logic Pro" 2>/dev/null
   ```

2. **Open Logic Pro**

3. **Create or open a project**

4. **Check if plugin appears:**
   - Go to Audio FX menu
   - Look for "NIH-Plug AUv3" under Audio Units
   - Look in manufacturer category "NPLG"

5. **Report Results:**
   - ✅ **If plugin appears:** Fixes 1+2 SUCCESSFUL! Document findings and consider work done for this phase.
   - ❌ **If plugin does NOT appear:** Proceed to Fix 3 (Switch to UI Extension Point) in next iteration.

### What to Look For

**Success Indicators:**
- Plugin appears in Logic Pro Audio FX menu
- Can instantiate plugin (even if it has issues)
- Plugin listed under "NPLG" manufacturer or "Audio Units" category

**Failure Indicators:**
- Plugin does not appear in any menu
- No new entries in Audio FX list
- Same behavior as iterations 046-047

## Comparison with Apple FilterDemo

| Aspect | Apple FilterDemo | NIH-Plug (Before) | NIH-Plug (After 047) | NIH-Plug (After 048) |
|--------|-----------------|-------------------|----------------------|----------------------|
| AudioComponents Location | Inside NSExtensionAttributes | Top-level ❌ | Inside NSExtensionAttributes ✅ | Inside NSExtensionAttributes ✅ |
| NSExtensionServiceRoleType | Present | Missing ❌ | Present ✅ | Present ✅ |
| Sandbox Entitlement | Yes | No ❌ | No ❌ | Yes ✅ |
| NSExtensionPointIdentifier | com.apple.AudioUnit-UI | com.apple.AudioUnit | com.apple.AudioUnit | com.apple.AudioUnit |
| AudioComponentBundle | Points to framework | Not present | Not present | Not present |
| Separate Framework | Yes | No | No | No |

**Fixes 1+2 now implemented. If Logic Pro testing fails, Fixes 3-4 remain available.**

## Next Steps (Depending on Logic Pro Test Results)

### If Fixes 1+2 Work ✅
1. Update fix_plan.md to mark Fixes 1+2 as successful
2. Document findings for upstream PR
3. Consider whether to implement Fixes 3-4 for robustness
4. Proceed to Phase 8 (Automated Validation)

### If Fixes 1+2 Don't Work ❌
1. Proceed to Fix 3: Switch to UI Extension Point
   - Change NSExtensionPointIdentifier to `com.apple.AudioUnit-UI`
   - Create minimal NSViewController subclass
   - Update NSExtensionPrincipalClass
   - Rebuild and test in Logic Pro again

2. If Fix 3 doesn't work, proceed to Fix 4: Framework Architecture (major refactor)

## Files Created/Modified
- `/Users/ludvig/Desktop/nih-plug/src/wrapper/auv3/swift/NIHPlugAUv3/NIHPlugAUv3.entitlements` - Updated with app sandbox entitlement
- `/Users/ludvig/Desktop/nih-plug/nih_plug_xtask/src/lib.rs` - Added entitlements support and .swiftmodule removal
- `/Applications/gain.app/` - Rebuilt and reinstalled plugin with entitlements
- `/Users/ludvig/Desktop/nih-plug/handoffs/iteration_048.md` - This handoff document

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
│   │           ├── Info.plist  ← FIX 1 APPLIED
│   │           └── (no .swiftmodule - removed before signing)
│   └── Info.plist
```

**Code Signing:** Adhoc signing with entitlements applied ✅

**Architectures:** Universal binary (x86_64 + arm64)

**Entitlements Applied:**
- `com.apple.security.app-sandbox = true` ✅
- `com.apple.security.get-task-allow = true` ✅

## Success Criteria for This Iteration

- ✅ Fix 2 implemented correctly
- ✅ Entitlements file created with app sandbox entitlement
- ✅ Bundler updated to sign with entitlements
- ✅ .swiftmodule signing issue fixed
- ✅ Plugin builds successfully
- ✅ Plugin installed to /Applications/
- ✅ Entitlements verified in signed binaries
- ⏳ **Awaiting Logic Pro testing** (human checkpoint)

## Code Quality Notes

- Changes are focused and minimal (Fix 2 only)
- Added backward-compatible function for entitlements support
- Fixed .swiftmodule signing issue that was preventing proper code signing
- No breaking changes to existing code
- Both source entitlements file and bundler updated

## Technical Challenges Solved

1. **Code Signing Errors:** Initial attempt failed with "bundle format unrecognized" errors
   - **Root Cause:** .swiftmodule directory inside .appex bundle
   - **Solution:** Remove .swiftmodule before signing (matches Apple's approach)

2. **Entitlements Not Embedded:** First build didn't embed entitlements due to signing errors
   - **Root Cause:** Signing failed before entitlements could be applied
   - **Solution:** Fix .swiftmodule issue first, then entitlements work correctly

## Lessons Learned

1. **Code signing order matters:** Must fix bundle structure issues before entitlements can be applied
2. **.swiftmodule should not be in final bundle:** Only needed at build time, causes signing issues
3. **Verify entitlements after signing:** Use `codesign -d --entitlements` to confirm they're embedded
4. **Ad-hoc signing can use entitlements:** Don't need a developer certificate to test entitlements

## Blockers

- ⏳ Waiting for human Logic Pro testing to determine if Fixes 1+2 resolved the discovery issue

## Time Spent

- Understanding Fix 2 requirements: ~5 minutes
- Implementing entitlements support: ~15 minutes
- Debugging .swiftmodule signing issue: ~15 minutes
- Building and verifying: ~10 minutes
- Creating handoff documentation: ~15 minutes
- Total: ~60 minutes

---

**Confidence: 75** - Fix 2 implemented correctly based on FilterDemo comparison. App sandbox entitlement is now present in both .app and .appex bundles, matching Apple's working example. Moderate-high confidence this combined with Fix 1 will help with discovery, but requires Logic Pro testing to confirm.

**Next Human Action:** Test in Logic Pro and report whether plugin appears in Audio FX menu.
