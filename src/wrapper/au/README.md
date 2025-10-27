# Audio Unit v2 Wrapper

This directory contains the implementation of the Audio Unit v2 (AUv2) wrapper for NIH-plug.

## Overview

The AUv2 wrapper allows NIH-plug plugins to be exported as Audio Unit plugins on macOS. This implementation follows similar patterns to the VST3 and CLAP wrappers but adapts them for Apple's Audio Component API.

## Architecture

### Key Components

1. **bindings.rs** - Rust definitions for AU C API types and constants
2. **factory.rs** - Factory function for creating plugin instances
3. **wrapper.rs** - Main wrapper that bridges Plugin trait to AU API
4. **context.rs** - InitContext and ProcessContext implementations
5. **util.rs** - Helper functions and macros

### Plugin Lifecycle

1. Host calls factory function with AudioComponentDescription
2. Factory creates AudioComponentPlugInInstance
3. Host calls Open() to initialize the instance
4. Host configures plugin (sample rate, buffer size, stream format)
5. Host calls Render() to process audio
6. Host calls Close() to cleanup

## Bundle Structure

Audio Unit plugins are distributed as `.component` bundles with this structure:

```
MyPlugin.component/
  Contents/
    Info.plist
    MacOS/
      MyPlugin
    Resources/      (optional)
```

### Info.plist Example

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>English</string>
    <key>CFBundleExecutable</key>
    <string>MyPlugin</string>
    <key>CFBundleIdentifier</key>
    <string>com.mycompany.myplugin.au</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>MyPlugin</string>
    <key>CFBundlePackageType</key>
    <string>BNDL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>

    <key>AudioComponents</key>
    <array>
        <dict>
            <key>type</key>
            <string>aufx</string>           <!-- Effect: aufx, Instrument: aumu -->
            <key>subtype</key>
            <string>MYPL</string>           <!-- Your unique 4-char code -->
            <key>manufacturer</key>
            <string>MYCO</string>           <!-- Your manufacturer 4-char code -->
            <key>name</key>
            <string>MyCompany: MyPlugin</string>
            <key>version</key>
            <integer>65536</integer>        <!-- Version in hex (1.0.0 = 0x10000) -->
            <key>factoryFunction</key>
            <string>AU_FACTORY</string>     <!-- Factory function name from Rust -->
            <key>sandboxSafe</key>
            <true/>
        </dict>
    </array>
</dict>
</plist>
```

## Component Types and Codes

### Component Types (4-character codes)

- `aufx` - Audio Effect (processes audio)
- `aumu` - Music Device (synthesizer/instrument)
- `aumf` - Music Effect (processes both MIDI and audio)
- `augn` - Generator (generates audio without input)
- `auou` - Output (renders to hardware)

### Choosing Your Codes

- **Subtype**: A unique 4-character code for your plugin (e.g., "GAIN" for a gain plugin)
- **Manufacturer**: A unique 4-character code for your company (register with Apple or use your own)

**Important**: These codes should be unique to avoid conflicts with other plugins.

## Usage

In your plugin's `lib.rs`:

```rust
use nih_plug::prelude::*;

struct MyPlugin;

impl Plugin for MyPlugin {
    const NAME: &'static str = "My Plugin";
    const VENDOR: &'static str = "My Company";
    // ... other required constants
}

impl ClapPlugin for MyPlugin {
    const CLAP_ID: &'static str = "com.mycompany.myplugin";
}

// Export as Audio Unit
nih_export_au!(MyPlugin);
```

## Building

The NIH-plug bundler (nih_plug_xtask) will need to be updated to:

1. Compile the plugin as a dynamic library
2. Create the .component bundle structure
3. Generate the Info.plist with correct metadata
4. Sign the bundle for distribution (optional but recommended)

## Current Status

### Implemented
- ✅ Basic module structure
- ✅ AU bindings (types, constants)
- ✅ Factory function framework
- ✅ Wrapper skeleton
- ✅ Context implementations (skeleton)

### In Progress
- 🔄 AudioComponentPlugInInstance callbacks
- 🔄 Audio processing pipeline
- 🔄 Parameter management

### TODO
- ❌ Initialize/Uninitialize callbacks
- ❌ Render (process) callback
- ❌ GetProperty/SetProperty
- ❌ Parameter automation
- ❌ MIDI event handling
- ❌ State save/load
- ❌ GUI integration
- ❌ Bundler integration
- ❌ Testing in DAWs

## References

- [Audio Unit Programming Guide](https://developer.apple.com/library/archive/documentation/MusicAudio/Conceptual/AudioUnitProgrammingGuide/)
- [Technical Note TN2247](https://developer.apple.com/library/archive/technotes/tn2247/)
- [Audio Component Services](https://developer.apple.com/documentation/audiounit/audio_component_services)
