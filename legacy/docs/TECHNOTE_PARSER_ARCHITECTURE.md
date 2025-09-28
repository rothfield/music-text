# TECHNOTE: Parser Architecture Clarification

## The Confusion

Our discussion revealed fundamental confusion about what constitutes "the parser" in a Pest-based system. The file `document_parser.rs` is misleadingly named - it's 95% tree transformation code, not parser code.

## How Pest Actually Works

1. **Compile Time**: The `#[derive(Parser)]` macro reads `grammar.pest` and generates a complete parser as Rust code
2. **Runtime**: `MusicParser::parse()` is a static method call to compiled machine code - no parser object is created
3. **The Parser**: Exists as generated code in the binary, not as a runtime object or data structure

This differs from runtime parser generators (like Instaparse) where you create a parser object from a grammar string.

## The Real Architecture

```
grammar.pest → [Pest macro at compile time] → Generated parser code in binary
                                                         ↓
Input string → MusicParser::parse() → Parse tree → Tree walker → AST
                      ↑                                 ↑
              Static method call              Our transformation code
```

## Current Problem

The file `document_parser.rs` conflates two distinct responsibilities:
- **Parser generation** (5%): The `#[derive(Parser)]` declaration
- **Tree transformation** (95%): Walking the parse tree and building domain structures

This conflation led to misunderstandings about:
- What is "the parser" (it's the generated code, not our tree walker)
- Where syntax vs. semantics are handled (both are syntax - just at different levels)
- What "no parser changes needed" means (really means "no tree walker changes needed")

## Recommendation: Self-Documenting Structure

```
src/
  document/
    grammar.pest       # Parser specification (becomes the actual parser)
    parser.rs          # Parser generation and interface (5-10 lines)
    ast_builder.rs     # Tree transformation logic (the current 95%)
    types.rs           # Domain types (Document, Stave, etc.)
    mod.rs            # Module interface
```

## Why This Matters

1. **Clarity**: The file structure immediately communicates the architecture
2. **Education**: New developers understand that Pest generates the parser
3. **Separation**: Parser generation is visibly separate from tree transformation
4. **Self-Documenting**: No need to explain - the structure IS the explanation

## Key Insight

The very confusion in our discussion proves the need for this separation. Even though `parser.rs` would be tiny (5-10 lines), its existence as a separate file provides crucial architectural clarity. The physical separation makes the conceptual separation obvious.

## Implementation Impact

- No functional changes required
- Simply split `document_parser.rs` into `parser.rs` and `ast_builder.rs`
- Update imports in other files
- The code becomes self-documenting through structure alone

## Example parser.rs Content

```rust
// parser.rs - This file's existence tells the story
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "document/grammar.pest"] 
pub struct MusicParser;

pub fn parse(input: &str) -> Result<Pairs<Rule>, Error<Rule>> {
    MusicParser::parse(Rule::document, input)
}
```

Those few lines make it impossible to misunderstand that:
1. The parser is generated from the grammar
2. This file is just the interface to that generated parser
3. Everything else (in `ast_builder.rs`) is post-processing