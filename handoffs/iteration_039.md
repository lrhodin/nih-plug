# Iteration 039 Handoff

## Confidence Score: N/A (Research Iteration)

Comprehensive research completed on AUv3 macOS support. Critical architectural decision required.

## Context Usage
- Tasks completed this iteration: 1 (comprehensive AUv3/AUv2 research)
- Token usage: Web search research agent
- Reason for handoff: Critical architectural decision required

## Health Check
- [x] All tests passing? N/A - research iteration
- [x] Any known issues or bugs? Root cause of plugin recognition identified
- [x] Any incomplete implementations? AUv3 deployment strategy is incorrect
- [x] Code follows project conventions? N/A - research iteration

## Key Learnings

### CRITICAL FINDING: AUv3 IS Supported on macOS

**Confirmed:** AUv3 is fully supported on macOS since OS X 10.11 (2015)

**However:** Our deployment strategy is fundamentally wrong.

### Root Cause of Plugin Recognition Failure

**Current (Wrong) Approach:**
```
~/Library/Audio/Plug-Ins/Components/NIHPlugAUv3Host.app/
```

**Correct Approach:**
```
/Applications/YourPlugin.app/
├── Contents/
│   └── PlugIns/
│       └── YourPluginAU.appex/
```

**Why it doesn't work:**
1. `~/Library/Audio/Plug-Ins/Components/` is **ONLY for AUv2 `.component` bundles**
2. AUv3 uses **PlugInKit discovery**, not AudioComponent scanning
3. PlugInKit only discovers extensions in **launched applications**
4. Containing apps must be in **`/Applications`** or user-accessible locations

### How AUv3 Actually Works on macOS

**Registration Process:**
1. Install containing app to `/Applications/`
2. **Launch the app once** (triggers PlugInKit registration)
3. System automatically registers the .appex
4. Plugin becomes available to DAWs via AudioComponentFindNext()

**Quote from Research:**
> "The plug-in (.appex) is registered by pluginkit when the parent application launches by double clicking, or running from the Xcode debugger."

### AUv2 vs AUv3 Trade-offs

**AUv2 (Traditional .component bundles):**
- ✅ **Simpler deployment**: Copy to `/Library/Audio/Plug-Ins/Components/`
- ✅ **Universal DAW support**: Logic, Ableton, Pro Tools, Reaper, etc.
- ✅ **No containing app required**
- ✅ **Automatic discovery** - no registration step
- ✅ **Standard plugin format** - familiar to users
- ✅ **Partially implemented** in `src/wrapper/au/`
- ❌ In-process loading (less isolation)

**AUv3 (Modern .appex extensions):**
- ✅ Out-of-process loading (better stability)
- ✅ MIDI output support
- ✅ Better sandboxing
- ❌ **Complex deployment** (requires /Applications + launch step)
- ❌ **Limited DAW support** (Logic, some others - NOT Ableton)
- ❌ **User confusion** (plugin appears as app)
- ❌ **Must launch app** at least once
- ❌ **38 iterations** spent on deployment issues

## Architectural Decision Required

### Option A: Fix AUv3 Deployment

**Required Changes:**
1. Change output location to `/Applications/`
2. Create minimal host app executable (Swift stub)
3. Add post-installation launch step to trigger registration
4. Handle admin privileges for `/Applications` installation

**Pros:**
- Leverages existing 38 iterations of AUv3 work
- Modern plugin format

**Cons:**
- Additional 5-10 iterations for deployment fixes
- Complex build system changes
- Limited DAW support
- Confusing UX (plugin as app)
- Requires app launch for registration

### Option B: Pivot to AUv2 (RECOMMENDED BY RESEARCH)

**Required Changes:**
1. Complete the AUv2 wrapper in `src/wrapper/au/` (partially done)
2. Implement missing callbacks (render, properties, parameters)
3. Add bundler support for `.component` creation
4. Copy to `/Library/Audio/Plug-Ins/Components/`

**Pros:**
- Simpler deployment model
- Universal DAW compatibility
- Standard plugin UX
- Automatic discovery (no registration)
- Estimated 10 iterations to complete
- **Not deprecated** - fully supported by Apple

**Cons:**
- 38 iterations of AUv3 work becomes research/learning
- In-process loading (minor issue)

### Option C: Hybrid Approach

1. **Phase 1**: Implement AUv2 (primary format)
2. **Phase 2**: Optionally add AUv3 for advanced features

**Pros:**
- Universal compatibility via AUv2
- Optional modern features via AUv3
- Best of both worlds

## Industry Practice

**From Research:**
- **JUCE**: Supports both, defaults to AUv2
- **iPlug2**: Supports both formats
- **Most commercial plugins**: Ship AUv2 as primary, AUv3 optional
- **Community consensus**: "AUv2 is currently recommended for most scenarios"

## Estimated Completion Time

**Option A (Fix AUv3):** 5-10 additional iterations
- Deployment fixes
- Registration mechanism
- Installation handling

**Option B (Implement AUv2):** ~10 iterations
- Complete wrapper (5-8 iterations)
- Bundler support (2-3 iterations)
- Testing (2 iterations)

**Both options take similar time, but AUv2 provides better UX and compatibility.**

## Next Focus

### Immediate Priority: Architectural Decision

**This requires human input:**
1. Should we fix AUv3 deployment to `/Applications`?
2. Should we pivot to AUv2 for simpler deployment?
3. Should we implement both formats?

**Recommendation:** Pivot to AUv2 based on:
- Simpler deployment model
- Better DAW compatibility
- Standard plugin UX
- Similar development time
- Research strongly supports this

## Technical Details

### If AUv3 is chosen:
```bash
# Correct installation location
/Applications/YourPlugin.app/

# Registration command
open /Applications/YourPlugin.app

# Verification
pluginkit -m -v | grep -i yourplugin
```

### If AUv2 is chosen:
```bash
# Installation location
/Library/Audio/Plug-Ins/Components/YourPlugin.component/

# No registration needed - automatic discovery
```

## Final Thoughts

After 38 iterations, we've discovered that AUv3 **IS** supported on macOS, but our deployment strategy was fundamentally incorrect. The plugin recognition failure is due to:
1. Wrong installation location (should be `/Applications`, not `~/Library/Audio/Plug-Ins/Components/`)
2. Missing registration step (app must be launched once)

We have two viable paths forward:
- **Fix AUv3**: Change to `/Applications` deployment
- **Pivot to AUv2**: Simpler, better compatibility (recommended)

**This is a strategic decision that will determine the next 10+ iterations.**

---

**Context usage at handoff**: Research complete

**Git status**: Clean (no code changes this iteration)

**Compilation status**: N/A (research iteration)

**Phase 8 Status**: ⚠️ BLOCKED - Architectural decision required before proceeding
