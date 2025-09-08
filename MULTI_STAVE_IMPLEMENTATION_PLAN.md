# Multi-Stave Support Implementation Plan

## Overview

This document outlines a comprehensive plan to add multi-stave support to music-text, enabling connected staff groups like piano music and conductor scores. The implementation focuses on grammar extension and LilyPond rendering while preserving backward compatibility.

## Research Summary

### LilyPond Staff Group Types
Based on research, LilyPond provides several staff grouping contexts:

1. **PianoStaff**: Brace connection + connected barlines (keyboard music)
2. **GrandStaff**: Brace connection + connected barlines (orchestral scores)  
3. **StaffGroup**: Bracket connection + disconnected barlines (ensemble music)
4. **ChoirStaff**: Bracket connection + disconnected barlines (vocal music)

### Key Visual Distinctions
- **Brace** (curly): PianoStaff, GrandStaff - for unified instruments
- **Bracket** (straight): StaffGroup, ChoirStaff - for separate instruments
- **Connected barlines**: PianoStaff, GrandStaff
- **Disconnected barlines**: StaffGroup, ChoirStaff

## Grammar Design

### Proposed Syntax

Drawing inspiration from markdown fenced blocks and ABC notation's `%%score` directive, we propose bracket-fence syntax:

```
{piano
treble: |1 2 3 4|
bass: |5 4 3 2|
}

{group
violin: |1 2 3 4|  
viola: |5 4 3 2|
}

{choir
soprano: |1 2 3 4|
alto: |5 4 3 2|  
tenor: |3 2 1 2|
bass: |1 1 2 2|
}
```

### Alternative Syntax Considerations

#### Option 1: User's Suggestion - Line-Based Brackets
```
{_____________
|1 2 3 4|
|5 4 3 2|  
}_____________
```

#### Option 2: ABC-Inspired Score Directive
```
%%score {treble bass}
treble: |1 2 3 4|
bass: |5 4 3 2|
```

#### Option 3: YAML-Like Frontmatter
```
---
type: piano
staves: [treble, bass]
---
treble: |1 2 3 4|
bass: |5 4 3 2|
```

### Recommended Syntax: Named Staff Groups

We recommend **Option 1 with named staff types** for the following reasons:

1. **Markdown-like**: Familiar fenced block pattern `{grouptype ... }`
2. **Explicit staff roles**: Clear semantic meaning (treble, bass, violin, etc.)
3. **LilyPond mapping**: Direct mapping to LilyPond staff group types
4. **Extensible**: Easy to add new group types and staff roles
5. **Readable**: Self-documenting syntax

```
{piano
treble: |1 2 3 4|
bass: |5 4 3 2|
}

{group
staff1: |1 2 3 4|  
staff2: |5 4 3 2|
}
```

## Grammar Implementation

### Grammar Extensions

```pest
// Add to grammar.pest

document = { SOI ~ mixed_content? ~ trailing_whitespace? ~ EOI }

mixed_content = { (mixed_line | staff_group | blank_line) ~ (stave_separator ~ (mixed_line | staff_group | blank_line))* ~ stave_separator? }

// New staff group rule
staff_group = { 
    staff_group_start ~ 
    staff_group_content ~ 
    staff_group_end 
}

staff_group_start = { "{" ~ group_type ~ NEWLINE }
group_type = { "piano" | "grand" | "group" | "choir" }

staff_group_content = { named_stave+ }
named_stave = { staff_name ~ ":" ~ stave ~ NEWLINE }
staff_name = { ASCII_ALPHANUMERIC+ }

staff_group_end = { "}" ~ NEWLINE? }

// Existing stave rule remains unchanged for backward compatibility
stave = { 
    text_lines* ~ 
    content_line ~ 
    text_lines*
}
```

### Data Model Extensions

```rust
// Extend document/model.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub content: Vec<DocumentElement>,  // Changed from just staves
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentElement {
    SingleStave(Stave),
    StaffGroup(StaffGroup),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffGroup {
    pub group_type: StaffGroupType,
    pub staves: Vec<NamedStave>,
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StaffGroupType {
    Piano,    // Maps to PianoStaff (brace + connected bars)
    Grand,    // Maps to GrandStaff (brace + connected bars)
    Group,    // Maps to StaffGroup (bracket + disconnected bars)  
    Choir,    // Maps to ChoirStaff (bracket + disconnected bars)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedStave {
    pub name: String,      // "treble", "bass", "violin1", etc.
    pub stave: Stave,      // Existing Stave structure
    pub source: Source,
}
```

## LilyPond Rendering

### Renderer Extensions

```rust
// Extend renderers/lilypond/renderer.rs

impl LilyPondRenderer {
    pub fn render_staff_group(&self, staff_group: &StaffGroup) -> String {
        let lily_context = match staff_group.group_type {
            StaffGroupType::Piano => "PianoStaff",
            StaffGroupType::Grand => "GrandStaff", 
            StaffGroupType::Group => "StaffGroup",
            StaffGroupType::Choir => "ChoirStaff",
        };
        
        let mut result = format!("\\new {} <<\n", lily_context);
        
        for named_stave in &staff_group.staves {
            let stave_content = self.convert_staves_to_notes_and_lyrics(&[named_stave.stave.clone()]);
            let clef = self.infer_clef_from_name(&named_stave.name);
            
            result.push_str(&format!(
                "  \\new Staff = \"{}\" {{\n",
                named_stave.name
            ));
            
            if let Some(clef) = clef {
                result.push_str(&format!("    \\clef {}\n", clef));
            }
            
            result.push_str(&format!("    {}\n", stave_content.0));
            result.push_str("  }\n");
        }
        
        result.push_str(">>\n");
        result
    }
    
    fn infer_clef_from_name(&self, name: &str) -> Option<&'static str> {
        match name.to_lowercase().as_str() {
            "treble" | "soprano" | "alto" | "violin" => Some("treble"),
            "bass" | "cello" | "contrabass" => Some("bass"),
            "viola" | "tenor" => Some("alto"), 
            _ => None, // Let LilyPond use default
        }
    }
}
```

### Template Extensions

```mustache
{{! Extend web-fast.ly.mustache }}

{{#staff_groups}}
\new {{group_type}} <<
{{#staves}}  
  \new Staff = "{{name}}" {
    {{#clef}}\clef {{clef}}{{/clef}}
    \fixed c' {
      \autoBeamOff
      {{{content}}}
    }
  }
{{/staves}}
>>
{{/staff_groups}}

{{#single_staves}}
\new Staff {
  \fixed c' {
    \autoBeamOff
    {{{content}}}
  }
}
{{/single_staves}}
```

## Use Cases and Examples

### Piano Music
```
{piano
treble: |1 2 3 4|
bass: |5 4 3 2|
}
```

**LilyPond Output:**
```lilypond
\new PianoStaff <<
  \new Staff = "treble" {
    \clef treble
    c'4 d'4 e'4 f'4
  }
  \new Staff = "bass" {
    \clef bass  
    g4 f4 e4 d4
  }
>>
```

### String Quartet
```
{group
violin1: |1 3 5 3|
violin2: |1 2 3 2|
viola: |5 4 3 4|
cello: |1 1 1 1|
}
```

**LilyPond Output:**
```lilypond
\new StaffGroup <<
  \new Staff = "violin1" {
    \clef treble
    c'4 e'4 g'4 e'4
  }
  \new Staff = "violin2" {
    \clef treble
    c'4 d'4 e'4 d'4
  }
  \new Staff = "viola" {
    \clef alto
    g4 f4 e4 f4
  }
  \new Staff = "cello" {
    \clef bass
    c4 c4 c4 c4
  }
>>
```

### SATB Choir
```
{choir
soprano: |1 2 3 4|
alto: |5 6 7 1|
tenor: |3 4 5 6|
bass: |1 1 1 1|
}
```

## Implementation Phases

### Phase 1: Grammar and Parsing
- [ ] Extend grammar.pest with staff group rules
- [ ] Update document/parser.rs to handle new syntax
- [ ] Add StaffGroup data structures to document/model.rs
- [ ] Update tree_transformer to build StaffGroup elements

### Phase 2: LilyPond Rendering  
- [ ] Extend LilyPond renderer with staff group support
- [ ] Add clef inference logic based on staff names
- [ ] Update mustache templates for multi-stave output
- [ ] Test all four staff group types (Piano, Grand, Group, Choir)

### Phase 3: Testing and Integration
- [ ] Add comprehensive test cases for all syntax variants
- [ ] Test piano music examples with treble/bass staves
- [ ] Test orchestral examples with multiple instruments
- [ ] Verify backward compatibility with single staves
- [ ] Test web interface with multi-stave rendering

### Phase 4: Documentation and Polish
- [ ] Update grammar documentation with new syntax
- [ ] Add usage examples for different musical contexts
- [ ] Document clef inference rules and customization
- [ ] Add troubleshooting guide for common issues

## Technical Considerations

### Backward Compatibility
- Single staves continue to work unchanged
- Existing documents parse and render identically  
- New syntax is purely additive

### Error Handling
- Validate staff group types (piano, grand, group, choir)
- Ensure staff names are valid identifiers
- Provide clear error messages for syntax mistakes
- Handle empty staff groups gracefully

### Performance
- Minimal impact on single-stave documents
- Staff group parsing is localized and efficient
- LilyPond rendering scales linearly with staff count

### Extensibility
- Easy to add new staff group types
- Staff naming is flexible and user-controlled  
- Clef inference can be customized or overridden
- Template system supports advanced LilyPond features

## Alternative Approaches Considered

### Why Not Inline Attributes?
```
|1 2 3 4| staff=treble group=piano
|5 4 3 2| staff=bass group=piano
```
**Rejected**: Verbose, doesn't clearly show grouping structure

### Why Not Indentation-Based?
```
piano:
  treble: |1 2 3 4|
  bass: |5 4 3 2|
```  
**Rejected**: Indentation is error-prone in plain text formats

### Why Not HTML-Like Tags?
```
<piano>
<treble>|1 2 3 4|</treble>
<bass>|5 4 3 2|</bass>
</piano>
```
**Rejected**: Too verbose, not markdown-like

## Conclusion

The proposed syntax provides a clean, extensible approach to multi-stave support that:

1. **Maintains backward compatibility** with existing single-stave documents
2. **Maps naturally to LilyPond** staff grouping constructs  
3. **Uses familiar markdown-like syntax** for ease of learning
4. **Supports all major use cases** (piano, orchestral, choral music)
5. **Provides clear semantic meaning** through named staff roles

The implementation can be done incrementally without disrupting existing functionality, and the syntax is flexible enough to support future extensions while remaining readable and maintainable.