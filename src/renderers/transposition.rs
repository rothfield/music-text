/// Shared transposition utilities for tonic-based movable-do system
use crate::models::PitchCode;

/// Transpose a PitchCode and octave based on the tonic (movable-do system)
/// Returns (transposed_pitchcode, adjusted_octave)
pub fn transpose_pitchcode_with_octave(pitchcode: PitchCode, octave: i8, tonic: PitchCode) -> (PitchCode, i8) {
    // Major scale interval pattern (semitones from tonic): 0, 2, 4, 5, 7, 9, 11
    let major_scale_semitones = [0, 2, 4, 5, 7, 9, 11];
    
    // Get the pitch and tonic as scale degrees (0-6) and semitone offsets
    let (pitch_degree, pitch_semitone_offset) = pitchcode_to_scale_position(pitchcode);
    let (tonic_degree, tonic_semitone_offset) = pitchcode_to_scale_position(tonic);
    
    // Calculate the target semitone value
    // Start with the major scale interval for this scale degree
    let target_semitones = major_scale_semitones[pitch_degree];
    
    // Add the pitch's semitone offset from its scale degree
    let pitch_offset = pitch_semitone_offset;
    
    // Add the tonic's base semitone position + offset
    let tonic_base_semitones = major_scale_semitones[tonic_degree];  
    let tonic_offset = tonic_semitone_offset;
    
    // Total semitones = target interval + pitch offset + tonic position + tonic offset
    let total_semitones = target_semitones + pitch_offset + tonic_base_semitones + tonic_offset;
    
    // Convert back to scale position and calculate octave wrap
    let (final_scale_pos, final_semitone_offset, octave_adjustment) = semitones_to_scale_position(total_semitones);
    
    let transposed_pitchcode = scale_position_to_pitchcode(final_scale_pos, final_semitone_offset);
    let adjusted_octave = octave + octave_adjustment;
    
    (transposed_pitchcode, adjusted_octave)
}

/// Convert total semitones back to scale position, semitone offset, and octave
fn semitones_to_scale_position(total_semitones: i8) -> (usize, i8, i8) {
    // Handle negative semitones and octave wrapping
    let octave_adjustment = (total_semitones / 12) as i8;
    let semitones_in_octave = ((total_semitones % 12) + 12) % 12; // Ensure positive
    
    // Map semitones to scale positions (prefer natural notes when possible)
    match semitones_in_octave {
        0 => (0, 0, octave_adjustment),   // C
        1 => (0, 1, octave_adjustment),   // C#
        2 => (1, 0, octave_adjustment),   // D  
        3 => (2, -1, octave_adjustment),  // Eb (prefer Eb over D#)
        4 => (2, 0, octave_adjustment),   // E
        5 => (3, 0, octave_adjustment),   // F
        6 => (3, 1, octave_adjustment),   // F#
        7 => (4, 0, octave_adjustment),   // G
        8 => (5, -1, octave_adjustment),  // Ab (prefer Ab over G#) 
        9 => (5, 0, octave_adjustment),   // A
        10 => (6, -1, octave_adjustment), // Bb (prefer Bb over A#)
        11 => (6, 0, octave_adjustment),  // B
        _ => (0, 0, octave_adjustment),   // Fallback to C
    }
}

/// Transpose a PitchCode based on the tonic (movable-do system) 
pub fn transpose_pitchcode(pitchcode: PitchCode, tonic: PitchCode) -> PitchCode {
    let (transposed_pitchcode, _) = transpose_pitchcode_with_octave(pitchcode, 0, tonic);
    transposed_pitchcode
}

/// Convert PitchCode to scale position (0-6) and semitone offset
fn pitchcode_to_scale_position(pitchcode: PitchCode) -> (usize, i8) {
    use PitchCode::*;
    match pitchcode {
        N1bb => (0, -2), N1b => (0, -1), N1 => (0, 0), N1s => (0, 1), N1ss => (0, 2),
        N2bb => (1, -2), N2b => (1, -1), N2 => (1, 0), N2s => (1, 1), N2ss => (1, 2),
        N3bb => (2, -2), N3b => (2, -1), N3 => (2, 0), N3s => (2, 1), N3ss => (2, 2),
        N4bb => (3, -2), N4b => (3, -1), N4 => (3, 0), N4s => (3, 1), N4ss => (3, 2),
        N5bb => (4, -2), N5b => (4, -1), N5 => (4, 0), N5s => (4, 1), N5ss => (4, 2),
        N6bb => (5, -2), N6b => (5, -1), N6 => (5, 0), N6s => (5, 1), N6ss => (5, 2),
        N7bb => (6, -2), N7b => (6, -1), N7 => (6, 0), N7s => (6, 1), N7ss => (6, 2),
    }
}

/// Convert scale position and semitone offset back to PitchCode
fn scale_position_to_pitchcode(scale_pos: usize, semitone_offset: i8) -> PitchCode {
    use PitchCode::*;
    match (scale_pos, semitone_offset) {
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
    fn test_transpose_pitchcode_7_in_d_major() {
        // Scale degree 7 in D major should be C# (N1s) in octave above
        let result = transpose_pitchcode_with_octave(PitchCode::N7, 0, PitchCode::N2);
        assert_eq!(result, (PitchCode::N1s, 1));
    }
    
    #[test] 
    fn test_transpose_pitchcode_1_in_d_major() {
        // Scale degree 1 in D major should be D (N2) in same octave
        let result = transpose_pitchcode_with_octave(PitchCode::N1, 0, PitchCode::N2);
        assert_eq!(result, (PitchCode::N2, 0));
    }
    
    #[test]
    fn test_transpose_pitchcode_4_in_g_major() {
        // Scale degree 4 in G major should be C (N1) in octave above
        let result = transpose_pitchcode_with_octave(PitchCode::N4, 0, PitchCode::N5);
        assert_eq!(result, (PitchCode::N1, 1));
    }
}