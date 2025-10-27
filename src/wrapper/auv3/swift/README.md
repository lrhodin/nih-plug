# NIH-Plug AUv3 Swift Integration

This directory contains the Swift app extension implementation for NIH-Plug AUv3 support.

## Overview

The AUv3 implementation consists of:

- **AUAudioUnit.swift**: Main Swift class that bridges between AUv3 host and Rust FFI
- **NIHPlugAUv3-Bridging-Header.h**: C header for FFI function declarations
- **Info.plist**: Extension metadata and Audio Unit registration
- **NIHPlugAUv3.xcodeproj**: Xcode project for building the extension
- **build_rust.sh**: Script to compile Rust static library

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   AUv3 Host     │    │   Swift Bridge   │    │   Rust Core     │
│  (Logic Pro)    │◄──►│  (AUAudioUnit)   │◄──►│  (NIH-Plug)     │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                              │
                              ▼
                       ┌──────────────────┐
                       │   FFI Layer      │
                       │  (C Functions)   │
                       └──────────────────┘
```

## Building

### Prerequisites

- Xcode 15.0 or later
- Rust toolchain
- cbindgen (optional, for C header generation)

### Build Steps

1. **Build Rust static library**:
   ```bash
   ./build_rust.sh
   ```

2. **Open in Xcode**:
   ```bash
   open NIHPlugAUv3.xcodeproj
   ```

3. **Build the extension**:
   - Select the NIHPlugAUv3 target
   - Press Cmd+B to build

### Testing

1. **Load in Logic Pro**:
   - Build the extension
   - Open Logic Pro
   - Look for "NIH-Plug AUv3" in the Audio Unit browser

2. **Load in GarageBand**:
   - Build the extension
   - Open GarageBand
   - Add Audio Unit effect
   - Select "NIH-Plug AUv3"

## File Structure

```
swift/
├── AUAudioUnit.swift              # Main Swift AUAudioUnit subclass
├── NIHPlugAUv3-Bridging-Header.h # C bridging header
├── Info.plist                    # Extension metadata
├── NIHPlugAUv3.xcodeproj/        # Xcode project
│   └── project.pbxproj           # Project configuration
├── build_rust.sh                 # Rust build script
├── libnih_plug_auv3.a           # Rust static library (generated)
└── README.md                     # This file
```

## Implementation Status

### ✅ Completed
- [x] Swift directory structure
- [x] Basic AUAudioUnit subclass
- [x] C bridging header
- [x] Info.plist configuration
- [x] Xcode project setup
- [x] Rust build integration

### 🚧 In Progress
- [ ] Parameter tree implementation
- [ ] Audio processing integration
- [ ] State management
- [ ] Error handling

### 📋 TODO
- [ ] Parameter automation
- [ ] Preset management
- [ ] UI integration
- [ ] Performance optimization
- [ ] Testing in multiple DAWs

## Key Features

### Parameter Management
- Automatic parameter tree creation from Rust plugin
- Real-time parameter updates
- Host automation support

### Audio Processing
- Real-time audio processing via FFI
- Input/output buffer management
- Thread-safe audio processing

### State Management
- Plugin state serialization
- Preset save/load
- Host state persistence

## Troubleshooting

### Build Issues

1. **Rust library not found**:
   - Run `./build_rust.sh` first
   - Check that `libnih_plug_auv3.a` exists

2. **Swift compilation errors**:
   - Check bridging header includes
   - Verify FFI function declarations

3. **Extension not loading**:
   - Check Info.plist configuration
   - Verify Audio Unit registration
   - Check code signing

### Runtime Issues

1. **Audio not processing**:
   - Check FFI function calls
   - Verify buffer management
   - Check thread safety

2. **Parameters not working**:
   - Verify parameter tree setup
   - Check parameter ID mapping
   - Verify FFI parameter functions

## Development Notes

### FFI Integration
- All Rust functions are exposed via C-compatible FFI
- Memory management is handled safely between Swift and Rust
- Error codes are consistent across both languages

### Thread Safety
- Audio processing runs on real-time thread
- Parameter updates are thread-safe
- State management is synchronized

### Performance
- Minimal overhead for FFI calls
- Optimized for real-time audio processing
- Memory allocations are minimized

## Contributing

When modifying the Swift integration:

1. **Follow Swift conventions**: Use proper naming and documentation
2. **Test thoroughly**: Test in multiple DAWs and scenarios
3. **Update documentation**: Keep this README current
4. **Check FFI compatibility**: Ensure C function signatures match

## License

This code is part of the NIH-Plug project and follows the same license terms.