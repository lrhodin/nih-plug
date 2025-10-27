# nih-plug - Overview

## Project Description

Add Audio Unit v3 (AUv3) support to NIH-Plug.

This is for a pull request to https://github.com/robbert-vdh/nih-plug

**CRITICAL: We are implementing AUv3 (Audio Unit version 3), NOT AUv2.**
- AUv3 uses the modern AVAudioUnit framework
- AUv3 plugins are app extensions (.appex), not components (.component)
- AUv3 uses Swift/Objective-C, with Rust core called via FFI
- AUv2 is deprecated and should NOT be used

Work in phases that end at human testing checkpoints. Each phase should:
- Implement one testable milestone
- Stop when human DAW testing is needed
- Document what needs testing
- Reaching a testing checkpoint IS success for that iteration

Follow NIH-Plug's existing patterns and abstractions.
Research the codebase to understand how VST3/CLAP work, then apply similar patterns to AUv3.

This should be production-quality code suitable for upstream merge.

## ⚠️ ARCHITECTURAL PIVOT NOTE

**Iterations 1-2 built AUv2 infrastructure by mistake.**

The work done was valuable for learning NIH-plug patterns, but:
- AUv2 uses C AudioComponent API (deprecated)
- AUv3 uses AVAudioUnit framework (modern, required for iOS)
- The architectures are fundamentally different

**Starting from iteration 4+, we are pivoting to AUv3.**

Study the AUv2 code for NIH-plug integration patterns, but do NOT reuse the AU wrapper code directly.

## Goals

1. **Enable NIH-plug plugins to export as AUv3**
   - Provide `nih_export_auv3!()` macro
   - Generate app extension scaffolding
   - Bridge Rust plugin to AVAudioUnit

2. **Support macOS and iOS**
   - AUv3 works on both platforms
   - Follow Apple's best practices

3. **Maintain NIH-plug patterns**
   - Follow VST3/CLAP wrapper architecture where applicable
   - Reuse parameter, state, and context abstractions

4. **Production quality**
   - Ready for upstream merge
   - Well-documented
   - Tested in real DAWs
   - **Validated with pluginval** - All AUv3 plugins must pass pluginval strictness level 5

## Architecture

### Audio Unit v3 (AUv3) Implementation

**AUv3 uses a completely different architecture from AUv2:**

1. **App Extension Structure**
   - AUv3 plugins are app extensions (.appex)
   - Contains Swift/Objective-C code that subclasses AUAudioUnit
   - Rust plugin core is compiled as static library (.a) and linked

2. **Wrapper Layer**
   - Swift AUAudioUnit subclass in app extension
   - Calls into Rust FFI layer
   - Rust wrapper translates to NIH-plug Plugin trait

3. **Key Differences from AUv2**
   - No C AudioComponent API
   - Uses AVAudioUnit framework
   - Parameters via AUParameter/AUParameterTree
   - Rendering via AVAudioPCMBuffer
   - State via presets/fullState

4. **Bundle Structure**
   ```
   MyPlugin.appex/
     Contents/
       MacOS/MyPlugin          # Mach-O app extension
       Info.plist              # Extension metadata
       Resources/
   ```

5. **Integration Strategy**
   - Build Rust plugin as staticlib
   - Swift extension imports C headers (FFI)
   - Calls Rust functions for all plugin operations
   - Wraps results in AUAudioUnit interface

## Technical Decisions

### Use Swift for AUAudioUnit Subclass
AUv3 requires subclassing AUAudioUnit, which is an Objective-C/Swift class. We'll use Swift for the subclass and call into Rust via FFI.

### Static Library Linking
Rust plugin compiles as staticlib (.a) and links into the app extension. This is cleaner than dynamic loading.

### FFI Safety Layer
Create a safe FFI boundary:
- C-compatible function signatures
- Proper error handling across FFI
- No panics across FFI boundary

### Reuse NIH-plug Abstractions
Where possible, reuse:
- Parameter mapping and normalization
- State serialization
- Buffer and context patterns
- Follow VST3/CLAP wrapper structure

## Validation Requirements

### Automated Validation with pluginval

All AUv3 plugins built with NIH-plug must pass automated validation using [pluginval](https://github.com/Tracktion/pluginval).

**Setup:**
1. Download pluginval binary for macOS from [releases](https://github.com/Tracktion/pluginval/releases)
2. Install to a known location (e.g., `~/bin/pluginval`)
3. Make executable: `chmod +x ~/bin/pluginval`

**Validation Process:**
1. Build a test AUv3 plugin using NIH-plug (e.g., gain example)
2. Run pluginval in headless mode:
   ```bash
   pluginval --strictness-level 5 --validate-in-process --verbose /path/to/plugin.appex
   ```
3. Check exit code: 0 = pass, 1 = fail
4. Review console output for any warnings or errors

**Success Criteria:**
- Exit code must be 0 (all tests pass)
- No crashes during parameter fuzzing
- No memory leaks detected
- State save/restore works correctly
- Audio processing doesn't produce NaN values

**Integration:**
- Add pluginval validation as final step before requesting human testing
- Document validation results in handoff notes
- If validation fails, fix issues before proceeding to DAW testing
