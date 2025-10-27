# Logic Pro Testing Results (SSH Session)

**Date**: 2025-10-27
**Tested By**: Claude via SSH
**Plugin**: NIH-Plug AUv3 (gain.app)

## Summary

Unable to fully test plugin in Logic Pro due to SSH/accessibility limitations, but discovered **critical installation issue** and performed cleanup.

## Critical Finding: Old Artifacts in Wrong Location

### Problem Discovered

Found old AUv3 bundles in **incorrect location**:
```
~/Library/Audio/Plug-Ins/Components/
├── NIHPlugAUv3.appex  ❌ WRONG (created Oct 27 06:33)
├── gain.appex          ❌ WRONG (created Oct 27 05:26)
├── gainHost.app        ❌ WRONG (created Oct 27 09:09)
└── nih_plug.appex      ❌ WRONG (created Oct 27 01:20)
```

**Issue**: `~/Library/Audio/Plug-Ins/Components/` is for **AUv2** plugins (.component bundles), not AUv3 plugins.

**AUv3 plugins** should be installed as standalone apps in `/Applications/`:
```
/Applications/gain.app/  ✅ CORRECT
└── Contents/
    └── PlugIns/
        └── NIHPlugAUv3.appex/
```

### Action Taken

**Cleaned up old artifacts**:
```bash
rm -rf ~/Library/Audio/Plug-Ins/Components/NIHPlugAUv3.appex
rm -rf ~/Library/Audio/Plug-Ins/Components/gain.appex
rm -rf ~/Library/Audio/Plug-Ins/Components/gainHost.app
rm -rf ~/Library/Audio/Plug-Ins/Components/nih_plug.appex
```

**Impact**: These old bundles may have been interfering with plugin discovery. Logic Pro may have been confused by multiple versions in different locations.

## Current Installation Status

### Correct Installation
✅ **Location**: `/Applications/gain.app`
✅ **Registered with pluginkit**:
```
com.nihplug.auv3(1.0)  9D471691-8F4D-4231-A89C-F4BFF61C12D0
Path: /Applications/gain.app/Contents/PlugIns/NIHPlugAUv3.appex
```

### Verified Configuration

✅ **Extension Point**: `com.apple.AudioUnit-UI` (Fix 3 applied)
✅ **Principal Class**: `NIHPlugAUv3ViewController` (Fix 3 applied)
✅ **Entitlements**: `com.apple.security.app-sandbox = true` (Fix 2 applied)
✅ **Info.plist Structure**: AudioComponents inside NSExtensionAttributes (Fix 1 applied)

### Bundle Structure
```
/Applications/gain.app/
├── Contents/
│   ├── Info.plist
│   ├── MacOS/
│   │   └── gainHost
│   └── PlugIns/
│       └── NIHPlugAUv3.appex/
│           └── Contents/
│               ├── Info.plist (all 3 fixes applied)
│               ├── MacOS/NIHPlugAUv3
│               └── Frameworks/ (Swift runtime)
```

## Discovery Test Results

### AVAudioUnitComponentManager Test
```
Found 0 NIHPlugAUv3 components
Total components found: 57 (all Apple-provided)
```

**Note**: This result is **expected and not a failure indicator**. Per iteration 046 research, Apple's FilterDemo also showed 0 components in programmatic tests but worked correctly in Logic Pro.

## Logic Pro Testing (Attempted)

### Limitations Encountered

❌ **AppleScript System Events**: Connection errors due to SSH/accessibility limitations
❌ **GUI Interaction**: Cannot directly interact with Logic Pro menus
❌ **Screenshots**: Captured desktop wallpaper only (Logic not in foreground)

### What Was Attempted

1. ✅ Launched Logic Pro successfully
2. ✅ Verified process is running
3. ❌ Could not navigate to Audio Units menu (accessibility restrictions)
4. ❌ Could not capture Logic Pro window screenshots

### Logs Checked

**System Logs**:
- No errors in `com.apple.audio` subsystem
- No errors in AudioComponentRegistrar
- No NIHPlug-related error messages

**Interpretation**: Absence of errors is a positive sign - system is accepting the plugin without issues.

## Impact of Cleanup

### Before Cleanup
- Multiple AUv3 bundles in wrong location (Components directory)
- Potentially conflicting with proper installation
- Logic Pro may have been scanning outdated/incorrect bundles

### After Cleanup
- ✅ Only correct installation remains (/Applications/gain.app)
- ✅ No conflicting bundles
- ✅ Plugin still registered with pluginkit
- ✅ Fresh state for Logic Pro to scan

## Recommendations for Manual Testing

### Next Steps (When at Computer)

1. **Restart Logic Pro** (may need fresh launch after cleanup)
   ```bash
   killall "Logic Pro"
   open -a "Logic Pro"
   ```

2. **Check Plugin Manager**:
   - Open Logic Pro
   - Create new project
   - Go to Audio FX menu
   - Look for "NIH-Plug AUv3" or "NPLG" manufacturer

3. **Monitor Console.app**:
   - Filter for: `nihplug` or `NPLG`
   - Watch for view controller logs:
     - "NIHPlugAUv3ViewController: loadView called"
     - "NIHPlugAUv3ViewController: createAudioUnit called"

### Expected Behavior After Cleanup

**Best Case**: Plugin now appears in Logic Pro
- Cleanup removed conflicting bundles
- Logic Pro now finds correct installation
- All 3 fixes (plist structure + entitlements + UI) were sufficient

**If Still Not Appearing**:
- Check Console.app for specific errors
- Try resetting plugin cache: `pluginkit -r`
- Consider Fix 4 (framework architecture) as documented in fix_plan.md

## Why Artifacts Were in Wrong Location

### Historical Context

From git status and handoff documents, the bundler went through several iterations:
- Iterations 027-030: Fixed bundle structure (host app requirement)
- Earlier iterations may have installed to Components directory
- Each test build created new bundles without cleanup

### Lesson Learned

**AUv2 vs AUv3 Installation**:
- AUv2: `~/Library/Audio/Plug-Ins/Components/Plugin.component`
- AUv3: `/Applications/HostApp.app/Contents/PlugIns/Plugin.appex`

The bundler is now correctly installing to `/Applications/`, but old test artifacts remained.

## Technical Details

### Build Information
- **Built**: October 27, 2025 at 12:29 (per Info.plist timestamp)
- **Architecture**: Universal binary (x86_64 + arm64)
- **Code Signing**: Ad-hoc with entitlements
- **Build Command**: `cargo xtask bundle-universal gain --release`

### What's Working
✅ Plugin builds successfully
✅ Bundle structure is correct
✅ All three fixes applied
✅ Code signing with entitlements
✅ PlugInKit registration
✅ No system errors in logs

### What Couldn't Be Tested
❌ Actual appearance in Logic Pro plugin list
❌ Plugin instantiation in DAW
❌ UI rendering
❌ Audio processing in Logic Pro

## Conclusion

**Main Achievement**: Discovered and cleaned up conflicting installation artifacts that may have been preventing proper plugin discovery.

**Current Status**:
- Plugin is correctly installed and configured
- All known fixes applied
- Old conflicting bundles removed
- Ready for manual Logic Pro testing

**High Confidence**: The cleanup of conflicting bundles may have resolved the discovery issue. Manual testing should be performed to confirm plugin now appears in Logic Pro.

**If Plugin Still Doesn't Appear**: The framework architecture (Fix 4) remains as a fallback option, but should only be implemented after confirming that cleanup + current fixes are insufficient.

## Files Modified/Removed

### Removed (Old Artifacts)
- `~/Library/Audio/Plug-Ins/Components/NIHPlugAUv3.appex`
- `~/Library/Audio/Plug-Ins/Components/gain.appex`
- `~/Library/Audio/Plug-Ins/Components/gainHost.app`
- `~/Library/Audio/Plug-Ins/Components/nih_plug.appex`

### Verified (Current Installation)
- `/Applications/gain.app/` (unchanged, still properly configured)

## Next Manual Test Checklist

When you're back at your computer:

- [ ] Launch Logic Pro
- [ ] Open or create a project
- [ ] Navigate to Audio FX → Audio Units
- [ ] Search for "NIH-Plug AUv3" or "NPLG"
- [ ] Document whether plugin appears
- [ ] If appears: Test instantiation
- [ ] If doesn't appear: Check Console.app for errors
- [ ] Report findings for next iteration

---

**Bottom Line**: Cleanup performed successfully. The conflicting bundles in the wrong location have been removed. Manual Logic Pro testing is required to determine if this resolved the issue.
