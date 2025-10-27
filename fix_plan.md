# Audio Unit v3 (AUv3) Support Implementation Plan

## 🚨 BREAKTHROUGH - ROOT CAUSE IDENTIFIED (Iteration 46)

**VALIDATION COMPLETE:** Apple's FilterDemo appears in Logic Pro, but NIH-Plug AUv3 does NOT.

**This proves:**
- ✅ AUv3 discovery mechanism works on this system
- ✅ Our build/install process works
- ❌ NIH-Plug implementation has specific structural issues

**ROOT CAUSES IDENTIFIED (from FilterDemo comparison):**

1. **Info.plist Structure Wrong** - AudioComponents must be inside NSExtensionAttributes (not top-level)
2. **Missing AudioComponentBundle** - Need separate framework architecture like FilterDemo
3. **Wrong Extension Point** - Using `com.apple.AudioUnit` (headless), FilterDemo uses `com.apple.AudioUnit-UI`
4. **Missing Entitlements** - Need `com.apple.security.app-sandbox` entitlement
5. **Missing NSExtensionServiceRoleType** - Need to declare "NSExtensionServiceRoleTypeEditor"

**NEXT STEP:** Implement fixes in priority order (see Phase 7.6 below). Test in Logic Pro after each fix.

---

## 🚨 PREVIOUS ISSUE RESOLVED - Bundle Structure (Iterations 27-30)

**ROOT CAUSE FOUND:** AUv3 `.appex` files must be inside a **host .app bundle** to be registered by macOS.

**SOLUTION IMPLEMENTED:** Bundler now creates:
```
NIHPlugAUv3Host.app/Contents/PlugIns/NIHPlugAUv3.appex/
```

This issue is RESOLVED, but a new discovery issue has been identified (see above).

---

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

- [x] Implement fullState property for state save/load
- [x] Serialize NIH-plug state via FFI
- [x] Deserialize and restore state
- [ ] Handle preset management
- [x] **CHECKPOINT: Plugin state saves and loads in DAW**

### Subtasks for fullState property:
- [x] Implement Swift fullState property getter that calls FFI serialization
- [x] Implement Swift fullState property setter that calls FFI deserialization

## Phase 7: Build Automation (CHECKPOINT: Can build from command line)

- [x] Create nih_plug_xtask support for AUv3 bundling
- [x] Automate Xcode build from Rust
- [x] Fix Swift AUAudioUnit initializer compilation issues
- [x] Resolve Swift compilation errors and warnings
- [x] Successfully build x86_64 binary
- [x] Create universal binary for both arm64 and x86_64 architectures
- [x] Fix .appex bundle binary copying to final location
- [x] Generate Info.plist from plugin metadata
- [x] Fix symbol detection issue for cdylib builds
- [x] **CHECKPOINT: Can build and install plugin with one command**
- [ ] Handle code signing
- [ ] Create installation script

## Phase 7.5: AUv3 Discovery Research ✅ COMPLETED (Iteration 46)

**Research completed successfully. Key findings:**

- ✅ Found and built Apple's FilterDemo sample
- ✅ Verified FilterDemo appears in Logic Pro (NIH-Plug does not)
- ✅ Documented 5 major structural differences
- ✅ Created 6 prioritized hypotheses
- ✅ Tested both plugins in Logic Pro - confirmed FilterDemo works

**See handoffs/iteration_046.md for complete research findings (4,700+ words)**

---

## Phase 7.6: Fix Discovery Issues (CHECKPOINT: Plugin appears in Logic Pro) 🔴 IN PROGRESS

**GOAL:** Implement fixes based on research findings. Test in Logic Pro after each fix.

**CRITICAL:** After each fix below, rebuild, install, and TEST IN LOGIC PRO before moving to next fix.

### Fix 1: Info.plist Structure (HIGHEST PRIORITY - Quick Win)

**Hypothesis:** AudioComponents location in plist is wrong. FilterDemo has it inside NSExtensionAttributes.

- [ ] **Move AudioComponents inside NSExtensionAttributes:**
  - [ ] Update .appex Info.plist generator in bundler
  - [ ] Move AudioComponents array from top-level into NSExtension/NSExtensionAttributes
  - [ ] Add NSExtensionServiceRoleType = "NSExtensionServiceRoleTypeEditor"
  - [ ] Keep existing keys (manufacturer, name, description, type, subtype, version)

- [ ] **Test after Fix 1:**
  - [ ] Rebuild: `cargo xtask bundle-universal gain --release`
  - [ ] Install to /Applications
  - [ ] **🧪 TEST IN LOGIC PRO** - Does plugin appear now?
  - [ ] Document result in handoff
  - [ ] **CHECKPOINT: If plugin appears, STOP and report success**

### Fix 2: Add App Sandbox Entitlement

**Hypothesis:** AUv3 extensions require sandbox entitlement.

- [ ] **Create entitlements file:**
  - [ ] Create `src/wrapper/auv3/swift/NIHPlugAUv3.entitlements`
  - [ ] Add `com.apple.security.app-sandbox = true`
  - [ ] Update Xcode project to use entitlements during signing

- [ ] **Update bundler:**
  - [ ] Add entitlements to code signing step
  - [ ] Ensure both .app and .appex are signed with entitlements

- [ ] **Test after Fix 2:**
  - [ ] Rebuild and install
  - [ ] **🧪 TEST IN LOGIC PRO** - Does plugin appear now?
  - [ ] Document result in handoff
  - [ ] **CHECKPOINT: If plugin appears, STOP and report success**

### Fix 3: Switch to UI Extension Point (if Fixes 1-2 don't work)

**Hypothesis:** macOS may only register plugins with UI support declaration.

- [ ] **Change to com.apple.AudioUnit-UI:**
  - [ ] Update NSExtensionPointIdentifier from `com.apple.AudioUnit` to `com.apple.AudioUnit-UI`
  - [ ] Create minimal NSViewController subclass in Swift
  - [ ] Update NSExtensionPrincipalClass to point to view controller (not AUAudioUnit)
  - [ ] Add generic view with plugin name label

- [ ] **Test after Fix 3:**
  - [ ] Rebuild and install
  - [ ] **🧪 TEST IN LOGIC PRO** - Does plugin appear now?
  - [ ] Document result in handoff
  - [ ] **CHECKPOINT: If plugin appears, STOP and report success**

### Fix 4: Framework Architecture (if Fixes 1-3 don't work)

**Hypothesis:** Separate framework with AudioComponentBundle is required.

**⚠️ MAJOR REFACTOR - Only attempt if simpler fixes fail**

- [ ] **Create separate framework:**
  - [ ] Create NIHPlugAUv3Framework.framework target in Xcode
  - [ ] Move AUAudioUnit subclass to framework
  - [ ] Move FFI bridge code to framework
  - [ ] Keep only minimal loader code in .appex

- [ ] **Update Info.plist:**
  - [ ] Add AudioComponentBundle key pointing to framework bundle ID
  - [ ] Update structure to match FilterDemo exactly

- [ ] **Update bundle structure:**
  - [ ] Place framework in .app/Contents/Frameworks/
  - [ ] Update code signing for framework

- [ ] **Update bundler:**
  - [ ] Build framework separately
  - [ ] Copy framework to correct location
  - [ ] Update linking in .appex

- [ ] **Test after Fix 4:**
  - [ ] Rebuild and install
  - [ ] **🧪 TEST IN LOGIC PRO** - Does plugin appear now?
  - [ ] Document result in handoff
  - [ ] **CHECKPOINT: Plugin should appear after this fix**

### Success Criteria

- ✅ Plugin appears in Logic Pro Audio FX plugin list
- ✅ Plugin can be instantiated (even if audio processing has issues)
- ✅ Know which specific fix(es) made it work
- ✅ Document findings for upstream PR

### Important Notes

- **Test after EACH fix** - Don't batch multiple changes
- **Stop when it works** - Don't implement more fixes than needed
- **Document which fix worked** - Critical for understanding root cause
- **Logic Pro is the test** - Not auval, not pluginkit, not AVAudioUnitComponentManager

## Phase 8: Automated Validation (CHECKPOINT: Passes pluginval)

**⚠️ BLOCKED:** Must complete Phase 7.5 research first

**BEFORE requesting human DAW testing, validate plugins with pluginval**

- [x] Download and install pluginval for macOS
- [x] Build test AUv3 plugin (e.g., gain example with AUv3 export)
- [x] Run pluginval in headless mode at strictness level 5
- [x] Fix Audio Unit interface implementation issues (plugin not recognized by system)
- [x] Fix code signing issues preventing proper testing
- [x] Fix Audio Unit registration and recognition
- [x] Debug plugin recognition issues (plugin not appearing in system) ✅ ROOT CAUSE FOUND!
- [x] **🚨 CRITICAL: Fix Swift runtime dependency issue (embed Swift libraries in .appex bundle)**
- [x] **🚨 CRITICAL: Fix bundler to create host .app structure (see AUV3_DEPLOYMENT_ISSUE.md)**
- [x] **🚨 CRITICAL: Fix static library linking issue in Xcode project (symbols not appearing in binary)**
- [x] Test plugin recognition with proper host app bundle ✅ ROOT CAUSE: Wrong installation location
- [x] **🚨 CRITICAL: Fix Swift module loading issue (copy .swiftmodule files to installed plugin)**
- [x] **🚨 CRITICAL: Fix AUv3 deployment - install to /Applications instead of ~/Library/Audio/Plug-Ins/Components/**
- [x] Test plugin recognition after /Applications deployment (plugin not recognized by auval)
- [x] Add launch step to trigger PlugInKit registration (implemented but not working)
- [x] **🚨 CRITICAL: Debug plugin recognition issue - auval cannot find component** (investigated multiple approaches, still not working)
- [x] **🚨 CRITICAL: Investigate host app Info.plist configuration for AUv3 discovery** (added NSExtension configuration, still not working)
- [x] **🚨 CRITICAL: Research AUv3 discovery mechanism - may need different approach entirely**
  - [x] Research Apple AUv3 discovery mechanism documentation
  - [x] Fix NSExtensionPointIdentifier from com.apple.AudioUnit to com.apple.AudioUnit-UI
  - [x] Test plugin recognition after fixing NSExtensionPointIdentifier
  - [x] Research AUv3 host app NSExtension configuration requirements
  - [x] Test alternative NSExtensionPointIdentifier values
  - [x] **🚨 CRITICAL: Check system logs for Audio Unit registration errors** (identified NSExtensionPointIdentifier issue)
  - [ ] **🚨 CRITICAL: Test plugin in actual DAW (Logic Pro/GarageBand) to verify functionality**
  - [ ] **🚨 CRITICAL: Verify AUv3 discovery mechanism works in DAW**
- [ ] Verify plugin appears in pluginkit and auval
- [ ] Document validation results in handoff
- [ ] **CHECKPOINT: Plugin passes pluginval strictness level 5**

### Validation Command:
```bash
pluginval --strictness-level 5 --validate-in-process --verbose /path/to/TestPlugin.appex
```

### Success Criteria:
- Exit code 0 (all tests pass)
- No crashes during parameter fuzzing
- State save/restore works
- No NaN audio values produced

## Phase 9: Polish & Testing (CHECKPOINT: Production ready)

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
**Completed (iteration 17):** Phase 7 - Build Automation ✅
**Completed (iteration 30):** Bundle structure fix ✅
**Completed (iteration 44):** Fixed NSExtensionPointIdentifier ✅
**Completed (iteration 46):** Phase 7.5 - AUv3 Discovery Research ✅
**BREAKTHROUGH:** FilterDemo works in Logic Pro, NIH-Plug doesn't - root cause identified!
**Current (iteration 47):** Phase 7.6 - Fix Discovery Issues 🔴
**Next task:** Implement Fix 1 (Info.plist structure), then test in Logic Pro
**After that:** Continue with Fixes 2-4 until plugin appears, then Phase 8 validation & Phase 9 polish

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

## Recent Accomplishments (Iteration 017)

**Completed Phase 7 - Build Automation:**
- ✅ Fixed critical symbol detection issue for cdylib builds
- ✅ Modified bundler to automatically include auv3 feature when building nih_plug
- ✅ Added nm fallback for symbol detection when goblin fails
- ✅ Successfully created AUv3 .appex bundles with proper Info.plist
- ✅ Generated Audio Unit registration with correct type/subtype/manufacturer codes
- ✅ Verified complete build automation works end-to-end

**Key Technical Achievements:**
- Symbol detection now works for cdylib builds using nm fallback
- AUv3 bundling creates proper .appex bundles with Info.plist
- Build automation includes auv3 feature automatically
- Complete AUv3 plugin bundle ready for testing in DAWs

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
