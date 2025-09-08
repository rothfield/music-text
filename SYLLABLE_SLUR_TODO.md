# Syllable Application and Slur Interaction - TODO

## Current Issue

The syllable post-processing added in `src/parser/horizontal.rs` uses simple left-to-right distribution that doesn't respect slur boundaries. This overrides the existing @doremi-script logic that properly handled slur-syllable interaction.

## Problem

Current post-processing logic:
```rust
// Step 3: Redistribute split syllables back to notes
// Strategy: Extra syllables go to the LAST note
```

This naive approach ignores:
1. **Slur boundaries** - notes within a slur should share the first syllable
2. **Melismatic passages** - slurred notes after the first should get placeholder "_" 
3. **Existing slur-aware logic** - @doremi-script likely already handled this correctly

## Correct Musical Behavior

```
Input: 1_2_3 4
Syllables: ka-ta ge-na
Expected: 
- Note 1: "ka-" (first note of slur gets syllable)
- Note 2: "_" (continuation of slur, placeholder)  
- Note 3: "ta" (end of slur, still uses first syllable)
- Note 4: "ge-" (new syllable for non-slurred note)
```

## Solution Strategy

Instead of replacing syllable distribution, modify approach to:

1. **Split hyphens FIRST** - before any distribution logic
2. **Preserve existing slur-aware logic** - let @doremi-script handle slur-syllable interaction
3. **Find the original syllable distribution** - likely in models/lyrics.rs or similar
4. **Apply hyphen splitting at the input level** - so existing logic sees split syllables

## Files to Investigate

- `src/models/lyrics.rs` - likely contains slur-aware distribution
- `src/parser/vertical.rs` - syllable application logic
- Original @doremi-script implementation - proper slur handling

## Action Items

1. Find existing slur-aware syllable distribution logic
2. Move hyphen splitting to earlier in pipeline (before slur processing)  
3. Remove naive left-to-right redistribution from horizontal.rs
4. Test that slurs + syllables work correctly together
5. Ensure placeholders ("_") are properly generated for melismas

## Test Cases Needed

```
1_2_3 4
ka-ta ge-na
-> Note1: "ka-", Note2: "_", Note3: "ta", Note4: "ge-"

1 2_3_4
a-b-c d-e
-> Note1: "a-", Note2: "b-", Note3: "_", Note4: "c", continuing...
```