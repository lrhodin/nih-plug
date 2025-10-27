# AUv3 App Extension Bundle Structure (.appex format)

## Overview

AUv3 plugins are distributed as app extensions (.appex bundles) rather than traditional .component bundles. This represents a fundamental shift from AUv2 to AUv3 architecture.

## Bundle Structure

### Directory Layout

```
MyPlugin.appex/
├── Contents/
│   ├── MacOS/
│   │   └── MyPlugin                    # Mach-O app extension binary
│   ├── Info.plist                      # Extension metadata and configuration
│   ├── Resources/                      # Optional resources (icons, etc.)
│   │   ├── icon.icns                   # Plugin icon (optional)
│   │   └── other resources...
│   └── _CodeSignature/                 # Code signature (if signed)
│       └── CodeResources
└── MyPlugin.a                          # Rust static library (linked into binary)
```

### Key Differences from AUv2 (.component)

| Aspect | AUv2 (.component) | AUv3 (.appex) |
|--------|-------------------|---------------|
| **Bundle Type** | Component bundle | App extension bundle |
| **Package Type** | `BNDL` | `XPC!` (App Extension) |
| **Binary Location** | `Contents/MacOS/` | `Contents/MacOS/` |
| **Metadata** | `Info.plist` | `Info.plist` (different keys) |
| **Code Signing** | Optional | Required for distribution |
| **Platform Support** | macOS only | macOS + iOS |

## Info.plist Structure

### Required Keys for AUv3

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist>
  <dict>
    <!-- Basic Bundle Information -->
    <key>CFBundleExecutable</key>
    <string>MyPlugin</string>
    
    <key>CFBundleIdentifier</key>
    <string>com.company.MyPlugin</string>
    
    <key>CFBundleName</key>
    <string>MyPlugin</string>
    
    <key>CFBundleDisplayName</key>
    <string>MyPlugin</string>
    
    <key>CFBundlePackageType</key>
    <string>XPC!</string>  <!-- App Extension -->
    
    <key>CFBundleSignature</key>
    <string>????</string>
    
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    
    <key>CFBundleVersion</key>
    <string>1.0.0</string>
    
    <!-- App Extension Configuration -->
    <key>NSExtension</key>
    <dict>
      <key>NSExtensionPointIdentifier</key>
      <string>com.apple.AudioUnit-UI</string>  <!-- For AUv3 -->
      
      <key>NSExtensionPrincipalClass</key>
      <string>MyPluginAUAudioUnit</string>  <!-- Swift class name -->
      
      <key>NSExtensionAttributes</key>
      <dict>
        <key>AudioComponents</key>
        <array>
          <dict>
            <key>name</key>
            <string>MyPlugin</string>
            
            <key>description</key>
            <string>MyPlugin Audio Unit</string>
            
            <key>manufacturer</key>
            <string>MyCompany</string>
            
            <key>type</key>
            <string>aumu</string>  <!-- Audio Unit Music Device -->
            
            <key>subtype</key>
            <string>mplg</string>  <!-- Plugin-specific subtype -->
            
            <key>version</key>
            <integer>65536</integer>  <!-- 1.0.0 as integer -->
            
            <key>sandboxSafe</key>
            <true/>
            
            <key>hasCustomView</key>
            <true/>  <!-- If plugin has custom UI -->
          </dict>
        </array>
      </dict>
    </dict>
    
    <!-- Audio Unit Specific -->
    <key>AudioComponents</key>
    <array>
      <dict>
        <key>name</key>
        <string>MyPlugin</string>
        
        <key>description</key>
        <string>MyPlugin Audio Unit</string>
        
        <key>manufacturer</key>
        <string>MyCompany</string>
        
        <key>type</key>
        <string>aumu</string>  <!-- aumu = Music Device, auef = Effect -->
        
        <key>subtype</key>
        <string>mplg</string>  <!-- 4-character code -->
        
        <key>version</key>
        <integer>65536</integer>
        
        <key>factoryFunction</key>
        <string>AudioUnitFactory</string>  <!-- C function name -->
      </dict>
    </array>
    
    <!-- Platform Support -->
    <key>LSMinimumSystemVersion</key>
    <string>10.11</string>  <!-- macOS 10.11+ for AUv3 -->
    
    <key>NSHumanReadableCopyright</key>
    <string>Copyright © 2024 MyCompany</string>
    
    <!-- High Resolution Support -->
    <key>NSHighResolutionCapable</key>
    <true/>
    
    <!-- Sandboxing -->
    <key>com.apple.security.app-sandbox</key>
    <true/>
    
    <key>com.apple.security.files.user-selected.read-only</key>
    <true/>
  </dict>
</plist>
```

## Audio Unit Types and Subtypes

### Component Types

| Type | Code | Description |
|------|------|-------------|
| `aumu` | Music Device | Instrument/synthesizer |
| `auef` | Effect | Audio effect processor |
| `aumf` | Music Effect | MIDI-controlled effect |
| `aufx` | Effect | Audio effect (legacy) |

### Subtype Codes

- Must be 4 characters
- Use reverse domain notation: `com.company.plugin` → `mplg`
- Must be unique within the manufacturer's namespace

## Build Process Integration

### 1. Rust Static Library

```toml
# Cargo.toml
[lib]
crate-type = ["staticlib"]
name = "my_plugin_rust"
```

### 2. Swift App Extension

```swift
// MyPluginAUAudioUnit.swift
import AudioToolbox
import AVFoundation

@objc(MyPluginAUAudioUnit)
class MyPluginAUAudioUnit: AUAudioUnit {
    // Implementation
}
```

### 3. Xcode Project Structure

```
MyPlugin.xcodeproj/
├── MyPlugin/
│   ├── MyPluginAUAudioUnit.swift
│   ├── MyPlugin-Bridging-Header.h
│   └── Info.plist
├── Rust/
│   └── libmy_plugin_rust.a
└── MyPlugin.xcodeproj/
    └── project.pbxproj
```

## Integration with NIH-plug Bundler

### Current Bundler Support

The existing `nih_plug_xtask` bundler supports:
- VST2 bundles (`.vst`)
- VST3 bundles (`.vst3`)
- CLAP bundles (`.clap`)

### Required Extensions for AUv3

1. **Detect AUv3 Support**:
   ```rust
   let bundle_auv3 = symbols::exported(first_lib_path, "AudioUnitFactory")
       .with_context(|| format!("Could not parse '{}'", first_lib_path.display()))?;
   ```

2. **Create .appex Bundle**:
   ```rust
   if bundle_auv3 {
       let auv3_bundle_library_name = auv3_bundle_library_name(&bundle_name, compilation_target);
       let auv3_lib_path = bundle_home_dir.join(&auv3_bundle_library_name);
       
       // Create bundle structure
       fs::create_dir_all(auv3_lib_path.parent().unwrap())?;
       
       // Copy Swift binary and Rust library
       // Generate Info.plist
       // Code sign
   }
   ```

3. **Generate Info.plist**:
   - Extract plugin metadata from Rust plugin
   - Generate AudioComponents array
   - Set proper bundle identifiers
   - Configure app extension settings

## Code Signing Requirements

### Development
- Self-signed certificates work for development
- Use `codesign -f -s - MyPlugin.appex`

### Distribution
- Requires Apple Developer Program membership
- Use distribution certificate
- May require notarization for macOS distribution

## Installation and Discovery

### macOS
- Install to `~/Library/Audio/Plug-Ins/Components/` (AUv2)
- Install to `~/Library/Audio/Plug-Ins/Components/` (AUv3)
- System-wide: `/Library/Audio/Plug-Ins/Components/`

### iOS
- Distributed through App Store
- Installed as part of host app
- Sandboxed execution

## Key Implementation Considerations

### 1. Swift-Rust FFI
- Rust compiles to static library (.a)
- Swift imports C headers for FFI
- Careful memory management across FFI boundary

### 2. Bundle Generation
- Xcode project generation from Rust metadata
- Info.plist generation from plugin descriptor
- Automated build process integration

### 3. Platform Compatibility
- macOS 10.11+ requirement
- iOS 9+ support
- Universal binary support (x86_64 + arm64)

### 4. Testing and Validation
- AU Lab for basic functionality
- Logic Pro for real-world testing
- iOS Simulator for iOS compatibility

## Next Steps

1. Research Swift-to-Rust FFI patterns
2. Study AUParameter and AUParameterTree implementation
3. Research internalRenderBlock audio processing
4. Create comprehensive architecture summary
5. Implement FFI layer and Swift wrapper
6. Integrate with existing bundler system