# nih-plug - Overview

## Project Description

Add Audio Unit (AU) support to NIH-Plug.

This is for a pull request to https://github.com/robbert-vdh/nih-plug

Work in phases that end at human testing checkpoints. Each phase should:
- Implement one testable milestone
- Stop when human DAW testing is needed
- Document what needs testing
- Reaching a testing checkpoint IS success for that iteration

Follow NIH-Plug's existing patterns and abstractions.
Research the codebase to understand how VST3/CLAP work, then apply similar patterns to AU.

This should be production-quality code suitable for upstream merge.


## Goals

List the main goals of this project here.

## Architecture

### Audio Unit (AUv2) Implementation

NIH-plug's AU support follows the same architectural pattern as VST3 and CLAP:

1. **Wrapper Layer** (`src/wrapper/au/`)
   - Translates between AU C API and NIH-plug's Rust Plugin trait
   - Handles AU component lifecycle (factory, open, close, render)
   - Manages parameter mapping and automation
   - Provides AU-specific contexts for initialization and processing

2. **Component Structure**
   - `AudioComponentPlugInInstance` - The main plugin instance struct
   - `AudioComponentPlugInInterface` - Function table with AU callbacks
   - Factory function - Creates instances from AudioComponentDescription

3. **Bundle Distribution**
   - Plugins are distributed as `.component` bundles
   - Contains dylib, Info.plist, and optional resources
   - Info.plist specifies component type, codes, and factory function

## Key Features

- Feature 1
- Feature 2
- Feature 3

## Technical Decisions

Document important technical decisions here as you make them.
