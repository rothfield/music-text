/// Shared transposition utilities for both LilyPond and VexFlow converters
use crate::models::Degree;

/// Transpose a degree and octave based on the tonic (movable-do system)
/// Returns (transposed_degree, adjusted_octave)
pub fn transpose_degree_with_octave(degree: Degree, octave: i8, tonic: Degree) -> (Degree, i8) {
    // Get scale degree positions (0-6 for natural notes, with accidentals as offsets)
    let (degree_scale_pos, degree_accidental) = degree_to_scale_position(degree);
    let (tonic_scale_pos, tonic_accidental) = degree_to_scale_position(tonic);
    
    // Calculate transposition: shift by tonic's position
    let transposed_scale_pos_raw = degree_scale_pos + tonic_scale_pos;
    let transposed_scale_pos = transposed_scale_pos_raw % 7;
    let octave_adjustment = (transposed_scale_pos_raw / 7) as i8;
    let transposed_accidental = degree_accidental + tonic_accidental;
    
    // Convert back to Degree
    let transposed_degree = scale_position_to_degree(transposed_scale_pos, transposed_accidental);
    let adjusted_octave = octave + octave_adjustment;
    
    (transposed_degree, adjusted_octave)
}

/// Transpose a degree based on the tonic (movable-do system)
pub fn transpose_degree(degree: Degree, tonic: Degree) -> Degree {
    let (transposed_degree, _) = transpose_degree_with_octave(degree, 0, tonic);
    transposed_degree
}

/// Convert degree to scale position (0-6) and accidental offset
fn degree_to_scale_position(degree: Degree) -> (usize, i8) {
    use Degree::*;
    match degree {
        N1bb => (0, -2), N1b => (0, -1), N1 => (0, 0), N1s => (0, 1), N1ss => (0, 2),
        N2bb => (1, -2), N2b => (1, -1), N2 => (1, 0), N2s => (1, 1), N2ss => (1, 2),
        N3bb => (2, -2), N3b => (2, -1), N3 => (2, 0), N3s => (2, 1), N3ss => (2, 2),
        N4bb => (3, -2), N4b => (3, -1), N4 => (3, 0), N4s => (3, 1), N4ss => (3, 2),
        N5bb => (4, -2), N5b => (4, -1), N5 => (4, 0), N5s => (4, 1), N5ss => (4, 2),
        N6bb => (5, -2), N6b => (5, -1), N6 => (5, 0), N6s => (5, 1), N6ss => (5, 2),
        N7bb => (6, -2), N7b => (6, -1), N7 => (6, 0), N7s => (6, 1), N7ss => (6, 2),
    }
}

/// Convert scale position and accidental back to Degree
fn scale_position_to_degree(scale_pos: usize, accidental: i8) -> Degree {
    use Degree::*;
    match (scale_pos, accidental) {
        (0, -2) => N1bb, (0, -1) => N1b, (0, 0) => N1, (0, 1) => N1s, (0, 2) => N1ss,
        (1, -2) => N2bb, (1, -1) => N2b, (1, 0) => N2, (1, 1) => N2s, (1, 2) => N2ss,
        (2, -2) => N3bb, (2, -1) => N3b, (2, 0) => N3, (2, 1) => N3s, (2, 2) => N3ss,
        (3, -2) => N4bb, (3, -1) => N4b, (3, 0) => N4, (3, 1) => N4s, (3, 2) => N4ss,
        (4, -2) => N5bb, (4, -1) => N5b, (4, 0) => N5, (4, 1) => N5s, (4, 2) => N5ss,
        (5, -2) => N6bb, (5, -1) => N6b, (5, 0) => N6, (5, 1) => N6s, (5, 2) => N6ss,
        (6, -2) => N7bb, (6, -1) => N7b, (6, 0) => N7, (6, 1) => N7s, (6, 2) => N7ss,
        _ => N1, // Fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transpose_degree_7_in_d_major() {
        // Scale degree 7 in D major should be C# (N1s) in octave above
        let result = transpose_degree_with_octave(Degree::N7, 0, Degree::N2);
        assert_eq!(result, (Degree::N1s, 1));
    }
    
    #[test] 
    fn test_transpose_degree_1_in_d_major() {
        // Scale degree 1 in D major should be D (N2) in same octave
        let result = transpose_degree_with_octave(Degree::N1, 0, Degree::N2);
        assert_eq!(result, (Degree::N2, 0));
    }
    
    #[test]
    fn test_transpose_degree_4_in_g_major() {
        // Scale degree 4 in G major should be C (N1) in octave above
        let result = transpose_degree_with_octave(Degree::N4, 0, Degree::N5);
        assert_eq!(result, (Degree::N1, 1));
    }
}