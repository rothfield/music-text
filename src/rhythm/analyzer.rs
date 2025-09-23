// Rhythm analyzer FSM based on bak.src/rhythm_fsm.rs
// Adapted to work with current parse model structures
use crate::parse::model::{Document, DocumentElement, StaveLine, ContentElement, Beat, BeatElement, Note, Rest, Dash};
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
            analyze_beat_rhythm_fsm(beat)?;
        }
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
enum State {
    Initial,
    InNote { note_index: usize },
}

/// FSM-based rhythm analyzer for a single beat
fn analyze_beat_rhythm_fsm(beat: &mut Beat) -> Result<(), String> {
    let element_count = beat.elements.len();

    if element_count == 0 {
        return Ok(());
    }

    // Check for leading dashes before any note
    let mut leading_dash_count = 0;
    let mut first_dash_index = None;
    for (i, element) in beat.elements.iter().enumerate() {
        match element {
            BeatElement::Dash(_) => {
                if first_dash_index.is_none() {
                    first_dash_index = Some(i);
                }
                leading_dash_count += 1;
            }
            BeatElement::Note(_) => {
                // Found a note, stop checking
                break;
            }
            BeatElement::BreathMark(_) => {
                // Continue checking past breath marks
            }
            BeatElement::Rest(_) => {
                // Shouldn't happen in initial parsing, but handle gracefully
                break;
            }
        }
    }

    // If we have leading dashes, set tied_to_previous
    // The renderer will decide whether to create a tie (if previous note exists) or rest
    if leading_dash_count > 0 {
        beat.tied_to_previous = Some(true);

        // Convert leading dashes to a Rest element if needed
        // This will be handled by checking tied_to_previous in context
    }

    // Track subdivision counts for each note
    let mut note_subdivisions = Vec::new();
    let mut state = State::Initial;
    let mut total_subdivisions = 0;

    // FSM processing
    for element in &beat.elements {
        match (&state, element) {
            // Note encountered
            (State::Initial, BeatElement::Note(_)) => {
                note_subdivisions.push(1);
                total_subdivisions += 1;
                state = State::InNote { note_index: note_subdivisions.len() - 1 };
            }
            (State::InNote { note_index }, BeatElement::Note(_)) => {
                // Start new note
                note_subdivisions.push(1);
                total_subdivisions += 1;
                state = State::InNote { note_index: note_subdivisions.len() - 1 };
            }

            // Dash encountered (duration extender)
            (State::InNote { note_index }, BeatElement::Dash(_)) => {
                if let Some(subdivisions) = note_subdivisions.get_mut(*note_index) {
                    *subdivisions += 1;
                    total_subdivisions += 1;
                }
                // Stay in same state
            }
            (State::Initial, BeatElement::Dash(_)) => {
                // Dash without preceding note - treat as rest (ignore for now)
                total_subdivisions += 1;
            }

            // Rest encountered
            (State::Initial, BeatElement::Rest(_)) => {
                // Rest at beginning
                total_subdivisions += 1;
            }
            (State::InNote { .. }, BeatElement::Rest(_)) => {
                // Rest after note - treat as new element
                total_subdivisions += 1;
            }

            // Breath mark encountered (ignored)
            (_, BeatElement::BreathMark(_)) => {
                // Breath marks are ignored, state unchanged
            }
        }
    }

    // Set beat-level metadata
    beat.divisions = Some(total_subdivisions);
    beat.total_duration = Some(Fraction::new(1u64, 4u64)); // Beat is always 1/4 note

    // Determine if this is a tuplet (non-power of 2 divisions)
    let is_tuplet = is_tuplet_division(total_subdivisions);
    beat.is_tuplet = Some(is_tuplet);

    // Calculate tuplet ratio if applicable
    if is_tuplet {
        beat.tuplet_ratio = Some(calculate_tuplet_ratio(total_subdivisions));
    }

    // Apply calculated durations to notes
    let mut note_index = 0;
    for beat_element in &mut beat.elements {
        if let BeatElement::Note(note) = beat_element {
            if note_index < note_subdivisions.len() {
                let subdivisions = note_subdivisions[note_index];

                // Calculate duration as fraction of beat
                let duration = Fraction::new(subdivisions as u64, total_subdivisions as u64) * Fraction::new(1u64, 4u64);

                // Convert to numerator/denominator
                let numer = *duration.numer().unwrap() as u32;
                let denom = *duration.denom().unwrap() as u32;
                note.numerator = Some(numer);
                note.denominator = Some(denom);

                note_index += 1;
            }
        }
    }

    Ok(())
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::model::*;

    #[test]
    fn test_fsm_dash_extension() {
        // Create a beat with pattern: Note, Dash, Dash, Note (1--2)
        let mut beat = Beat {
            elements: vec![
                BeatElement::Note(Note::new(Some("1".to_string()), 0, PitchCode::N1, NotationSystem::Number)),
                BeatElement::Dash(Dash { value: Some("-".to_string()), char_index: 1, consumed_elements: vec![] }),
                BeatElement::Dash(Dash { value: Some("-".to_string()), char_index: 2, consumed_elements: vec![] }),
                BeatElement::Note(Note::new(Some("2".to_string()), 3, PitchCode::N2, NotationSystem::Number)),
            ],
            value: Some("1--2".to_string()),
            char_index: 0,
            consumed_elements: vec![],
            divisions: None,
            total_duration: None,
            is_tuplet: None,
            tuplet_ratio: None,
            tied_to_previous: None,
        };

        analyze_beat_rhythm_fsm(&mut beat).unwrap();

        // Check that first note gets 3/4 of beat (3 subdivisions out of 4)
        if let BeatElement::Note(note1) = &beat.elements[0] {
            assert_eq!(note1.numerator, Some(3));
            assert_eq!(note1.denominator, Some(16)); // 3/4 of 1/4 = 3/16
        }

        // Check that second note gets 1/4 of beat (1 subdivision out of 4)
        if let BeatElement::Note(note2) = &beat.elements[3] {
            assert_eq!(note2.numerator, Some(1));
            assert_eq!(note2.denominator, Some(16)); // 1/4 of 1/4 = 1/16
        }

        assert_eq!(beat.divisions, Some(4)); // Total subdivisions
    }
}
