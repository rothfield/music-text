# Doremi-Script Grammar Reference Files

This directory contains reference files from the original doremi-script project to help complete the grammar migration to music-text.

## Files:

- `doremi_script_grammar.ebnf` - Original EBNF grammar (Clojure data format)
- `grammar_compiler.clj` - Shows how the original grammar was generated
- `test_patterns/` - Key test patterns that should work in music-text

## Purpose:

The current music-text hand-written parser has been migrated from PEST grammar.

Use these files to:
1. Compare working vs broken patterns
2. Validate test coverage
3. Debug parsing issues

## Usage:

```bash
# Test a pattern from doremi-script
./target/release/music-txt --input "$(cat src/reference/test_patterns/slur.txt)"

# Compare with original doremi-script behavior
```

Generated: $(date)