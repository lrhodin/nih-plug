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

## Phase 1: Research & Planning (CHECKPOINT: Understand AUv3 architecture) ✅ COMPLETED

### Research Tasks (small, focused - max 20 min each)

- [x] Document key AUv2 vs AUv3 architectural differences (brief summary)
- [x] Research: AVAudioUnit class hierarchy and core methods
- [x] Research: AUv3 app extension bundle structure (.appex format)
- [x] Research: Basic Swift-to-Rust FFI examples (simple function calls)
- [x] Research: AUParameter and AUParameterTree for parameter management
- [x] Research: AUv3 audio processing (internalRenderBlock)
- [x] Create: One-page AUv3 architecture summary for NIH-plug integration
- [x] **CHECKPOINT: Present architecture plan for human review**

## Phase 2: FFI Foundation (CHECKPOINT: Rust plugin exports via FFI)

### FFI Tasks (small, incremental)

- [ ] Create `src/wrapper/auv3/ffi.rs` with basic module structure
- [ ] Implement FFI: Plugin initialization (init/deinit only)
- [ ] Implement FFI: Parameter count and info queries
- [ ] Implement FFI: Parameter get/set functions
- [ ] Implement FFI: Audio processing callback signature
- [ ] Implement FFI: State serialization/deserialization
- [ ] Generate C headers using cbindgen or write manually
- [ ] Create simple C test program to verify FFI calls work
- [ ] **CHECKPOINT: Can call Rust plugin from C test program**

## Phase 3: Swift App Extension Scaffold (CHECKPOINT: Extension compiles)

### Swift Extension Tasks (small, buildable steps)

- [ ] Create basic app extension directory structure
- [ ] Generate minimal Xcode project for AUv3 extension
- [ ] Add Rust staticlib to Xcode project (link only, no bridging yet)
- [ ] Create empty AUAudioUnit subclass in Swift
- [ ] Add bridging header for C FFI imports
- [ ] Implement AUAudioUnit init method (minimal stub)
- [ ] Implement inputBusses and outputBusses properties (stub)
- [ ] Verify extension builds (even with stubs)
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
- [x] Serialize NIH-plug state via FFI
- [x] Deserialize and restore state
- [ ] Handle preset management
- [ ] **CHECKPOINT: Plugin state saves and loads in DAW**

### Subtasks for fullState property:
- [ ] Implement Swift fullState property getter that calls FFI serialization
- [ ] Implement Swift fullState property setter that calls FFI deserialization

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

**Completed (iterations 1-5):** AUv2 foundation with working audio processing
**Completed (iteration 6):** Phase 1 - Research AUv3 architecture ✅
**Next task:** Phase 2 - FFI Foundation

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

## Recent Accomplishments (Iteration 006)

**Completed Phase 1 - Research & Planning:**
- ✅ Documented key AUv2 vs AUv3 architectural differences
- ✅ Researched AVAudioUnit class hierarchy and core methods
- ✅ Researched AUv3 app extension bundle structure (.appex format)
- ✅ Researched Swift-to-Rust FFI examples and patterns
- ✅ Researched AUParameter and AUParameterTree for parameter management
- ✅ Researched AUv3 audio processing (internalRenderBlock)
- ✅ Created comprehensive AUv3 architecture summary for NIH-plug integration

**Key Research Findings:**
- AUv3 requires complete architectural rewrite from AUv2
- Swift AUAudioUnit subclass + FFI layer needed for Rust integration
- AUParameterTree provides automatic parameter automation and host integration
- internalRenderBlock handles real-time audio processing with AVAudioPCMBuffer
- .appex bundle format with Xcode project generation required
- Comprehensive FFI interface design completed for all plugin functionality

## Notes

- Each phase ends with a human testing checkpoint
- AUv3 requires macOS 10.11+ (iOS 9+)
- Swift code must be compatible with NIH-plug's Rust-first approach
- Consider both macOS and iOS support from the start
- AUv3 plugins can be loaded in-process (unlike AUv2)
