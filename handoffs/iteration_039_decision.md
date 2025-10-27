# Iteration 039 - Architectural Decision: Continue with AUv3

## Decision Made: CONTINUE WITH AUv3

**Date:** 2025-10-27
**Iteration:** 39 (research + decision)
**Decision maker:** Project owner

### Decision Rationale

Despite research showing AUv2 is the current industry standard, the strategic decision is to:
- ✅ **Continue with AUv3** as the forward-looking format
- ❌ **Skip AUv2** - considered a "dead format" for new development
- 🎯 **Target greenfield plugins** and complete ports from other systems

**Reasoning:**
- AUv3 is the modern, future-proof format
- Any plugins built with nih-plug for AU will be new or complete ports
- Better to invest in the future than support legacy formats
- 38 iterations of AUv3 work should not be abandoned

## What We Know (Iteration 39 Research)

### Root Cause of Plugin Recognition Failure

**Current (Wrong) Deployment:**
```
~/Library/Audio/Plug-Ins/Components/NIHPlugAUv3Host.app/  ❌
```

**Correct Deployment:**
```
/Applications/YourPlugin.app/  ✅
├── Contents/
│   ├── Info.plist (host app)
│   ├── MacOS/
│   │   └── YourPlugin (minimal executable)
│   └── PlugIns/
│       └── YourPluginAU.appex/
```

### Why Current Deployment Fails

1. **Wrong location:** `~/Library/Audio/Plug-Ins/Components/` is ONLY for AUv2 `.component` bundles
2. **Discovery mechanism:** AUv3 uses PlugInKit (requires apps in `/Applications`)
3. **Registration:** App must be launched once to register the .appex with PlugInKit
4. **System requirement:** macOS only discovers extensions inside launched applications

### What Works Already (Iterations 1-38)

✅ **Build Infrastructure:**
- Host app bundle structure
- Swift runtime libraries embedded
- Static library linking with FFI symbols
- Swift module files copying
- Code signing
- Bundler automation

✅ **Plugin Implementation:**
- Swift AUAudioUnit subclass
- FFI integration (Rust ↔ Swift)
- Parameter system
- Audio processing
- State serialization

**The only issue:** Wrong deployment location and registration mechanism.

## Next Steps: Iteration 40+

### Immediate Priority: Fix AUv3 Deployment

**Iteration 40 should implement:**

1. **Change installation location**
   ```rust
   // In nih_plug_xtask/src/lib.rs
   let install_path = PathBuf::from("/Applications")
       .join(format!("{}.app", plugin_name));
   ```

2. **Update bundler to install to /Applications**
   - Modify `install_auv3_plugin()` or equivalent function
   - Copy entire host app bundle to `/Applications`
   - Handle permissions (may need sudo for system-wide install)

3. **Add registration step**
   ```bash
   # After installation, launch the app to trigger PlugInKit registration
   open "/Applications/YourPlugin.app"
   ```

4. **Update documentation**
   - Document that AUv3 plugins appear in /Applications
   - Explain the launch-once registration requirement
   - Provide uninstall instructions (drag to trash)

### Expected Timeline

**Iteration 40:** Implement /Applications deployment (1 iteration)
**Iteration 41:** Test plugin recognition with correct deployment (1 iteration)
**Iteration 42:** Run pluginval validation (1 iteration)
**Iteration 43:** Polish and documentation (1 iteration)

**Total:** ~4 iterations to complete AUv3 support

### Success Criteria

**Plugin recognized when:**
```bash
# After installation and launch
pluginkit -m -v | grep -i yourplugin  # Shows plugin
auval -a | grep -i yourplugin          # Lists plugin
auval -v aufx PLUG MFGR                # Validates successfully
```

**Plugin works in Logic Pro** (primary target DAW for AUv3)

## Current Codebase State

### Completed Components

**Location:** `src/wrapper/auv3/`
- ✅ `ffi.rs` - Complete FFI layer
- ✅ Swift AUAudioUnit subclass
- ✅ Swift bridging header with FFI declarations
- ✅ Parameter integration
- ✅ Audio processing (internalRenderBlock)
- ✅ State management (fullState)

**Location:** `nih_plug_xtask/src/lib.rs`
- ✅ AUv3 bundler with host app structure
- ✅ Static library support
- ✅ Swift runtime embedding
- ✅ Swift module copying
- ✅ Code signing
- ✅ Universal binary creation

**Location:** `src/wrapper/auv3/swift/`
- ✅ Xcode project
- ✅ Swift source files
- ✅ Build configuration
- ✅ Info.plist templates

### What Needs Changing

**Only location:** `nih_plug_xtask/src/lib.rs`
- Change installation path from `~/Library/Audio/Plug-Ins/Components/` to `/Applications/`
- Add optional launch step to trigger registration
- ~50 lines of code change

**Everything else works correctly.**

## Important Notes for Next Session

### AUv3 Deployment Requirements

1. **Installation location:** `/Applications/` (not Components directory)
2. **Registration:** Launch app once via `open` command
3. **Discovery:** PlugInKit finds extensions in launched apps
4. **Unregistration:** Trashing the app unregisters the plugin

### Testing Commands

```bash
# Build and install
cargo xtask bundle-universal gain --release

# Verify installation
ls -la /Applications/*.app/Contents/PlugIns/*.appex

# Launch to register
open /Applications/YourPlugin.app

# Verify registration
pluginkit -m -v | grep -i yourplugin

# Test recognition
auval -a | grep -i nplg
auval -v aufx Nplg NPLG

# Validate
pluginval --strictness-level 5 --validate-in-process --verbose /Applications/YourPlugin.app/Contents/PlugIns/*.appex
```

### DAW Support Reality

**Works with AUv3:**
- ✅ Logic Pro (primary target)
- ✅ GarageBand
- ✅ MainStage
- ✅ Reaper
- 🟡 Ableton Live (limited, Apple Silicon only)

**Does NOT work with AUv3:**
- ❌ Studio One
- ❌ Digital Performer
- ❌ Pro Tools (AAX only)

**This is acceptable** - AUv3 targets modern Apple ecosystem.

## Orchestrator Instructions

### When Resuming (Iteration 40)

**Context files to read:**
1. `./handoffs/iteration_039_decision.md` (this file)
2. `./handoffs/iteration_039.md` (research findings)
3. `./fix_plan.md` (task list)
4. `./.ralph/current_iteration.txt` (should be 40)

**Task for iteration 40:**
"Modify bundler to install AUv3 plugins to /Applications instead of ~/Library/Audio/Plug-Ins/Components/"

**Subtasks:**
1. Change installation path in `nih_plug_xtask/src/lib.rs`
2. Update install function to copy to /Applications
3. Add optional launch step for registration
4. Test plugin recognition after deployment
5. Update documentation

**Expected outcome:**
- Plugin installs to `/Applications/PluginName.app`
- Plugin can be launched to trigger registration
- Plugin appears in `pluginkit -m -v` output
- Plugin recognized by `auval -a`

### Iteration Count

- **Current:** 39 (research + decision)
- **Next:** 40 (implement /Applications deployment)
- **Estimated completion:** Iteration 43 (~4 more iterations)

## References

**Key research documents:**
- `./handoffs/iteration_039.md` - Comprehensive AUv3/AUv2 research
- `./AUV3_DEPLOYMENT_ISSUE.md` - Original deployment issue documentation

**Critical finding:**
> "The plug-in (.appex) is registered by pluginkit when the parent application launches by double clicking, or running from the Xcode debugger."

**Installation location requirement:**
> "All the AUv3's downloaded from the Appstore store the AUv3 Plug-in inside the App Bundle in the Applications folder."

## Status Summary

**Phase 8 (Validation):** IN PROGRESS - Blocked on deployment fix

**Blockers removed:** None - clear path forward

**Confidence:** HIGH - We know exactly what needs to be fixed

**Risk:** LOW - Small, well-understood change required

---

**Ready for iteration 40 when orchestrator restarts.**
