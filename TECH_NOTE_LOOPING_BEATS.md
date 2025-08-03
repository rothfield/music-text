# Tech Note: Looping Beats Visualization

## Overview
Traditional manuscripts (mss) use lower loops to visually demarcate beats, making musical notation significantly more readable by showing rhythmic structure at a glance.

## Visual Concept
```
| S  R  G  M | P  D  N  S |
  \____/  \__/   \______/
   beat1  beat2    beat3
```

The curved loops underneath groups of notes serve as visual beat boundaries, helping musicians:
- Parse rhythmic groupings instantly
- Distinguish multi-note beats from single notes  
- Understand time signatures and subdivisions
- Read complex passages with clear structure

## Current Implementation Context

### Existing Components
- **Data Generator** (`src/bin/data_generator.rs`): Already generates lower loops for multi-character words with 30% probability
- **BEAT Nodes**: Parser produces BEAT structures with divisions and multiple pitches
- **VexFlow Integration**: SVG rendering capabilities in place
- **Spatial Analysis**: Parser tracks positional relationships between notes

### Beat Detection Logic
The parser's `group_nodes_into_lines_and_beats` phase already identifies:
- Multi-pitch beats (beats containing multiple notes)
- Beat divisions and subdivisions
- Temporal groupings within measures

## Proposed Implementation

### Phase 1: Beat Loop Detection
- Identify beats with `divisions > 1` or multiple pitch children
- Calculate horizontal span of beat content
- Determine optimal loop positioning relative to staff

### Phase 2: VexFlow Rendering
- Generate SVG curved paths below staff lines
- Position loops to avoid conflicts with lyrics/ornaments
- Style loops to match manuscript conventions (thin, curved lines)

### Phase 3: User Controls
- Toggle option: "Show Beat Loops"
- Configurable loop styles (thickness, curvature, offset)
- Automatic vs manual loop placement modes

## Benefits
- **Historical Accuracy**: Matches traditional manuscript notation practices
- **Pedagogical Value**: Helps students learn rhythmic reading
- **Visual Clarity**: Reduces cognitive load when parsing complex rhythms
- **Accessibility**: Makes notation more approachable for learners

## Technical Considerations
- Loop curves must avoid colliding with existing notation elements
- Positioning algorithm needs to account for varying beat widths
- SVG path generation for smooth, consistent curve appearance
- Performance impact minimal (simple path drawing)

## Future Extensions
- Phrase loops (longer arcs spanning multiple beats/measures)
- Hierarchical loops (beat subdivisions within larger groupings)
- Color-coded loops for different rhythmic levels
- Interactive loop editing in web interface

## References
- Traditional manuscript notation practices
- Current data generator implementation in `src/bin/data_generator.rs:generate_lower_loop`
- Parser beat detection in `src/parser.rs:group_nodes_into_lines_and_beats`