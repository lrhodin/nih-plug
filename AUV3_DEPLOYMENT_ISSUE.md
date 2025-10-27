# AUv3 Deployment Issue - CRITICAL FIX NEEDED

## Problem Discovered

The plugin is NOT being registered with macOS Audio Unit system.

**Test Results:**
- ✅ Plugin builds successfully with correct Info.plist
- ✅ All required AUAudioUnit properties implemented
- ✅ Code signing works (adhoc)
- ❌ Plugin NOT found by AudioComponentFindNext()
- ❌ pluginkit shows NO AudioUnit extensions registered
- ❌ auval cannot find plugin

**Root Cause:**
Standalone `.appex` files in `~/Library/Audio/Plug-Ins/Components/` are NOT automatically registered by macOS. That directory is for AUv2 `.component` bundles only.

## Apple's AUv3 Requirements

Per Apple documentation, AUv3 plugins must be EITHER:

### Option 1: Inside a Host Application (Standard Approach)
```
MyApp.app/
└── Contents/
    └── PlugIns/
        └── MyAU.appex/    # Extension bundle goes here
```

The host app can be minimal (just a shell), but it must exist for the extension to be discovered.

### Option 2: Use AudioComponentRegister() API
For in-process loading without app bundle:
```swift
// Register at runtime instead of automatic discovery
AudioComponentRegister(&desc, name, version, constructor)
```

## Solution for NIH-Plug

**Recommended: Create Minimal Host App**

The bundler (`nih_plug_xtask`) needs to:

1. Generate a minimal `.app` bundle:
   ```
   NIHPlugAUv3Host.app/
   ├── Contents/
   │   ├── Info.plist         # Host app plist
   │   ├── MacOS/
   │   │   └── NIHPlugAUv3Host  # Minimal executable
   │   └── PlugIns/
   │       └── NIHPlugAUv3.appex/  # Our extension
   ```

2. Install to: `~/Library/Audio/Plug-Ins/Components/NIHPlugAUv3Host.app`

3. Host app Info.plist must declare extension:
   ```xml
   <key>NSExtension</key>
   <dict>
       <key>NSExtensionPointIdentifier</key>
       <string>com.apple.AudioUnit</string>
   </dict>
   ```

## What Needs To Change

### File: `nih_plug_xtask/src/lib.rs`

The `bundle_auv3()` function currently creates standalone `.appex`. It needs to:

1. Create host `.app` bundle structure
2. Place `.appex` inside `Contents/PlugIns/`
3. Create minimal host executable (can be empty stub)
4. Generate host app Info.plist
5. Code sign both host and extension

### Minimal Host Executable

Can be a tiny Swift program:
```swift
import Cocoa
// AUv3 host app - extension loaded by system
NSApplicationMain(CommandLine.argc, CommandLine.unsafeArgv)
```

Or even simpler - just exit immediately since the extension is what matters.

## Testing After Fix

Once host app structure is created:

```bash
# Install
cp -r NIHPlugAUv3Host.app ~/Library/Audio/Plug-Ins/Components/

# Register
pluginkit -a ~/Library/Audio/Plug-Ins/Components/NIHPlugAUv3Host.app

# Verify extension is registered
pluginkit -m -v | grep -i nih

# Test with Swift
swift test_plugin_loading_fixed.swift

# Test with auval
auval -a | grep NPLG
auval -v aufx Nplg NPLG
```

## References

- Apple Technical Q&A: "Packaging Audio Unit Extensions"
- App Extension Programming Guide
- Audio Unit v3 documentation

## Current Status

- All Swift implementation is CORRECT
- All Info.plist configuration is CORRECT
- **Only issue**: Wrong bundle structure/installation location

This explains why iterations 27-29 couldn't get recognition working - it's an architectural issue with how the bundle is packaged and installed, not a code issue.
