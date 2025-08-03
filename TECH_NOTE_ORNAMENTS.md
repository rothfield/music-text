# Technical Note: Ornament Implementation

## Overview
Ornaments in musical notation are decorative note sequences that embellish main melodic notes. In our parser, ornaments should work similarly to how lyrics are currently implemented - as sequences of elements that get spatially attached to target notes.

## Current Lyrics Implementation (Reference Model)
The parser already successfully handles complex spatial relationships with lyrics:

```
Song title
| S R G M |
  la la la
```

- **Lyrics line**: Contains WORD tokens that get attached to notes above
- **Spatial alignment**: Words below notes get attached as child nodes
- **LilyPond rendering**: `\addlyrics { "la" "la" "la" }`

## Ornament Requirements

### Input Pattern
```
NRSNS               <- Ornament sequence (above music line)
SS RRR GG MM |      <- Main music line
```

### Expected Behavior
1. **Detection**: `NRSNS` should be detected as an ornament sequence above the music line
2. **Attachment**: The sequence should attach to the first `S` (spatially aligned at column 0)
3. **LilyPond Output**: `\grace { b8 d8 c8 b8 c8 } c8 c8 \tuplet 3/2 { d8 d8 d8 } e8 e8 fs8 fs8`

## Technical Approach (Based on Lyrics Pattern)

### 1. Spatial Detection
- **Current**: Lyrics below music lines → WORD tokens attach to PITCH tokens above
- **Ornaments**: Pitch sequences above music lines → PITCH tokens attach to PITCH tokens below
- **Alignment**: Column-based spatial relationship (same as lyrics)

### 2. Processing Location
- **Spatial Relationship Processing**: Same place where octave markers and lyrics are handled
- **Existing pattern**: Look for elements in adjacent lines that align spatially
- **New pattern**: Look for PITCH sequences in lines above music lines

### 3. Node Structure
```yaml
- type: PITCH
  val: S
  pitch_code: C
  nodes:
    - type: ORNAMENT
      val: N
      pitch_code: B
    - type: ORNAMENT  
      val: R
      pitch_code: D
    - type: ORNAMENT
      val: S
      pitch_code: C
    # ... rest of ornament sequence
```

### 4. LilyPond Conversion
- **Current lyrics**: `for child in beat_element.nodes: if child.node_type == "WORD"`
- **New ornaments**: `for child in beat_element.nodes: if child.node_type == "ORNAMENT"`
- **Rendering**: Convert ornament children to `\grace { ... }` before the main note

## Implementation Steps

### Phase 1: Spatial Detection
```rust
// In flatten_spatial_relationships(), similar to octave markers
// Look for PITCH sequences in lines above music lines
// Attach as ORNAMENT child nodes to spatially aligned target notes
```

### Phase 2: LilyPond Integration  
```rust
// In lilypond_converter.rs, similar to lyrics handling
// Detect ORNAMENT child nodes and render as grace notes
if beat_element.nodes.iter().any(|child| child.node_type == "ORNAMENT") {
    let grace_notes = collect_ornament_notes(beat_element);
    note_str = format!("\\grace {{ {} }} {}", grace_notes, note_str);
}
```

## Advantages of This Approach
1. **Reuses existing patterns**: Leverages proven spatial relationship logic
2. **Consistent with lyrics**: Same architectural pattern, familiar codebase
3. **Minimal changes**: Extends existing systems rather than creating new ones
4. **Robust**: Benefits from existing boundary detection and alignment logic

## Key Differences from Lyrics
- **Direction**: Ornaments above music lines (vs lyrics below)
- **Token type**: PITCH tokens (vs WORD tokens)  
- **LilyPond output**: Grace notes (vs `\addlyrics`)
- **Timing**: Ornaments affect the timing of the target note

## Example Processing Flow
1. **Input**: `NRSNS` above `SS RRR GG MM |`
2. **Spatial detection**: Identifies NRSNS as ornament sequence for first S
3. **Node attachment**: NRSNS becomes ORNAMENT child nodes of first S
4. **Beat grouping**: First S becomes part of SS beat with ornament children
5. **LilyPond conversion**: `\grace { b8 d8 c8 b8 c8 } c8 c8`

This approach treats ornaments as a natural extension of the existing spatial relationship system, maintaining consistency with the established lyrics pattern while adding sophisticated musical notation capabilities.