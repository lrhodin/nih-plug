# Audio Unit v3 (AUv3) Support Implementation Plan

## ⚠️ ARCHITECTURAL PIVOT - READ FIRST

**Iterations 1-3 built AUv2 (deprecated). Starting iteration 4, we're pivoting to AUv3.**

AUv3 is fundamentally different:
- App extensions (.appex), not components (.component)
- Swift AUAudioUnit subclass, not C AudioComponent API
- AVAudioUnit framework, not AudioToolbox
- Requires Rust FFI layer + Swift wrapper

Study the AUv2 code in `src/wrapper/au/` for NIH-plug integration patterns, but **do NOT reuse it directly**.

---

## Phase 1: Research & Planning (CHECKPOINT: Understand AUv3 architecture)

- [ ] Research AUv3 architecture and AVAudioUnit framework
- [ ] Study Apple's AUv3 documentation and examples
- [ ] Understand app extension structure and requirements
- [ ] Research Rust FFI patterns for Swift interop
- [ ] Document AUv3 vs AUv2 differences
- [ ] Create architectural plan for NIH-plug AUv3 integration
- [ ] **CHECKPOINT: Present architecture plan for human review**

## Phase 2: FFI Foundation (CHECKPOINT: Rust plugin exports via FFI)

- [ ] Design C FFI interface for plugin operations
- [ ] Create Rust FFI module (`src/wrapper/auv3/ffi.rs`)
- [ ] Implement FFI functions for:
  - Plugin initialization/deinitialization
  - Parameter queries (count, info, get/set)
  - Audio processing callback
  - State save/load
- [ ] Add C header generation (cbindgen or manual)
- [ ] Create test harness to verify FFI from C
- [ ] **CHECKPOINT: Can call Rust plugin from C test program**

## Phase 3: Swift App Extension Scaffold (CHECKPOINT: Extension compiles)

- [ ] Create app extension directory structure
- [ ] Generate Xcode project for AUv3 extension
- [ ] Create AUAudioUnit subclass in Swift
- [ ] Integrate Rust staticlib into Xcode build
- [ ] Bridge C headers to Swift
- [ ] Implement minimal AUAudioUnit methods (init, inputBusses, outputBusses)
- [ ] **CHECKPOINT: Extension builds and can be loaded (even if non-functional)**

## Phase 4: Parameter System (CHECKPOINT: Parameters visible in DAW)

- [ ] Implement AUParameterTree creation in Swift
- [ ] Map NIH-plug parameters to AUParameter instances
- [ ] Implement parameter observers (get/set from host)
- [ ] Bridge parameter changes to Rust via FFI
- [ ] Handle parameter automation
- [ ] **CHECKPOINT: Parameters appear and respond in Logic/GarageBand**

## Phase 5: Audio Processing (CHECKPOINT: Audio processes correctly)

- [ ] Implement internalRenderBlock in Swift
- [ ] Convert AVAudioPCMBuffer to raw buffers for Rust
- [ ] Call Rust audio processing via FFI
- [ ] Handle input/output buffer management
- [ ] Implement proper thread safety
- [ ] **CHECKPOINT: Plugin processes audio in DAW**

## Phase 6: State Management (CHECKPOINT: State persists)

- [ ] Implement fullState property for state save/load
- [ ] Serialize NIH-plug state via FFI
- [ ] Deserialize and restore state
- [ ] Handle preset management
- [ ] **CHECKPOINT: Plugin state saves and loads in DAW**

## Phase 7: Build Automation (CHECKPOINT: Can build from command line)

- [ ] Create nih_plug_xtask support for AUv3 bundling
- [ ] Automate Xcode build from Rust
- [ ] Generate Info.plist from plugin metadata
- [ ] Handle code signing
- [ ] Create installation script
- [ ] **CHECKPOINT: Can build and install plugin with one command**

## Phase 8: Polish & Testing (CHECKPOINT: Production ready)

- [ ] Add comprehensive error handling
- [ ] Add logging and diagnostics
- [ ] Test in multiple DAWs (Logic, GarageBand, Ableton Live, etc.)
- [ ] Test on iOS (if applicable)
- [ ] Performance profiling and optimization
- [ ] Write documentation
- [ ] Create example plugins
- [ ] **CHECKPOINT: Ready for upstream PR**

---

## Current Status

**Completed (iterations 1-4):** AUv2 foundation with working audio processing
**Next task:** Phase 1 - Research AUv3 architecture

## Recent Accomplishments (Iteration 005)

**Fixed Critical Audio Processing Bug:**
- ✅ Fixed AU audio processing to handle both input and output buffers correctly
- ✅ Added proper buffer splitting logic for input/output channels
- ✅ Added comprehensive tests for buffer handling (input/output and generator plugins)
- ✅ All tests passing (82/82)

**Key Technical Details:**
- Audio processing now correctly splits AU buffers into input and output sections
- Supports both effect plugins (with input) and generator plugins (output only)
- Proper handling of channel counts based on AudioIOLayout
- Safe pointer casting for input buffers (read-only to mutable for NIH-plug compatibility)

## Notes

- Each phase ends with a human testing checkpoint
- AUv3 requires macOS 10.11+ (iOS 9+)
- Swift code must be compatible with NIH-plug's Rust-first approach
- Consider both macOS and iOS support from the start
- AUv3 plugins can be loaded in-process (unlike AUv2)
