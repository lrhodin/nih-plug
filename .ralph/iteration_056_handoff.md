# Iteration 056 - Plugin Renamed to Force Fresh Load

## Mission Complete

Successfully renamed the AUv3 plugin to force Logic Pro to treat it as a completely new plugin, bypassing any cached versions.

## Changes Made

### 1. Info.plist Updates (`/Users/ludvig/Desktop/nih-plug/src/wrapper/auv3/swift/NIHPlugAUv3/Info.plist`)

Changed the following fields to add "v2" suffix:
- **CFBundleDisplayName**: `NIH-Plug AUv3` → `NIH-Plug AUv3 v2`
- **subtype**: `Nplg` → `Npl2` (4-character Audio Unit subtype code)
- **name**: `NPLG: NIH-Plug AUv3` → `NPLG: NIH-Plug AUv3 v2`

### 2. Xcode Project Updates (`/Users/ludvig/Desktop/nih-plug/src/wrapper/auv3/swift/NIHPlugAUv3.xcodeproj/project.pbxproj`)

Changed bundle identifier in both Debug and Release configurations:
- **PRODUCT_BUNDLE_IDENTIFIER**: `com.nihplug.auv3` → `com.nihplug.auv3.v2`

### 3. Bundler Configuration (`/Users/ludvig/Desktop/nih-plug/bundler.toml`)

Added new entry for gain plugin:
```toml
[gain]
name = "gainv2"
```

This ensures the host app is named `gainv2.app` instead of `gain.app`.

### 4. Installation Cleanup

- Removed old installation: `/Applications/gainHost.app` (old version from previous build)
- No old `gain.app` was found (it was already removed in previous iterations)

### 5. Rebuild and Installation

- Successfully rebuilt with: `cargo xtask bundle-universal gain --release`
- Plugin bundles created:
  - CLAP: `/Users/ludvig/Desktop/nih-plug/target/bundled/gainv2.clap`
  - VST3: `/Users/ludvig/Desktop/nih-plug/target/bundled/gainv2.vst3`
  - AUv3: `/Applications/gainv2.app`

## Verification Results

### Installation Verified
```bash
$ ls -la /Applications/gain*.app
drwxr-xr-x@  3 ludvig  admin    96 Oct 27 16:11 gainv2.app
```

### Pluginkit Registration Verified
```bash
$ pluginkit -m -v | grep -i nih
com.nihplug.auv3.v2(1.0)	0EBAAB2D-7350-43A6-9585-7789D5111FE1	2025-10-27 23:12:17 +0000	/Applications/gainv2.app/Contents/PlugIns/NIHPlugAUv3.appex
```

### Key Identifiers
- **Bundle Identifier**: `com.nihplug.auv3.v2` (changed from `com.nihplug.auv3`)
- **Subtype Code**: `Npl2` (changed from `Nplg`)
- **Plugin UUID**: `0EBAAB2D-7350-43A6-9585-7789D5111FE1` (NEW - different from old UUID)
- **Display Name**: `NPLG: NIH-Plug AUv3 v2`
- **App Name**: `gainv2.app`
- **App Location**: `/Applications/gainv2.app`

### Info.plist Verification
```bash
$ plutil -p /Applications/gainv2.app/Contents/PlugIns/NIHPlugAUv3.appex/Contents/Info.plist
CFBundleDisplayName: "NIH-Plug AUv3 v2" ✓
CFBundleIdentifier: "com.nihplug.auv3.v2" ✓
name: "NPLG: NIH-Plug AUv3 v2" ✓
subtype: "Npl2" ✓
```

## Success Criteria - All Met

- ✅ Old plugin removed (`gainHost.app` deleted)
- ✅ New plugin with "v2" name installed at `/Applications/gainv2.app`
- ✅ Registered with pluginkit under new identifier `com.nihplug.auv3.v2`
- ✅ New 4-character subtype code `Npl2` (was `Nplg`)
- ✅ New UUID assigned by system: `0EBAAB2D-7350-43A6-9585-7789D5111FE1`
- ✅ Ready for testing in Logic Pro as a brand new plugin

## Cache Clearing

- Attempted to clear AudioUnitCache (directory was empty - no cache present)
- coreaudiod was not running (no restart needed)

## What This Achieves

By changing:
1. **Bundle Identifier** (`com.nihplug.auv3.v2`) - Forces macOS to see this as a completely different app
2. **Subtype Code** (`Npl2`) - Forces Audio Unit system to see this as a different plugin type
3. **Display Name** (`NIH-Plug AUv3 v2`) - Makes it easy to identify the new version
4. **App Name** (`gainv2.app`) - Prevents conflicts with any old installations

Logic Pro and macOS will treat this as a **completely new plugin** with:
- No cached information from the old version
- No old plugin scan results
- No old validation data
- A fresh UUID assigned by the system

## Next Steps for Testing

1. **Launch Logic Pro**
   ```bash
   open -a "Logic Pro"
   ```

2. **Create a new project or open existing one**

3. **Look for the plugin in Audio Units > Audio Units (v2) > Effect**
   - Should appear as: **NPLG: NIH-Plug AUv3 v2**
   - Manufacturer: **NPLG**
   - Type: **Effect (aufx)**

4. **Load the plugin on a track**
   - Add it to an audio or instrument track
   - Verify it appears in the plugin slot
   - Check if the UI loads correctly

5. **Test basic functionality**
   - Verify the plugin processes audio
   - Test parameter controls
   - Check for any console errors

## Files Modified

1. `/Users/ludvig/Desktop/nih-plug/src/wrapper/auv3/swift/NIHPlugAUv3/Info.plist`
2. `/Users/ludvig/Desktop/nih-plug/src/wrapper/auv3/swift/NIHPlugAUv3.xcodeproj/project.pbxproj`
3. `/Users/ludvig/Desktop/nih-plug/bundler.toml`

## Summary

The plugin has been successfully renamed to "v2" with all new identifiers. This is now a **completely fresh plugin** in the eyes of Logic Pro and macOS. Any cached information, validation results, or recognition issues from the old version will not affect this new version.

The plugin is ready for testing in Logic Pro. If it doesn't appear immediately, try:
1. Restarting Logic Pro
2. Running: `pluginkit -m -v` to confirm registration
3. Checking Logic Pro's Plugin Manager to enable it if needed

---
**Status**: ✅ Complete - Ready for Logic Pro Testing
**Date**: 2025-10-27
**Iteration**: 056
