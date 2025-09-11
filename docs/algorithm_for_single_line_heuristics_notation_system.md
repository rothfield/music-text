# Algorithm for Single Line Heuristics Notation System Detection

## Problem Statement
Enable parsing of single-line musical input without requiring barlines, while auto-detecting whether a line is musical content or plain text.

## Key Insight
Work with already-parsed elements from PEST, not raw character analysis. The parser has already classified each element by Rule type.

## Detection Algorithm

### Input
- Parsed `Pair<Rule>` elements from PEST parser
- Line has been parsed as `simple_content_line` (no barlines required)

### Detection Logic
```
1. Initialize: pitch_count = 0, current_system = None

2. For each parsed element:
   - If Rule::number_pitch:
     - Set/verify system = Number
     - Increment pitch_count
   - If Rule::western_pitch:
     - Set/verify system = Western  
     - Increment pitch_count
   - If Rule::sargam_pitch:
     - Set/verify system = Sargam
     - Increment pitch_count
   - If Rule::bhatkhande_pitch:
     - Set/verify system = Bhatkhande
     - Increment pitch_count
   - If Rule::space:
     - Continue (don't reset count)
   - Else:
     - Reset pitch_count to 0
     
3. If pitch_count >= 3:
   - Return Some(detected_system)
   - Transform to Stave
4. Else:
   - Return None
   - Treat as text annotation
```

## Examples

### Musical Lines (Auto-detect as Stave)
- `"123"` → 3 number_pitch rules → Some(Number)
- `"SRG"` → 3 sargam_pitch rules → Some(Sargam)  
- `"CDE"` → 3 western_pitch rules → Some(Western)
- `"1 2 3"` → 3 number_pitch + spaces → Some(Number)

### Non-Musical Lines (Remain as Text)
- `"12"` → 2 pitches → None
- `"SR"` → 2 pitches → None
- `"Hello"` → No pitch rules → None

## Implementation Notes

### Efficiency
- **No character-by-character analysis** - work with parsed Rule types
- **No string matching** - parser already classified elements
- **Single pass** - count while iterating parsed elements

### Ambiguous Pitches
- `"G"` and `"D"` can be Western or Sargam
- PEST's ordered choice resolves this (western_pitch tried first)
- If context suggests different system, could reparse with hints

### Grammar Changes Required
```pest
// Add to grammar.pest
simple_content_line = { musical_element_no_barline+ }
mixed_line = { stave | simple_content_line }
```

### Parser Changes Required
1. Handle `mixed_content` in document transformer
2. Add detection function working with `Pair<Rule>`
3. Auto-create Stave when detection returns Some(system)

## Critique

### Strengths
- **Efficient**: Uses already-parsed elements
- **Simple**: Just counting and pattern matching on Rules
- **Accurate**: 3+ pitch heuristic avoids false positives

### Considerations  
- Rule ordering in grammar affects ambiguous pitch classification
- Could extend to support mixed-system detection if needed
- Threshold of 3 pitches is configurable

## Future Extensions
- Support for accidentals (#, b) in detection
- Configurable pitch threshold
- Mixed notation system handling
- Context-aware reparsing for ambiguous cases