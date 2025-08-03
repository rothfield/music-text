# Technical Note: Spatial Rhythmic Notation with Cross-Beat Extensions and Variable Subdivisions

## Summary

- **Dashes (`-`) extend preceding notes across beats**.
- Each beat may have a **different number of subdivisions**.
- Durations are computed by summing subdivisions weighted by **beat-specific subdivision durations**.
- Fractional durations often yield **non-power-of-two values**, requiring tuplets or tied notes for accurate representation.

---

## Core Concept: Duration Calculation

Given:

- \( B_i \) = duration of beat \( i \) (usually 1 beat = 1 whole beat unit)
- \( S_i \) = number of subdivisions in beat \( i \)
- Subdivision duration in beat \( i \):  
\\[
d_i = \\frac{B_i}{S_i}
\\]

For a note extending over several subdivisions possibly crossing multiple beats:

\\[
\\text{Duration} = \\sum_{i} n_i \\times d_i
\\]

where \( n_i \) is the number of subdivisions the note occupies in beat \( i \).

---

## Examples

### Example 1: Simple extension within and across beats

Input:  

