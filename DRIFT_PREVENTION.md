# Drift Prevention Strategy

This document outlines our comprehensive strategy to prevent drift between the CLI binary and WASM/webapp implementations.

## 🎯 Goals

1. **Identical Core Logic**: Both CLI and WASM use the same core parsing functions
2. **Consistent Output**: Identical inputs produce identical outputs across platforms
3. **Automated Testing**: Continuous verification of consistency
4. **Early Detection**: Catch drift before it reaches production

## 🏗️ Architecture

### Shared Core Library
Both CLI and WASM use the same core functions from `src/lib.rs`:

- **CLI**: Uses `unified_parser()` directly
- **WASM**: Uses `parse_notation()` which calls `parse_notation_internal()` which uses `unified_parser()`

### Key Shared Functions
- `unified_parser()` - Core parsing logic
- `convert_to_lilypond()` - LilyPond generation
- `generate_outline()` - Outline generation
- `colorize_string()` - Syntax highlighting
- Vexflow conversion functions

## 🧪 Testing Strategy

### 1. Automated Cross-Platform Tests

**Rust Integration Tests** (`tests/cross_platform_tests.rs`):
- Tests core `unified_parser()` function directly  
- Tests WASM functions (`parse_notation()`, `get_lilypond_output()`)
- Verifies identical outputs for identical inputs
- Performance benchmarking to catch regressions

**Shell Script Tests** (`test_cross_platform_consistency.sh`):
- Tests CLI binary vs Web API endpoints
- Real-world simulation of both interfaces
- Runs same test cases against both platforms

### 2. Continuous Integration

**GitHub Actions** (`.github/workflows/cross-platform-consistency.yml`):
- Runs on every push/PR
- Daily scheduled runs
- Tests both CLI and web server
- Uploads artifacts on failure for debugging

**Pre-commit Hooks** (`.git/hooks/pre-commit`):
- Runs consistency tests before commits
- Only triggers when core parsing files are modified
- Prevents inconsistent code from being committed

### 3. Development Workflow

**Makefile targets**:
```bash
make test-consistency    # Run all consistency tests
make quick-check        # Fast consistency verification  
make ci                 # Full CI pipeline locally
```

## 📋 Test Cases

Our consistency tests cover these scenarios:

1. **Simple Sargam**: `| S R G M |` → `c4 d4 e4 fs4`
2. **Western notation**: `| C D E F |` → `c4 d4 e4 f4`  
3. **Number notation**: `| 1 2 3 4 |` → `c4 d4 e4 f4`
4. **Multi-line with metadata**: Title/Author + notation
5. **Chromatic notes**: Sharps and flats handling
6. **Octave markers**: Dot notation for octaves

## 🚨 Drift Detection

### What We Monitor
- **LilyPond Output**: Note sequences, durations, formatting
- **System Detection**: Sargam vs Western vs Numbers
- **VexFlow Output**: JSON structure and note data
- **Error Handling**: Consistent error messages
- **Performance**: Timing differences between platforms

### Automatic Alerts
- CI fails if consistency tests fail
- Pre-commit hooks prevent bad commits
- Daily CI runs catch environmental drift

## 🔧 Development Guidelines

### When Making Changes

1. **Always run consistency tests**: `make test-consistency`
2. **Test both platforms manually** when changing core logic
3. **Update test cases** when adding new features
4. **Document breaking changes** that affect both platforms

### Adding New Features

1. **Implement in core library first** (`src/lib.rs`)
2. **Expose via WASM bindings** with `#[wasm_bindgen]`
3. **Add to CLI** if relevant
4. **Add test cases** to `cross_platform_tests.rs`
5. **Update shell script tests** if needed

### Code Review Checklist

- [ ] Core logic changes are in shared library
- [ ] Both CLI and WASM use same core functions
- [ ] New test cases added for new functionality
- [ ] Consistency tests pass locally
- [ ] Performance impact considered

## 🚀 Usage

### Running Consistency Tests

```bash
# Full test suite
make test-consistency

# Quick check during development  
make quick-check

# Test specific functionality
cargo test cross_platform_tests::test_wasm_functions_consistency

# Test with running server
./test_cross_platform_consistency.sh
```

### CI Integration

The GitHub Actions workflow automatically:
1. Builds both CLI and WASM
2. Starts the web server
3. Runs all consistency tests
4. Reports failures with detailed logs

## 📈 Monitoring

### Success Metrics
- **100% test pass rate** on consistency tests
- **Zero drift incidents** in production
- **Fast feedback** (tests complete in <2 minutes)

### Warning Signs
- Tests failing intermittently
- Performance degradation on one platform
- Different error messages between platforms
- Manual testing reveals differences

## 🔄 Maintenance

### Regular Tasks
- Review test cases monthly for completeness
- Update test expectations when intentionally changing behavior  
- Monitor CI performance and optimize if needed
- Add new test cases for user-reported edge cases

### When Drift is Detected
1. **Stop development** - fix drift before continuing
2. **Investigate root cause** - why did tests not catch it?
3. **Fix the inconsistency** - usually in shared core logic
4. **Improve tests** - add cases to prevent regression
5. **Review process** - strengthen prevention measures

---

This strategy ensures that users get consistent behavior whether they use the CLI tool or the web interface. The automated testing provides confidence that both platforms remain synchronized as the codebase evolves.