# nih-plug - Build & Run Guide

## Prerequisites

- Rust toolchain (stable)
- Xcode (for AUv3 Swift compilation)
- macOS (for Audio Unit development)

## Build

### Build the AUv3 plugin (gain example):

```bash
cargo xtask bundle-universal gain --release
```

This will:
1. Compile the Rust plugin code
2. Build the Swift AUv3 wrapper using Xcode
3. Create a universal binary (arm64 + x86_64)
4. Install to `/Applications/gain.app/Contents/PlugIns/NIHPlugAUv3.appex`

### Build and install specific plugin:

```bash
cargo xtask bundle-universal <plugin-name> --release
```

## Test

### Run Rust tests:

```bash
cargo test
```

### Check if plugin is discovered by system:

```bash
# Check if plugin appears in auval (Audio Unit validation tool)
auval -a | grep -i nplg

# Check pluginkit registration
pluginkit -m -v -p com.apple.audio-unit

# Check if plugin exists on disk
ls -la /Applications/gain.app/Contents/PlugIns/NIHPlugAUv3.appex
```

### Test in Logic Pro:

1. Open Logic Pro
2. Create new project
3. Add Audio FX plugin
4. Look for "NIH-Plug AUv3" in the plugin list
5. Load and test

## Development

### Current Status (Iteration 46):

**Phase 7.5: AUv3 Discovery Research**

The plugin builds and installs correctly, but is NOT discovered by macOS (doesn't appear in auval, pluginkit, or Logic Pro).

**Current focus:** RESEARCH ONLY
- Find working AUv3 examples
- Compare with NIH-Plug implementation
- Identify why plugin isn't discovered

### Debugging Commands:

```bash
# Check FFI symbols in binary
nm /Applications/gain.app/Contents/PlugIns/NIHPlugAUv3.appex/Contents/MacOS/NIHPlugAUv3 | grep -i nih_plug

# Check Info.plist configuration
cat /Applications/gain.app/Contents/PlugIns/NIHPlugAUv3.appex/Contents/Info.plist | grep -A 10 NSExtension

# Monitor system logs during plugin registration
log stream --predicate 'subsystem == "com.apple.audio" or process == "AudioComponentRegistrar"' --level debug

# Clear plugin cache
killall -9 AudioComponentRegistrar
```
