# Abstraction Discussion: Why We Reverted Standardized Function Names

## Context

During development, we considered implementing a "meta algorithm" to unify VexFlow and LilyPond converters. We decided against it but implemented "standardized function names" as a middle ground.

## The Problem We Thought We Had

- VexFlow and LilyPond converters had similar logic for creating notes, rests, tuplets
- Recent bugs (tuplet ratio, tie handling) affected both converters
- Different function names made the code harder to understand across converters
- Copy/paste errors could occur due to different parameter orders

## What We Implemented

**Before standardization:**
```rust
// VexFlow - inline element creation
beat_notes.push(VexFlowElement::Note {
    keys: vec![key.clone()],
    duration: vexflow_duration.clone(),
    dots: *dots,
    accidentals: accidentals.clone(),
    tied: should_tie,
    original_duration: Some(format!("{}", beat_element.tuplet_duration)),
    beam_start: false,
    beam_end: false,
    syl: None,
});

// LilyPond - inline string formatting  
let lily_note = pitch_code_to_lilypond(pitch_code, octave, note_names)?;
let duration_string = fraction_to_lilypond_note(duration);
notes.push(format!("{}{}", lily_note, duration_string));
```

**After standardization:**
```rust
// VexFlow - helper function (incorrectly placed on KeyTransposer)
let note_elements = transposer.create_note(pitch_code, octave, duration);
beat_notes.extend(note_elements);

// LilyPond - helper function (new LilyPondElementCreator struct)
let creator = LilyPondElementCreator::new(note_names);
let note = creator.create_note(pitch_code, octave, duration)?;
notes.push(note);
```

## What We Actually Gained

- Consistent function names: `create_note()`, `create_rest()`, `create_tuplet()` 
- ~15 lines of code moved into helper functions
- Theoretical consistency between converters

## What We Lost

- **Code clarity** - Element creation logic now hidden behind function calls
- **Directness** - Can't see exactly what VexFlow/LilyPond elements are being created
- **Simplicity** - Added struct overhead and indirection layers
- **Maintainability** - Code now split across more places

## Design Mistakes Made

1. **Violated Single Responsibility Principle** - Put VexFlow creation methods on `KeyTransposer` (which should only handle transposition)
2. **Asymmetric design** - LilyPond got dedicated creator struct, VexFlow hijacked existing struct
3. **Premature abstraction** - Created abstraction without proven need

## Key Insights

### The Real Problem Was Different
The bugs we experienced (tuplet ratio, tie handling) weren't caused by code duplication or inconsistent naming. They were caused by **converters not trusting FSM data**:
- VexFlow was ignoring `FSM.ratio` and calculating its own tuplet parameters
- This was a logic bug, not a naming/structure bug

### Abstraction Wouldn't Have Prevented This
Even with standardized functions, a `VexFlowAdapter` could still ignore FSM data and do its own calculations. The abstraction addresses the wrong layer.

### The Formats Are Actually Very Different
- **VexFlow**: Creates complex JavaScript objects with many fields
- **LilyPond**: Creates simple strings with specific syntax
- **Error handling**: VexFlow uses Option/Result, LilyPond uses string formatting
- **Output structure**: VexFlow returns `Vec<VexFlowElement>`, LilyPond returns `Vec<String>`

These differences make meaningful abstraction difficult and not particularly valuable.

## Better Solutions We Should Have Used

1. **Integration tests** - Verify both outputs stay equivalent for the same input
2. **Strong FSM types** - Make FSM data harder to misuse
3. **Documentation** - "FSM is source of truth" principle
4. **Code review** - Check that converters trust FSM data
5. **Keep it simple** - Both converters were already clean and working

## Decision

**Revert to inline element creation.** The original code was:
- More direct and readable
- Simpler to understand and maintain
- Already bug-free after trusting FSM data
- Appropriate for the actual complexity level

## Lessons Learned

1. **Question the premise** - Are we solving the right problem?
2. **YAGNI principle** - You Aren't Gonna Need It (more formats, more abstraction)
3. **Favor composition over inheritance** - But favor simplicity over both
4. **Abstraction has costs** - Indirection, complexity, cognitive overhead
5. **Different doesn't mean wrong** - VexFlow and LilyPond can legitimately have different approaches

## When Abstraction Would Make Sense

- If we actually needed 5+ output formats
- If the converters were 500+ lines each with significant duplication
- If we were building a general-purpose notation conversion library
- If format-specific bugs were a recurring pattern

For our use case (2 formats, ~100 lines each, stable requirements), the abstraction was premature.