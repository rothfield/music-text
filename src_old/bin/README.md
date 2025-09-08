# Test Binaries

This directory contains various test and utility binaries for the notation parser.

## Running Test Binaries

All test binaries are now properly managed by Cargo. To run them:

```bash
# Run a specific test binary
cargo run --bin simple_test

# Or build all binaries
cargo build

# Then run from target directory
./target/debug/simple_test
```

## Available Test Binaries

- `simple_test` - Basic parser functionality test
- `debug_test` - Debug output for parser
- `debug_unified` - Test unified parser
- `debug_document_difference` - Compare document outputs
- `keystroke_test` - Test keystroke parsing
- `quick_test` - Quick parser tests
- `test_notation_detection` - Test notation system detection
- `test_lexer` - Test the lexer component
- `test_fsm_bin` - Test the finite state machine

## Main Binaries

- `cli` - Main command-line interface for the notation parser
- `data_generator` - Generate synthetic test data
- `get_vexflow_fsm` - Extract VexFlow FSM data

## Note

Test outputs are now written to the `test_output/` directory by default when using the CLI with files. You can override this with the `NOTATION_OUTPUT_DIR` environment variable.