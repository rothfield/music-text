# Notation Parser - Claude Context

## Project Overview
A Rust-based parser that converts spatial musical notation (text with 2D positioning) into structured output formats like LilyPond. The parser handles multiple notation systems (Western, Sargam, Number notation) and preserves spatial relationships through a multi-stage pipeline.

## Key Documentation
- **Main README**: [README.md](README.md) - General project information and usage
- **Source README**: [src/README.md](src/README.md) - Detailed internal architecture and module responsibilities

## Core Architecture
```
[Raw Text] → lexer → [Vec<Token>] → parser → [Vec<Node>] → Document → lilypond_converter → [LilyPond String]
```

### Key Pipeline Stages
1. **Lexer** (`src/lexer/mod.rs`): Text → positioned tokens
2. **Parser** (`src/parser/mod.rs`): Two-phase parsing:
   - **Phase 1**: `attach_floating_elements` - spatial analysis (like puzzle pieces floating up to anchor points)
   - **Phase 2**: `group_nodes_into_lines_and_beats` - musical structuring
3. **Display** (`src/display/mod.rs`): Terminal visualization
4. **Colorizer** (`src/colorizer/mod.rs`): ANSI color formatting
5. **Converters**: Output format generation (LilyPond, etc.)

## Key Concepts
- **Spatial Relationships**: 2D positioned text elements (ornaments above pitches, lyrics below, etc.)
- **Floating Elements**: Scattered tokens that need to "float up" and attach to their anchor pitches based on column alignment
- **Token Types**: PITCH, BARLINE, WORD, SYMBOLS, METADATA, etc.
- **Node Hierarchy**: Parent-child relationships representing musical structure

## Development Notes
- Modular architecture with clear separation of concerns
- Each stage has well-defined input/output data structures
- Heavy use of spatial positioning (line, column) for relationship inference
- Support for multiple notation systems with automatic detection

## Current Focus Areas
- Parser refinement and spatial relationship handling
- Output format expansion
- Documentation maintenance and architectural clarity