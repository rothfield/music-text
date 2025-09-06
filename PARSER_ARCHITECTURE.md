# Parser Architecture: Clean Separation of Syntax and Semantics

## Overview

The music-text parser demonstrates a textbook implementation of **separation of concerns** between syntax validation (grammar) and semantic processing (parser). This document explains the architecture and design decisions.

## Two-Layer Architecture

### Layer 1: Syntax (Grammar)
**File:** `src/music_notation.pest`  
**Responsibility:** Define what is syntactically valid

The PEG grammar acts as a gatekeeper, enforcing all syntax rules:
- Structure requirements (e.g., barlines in content lines)
- Token definitions (pitches, accidentals, spaces)
- Hierarchical document structure (staves, lines)

**Example: Barline Requirement**
```pest
content_line = { 
    musical_element* ~ barline ~ musical_element*
}
```
This rule ensures every content line MUST contain at least one barline - enforced at parse time.

### Layer 2: Semantics (Parser)
**File:** `src/document_parser.rs`  
**Responsibility:** Build meaningful data structures from validated input

The parser is completely generic and trusts the grammar:
- Traverses the validated parse tree
- Builds typed Rust structures
- Tracks source positions
- Never validates syntax (grammar already did this)

## Key Design Principles

### 1. Single Source of Truth
Each syntax rule exists in exactly ONE place - the grammar file. No duplication or parallel validation in the parser.

### 2. Trust Boundary
The parser trusts that any input it receives has already been validated by the grammar. This eliminates redundant checks and simplifies the code.

### 3. Clean Interfaces
```rust
// Simple entry point - grammar handles complexity
pub fn parse_notation(input: &str) -> Result<Pairs<Rule>, Error<Rule>> {
    MusicParser::parse(Rule::document, input)
}
```

### 4. Separation of Concerns

| Component | Responsible For | NOT Responsible For |
|-----------|----------------|-------------------|
| Grammar | Syntax validation | Building data structures |
| Parser | Structure building | Syntax validation |
| Grammar | Defining valid tokens | Interpreting meaning |
| Parser | Position tracking | Error recovery |

## Benefits of This Architecture

### 1. Maintainability
- Changes to syntax rules require editing ONLY the grammar
- Parser code remains stable even as syntax evolves
- Clear boundaries make debugging easier

### 2. Correctness
- Invalid input never reaches the parser
- Type-safe data structures prevent runtime errors
- Position tracking preserved throughout

### 3. Performance
- Grammar validates once at parse time
- No redundant validation in parser
- Efficient tree traversal with Pest

### 4. Testability
- Grammar rules can be tested in isolation
- Parser logic can be tested with known-valid input
- Clear separation enables targeted testing

## Example: Adding a New Syntax Rule

**Scenario:** Make barlines optional in content lines

**Required Changes:**
1. Grammar only: Change `barline` to `barline?`
2. Parser: NO CHANGES NEEDED

The parser continues to work because it only processes what the grammar provides. If no barline exists, it simply won't appear in the elements list.

## Data Flow

```
Input String
    ↓
[Grammar Validation] ← Syntax rules enforced here
    ↓
Parse Tree (Pest)
    ↓
[Parser Traversal] ← Structure building only
    ↓
Document Structure
    ↓
[Downstream Processing] ← Musical semantics
```

## Parser Responsibilities

The `document_parser.rs` module has these specific responsibilities:

1. **Parse Tree Traversal**: Navigate the hierarchical structure produced by Pest
2. **Type Conversion**: Convert untyped parse nodes into typed Rust structures
3. **Position Tracking**: Preserve line/column information for all elements
4. **Structure Building**: Assemble the Document → Stave → ContentLine hierarchy
5. **JSON Export**: Provide debug visualization of parse trees

## What the Parser Does NOT Do

- ❌ Validate syntax (grammar's job)
- ❌ Interpret musical meaning (later stages)
- ❌ Generate output formats (converter's job)
- ❌ Handle error recovery (fails fast)
- ❌ Enforce business rules (domain layer)

## Testing Strategy

### Grammar Tests
Test syntax acceptance/rejection:
```rust
#[test]
fn test_content_line_requires_barline() {
    assert!(parse_notation("1 2 3").is_err());      // No barline - fails
    assert!(parse_notation("1 2 | 3").is_ok());     // Has barline - passes
}
```

### Parser Tests
Test structure building with valid input:
```rust
#[test]
fn test_document_structure() {
    let doc = parse_document_structure("1 | 2").unwrap();
    assert_eq!(doc.staves.len(), 1);
    // Test structure, not syntax
}
```

## Conclusion

This architecture provides a clean, maintainable separation between:
- **What is valid** (grammar)
- **What it means** (parser)

This separation allows each component to focus on its core responsibility, resulting in simpler, more reliable code.