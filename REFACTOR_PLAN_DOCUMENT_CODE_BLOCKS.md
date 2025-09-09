# Document & Code Blocks Pattern for Music Notation

## Rationale

Music notation documents are similar to HTML documents with embedded code blocks:
- **Document structure** (HTML) = paragraphs, directives, metadata
- **Code blocks** (JavaScript/CSS) = musical staves requiring specialized parsing
- **Syntax highlighting** (language-specific) = spatial/temporal music analysis

This pattern justifies separating document parsing from domain-specific musical analysis.

## Target Directory Structure

```
src/
├── document/
│   └── document_parser/     # (renamed from manual_parser/)
│       ├── document.rs      # top-level document structure parsing
│       ├── stave.rs         # basic stave identification (calls stave/ module)
│       ├── content_line.rs  # basic content line parsing
│       └── ...              # other document-level parsers
├── stave/
│   └── parser.rs           # (moved from stave_parser.rs)
│                           # spatial analysis: content/upper/lower/lyrics lines
└── rhythm/
    └── analyzer.rs         # (moved from rhythm_fsm.rs)
                           # temporal analysis: FSM processing, beat grouping
```

## Data Flow Pipeline

1. **Raw text** → `document/document_parser/` → **paragraphs** (structure identification)
2. **Stave paragraphs** → `stave/parser.rs` → **Stave objects** (spatial analysis)
3. **Stave objects** → `rhythm/analyzer.rs` → **ProcessedStave** (temporal analysis)
4. **ProcessedStave** → `renderers/` → **Output** (LilyPond/VexFlow)

## Implementation Steps

1. Rename `src/document/manual_parser/` → `src/document/document_parser/`
2. Create `src/stave/` and move `stave_parser.rs` → `stave/parser.rs`
3. Create `src/rhythm/` and move `rhythm_fsm.rs` → `rhythm/analyzer.rs`
4. Update all import paths throughout codebase
5. Add "no mod.rs files" rule to README coding guidelines

## Benefits

- **Clear domain separation** (like HTML parser vs code syntax highlighter)
- **Single responsibility** - each module has clear boundaries
- **Independent testing** - testable at each stage
- **Extensible architecture** - future enhancements don't require touching other modules
- **Proven pattern** - follows established practices from web development

## Module Responsibilities

### `document/document_parser/`
- Parse document structure (paragraphs, directives)
- Identify paragraph types (stave vs text vs directive)
- Handle document metadata and global settings
- **Does NOT** understand musical notation details

### `stave/parser.rs`
- Parse raw stave paragraph text
- Perform spatial analysis (upper/content/lower/lyrics lines)
- Extract musical elements and annotations
- Produce structured `Stave` objects
- **Does NOT** understand temporal/rhythm relationships

### `rhythm/analyzer.rs`
- Analyze `Stave` objects for temporal relationships
- Run FSM processing for beat detection
- Handle tuplets, ties, and rhythm grouping
- Produce `ProcessedStave` objects ready for rendering
- **Does NOT** understand document structure or spatial layout