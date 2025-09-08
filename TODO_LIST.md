# TODO List

## High Priority

1. **Purge semitones use old transposition system** - Remove semitone-based transposition system and revert to the old transposition system that was working

2. ~~**Execute Git Repository Reorganization Plan** - Implement the structured commit strategy outlined in `GIT_REORG_PLAN.md`:~~ ✅ **COMPLETED**
   - ~~Phase 1: Infrastructure commit (parsing, pitch coverage, converters)~~
   - ~~Phase 2: API enhancement commit (notation-to-SVG endpoint)~~
   - ~~Phase 3: Documentation commit (research notes)~~
   - ~~Clean up build artifacts and update .gitignore if needed~~

3. ~~**Integrate beat processing with rhythm_fsm and output renderers** - Connect the flat element parsing with beat-aware rhythm processing:~~ ✅ **COMPLETED**
   - ~~Integrate rhythm_fsm_v2.rs to process parsed elements into beat structures~~
   - ~~Update VexFlow converter to handle beat-grouped elements and dash extension logic~~
   - ~~Update LilyPond converter to handle beat-grouped elements and dash extension logic~~
   - ~~Ensure dash elements are properly interpreted as extensions vs rests per specification~~

4. **Fix Web UI LilyPond SVG Generation** - Correct architectural separation between CLI and web UI: 🔄 **IN PROGRESS**
   - ✅ Fixed LilyPond syntax error (nested Staff blocks)
   - ✅ Added infinite loop protection in tuplet calculation  
   - ❌ SVG generation endpoint still times out (parse endpoint works fine)
   - **Next**: Debug specific hang in SVG generation flow

5. ~~**Clean Up Dead Code and Unused Imports** - Remove all unused code and fix build warnings:~~ ✅ **COMPLETED**
   - ~~Remove unused imports: `Stave`, `PitchCode`, `GenerationResult`~~
   - ~~Remove dead functions: `convert_beat_element_to_lilypond`, `subdivisions_to_lilypond_duration`, `cleanup_old_files`, `word_wrap`~~
   - ~~Remove unused struct fields: `wrap_width`, `shared_rules`, `pitch_definition`~~
   - ~~Fix unused variables: `complete_pitch`, `base_pitch`, `octave`~~
   - ~~Remove unreachable patterns and dead enum variants: `InBeat`~~
   - ~~Remove old_models references and unify data structures~~
