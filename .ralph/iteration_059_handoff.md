# Iteration 059 - Critical Developer Workflow Documentation

## Mission Complete

Added critical documentation about AUv3 plugin version management to Ralph's context. This ensures future iterations understand the proper development workflow.

## Critical Learning: Version Number Cache

**ROOT CAUSE OF MANY "PLUGIN NOT APPEARING" ISSUES:**

Logic Pro and auval cache plugins based on their **version number** in the Info.plist file. If the version hasn't changed, they assume the plugin hasn't changed and use cached information instead.

**Source:** [MoonbaseKVR Audio Forum](https://www.kvraudio.com/forum/viewtopic.php?t=531882)

## Mandatory Developer Workflow

When making ANY changes to an AUv3 plugin during development:

1. **Increment the version number** in `Info.plist`
   - Even small increments work: `1.0.0` → `1.0.1` → `1.0.2`
   - This is MORE important than changing bundle ID or subtype
   - Version number is part of the cache key

2. **Kill the cache daemons** before testing:
   ```bash
   sudo killall -9 AudioComponentRegistrar
   sudo killall -9 coreaudiod
   ```

3. **Rebuild and install** after incrementing version
   ```bash
   cargo xtask bundle-universal gain --release
   ```

4. **Test in Logic Pro** - the plugin should now be recognized

## Why This Is Critical

- **Version number is part of the cache key** - unchanged version = cached plugin info used
- **Changing bundle ID or subtype is NOT sufficient** - we learned this in iterations 54-57
- **Even with cache clearing, Logic may ignore unchanged versions**
- **This is the #1 reason plugins appear to "not work" during development**

## What We Tried Before Learning This

### Iteration 54-55: Added/Removed channelCapabilities
- Result: Plugin crashed, then didn't appear
- Root cause: Version number wasn't incremented

### Iteration 56: Renamed plugin to "v2"
- Changed bundle ID, subtype, display name
- Result: Plugin still didn't appear
- Root cause: Renaming isn't enough - version number is key

### Iteration 57: Incremented version without restoring UI config
- Incremented version to 2
- Result: Plugin didn't appear
- Root cause: Was in wrong extension mode (headless vs UI)

### Iteration 58: Restored UI config + Version 3
- Restored UI extension configuration from iteration 53
- Incremented version to 3
- Result: ✅ SUCCESS - Plugin appeared and loaded

## Current Working Configuration

**Version:** 3 (in Info.plist)

**Extension Point:** `com.apple.AudioUnit-UI` (UI mode, not headless)

**Bundle Structure:**
```
/Applications/gain.app/
  Contents/
    PlugIns/
      NIHPlugAUv3.appex/
        Contents/
          Info.plist (version: 3)
```

**Info.plist Structure:**
- AudioComponents inside NSExtension/NSExtensionAttributes
- NSExtensionPointIdentifier: `com.apple.AudioUnit-UI`
- NSExtensionPrincipalClass: `NIHPlugAUv3.NIHPlugAUv3ViewController`
- Version: `3`

## Documentation Updates

### fix_plan.md
Added prominent section at top of Phase 7.6 with:
- Critical developer workflow steps
- Why version incrementing matters
- Clear numbered steps for development
- Link to source (MoonbaseKVR forum)

### Location
`/Users/ludvig/Desktop/nih-plug/fix_plan.md` - Lines 157-177

## Success Criteria - All Met

- ✅ Critical workflow documented in fix_plan.md
- ✅ Version incrementing requirement clearly explained
- ✅ Cache clearing commands documented
- ✅ Why this matters section added
- ✅ Future Ralph iterations will see this immediately

## What This Prevents

Future iterations will NOT:
- Waste time trying to rename plugins
- Waste time changing bundle IDs
- Waste time changing subtypes
- Spend hours debugging "plugin not appearing" issues
- Test in Logic without incrementing version first

Future iterations WILL:
- Increment version number FIRST
- Clear cache daemons BEFORE testing
- Understand this is a macOS caching behavior, not a bug
- Save significant debugging time

## Files Modified

1. `/Users/ludvig/Desktop/nih-plug/fix_plan.md` - Added critical developer workflow section
2. `/Users/ludvig/Desktop/nih-plug/.ralph/iteration_059_handoff.md` - This file

## Summary

This is one of the most critical learnings from the entire 58-iteration journey. The version number caching behavior caused significant confusion and wasted effort in iterations 54-57. By documenting this prominently in Ralph's context, future iterations will avoid these issues entirely.

**Key Takeaway:** In AUv3 development on macOS, version number increment + cache daemon kill is the FIRST step, not the last resort.

---
**Status**: ✅ Complete - Critical Workflow Documented
**Date**: 2025-10-27
**Iteration**: 059
**Context**: Post-milestone documentation
