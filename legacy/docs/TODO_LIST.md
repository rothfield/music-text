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

6. ~~**Document & Code Blocks Architecture Refactor** - Implement clean separation following HTML + code blocks pattern:~~ ✅ **COMPLETED**
   - ✅ Written refactor plan to `REFACTOR_PLAN_DOCUMENT_CODE_BLOCKS.md`
   - ✅ Renamed `src/document/manual_parser/` → `src/document/document_parser/`
   - ✅ Create `src/stave/` and move `src/stave_parser.rs` → `src/stave/parser.rs`
   - ✅ Create `src/rhythm/` and move `src/rhythm_fsm.rs` → `src/rhythm/analyzer.rs`
   - ✅ Update all import paths throughout codebase
   - ✅ Add "no mod.rs files" rule to README coding guidelines
   - ✅ Test build and functionality after refactor

7. **Extract old_models.rs to Logical Module Structure** - Eliminate confusing "old_models" terminology: 🔄 **IN PROGRESS**
   - ✅ Create `src/rhythm/types.rs` with core processing types
   - ❌ Create `src/rhythm/converters.rs` with rhythm utilities
   - ❌ Update `src/document/model.rs` with spatial annotations 
   - ❌ Update module exports and imports throughout codebase
   - ❌ Remove `old_models.rs` file and validate functionality

8. **Consolidate Duplicate Types and Improve Type Organization** - Address architectural inconsistencies from old_models extraction:
   - **Position struct duplication**: Unify `document::Position` and `rhythm::Position` 
   - **Type placement review**: Evaluate if SlurRole, BarlineType belong in current locations
   - **Import dependency analysis**: Ensure no circular dependencies between modules
   - **API consistency**: Review public interfaces for logical coherence
   - **Documentation**: Add module-level docs explaining type organization philosophy

9. **Review process_rhythm_batch Implementation** - Analyze rhythm processing pipeline for optimization opportunities:
   - **Current behavior**: Examine how `process_rhythm_batch()` handles multiple staves
   - **Performance analysis**: Check if batch processing provides actual benefits over individual processing
   - **Architecture alignment**: Ensure rhythm processing fits cleanly with document/stave/rhythm separation
   - **Context propagation**: Verify if cross-stave rhythm context (ties, extensions) is properly handled
   - **API simplification**: Consider if batch vs individual processing APIs can be consolidated

10. **Refactor app.js** - Clean up and organize the JavaScript code in the web interface:
   - **Code organization**: Split large functions, improve modularity
   - **Font management**: Clean up font-related code after font file reduction
   - **Event handling**: Review and optimize event listeners
   - **Error handling**: Improve error handling and user feedback
   - **Performance**: Optimize DOM updates and parsing calls

11. **Create dedicated pitch_parser.rs module** - Consolidate scattered pitch parsing logic into a clean, maintainable module:
   - **Extract from multiple files**: Move pitch parsing logic from `document/model.rs`, `document/document_parser/content_line.rs`, and `models/pitch.rs`
   - **Context-aware parsing**: Centralize notation system detection and ambiguous character resolution (G→Ga vs G→Western G)
   - **Comprehensive pitch support**: Handle all notation systems (Number, Western, Sargam, Bhatkhande, Tabla) in one place
   - **Accidental parsing**: Support #, b, bb, ## modifiers with proper validation
   - **Multi-character pitches**: Handle tabla syllables like "dha", "dhin", etc.
   - **Clean API**: Provide simple, well-tested functions for pitch string → PitchCode conversion
   - **Unit testing**: Add comprehensive tests for all notation systems and edge cases

12. **Understand and clarify classifier.rs module** - Investigate actual purpose and consider renaming:
   - **Current confusion**: Name suggests it classifies content lines or notation systems
   - **Actual purpose**: Only classifies annotation lines (pre/post content) as upper/lower/lyrics
   - **Parser responsibility**: `identify_content_line()` in parser identifies musical content lines
   - **Tokenizer responsibility**: `classify_notation_system()` detects notation systems (Western, Sargam, etc.)
   - **Consider renaming**: `AnnotationClassifier` or `LineTypeClassifier` for clarity
   - **Document**: Add clear module-level documentation explaining its limited scope
