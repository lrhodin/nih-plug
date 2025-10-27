# Audio Unit (AU) Support Implementation Plan

## Phase 1: Foundation & Research (CHECKPOINT: Can compile AU skeleton)
- [x] Research VST3/CLAP wrapper patterns
- [x] Set up AU dependencies (AudioToolbox framework bindings)
- [x] Create basic AU wrapper structure (src/wrapper/au/)
- [x] Create nih_export_au!() macro skeleton
- [x] Compiles successfully with AU feature
- [ ] Test: Can compile a minimal AU plugin that loads in Logic/GarageBand

## Phase 2: Core Audio Processing (CHECKPOINT: Can process audio)
- [x] Implement AU component factory structure
- [x] Create AudioComponentPlugInInstance and interface
- [x] Document bundle structure and Info.plist requirements
- [x] Define AU selectors and create selector dispatch system
- [x] Implement lookup callback with selector routing
- [x] Implement Initialize/Uninitialize callbacks
- [x] Implement GetProperty/SetProperty for basic properties
- [x] Implement Reset callback
- [ ] Implement audio rendering (Render callback)
- [ ] Implement buffer handling for AU format
- [ ] Test: Plugin processes audio in DAW

## Phase 3: Parameter System (CHECKPOINT: Parameters work in DAW)
- [ ] Map NIH-plug parameters to AU parameters
- [ ] Implement parameter change notifications
- [ ] Implement parameter automation
- [ ] Implement preset/state save/load
- [ ] Test: Can automate parameters in DAW

## Phase 4: MIDI & Events (CHECKPOINT: MIDI works)
- [ ] Implement MIDI input handling
- [ ] Implement MIDI output (if needed)
- [ ] Implement note event translation
- [ ] Test: Plugin responds to MIDI

## Phase 5: GUI Support (CHECKPOINT: GUI displays)
- [ ] Implement AU view/editor integration
- [ ] Connect existing NIH-plug editors to AU views
- [ ] Handle window lifecycle
- [ ] Test: GUI opens and responds in DAW

## Phase 6: Bundle & Distribution
- [ ] Update nih_plug_xtask bundler for AU format
- [ ] Create .component bundle structure
- [ ] Add Info.plist generation
- [ ] Test: Can install and load plugin system-wide

## Phase 7: Polish & Testing
- [ ] Add comprehensive tests
- [ ] Add documentation
- [ ] Test in multiple DAWs (Logic, GarageBand, Ableton, etc.)
- [ ] Performance optimization
- [ ] Create auditor program

## Notes
- Start with AUv2 format (most compatible)
- Consider AUv3 for future enhancement
- Follow existing VST3/CLAP wrapper patterns closely
- Each phase ends with a DAW testing checkpoint
