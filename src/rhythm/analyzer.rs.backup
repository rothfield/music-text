// Rhythm analyzer that adds duration information to parsed documents
use crate::parse::model::{Document, DocumentElement, StaveLine, ContentElement, Beat, BeatElement, Note};
use fraction::Fraction;

/// Analyze rhythm patterns and add duration information to the document
/// This function modifies the document in place, adding duration info to Notes and Beats
pub fn analyze_rhythm_into_document(document: &mut Document) -> Result<(), String> {
    // Walk through all staves and content lines
    for element in &mut document.elements {
        if let DocumentElement::Stave(stave) = element {
            for line in &mut stave.lines {
                if let StaveLine::ContentLine(content_line) = line {
                    analyze_content_line_rhythm(&mut content_line.elements)?;
                }
            }
        }
    }
    Ok(())
}

/// Analyze rhythm for a content line (sequence of beats and other elements)
fn analyze_content_line_rhythm(elements: &mut Vec<ContentElement>) -> Result<(), String> {
    for element in elements {
        if let ContentElement::Beat(beat) = element {
            analyze_beat_rhythm(beat)?;
        }
    }
    Ok(())
}

/// Analyze rhythm for a single beat
fn analyze_beat_rhythm(beat: &mut Beat) -> Result<(), String> {
    let element_count = beat.elements.len();

    if element_count == 0 {
        return Ok(());
    }

    // Analyze based on the number of elements in the beat
    // This implements the doremi-script rhythm logic where:
    // - Single element = quarter note (1/4)
    // - Multiple elements = subdivisions (1/8, 1/16, etc.)

    let (individual_duration, beat_total_duration) = determine_beat_durations(element_count);

    // Set beat-level metadata
    beat.divisions = Some(element_count);
    beat.total_duration = Some(beat_total_duration);

    // Determine if this is a tuplet (non-power of 2 divisions)
    let is_tuplet = is_tuplet_division(element_count);
    beat.is_tuplet = Some(is_tuplet);

    // Calculate tuplet ratio if applicable
    if is_tuplet {
        beat.tuplet_ratio = Some(calculate_tuplet_ratio(element_count));
    }

    // Set duration for each note in the beat
    for beat_element in &mut beat.elements {
        if let BeatElement::Note(note) = beat_element {
            // Convert fraction to simple numerator/denominator
            let numer = *individual_duration.numer().unwrap() as u32;
            let denom = *individual_duration.denom().unwrap() as u32;
            note.numerator = Some(numer);
            note.denominator = Some(denom);
        }
        // Dashes and breath marks don't get duration - they're handled differently
    }

    Ok(())
}

/// Determine duration based on the number of elements in a beat
/// Returns (individual_note_duration, total_beat_duration)
/// Check if a division count represents a tuplet (non-power of 2)
fn is_tuplet_division(divisions: usize) -> bool {
    // Powers of 2 (1, 2, 4, 8, 16, etc.) are normal subdivisions
    // Non-powers of 2 (3, 5, 6, 7, etc.) are tuplets
    divisions > 0 && (divisions & (divisions - 1)) != 0
}

/// Calculate tuplet ratio for a given number of divisions
fn calculate_tuplet_ratio(divisions: usize) -> (usize, usize) {
    match divisions {
        3 => (3, 2),   // Triplet: 3 notes in the time of 2
        5 => (5, 4),   // Quintuplet: 5 notes in the time of 4
        6 => (6, 4),   // Sextuplet: 6 notes in the time of 4
        7 => (7, 4),   // Septuplet: 7 notes in the time of 4
        9 => (9, 8),   // Nonuplet: 9 notes in the time of 8
        10 => (10, 8), // 10 notes in the time of 8
        11 => (11, 8), // 11 notes in the time of 8
        12 => (12, 8), // 12 notes in the time of 8
        _ => {
            // For other cases, find the nearest power of 2
            let mut power_of_two = 1;
            while power_of_two < divisions {
                power_of_two *= 2;
            }
            if power_of_two > divisions {
                power_of_two /= 2;
            }
            (divisions, power_of_two)
        }
    }
}

fn determine_beat_durations(element_count: usize) -> (Fraction, Fraction) {
    match element_count {
        1 => {
            // Single element = quarter note
            (Fraction::new(1u64, 4u64), Fraction::new(1u64, 4u64))
        }
        2 => {
            // Two elements = eighth notes
            (Fraction::new(1u64, 8u64), Fraction::new(1u64, 4u64))
        }
        3 => {
            // Three elements = eighth note triplet (each is 1/12)
            (Fraction::new(1u64, 12u64), Fraction::new(1u64, 4u64))
        }
        4 => {
            // Four elements = sixteenth notes
            (Fraction::new(1u64, 16u64), Fraction::new(1u64, 4u64))
        }
        6 => {
            // Six elements = sixteenth note triplet (each is 1/24)
            (Fraction::new(1u64, 24u64), Fraction::new(1u64, 4u64))
        }
        8 => {
            // Eight elements = thirty-second notes
            (Fraction::new(1u64, 32u64), Fraction::new(1u64, 4u64))
        }
        12 => {
            // Twelve elements = thirty-second note triplet (each is 1/48)
            (Fraction::new(1u64, 48u64), Fraction::new(1u64, 4u64))
        }
        16 => {
            // Sixteen elements = sixty-fourth notes
            (Fraction::new(1u64, 64u64), Fraction::new(1u64, 4u64))
        }
        n => {
            // For other counts, calculate based on subdividing a quarter note
            // This handles cases like 5, 7, 9, 10, 11, etc.
            let denominator = 4 * n as u64;
            (Fraction::new(1u64, denominator), Fraction::new(1u64, 4u64))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_beat_durations() {
        assert_eq!(determine_beat_durations(1), (Fraction::new(1u64, 4u64), Fraction::new(1u64, 4u64)));
        assert_eq!(determine_beat_durations(2), (Fraction::new(1u64, 8u64), Fraction::new(1u64, 4u64)));
        assert_eq!(determine_beat_durations(4), (Fraction::new(1u64, 16u64), Fraction::new(1u64, 4u64)));
        assert_eq!(determine_beat_durations(12), (Fraction::new(1u64, 48u64), Fraction::new(1u64, 4u64)));
    }
}