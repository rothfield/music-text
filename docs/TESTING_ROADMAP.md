# Testing Roadmap for Music-Text

## Current State Analysis

### Existing Test Structure (from music-text)
```
music-text/
├── test/                         # Clojure unit tests
│   └── music_text/
│       ├── core_test.clj        # Parser tests
│       └── grammar_test.clj     # Grammar tests
│
└── resources/fixtures/           # Test notation files
    ├── sargam_composition/       # 100+ Sargam test cases
    │   ├── slur.txt             # Simple: (S R)
    │   ├── tied_notes.txt       # Simple: S - S
    │   ├── all_tuples.txt       # Complex rhythms
    │   ├── FAILS_*.txt          # Expected failures
    │   └── ...                  # Many specific feature tests
    ├── doremi_composition/       # Western notation tests
    ├── hindi_composition/        # Hindi notation tests
    ├── number_composition/       # Number notation tests
    └── abc_composition/          # ABC notation tests
```

### Strengths of Current Approach
- ✅ **Comprehensive coverage** - 100+ test cases covering edge cases
- ✅ **Feature-focused naming** - Each file tests specific features
- ✅ **Expected failures marked** - Files prefixed with `FAILS_`
- ✅ **Multiple notation systems** - Tests for Sargam, Western, Hindi, Number
- ✅ **Descriptive file names** - Clear what each test validates

### Identified Weaknesses
- No visual regression testing (PNG comparison)
- No performance benchmarking
- No structured test metadata
- No automated baseline comparison
- Missing stress tests and error recovery tests

## Proposed Improvements

### 1. Test Directory Structure (Co-located Baselines)
```bash
tests/
├── regression/           # Baseline tests that must never break
│   ├── basic/           # Core functionality
│   │   ├── one_note.123           # Source notation
│   │   ├── one_note.baseline.ly   # Expected LilyPond output
│   │   └── one_note.baseline.png  # Expected PNG output
│   └── complex/         # Advanced features
│       ├── yesterday.123
│       ├── yesterday.baseline.ly
│       └── yesterday.baseline.png
│
├── features/            # Feature-specific tests
│   ├── slurs/
│   │   ├── basic_slur.123
│   │   ├── basic_slur.baseline.ly
│   │   └── basic_slur.baseline.png
│   ├── ties/
│   ├── octaves/
│   ├── ornaments/
│   └── lyrics/
│
├── performance/         # Performance benchmarks
│   └── large_files/
│
└── test_output/         # Generated during test runs
    ├── features/slurs/
    │   ├── basic_slur.generated.ly
    │   ├── basic_slur.generated.png
    │   └── basic_slur.diff.png
    └── regression/basic/
        ├── one_note.generated.ly
        ├── one_note.generated.png
        └── one_note.diff.png
```

### 2. Test Runner Script Features
```bash
#!/bin/bash
# test_lilypond.sh

# Features to implement:
# - Run all tests or specific categories
# - Generate LilyPond and PNG outputs
# - Compare with baselines
# - Generate HTML report with side-by-side comparisons
# - Track performance metrics
# - Support for --update-baselines flag

# File Cleanup Policy:
# - Keep .generated.ly files (for review)
# - Keep .generated.png files (for visual verification)
# - Delete .diff.png files on successful tests (only keep failures)
# - Optional --clean flag to delete all generated files
```

### 3. Test Metadata Format
Each test should have accompanying metadata:
```yaml
# slur.test.yaml
name: "Basic Slur"
category: slurs
notation: sargam
input: "(S R)"
expected:
  has_slur: true
  note_count: 2
  lilypond_contains: ["(", ")"]
visual_test: true
performance_threshold_ms: 50
```

### 4. Visual Testing Pipeline
```
For each test:
1. Parse notation → LilyPond
2. LilyPond → PNG (via lilypond --png)
3. Compare with baseline PNG (ImageMagick compare)
4. Generate diff image highlighting changes
5. HTML report with side-by-side comparison
```

### 5. Test Harness Implementation (Rust)
```rust
// test_harness.rs
struct TestCase {
    name: String,
    input: String,
    expected_lilypond: Option<String>,
    expected_vexflow: Option<String>,
    visual_baseline: Option<PathBuf>,
    should_fail: bool,
    performance_threshold_ms: Option<u64>,
}

impl TestCase {
    fn run(&self) -> TestResult {
        // 1. Parse
        // 2. Generate outputs
        // 3. Compare with expected
        // 4. Generate visuals
        // 5. Check performance
    }
}
```

## Future TODOs

### Phase 1: Foundation (Week 1-2)
- [ ] Create test directory structure
- [ ] Port existing music-text test cases to new format
- [ ] Implement basic test runner script
- [ ] Set up LilyPond → PNG generation pipeline
- [ ] Create initial baseline PNGs for core tests

### Phase 2: Test Infrastructure (Week 3-4)
- [ ] Implement test metadata YAML parser
- [ ] Create visual diff generation using ImageMagick
- [ ] Build HTML report generator with side-by-side comparisons
- [ ] Add performance benchmarking to test runner
- [ ] Set up baseline update mechanism

### Phase 3: Comprehensive Coverage (Week 5-6)
- [ ] Add stress tests (large files, deeply nested structures)
- [ ] Add error recovery tests (malformed input)
- [ ] Add Unicode support tests
- [ ] Add whitespace handling tests (tabs, CRLF, etc.)
- [ ] Create performance regression suite

### Phase 4: CI/CD Integration (Week 7-8)
- [ ] GitHub Actions workflow for test execution
- [ ] Automated visual diff comments on PRs
- [ ] Performance tracking dashboard
- [ ] Compatibility matrix tracking
- [ ] Automated baseline updates on approved changes

### Phase 5: Advanced Features (Future)
- [ ] Property-based testing (generate random valid notation)
- [ ] Mutation testing (verify test effectiveness)
- [ ] Cross-browser testing for WASM output
- [ ] Audio synthesis testing (verify MIDI output)
- [ ] Accessibility testing for generated output

## Test Categories to Implement

### Core Features
- [ ] Basic note parsing (all notation systems)
- [ ] Octave markers (dots, asterisks)
- [ ] Rhythmic patterns (ties, dashes)
- [ ] Barlines (single, double, repeat)
- [ ] Key signatures
- [ ] Time signatures

### Advanced Features
- [ ] Slurs and phrases
- [ ] Ornaments (mordents, grace notes)
- [ ] Dynamics and articulations
- [ ] Lyrics and syllable alignment
- [ ] Tuplets (triplets, quintuplets, etc.)
- [ ] Chords and harmonies

### Edge Cases
- [ ] Empty files
- [ ] Files with only metadata
- [ ] Very long single lines
- [ ] Deeply nested slurs
- [ ] Mixed notation systems
- [ ] Invalid/malformed input

### Performance Tests
- [ ] 1000+ note files
- [ ] 100+ measure compositions
- [ ] Complex nested structures
- [ ] Memory usage tracking
- [ ] Parse time benchmarks

## Success Metrics
- All regression tests passing
- <100ms parse time for typical files
- <1s parse time for large files
- Visual diffs <5% for minor changes
- 100% backward compatibility with music-text fixtures
- Zero memory leaks in WASM

## Notes
- Prioritize backward compatibility with existing music-text tests
- Visual testing is critical for ensuring LilyPond output quality
- Performance benchmarks should run on consistent hardware
- Consider using property-based testing for edge case discovery
- Maintain a CHANGELOG for test suite evolution