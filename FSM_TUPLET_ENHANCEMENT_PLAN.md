# FSM Tuplet Enhancement Plan

## Goal
Enhance the V2 FSM to calculate correct tuplet durations using the simple rule and provide denormalized access to common fields.

## Current Problems
1. **Complex converter heuristics** - LilyPond/VexFlow converters use hardcoded subdivision mapping
2. **Awkward field access** - `element.element.pitch_code` is verbose and error-prone  
3. **Missing tuplet context** - FSM doesn't provide tuplet ratio information
4. **String durations** - Should use Fraction type consistently
5. **Unclear naming** - `ElementV2` doesn't clearly indicate it's a beat component

## Proposed Structure Changes

### Enhanced BeatV2
```rust
#[derive(Debug, Clone)]
pub struct BeatV2 {
    pub divisions: usize,
    pub elements: Vec<BeatElement>,           // RENAMED: ElementV2 → BeatElement
    pub tied_to_previous: bool,
    pub is_tuplet: bool,                      // NEW: Fast boolean check  
    pub tuplet_ratio: Option<(usize, usize)>, // NEW: (divisions, power_of_2) for tuplets
}
```

### Fully Denormalized BeatElement (renamed from ElementV2)
```rust
#[derive(Debug, Clone)]
pub struct BeatElement {
    // Original element for reference/completeness
    pub element: ParsedElement,
    pub subdivisions: usize,
    
    // NEW: Dual duration system (replaces ParsedElement duration)
    pub duration: Fraction,        // Actual beat fraction: subdivisions/divisions  
    pub tuplet_duration: Fraction, // Simple rule duration: subdivisions * (1/4 ÷ power_of_2)
    
    // NEW: ALL ParsedElement attributes flattened to top level
    pub pitch_code: Option<PitchCode>, // Note/Dash: Some(code), Others: None
    pub octave: Option<i8>,            // Note/Dash: Some(octave), Others: None
    pub value: String,                 // Original text value from all elements
    pub position: Position,            // Position from all elements
    pub children: Vec<ParsedChild>,    // Note: actual children, Others: empty vec
    
    // Extracted convenience fields
    pub syl: Option<String>,           // Extracted from children
    pub ornaments: Vec<OrnamentType>,  // Extracted from children  
    pub octave_markers: Vec<String>,   // Extracted from children
    
    // Quick element type checks (avoid pattern matching)
    pub is_rest: bool,
    pub is_note: bool,
    pub is_dash: bool,
    pub is_barline: bool,
    pub is_slur_start: bool,
    pub is_slur_end: bool,
}
```

## Naming Rationale

**ElementV2 → BeatElement**: 
- BeatElements exist ONLY within beats (confirmed by FSM architecture)
- Non-rhythmic elements (barlines, slurs, breathmarks) bypass the beat system entirely  
- Clear semantic relationship: `Beat` contains `BeatElement`s
- Eliminates confusion about scope and usage

## Denormalization Strategy

**Bit Copy Approach with From<ParsedElement>**:
- **Single-pass conversion**: All field extraction happens once during `From<ParsedElement>` implementation
- **Zero runtime overhead**: No pattern matching or field population after construction
- **Performance optimized**: Compiler-optimized From trait, better cache locality
- **Clean API**: Eliminates complex helper functions, single source of truth for conversion
- **Type safety**: `ParsedElementType` enum cleaner than multiple boolean flags
- **Future-proof**: Adding new ParsedElement variants is straightforward

## Implementation Plan

### Phase 1: Update Data Structures
- [ ] Rename ElementV2 to BeatElement in `src/rhythm_fsm_v2.rs`
- [ ] Modify BeatV2 struct in `src/rhythm_fsm_v2.rs`
- [ ] Add helper functions for power-of-2 calculation
- [ ] Update all references to ElementV2 throughout codebase

### Phase 2: Enhance FSM Logic
- [ ] Add tuplet detection in beat finishing logic:
  ```rust
  fn finish_beat(&mut self) {
      if let Some(mut beat) = self.current_beat.take() {
          // Tuplet detection
          beat.is_tuplet = beat.divisions > 1 && (beat.divisions & (beat.divisions - 1)) != 0;
          
          if beat.is_tuplet {
              let power_of_2 = find_next_lower_power_of_2(beat.divisions);
              beat.tuplet_ratio = Some((beat.divisions, power_of_2));
              
              // Calculate both duration types
              let each_unit = Fraction::new(1, 4) / power_of_2;
              for beat_element in &mut beat.elements {
                  // Actual duration
                  beat_element.duration = Fraction::new(beat_element.subdivisions, beat.divisions);
                  // Simple rule duration  
                  beat_element.tuplet_duration = each_unit * beat_element.subdivisions;
                  // Denormalized fields
                  self.populate_denormalized_fields(beat_element);
              }
          } else {
              // Regular beat processing
              for beat_element in &mut beat.elements {
                  beat_element.duration = Fraction::new(beat_element.subdivisions, beat.divisions) * Fraction::new(1, 4);
                  beat_element.tuplet_duration = beat_element.duration; // Same for regular beats
                  self.populate_denormalized_fields(beat_element);
              }
          }
          
          self.output.push(OutputItemV2::Beat(beat));
      }
  }
  ```

- [ ] Add helper methods for FSM construction:
  ```rust
  impl BeatElement {
      pub fn with_subdivisions(mut self, subdivisions: usize) -> Self {
          self.subdivisions = subdivisions;
          self
      }
      
      pub fn extend_subdivision(&mut self) {
          self.subdivisions += 1;
      }
  }
  
  fn find_next_lower_power_of_2(n: usize) -> usize {
      let mut power = 1;
      while power * 2 < n {
          power *= 2;
      }
      power.max(2)
  }
  ```

### Phase 3: Simplify Converters
- [ ] Update LilyPond converter to use FSM-provided tuplet info:
  ```rust
  // Replace complex calculate_tuplet_duration with:
  if beat.is_tuplet {
      let (n, p) = beat.tuplet_ratio.unwrap();
      let note_name = fraction_to_lilypond_note(beat_element.tuplet_duration);
      format!("\\tuplet {}/{} {{ {}{}... }}", n, p, pitch, note_name)
  } else {
      let note_name = fraction_to_lilypond_note(beat_element.duration);
      format!("{}{}", pitch, note_name)
  }
  ```

- [ ] Update VexFlow converter similarly
- [ ] Remove complex heuristic functions (`convert_tuplet_duration_to_vexflow_v2`, `calculate_tuplet_duration`)

### Phase 4: Update Consumers
- [ ] Update converters to use denormalized fields: `beat_element.pitch_code`, `beat_element.octave`, `beat_element.syl`
- [ ] Replace pattern matching with type check fields: `beat_element.is_rest`, `beat_element.is_note`
- [ ] Keep `beat_element.element` access for specialized/edge case processing
- [ ] Add tests for new tuplet calculations and denormalized field access
- [ ] Update documentation to reflect BeatElement naming and selective denormalization strategy

## Expected Benefits

1. **Correct tuplet durations** - Uses mathematical simple rule instead of heuristics
2. **Cleaner converter code** - Just maps fractions to note names  
3. **Better performance** - Denormalized fields avoid deep drilling and pattern matching
4. **Easier debugging** - Clear separation of actual vs display durations
5. **Future-ready** - Proper duration data for MIDI/audio generation

## Test Cases

- [ ] "1-2" → 3/2 tuplet with correct durations
- [ ] "1-2-3" → 5/4 tuplet with c8 d8 e16 
- [ ] "1111111111111111111111111111111" → 31/16 tuplet with uniform 64th notes
- [ ] Regular beats still work correctly
- [ ] Non-rhythmic elements (barlines) don't break

## Migration Strategy

1. **Feature flag** - Add environment variable to enable new FSM
2. **Parallel testing** - Run both old and new FSM, compare outputs  
3. **Gradual rollout** - Enable new FSM in specific test cases first
4. **Full replacement** - Remove old converter heuristics once verified

This plan addresses the core rhythm calculation issues while improving code clarity and performance.