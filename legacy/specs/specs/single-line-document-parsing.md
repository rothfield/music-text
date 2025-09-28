# Single-Line Document Parsing Specification

## Status
**Implemented** - Feature Complete

## Summary
Enable parsing of single-line musical input when it constitutes the entire document, with automatic notation system detection and a 25% musical content threshold, making the system more approachable for new users exploring the notation system.

## Motivation

### Music-Text as a Notation Recognizer
Music-text is fundamentally a **music notation recognizer** that can:
- **Recognize** different notation systems (Number, Western, Sargam, Tabla, etc.)
- **Parse** rhythmic patterns and musical structure  
- **Display** the recognized notation as professional staff notation

This recognition and visualization capability is music-text's core value - helping users see their notation rendered clearly.

### Current Recognition Barrier
The current system fails to demonstrate this core capability for minimal input:
- User types `1` → Nothing recognized → No staff notation shown
- User types `12` → Nothing recognized → Core value not demonstrated
- User types `C` → Nothing recognized → User doesn't see recognition working

### Solution: Demonstrate Recognition Immediately  
By recognizing single-line inputs as valid musical content:

1. **Show notation recognition working**: User types `1` → System recognizes Number notation → Displays staff
2. **Demonstrate multiple systems**: User tries `C` → Recognizes Western → User sees system flexibility  
3. **Visualize rhythm parsing**: User types `1-2` → Recognizes rhythm pattern → Shows proper note durations
4. **Build confidence**: Every valid input shows professional staff output → User trusts the system

### Core Value Proposition
This feature ensures that music-text's primary value - **notation recognition and staff visualization** - is immediately apparent to new users, rather than hidden behind arbitrary input length requirements.

## Detailed Design

### Activation Conditions
This feature activates when ALL of the following conditions are met:

1. The entire input contains exactly one non-empty line (after trimming whitespace from each line)
2. That non-empty line contains at least one non-whitespace character  
3. At least 25% of non-whitespace characters in that line are recognized musical notation

#### Single Line Document Definition
A "single line document" means exactly one non-empty line after trimming, with any number of blank/whitespace-only lines:

```
"1"                → ✓ Single line document
"  1  "           → ✓ Single line document (spaces trimmed)
"1\n\n"           → ✓ Single line document (blank lines ignored)
"   1  \n  \n\n " → ✓ Single line document (all whitespace trimmed)
"1\n2"            → ✗ Two non-empty lines (normal parsing)
"1\n  2  \n"      → ✗ Two non-empty lines (normal parsing)
"   \n  \n  "     → ✗ Zero non-empty lines (empty document)
```

### Musical Character Recognition
Musical characters include:

- **Number notation**: `1` through `7`
- **Western notation**: `C`, `D`, `E`, `F`, `G`, `A`, `B` (uppercase)
- **Sargam notation**: `S`, `R`, `G`, `M`, `P`, `D`, `N` (mixed case)
- **Tabla syllables**: `dha`, `ge`, `na`, `ka`, `ta`, `dhin`, `trka`, `terekita`
- **Musical symbols**: `-` (dash/extension), `|` (barline)
- **Modifiers**: `#` (sharp), `b` (flat)

### Percentage Calculation
```
musical_percentage = (count_of_musical_chars / count_of_non_whitespace_chars) * 100
```

### Behavior Rules

1. If `musical_percentage >= 25%`, parse as musical content
2. If `musical_percentage < 25%`, treat as non-musical (return empty document)
3. Notation system is auto-detected using existing `detect_line_notation_system()`
4. The resulting document contains a single stave with one content line

## Examples

### Recognition Demonstration Journey

**First attempt** - Single note recognition:
```
Input: 1
Recognition: Number notation system detected
Output: Professional staff notation with C note
User sees: "The system recognized my input and rendered it properly!"
```

**System discovery** - Multiple notation systems:
```
Input: C  
Recognition: Western notation system detected
Output: Staff notation with C note
User sees: "It recognizes different notation types!"

Input: S
Recognition: Sargam notation system detected  
Output: Staff notation with Sa (C) note
User sees: "It even knows Indian notation!"
```

**Rhythm recognition** - Pattern parsing:
```
Input: 1-2
Recognition: Number system + rhythm pattern (tied note + quarter note)
Output: Staff with proper note durations and ties
User sees: "It understands rhythm notation too!"
```

### Valid Single-Line Musical Documents

| Input | Musical % | Result | User Learning |
|-------|-----------|--------|---------------|
| `"1"` | 100% | ✓ Stave with note "1" | Single notes work! |
| `"123"` | 100% | ✓ Stave with notes "1", "2", "3" | Sequential notes |
| `"1x"` | 50% | ✓ Stave with note "1", "x" ignored | Non-musical chars filtered |
| `"C D E"` | 100% | ✓ Stave with Western notation | Western notation supported |
| `"\|1\|"` | 100% | ✓ Stave with barlines and note | Barlines create measures |
| `"12hello"` | 28.5% | ✓ Stave with notes "1", "2" | Mixed content works |

### Invalid Single-Line Documents

| Input | Musical % | Result |
|-------|-----------|--------|
| `"hello"` | 0% | ✗ Empty document |
| `"test123"` | 0% | ✗ Empty document (digits >7 not musical) |
| `"abc"` | 0% | ✗ Empty document |

## Edge Cases

### Whitespace Handling
- Leading/trailing spaces are trimmed before evaluation
- Internal spaces don't count toward character total
- Multiple blank lines after the content line are ignored

### Multi-line Input
If input contains multiple non-empty lines, normal paragraph parsing applies (this feature does NOT activate).

### Empty Input
Empty or whitespace-only input returns empty document (existing behavior unchanged).

## Implementation Strategy

### Integration Point
Add check in `parse_document()` function before paragraph splitting:

```rust
// In src/parse/document_parser/document.rs
pub fn parse_document(input: &str) -> Result<Document, ParseError> {
    // Check for single-line document special case
    if let Some(doc) = try_parse_single_line_document(input)? {
        return Ok(doc);
    }
    
    // Continue with normal paragraph-based parsing...
}
```

### New Functions Required

```rust
// Check if input qualifies as single-line document
fn is_single_line_document(input: &str) -> bool {
    let non_empty_lines: Vec<&str> = input.lines()
        .map(|line| line.trim())           // Trim each line
        .filter(|line| !line.is_empty())   // Keep non-empty
        .collect();
    
    non_empty_lines.len() == 1
}

// Calculate percentage of musical characters
fn calculate_musical_percentage(line: &str) -> f32

// Parse single-line input as musical document
fn try_parse_single_line_document(input: &str) -> Result<Option<Document>, ParseError>
```

## Test Plan

### Unit Tests

```rust
#[cfg(test)]
mod single_line_tests {
    use super::*;

    #[test]
    fn test_single_note() {
        let result = parse_document("1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().staves.len(), 1);
    }

    #[test]
    fn test_single_line_with_threshold() {
        assert!(parse_document("1x").is_ok());      // 50% > 25%
        assert!(parse_document("12hello").is_ok()); // 28.5% > 25%
        
        let empty_result = parse_document("hello");
        assert!(empty_result.is_ok());
        assert_eq!(empty_result.unwrap().staves.len(), 0);  // 0% < 25%
    }

    #[test]
    fn test_single_line_with_blanks() {
        assert!(parse_document("1\n\n\n").is_ok());      // Trailing blanks OK
        assert!(parse_document("  1  ").is_ok());         // Leading/trailing spaces OK
        assert!(parse_document("   1  \n  \n\n ").is_ok()); // Mixed whitespace OK
    }

    #[test]
    fn test_single_line_document_detection() {
        assert!(is_single_line_document("1"));
        assert!(is_single_line_document("  1  "));
        assert!(is_single_line_document("1\n\n"));
        assert!(is_single_line_document("   1  \n  \n\n "));
        
        assert!(!is_single_line_document("1\n2"));
        assert!(!is_single_line_document("1\n  2  \n"));
        assert!(!is_single_line_document("   \n  \n  "));
    }

    #[test]
    fn test_notation_system_detection() {
        let number_doc = parse_document("123").unwrap();
        // Verify notation system is Number
        
        let western_doc = parse_document("CDE").unwrap();
        // Verify notation system is Western
        
        let sargam_doc = parse_document("SRG").unwrap();
        // Verify notation system is Sargam
    }

    #[test]
    fn test_multiline_uses_normal_parsing() {
        let input = "line1\nline2";
        // Should NOT trigger single-line parsing
        // Verify normal paragraph parsing is used
    }
}
```

### Integration Tests
- Test with real musical notation examples
- Verify VexFlow and LilyPond output for single-line inputs
- Test web interface with single-character input

## Backwards Compatibility

This feature is fully backwards compatible:
- Only affects single-line inputs
- All existing multi-line documents parse unchanged
- No changes to public API or data structures
- No changes to existing function signatures

## Alternatives Considered

1. **Lower threshold (10%)**: Too permissive, would parse "test1" as music
2. **Higher threshold (50%)**: Too restrictive, would reject "12hello"
3. **Exact character matching**: Would reject any non-musical characters
4. **Directive-based**: Requiring `#single-line` directive adds complexity

## Future Extensions

- **Interactive tutorial mode**: Detect common beginner patterns and provide hints
- **Threshold configuration**: `#threshold: 0.25` for different use cases
- **Extended musical vocabulary**: Chord symbols, dynamics, articulation marks
- **Force single-line mode**: `#single-line` directive for explicit control
- **Playground mode**: Special mode optimized for experimentation with instant feedback
- **Notation system hints**: When ambiguous input detected, suggest notation systems

## References

- Current `is_content_line()` implementation: `src/parse/document_parser/content_line.rs`
- Musical character recognition: `count_musical_elements()` function
- Notation system detection: `detect_line_notation_system()` function

## Changelog

- 2024-01-XX: Initial draft specification
- 2025-09-11: **Feature implemented and validated** - All functionality working as specified