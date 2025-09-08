# TODO List

## High Priority

1. **Purge semitones use old transposition system** - Remove semitone-based transposition system and revert to the old transposition system that was working

2. **Execute Git Repository Reorganization Plan** - Implement the structured commit strategy outlined in `GIT_REORG_PLAN.md`:
   - Phase 1: Infrastructure commit (parsing, pitch coverage, converters)
   - Phase 2: API enhancement commit (notation-to-SVG endpoint)
   - Phase 3: Documentation commit (research notes)
   - Clean up build artifacts and update .gitignore if needed

3. **Integrate beat processing with rhythm_fsm and output renderers** - Connect the flat element parsing with beat-aware rhythm processing:
   - Integrate rhythm_fsm_v2.rs to process parsed elements into beat structures
   - Update VexFlow converter to handle beat-grouped elements and dash extension logic
   - Update LilyPond converter to handle beat-grouped elements and dash extension logic
   - Ensure dash elements are properly interpreted as extensions vs rests per specification
