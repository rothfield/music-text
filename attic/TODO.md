# TODO List

## High Priority

### 🔴 Grammar Refactor (Critical)
- [ ] Implement multi-phase parsing architecture to fix "1\n." parsing failure
- [ ] Create position-aware grammars (`upper_grammar` vs `lower_grammar`)
- [ ] Build `RawStave` → classification → `Stave` pipeline
- [ ] Review implementation plans in:
  - `TECH_NOTE_GRAMMAR_REFACTOR.md`
  - `DETAILED_CODE_PLAN_GRAMMAR_REFACTOR.md`
  - `GRAMMAR_REFACTOR_CRITIQUE.md`

### 🎹 WYSIWYG Editor
- [ ] Review architecture proposal in `WYSIWYG_ARCHITECTURE_PROPOSAL.md`
- [ ] Design visual slur editing interface
- [ ] Implement spatial format generator from visual input
- [ ] Integrate with existing `unified_parser()` pipeline

## Medium Priority

### 🧪 Testing & Quality
- [ ] Expand Playwright test coverage for edge cases
- [ ] Add tests for tuplet rhythm parsing
- [ ] Test all notation systems (Western, Sargam, Number)
- [ ] Verify tonic transposition for all keys

### 🎵 Feature Enhancements
- [ ] Add support for dynamics notation
- [ ] Implement tempo markings
- [ ] Add articulation marks (staccato, accent, etc.)
- [ ] Support for time signature changes

## Low Priority

### 📚 Documentation
- [ ] Create user guide for notation syntax
- [ ] Add examples gallery with common patterns
- [ ] Document API endpoints for web server
- [ ] Create contribution guidelines

### 🔧 Technical Debt
- [ ] Optimize FSM performance for large inputs
- [ ] Refactor duration calculation utilities
- [ ] Clean up deprecated V1 code
- [ ] Improve error messages for parse failures

## Completed ✅
- [x] Fix slur positioning in spatial analysis
- [x] Correct tie logic (same pitch only)
- [x] Fix VexFlow crash with slur indexing
- [x] Implement V2 LilyPond tuplet generation
- [x] Set up Playwright testing framework

## Notes
- Grammar refactor is estimated at 2-4 weeks implementation time
- Always test changes in web UI at http://localhost:3000
- Run `npx playwright test` after major changes