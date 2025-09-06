# Grammar and Domain Decisions

This document captures the fundamental domain knowledge and grammar decisions established during the development session.

## Core Domain Concepts

### What is a Document?
A **document** consists of one or more **staves** separated by blank lines:
- Documents can have multiple staves for multi-part music
- Staves are separated by one or more blank lines (like paragraphs in text)
- Example: `|1 2\n\n|3 4` = 2 staves

### What is a Stave?
A **stave** is like a paragraph of musical text, containing:
- Optional text lines before the content line
- Exactly one **content line** (the musical notation)
- Optional text lines after the content line

**Critical Rule**: A stave is analogous to a paragraph - staves are separated by blank lines.

### What is a Content Line?
A **content line** contains the actual musical notation and has these requirements:
- **MUST contain at least one barline (`|`)** - this was the key grammar constraint
- Contains musical elements: pitches, spaces, and barlines
- Example: `|1 2 3` or `1 2 | 3 4` or `1 2 3|`

### What is a Text Line?
A **text line** is any line that:
- Is NOT a content line
- Is NOT a blank line  
- Can contain any character EXCEPT `|` (barlines)

## Valid Pitches

The grammar supports these pitch notations:

### Number Notation (Primary)
- `1`, `2`, `3`, `4`, `5`, `6`, `7` - basic scale degrees
- With accidentals: `1#`, `2b`, `3##`, `4bb` etc.

### Letter Notation  
- `A`, `B`, `C`, `D`, `E`, `F`, `G` - western note names
- With accidentals: `C#`, `Db`, `F##` etc.

### Grammar Rules for Pitches
```pest
pitch = { base_pitch ~ accidentals? }
base_pitch = { number_pitch | letter_pitch }
number_pitch = { '1'..'7' }
letter_pitch = { 'A'..'G' }
accidentals = { sharp+ | flat+ }
sharp = { "#" }
flat = { "b" }
```

## Critical Grammar Constraint: Barline Requirement

The most important grammar decision was **enforcing barline requirement**:

### Original Problem
Input like `"1"` was incorrectly parsing as valid, but music notation requires barlines.

### Solution  
Changed content_line definition from:
```pest
content_line = { musical_element+ }
```

To:
```pest  
content_line = { musical_element* ~ barline ~ musical_element* }
```

### What This Means
- Every content line MUST contain at least one barline
- Barlines can appear anywhere: start, middle, end, or multiple places
- Input `"1"` now correctly FAILS to parse
- Input `"|1"`, `"1|"`, or `"1|2"` correctly parse

### Why This Pattern Works
- `musical_element*` before barline = zero or more elements (flexible)
- `barline` = required barline (constraint)
- `musical_element*` after barline = zero or more elements (allows multiple barlines since musical_element includes barline)

## Multi-Stave Parsing

### Grammar Structure
```pest
document = { SOI ~ stave_list? ~ trailing_whitespace? ~ EOI }
stave_list = { stave ~ (stave_separator ~ stave)* ~ stave_separator? }
stave_separator = { NEWLINE{2,} }  // Two or more newlines
```

### Key Insights
- Staves are like paragraphs - separated by blank lines
- `NEWLINE{2,}` means "two or more newlines" = blank line separation
- `stave_separator?` at end handles trailing blank lines

### Examples
- `"|1 2 3"` = 1 stave
- `"|1 2\n\n|3 4"` = 2 staves (separated by blank line)
- `"|1\n\n|2\n\n|3"` = 3 staves

## Text Line Grammar Refinement

### The Critical Fix
Text lines must exclude both content lines AND blank lines:
```pest
text_line = { !content_line ~ !blank_line ~ (!NEWLINE ~ !"|" ~ ANY)* }
blank_line = { " "* }
```

### Why This Works
- `!content_line` = negative lookahead, not a content line
- `!blank_line` = negative lookahead, not a blank line  
- `(!NEWLINE ~ !"|" ~ ANY)*` = any characters except newlines and barlines

This prevents text lines from interfering with stave separation logic.

## Position Tracking

Every parsed element includes position information from PEST:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}
```

This enables detailed error reporting and source mapping.

## Parser Architecture Decisions

### No Parser Changes Required
The barline requirement was implemented ONLY in grammar - parser code remained unchanged.
This demonstrates clean separation between syntax (grammar) and semantics (parser).

### Pipeline Architecture  
```
Input → document_parser → stave_parser → ProcessingResult
```

Functional pipeline with clear data flow and transformations.

## Testing Philosophy

### Critical Test Cases
- `"1"` MUST fail (no barline)
- `"|1"` MUST succeed (has barline)
- `"|1 2\n\n|3 4"` MUST parse as 2 staves

### Grammar Validation
Every grammar change must pass existing tests to ensure backward compatibility while adding new constraints.