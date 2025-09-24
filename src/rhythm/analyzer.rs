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
pub fn analyze_content_line_rhythm(elements: &mut Vec<ContentElement>) -> Result<(), String> {
    // First pass: determine which beats should be tied
    let mut tie_flags = Vec::new();
    for (i, element) in elements.iter().enumerate() {
        if let ContentElement::Beat(beat) = element {
            let should_tie = should_tie_to_previous(beat, elements, i);
            tie_flags.push((i, should_tie)); // Store both index and tie decision
        }
    }

    // Second pass: apply rhythm analysis with tie information
    for element in elements.iter_mut() {
        if let ContentElement::Beat(beat) = element {
            // Find the tie decision for this beat by matching position
            let beat_char_index = beat.char_index;
            let should_tie = tie_flags.iter()
                .find(|(_, _)| {
                    // Match by the beat's char_index or just use order
                    true // For now, just use order
                })
                .map(|(_, tie)| *tie)
                .unwrap_or(false);

            // Remove the first entry since we're processing in order
            if !tie_flags.is_empty() {
                let (_, should_tie) = tie_flags.remove(0);
                analyze_beat_rhythm_fsm(beat, should_tie)?;
            }
        }
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
enum State {
    Initial,
    InNote { note_index: usize },
    InRest { rest_index: usize },
}

/// Check if a beat starting with dashes should be tied to a previous note/rest
fn should_tie_to_previous(beat: &Beat, all_elements: &[ContentElement], current_index: usize) -> bool {
    // Check if this beat starts with dashes
    let starts_with_dash = beat.elements.first()
        .map(|e| matches!(e, BeatElement::Dash(_)))
        .unwrap_or(false);


    if !starts_with_dash {
        return false;
    }

    // Look backwards through previous elements to find a note or rest
    for i in (0..current_index).rev() {
        match &all_elements[i] {
            ContentElement::Beat(prev_beat) => {
                // Check the last element of this beat
                if let Some(last_element) = prev_beat.elements.last() {
                    match last_element {
                        BeatElement::Note(_) => {
                            return true;
                        }
                        BeatElement::Rest(_) => {
                            return true;
                        }
                        BeatElement::BreathMark(_) => {
                            return false;
                        }
                        BeatElement::Dash(_) => {
                            continue;
                        }
                    }
                }
            }
            ContentElement::Barline(_) => {
                return false;
            }
            ContentElement::Whitespace(_) => {
                continue;
            }
            ContentElement::UnknownToken(_) => {
                // Unknown tokens behave like whitespace
                continue;
            }
        }
    }

    false
}

/// FSM-based rhythm analyzer for a single beat
fn analyze_beat_rhythm_fsm(beat: &mut Beat, should_tie: bool) -> Result<(), String> {
    let element_count = beat.elements.len();

    if element_count == 0 {
        return Ok(());
    }

    // Set tied_to_previous based on the analysis
    beat.tied_to_previous = Some(should_tie);

    // Track subdivision counts for each note and rest
    let mut note_subdivisions = Vec::new();
    let mut rest_subdivisions = Vec::new();
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
            (State::InRest { .. }, BeatElement::Note(_)) => {
                // Start new note after rest
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
            (State::InRest { rest_index }, BeatElement::Dash(_)) => {
                if let Some(subdivisions) = rest_subdivisions.get_mut(*rest_index) {
                    *subdivisions += 1;
                    total_subdivisions += 1;
                }
                // Stay in same state
            }
            (State::Initial, BeatElement::Dash(_)) => {
                // Dash without preceding note - start new rest
                rest_subdivisions.push(1);
                total_subdivisions += 1;
                state = State::InRest { rest_index: rest_subdivisions.len() - 1 };
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
            (State::InRest { .. }, BeatElement::Rest(_)) => {
                // Rest after rest - treat as new element
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

    // Apply calculated durations to notes and starting dashes (rests)
    let mut note_index = 0;
    let mut rest_index = 0;
    let mut current_state = State::Initial;

    for beat_element in &mut beat.elements {
        match beat_element {
            BeatElement::Note(note) => {
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
                    current_state = State::InNote { note_index: note_index - 1 };
                }
            }
            BeatElement::Dash(dash) => {
                match current_state {
                    State::Initial => {
                        // Starting dash - check if this beat is tied to previous
                        if should_tie {
                            // This dash is a tie continuation, don't assign rhythm data
                            // The renderer will handle the tie continuation
                        } else {
                            // Starting dash for a new rest, assign rhythm data
                            if rest_index < rest_subdivisions.len() {
                                let subdivisions = rest_subdivisions[rest_index];
                                let duration = Fraction::new(subdivisions as u64, total_subdivisions as u64) * Fraction::new(1u64, 4u64);
                                let numer = *duration.numer().unwrap() as u32;
                                let denom = *duration.denom().unwrap() as u32;
                                dash.numerator = Some(numer);
                                dash.denominator = Some(denom);

                                rest_index += 1;
                                current_state = State::InRest { rest_index: rest_index - 1 };
                            }
                        }
                    }
                    State::InNote { .. } => {
                        // Extending note - no rhythm data needed
                        // State stays the same
                    }
                    State::InRest { .. } => {
                        // Extending rest - no rhythm data needed
                        // State stays the same
                    }
                }
            }
            _ => {
                // Other elements don't change state
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

/// Calculate tuplet ratio using systematic subdivision approach
fn calculate_tuplet_ratio(divisions: usize) -> (usize, usize) {
    // Systematic subdivision by note count ranges
    let denominator = match divisions {
        1..=2 => 1,    // Quarter note subdivisions
        3..=3 => 2,    // Eighth note subdivisions
        4..=4 => 4,    // Quarter note (power of 2, not tuplet)
        5..=7 => 4,    // Sixteenth note subdivisions
        8..=8 => 8,    // Eighth note (power of 2, not tuplet)
        9..=15 => 8,   // Thirty-second note subdivisions
        16..=16 => 16, // Sixteenth note (power of 2, not tuplet)
        17..=31 => 16, // Sixty-fourth note subdivisions
        32..=32 => 32, // Thirty-second note (power of 2, not tuplet)
        33..=63 => 32, // 128th note subdivisions
        64..=64 => 64, // Sixty-fourth note (power of 2, not tuplet)
        65..=127 => 64, // 256th note subdivisions
        _ => {
            // For extreme cases, find next power of 2
            let mut power_of_two = 1;
            while power_of_two < divisions {
                power_of_two *= 2;
            }
            power_of_two / 2 // Use the subdivision count that fits in a quarter beat
        }
    };

    (divisions, denominator)
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
                BeatElement::Dash(Dash { value: Some("-".to_string()), char_index: 1, consumed_elements: vec![], numerator: None, denominator: None }),
                BeatElement::Dash(Dash { value: Some("-".to_string()), char_index: 2, consumed_elements: vec![], numerator: None, denominator: None }),
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

        analyze_beat_rhythm_fsm(&mut beat, false).unwrap(); // Not tied for this test

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

    #[test]
    fn test_standalone_dash_sequences() {
        // Test case: "-- -1" (2 beats: rest sequence + dash+note)
        // Beat 1: "--" should have first dash as rest, second as extender
        let mut beat1 = Beat {
            elements: vec![
                BeatElement::Dash(Dash { value: Some("-".to_string()), char_index: 0, consumed_elements: vec![], numerator: None, denominator: None }),
                BeatElement::Dash(Dash { value: Some("-".to_string()), char_index: 1, consumed_elements: vec![], numerator: None, denominator: None }),
            ],
            value: Some("--".to_string()),
            char_index: 0,
            consumed_elements: vec![],
            divisions: None,
            total_duration: None,
            is_tuplet: None,
            tuplet_ratio: None,
            tied_to_previous: None,
        };

        analyze_beat_rhythm_fsm(&mut beat1, false).unwrap(); // Not tied

        // Check first dash gets rhythm data (rest)
        if let BeatElement::Dash(dash1) = &beat1.elements[0] {
            assert_eq!(dash1.numerator, Some(1));
            assert_eq!(dash1.denominator, Some(4)); // 2/2 * 1/4 = 1/4
        }

        // Check second dash has no rhythm data (extender)
        if let BeatElement::Dash(dash2) = &beat1.elements[1] {
            assert_eq!(dash2.numerator, None);
            assert_eq!(dash2.denominator, None);
        }

        assert_eq!(beat1.divisions, Some(2)); // Total subdivisions

        // Beat 2: "-1" should have dash as rest, note gets rhythm data
        let mut beat2 = Beat {
            elements: vec![
                BeatElement::Dash(Dash { value: Some("-".to_string()), char_index: 3, consumed_elements: vec![], numerator: None, denominator: None }),
                BeatElement::Note(Note::new(Some("1".to_string()), 4, PitchCode::N1, NotationSystem::Number)),
            ],
            value: Some("-1".to_string()),
            char_index: 3,
            consumed_elements: vec![],
            divisions: None,
            total_duration: None,
            is_tuplet: None,
            tuplet_ratio: None,
            tied_to_previous: None,
        };

        analyze_beat_rhythm_fsm(&mut beat2, false).unwrap(); // Not tied

        // Check dash gets rhythm data (rest)
        if let BeatElement::Dash(dash) = &beat2.elements[0] {
            assert_eq!(dash.numerator, Some(1));
            assert_eq!(dash.denominator, Some(8)); // 1/2 * 1/4 = 1/8
        }

        // Check note gets rhythm data
        if let BeatElement::Note(note) = &beat2.elements[1] {
            assert_eq!(note.numerator, Some(1));
            assert_eq!(note.denominator, Some(8)); // 1/2 * 1/4 = 1/8
        }

        assert_eq!(beat2.divisions, Some(2)); // Total subdivisions
    }

    #[test]
    fn test_tied_dashes() {
        // Test case: "-547 -5" where -5 is tied to previous
        // Beat 1: "-547" should have dash as rest, notes get rhythm data
        let mut beat1 = Beat {
            elements: vec![
                BeatElement::Dash(Dash { value: Some("-".to_string()), char_index: 0, consumed_elements: vec![], numerator: None, denominator: None }),
                BeatElement::Note(Note::new(Some("5".to_string()), 1, PitchCode::N5, NotationSystem::Number)),
                BeatElement::Note(Note::new(Some("4".to_string()), 2, PitchCode::N4, NotationSystem::Number)),
                BeatElement::Note(Note::new(Some("7".to_string()), 3, PitchCode::N7, NotationSystem::Number)),
            ],
            value: Some("-547".to_string()),
            char_index: 0,
            consumed_elements: vec![],
            divisions: None,
            total_duration: None,
            is_tuplet: None,
            tuplet_ratio: None,
            tied_to_previous: None,
        };

        analyze_beat_rhythm_fsm(&mut beat1, false).unwrap(); // Not tied

        // Check dash gets rhythm data (rest)
        if let BeatElement::Dash(dash) = &beat1.elements[0] {
            assert_eq!(dash.numerator, Some(1));
            assert_eq!(dash.denominator, Some(16)); // 1/4 * 1/4 = 1/16
        }

        // Check all notes get rhythm data
        for i in 1..4 {
            if let BeatElement::Note(note) = &beat1.elements[i] {
                assert_eq!(note.numerator, Some(1));
                assert_eq!(note.denominator, Some(16)); // 1/4 * 1/4 = 1/16 each
            }
        }

        assert_eq!(beat1.divisions, Some(4)); // Total subdivisions

        // Beat 2: "-5" tied to previous - dash should NOT get rhythm data
        let mut beat2 = Beat {
            elements: vec![
                BeatElement::Dash(Dash { value: Some("-".to_string()), char_index: 5, consumed_elements: vec![], numerator: None, denominator: None }),
                BeatElement::Note(Note::new(Some("5".to_string()), 6, PitchCode::N5, NotationSystem::Number)),
            ],
            value: Some("-5".to_string()),
            char_index: 5,
            consumed_elements: vec![],
            divisions: None,
            total_duration: None,
            is_tuplet: None,
            tuplet_ratio: None,
            tied_to_previous: None,
        };

        analyze_beat_rhythm_fsm(&mut beat2, true).unwrap(); // TIED to previous

        // Check dash has NO rhythm data (tie continuation)
        if let BeatElement::Dash(dash) = &beat2.elements[0] {
            assert_eq!(dash.numerator, None);
            assert_eq!(dash.denominator, None);
        }

        // Check note gets rhythm data
        if let BeatElement::Note(note) = &beat2.elements[1] {
            assert_eq!(note.numerator, Some(1));
            assert_eq!(note.denominator, Some(8)); // 1/2 * 1/4 = 1/8
        }

        assert_eq!(beat2.divisions, Some(2)); // Total subdivisions
        assert_eq!(beat2.tied_to_previous, Some(true)); // Confirms tied
    }
}
