/// Rhythm FSM - Beat processing and tuplet detection
/// Converts flat element lists into beat-grouped structures with subdivision information

use crate::document::model::{MusicalElement, Note, PitchCode};
use fraction::Fraction;

// Beat processing data structures
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Beat {
    pub divisions: usize,
    pub elements: Vec<BeatElement>,
    pub is_tuplet: bool,
    pub tuplet_ratio: Option<(usize, usize)>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BeatElement {
    pub subdivisions: usize,
    pub element_type: BeatElementType,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum BeatElementType {
    Note(Note),
    Rest,
}

#[derive(Debug, PartialEq)]
enum FsmState {
    Initial,              // Between beats
    InBeat,              // Processing beat elements
    CollectingPitch,     // Extending current note with dashes
    CollectingRests,     // Processing leading dashes as rests
}

pub struct RhythmFSM {
    state: FsmState,
    extension_chain_active: bool,
    current_beat: Option<Beat>,
    output: Vec<ProcessedItem>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ProcessedItem {
    Beat(Beat),
    Barline,
    // Could add more items like breath marks, etc.
}

impl RhythmFSM {
    pub fn new() -> Self {
        Self {
            state: FsmState::Initial,
            extension_chain_active: false,
            current_beat: None,
            output: vec![],
        }
    }

    /// Process a flat list of musical elements into beat-grouped structure
    pub fn process(&mut self, elements: &[MusicalElement]) -> Vec<ProcessedItem> {
        for element in elements {
            match (&self.state, element) {
                // Initial state - start new beats or handle structural elements
                (FsmState::Initial, MusicalElement::Note(note)) => {
                    self.start_beat_with_note(note);
                },
                (FsmState::Initial, MusicalElement::Dash { .. }) => {
                    if self.extension_chain_active {
                        // This would be a cross-beat tie, but for now treat as rest
                        self.start_beat_with_rest();
                    } else {
                        self.start_beat_with_rest();
                    }
                },
                (FsmState::Initial, MusicalElement::Space { .. }) => {
                    // Spaces at initial state are ignored (beat separators)
                },
                (FsmState::Initial, MusicalElement::Barline { .. }) => {
                    self.emit_barline();
                },

                // In beat - add elements to current beat
                (FsmState::InBeat, MusicalElement::Note(note)) => {
                    self.add_note_to_beat(note);
                    self.state = FsmState::CollectingPitch;
                },
                (FsmState::InBeat, MusicalElement::Dash { .. }) => {
                    // Start collecting rest extensions
                    self.add_rest_to_beat();
                    self.state = FsmState::CollectingRests;
                },
                (FsmState::InBeat, MusicalElement::Space { .. }) => {
                    self.finish_beat();
                },
                (FsmState::InBeat, MusicalElement::Barline { .. }) => {
                    self.finish_beat();
                    self.emit_barline();
                },

                // Collecting pitch - extending notes with dashes
                (FsmState::CollectingPitch, MusicalElement::Dash { .. }) => {
                    self.extend_last_element();
                },
                (FsmState::CollectingPitch, MusicalElement::Note(note)) => {
                    self.add_note_to_beat(note);
                    // Stay in CollectingPitch for the new note
                },
                (FsmState::CollectingPitch, MusicalElement::Space { .. }) => {
                    self.finish_beat();
                },
                (FsmState::CollectingPitch, MusicalElement::Barline { .. }) => {
                    self.finish_beat();
                    self.emit_barline();
                },

                // Collecting rests - extending rests with dashes
                (FsmState::CollectingRests, MusicalElement::Dash { .. }) => {
                    self.extend_last_element();
                },
                (FsmState::CollectingRests, MusicalElement::Note(note)) => {
                    self.add_note_to_beat(note);
                    self.state = FsmState::CollectingPitch;
                },
                (FsmState::CollectingRests, MusicalElement::Space { .. }) => {
                    self.finish_beat();
                },
                (FsmState::CollectingRests, MusicalElement::Barline { .. }) => {
                    self.finish_beat();
                    self.emit_barline();
                },

                // Handle other element types by ignoring them for now
                _ => {}
            }
        }

        // Finish any pending beat
        if matches!(self.state, FsmState::InBeat | FsmState::CollectingPitch | FsmState::CollectingRests) {
            self.finish_beat();
        }

        self.output.clone()
    }

    fn start_beat_with_note(&mut self, note: &Note) {
        let mut beat = Beat {
            divisions: 1,
            elements: vec![],
            is_tuplet: false,
            tuplet_ratio: None,
        };
        
        beat.elements.push(BeatElement {
            subdivisions: 1,
            element_type: BeatElementType::Note(note.clone()),
        });
        
        self.current_beat = Some(beat);
        self.extension_chain_active = true;
        self.state = FsmState::CollectingPitch;
    }

    fn start_beat_with_rest(&mut self) {
        let mut beat = Beat {
            divisions: 1,
            elements: vec![],
            is_tuplet: false,
            tuplet_ratio: None,
        };
        
        beat.elements.push(BeatElement {
            subdivisions: 1,
            element_type: BeatElementType::Rest,
        });
        
        self.current_beat = Some(beat);
        self.extension_chain_active = false;
        self.state = FsmState::CollectingRests;
    }

    fn add_note_to_beat(&mut self, note: &Note) {
        if let Some(beat) = &mut self.current_beat {
            beat.divisions += 1;
            beat.elements.push(BeatElement {
                subdivisions: 1,
                element_type: BeatElementType::Note(note.clone()),
            });
            self.extension_chain_active = true;
        }
    }

    fn add_rest_to_beat(&mut self) {
        if let Some(beat) = &mut self.current_beat {
            beat.divisions += 1;
            beat.elements.push(BeatElement {
                subdivisions: 1,
                element_type: BeatElementType::Rest,
            });
            self.extension_chain_active = false;
        }
    }

    fn extend_last_element(&mut self) {
        if let Some(beat) = &mut self.current_beat {
            beat.divisions += 1;
            if let Some(last) = beat.elements.last_mut() {
                last.subdivisions += 1;
            }
        }
    }

    fn finish_beat(&mut self) {
        if let Some(mut beat) = self.current_beat.take() {
            // Tuplet detection: not a power of 2
            beat.is_tuplet = beat.divisions > 1 && (beat.divisions & (beat.divisions - 1)) != 0;
            
            if beat.is_tuplet {
                let power_of_2 = self.find_next_lower_power_of_2(beat.divisions);
                beat.tuplet_ratio = Some((beat.divisions, power_of_2));
            }
            
            self.output.push(ProcessedItem::Beat(beat));
        }
        self.state = FsmState::Initial;
    }

    fn emit_barline(&mut self) {
        self.output.push(ProcessedItem::Barline);
        self.extension_chain_active = false;
        self.state = FsmState::Initial;
    }

    fn find_next_lower_power_of_2(&self, n: usize) -> usize {
        let mut power = 1;
        while power * 2 < n {
            power *= 2;
        }
        power.max(2)
    }
}

/// Convenience function to process elements through rhythm FSM
pub fn process_rhythm(elements: &[MusicalElement]) -> Vec<ProcessedItem> {
    let mut fsm = RhythmFSM::new();
    fsm.process(elements)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::model::{PitchCode, NotationSystem, Source, Position};

    fn create_test_note(syllable: &str, pitch_code: PitchCode) -> Note {
        Note {
            syllable: syllable.to_string(),
            octave: 0,
            pitch_code,
            notation_system: NotationSystem::Number,
            source: Source {
                value: syllable.to_string(),
                position: Position { line: 1, column: 1 },
            },
            in_slur: false,
            in_beat_group: false,
        }
    }

    fn create_test_dash() -> MusicalElement {
        MusicalElement::Dash {
            source: Source {
                value: "-".to_string(),
                position: Position { line: 1, column: 1 },
            },
            in_slur: false,
            in_beat_group: false,
        }
    }

    fn create_test_space() -> MusicalElement {
        MusicalElement::Space {
            count: 1,
            source: Source {
                value: " ".to_string(),
                position: Position { line: 1, column: 1 },
            },
            in_slur: false,
            in_beat_group: false,
        }
    }

    #[test]
    fn test_simple_note_sequence() {
        let elements = vec![
            MusicalElement::Note(create_test_note("1", PitchCode::N1)),
            create_test_space(),
            MusicalElement::Note(create_test_note("2", PitchCode::N2)),
        ];

        let result = process_rhythm(&elements);
        assert_eq!(result.len(), 2);

        if let ProcessedItem::Beat(beat) = &result[0] {
            assert_eq!(beat.divisions, 1);
            assert_eq!(beat.elements.len(), 1);
            assert_eq!(beat.elements[0].subdivisions, 1);
            assert!(!beat.is_tuplet);
        } else {
            panic!("Expected beat");
        }
    }

    #[test]
    fn test_dash_extension() {
        let elements = vec![
            MusicalElement::Note(create_test_note("1", PitchCode::N1)),
            create_test_dash(),
            MusicalElement::Note(create_test_note("2", PitchCode::N2)),
        ];

        let result = process_rhythm(&elements);
        assert_eq!(result.len(), 1);

        if let ProcessedItem::Beat(beat) = &result[0] {
            assert_eq!(beat.divisions, 3);
            assert_eq!(beat.elements.len(), 2);
            assert_eq!(beat.elements[0].subdivisions, 2); // Note "1" extended
            assert_eq!(beat.elements[1].subdivisions, 1); // Note "2"
            assert!(beat.is_tuplet);
            assert_eq!(beat.tuplet_ratio, Some((3, 2)));
        } else {
            panic!("Expected beat");
        }
    }

    #[test]
    fn test_leading_dash_as_rest() {
        let elements = vec![
            create_test_dash(),
            MusicalElement::Note(create_test_note("1", PitchCode::N1)),
        ];

        let result = process_rhythm(&elements);
        assert_eq!(result.len(), 1);

        if let ProcessedItem::Beat(beat) = &result[0] {
            assert_eq!(beat.divisions, 2);
            assert_eq!(beat.elements.len(), 2);
            assert_eq!(beat.elements[0].subdivisions, 1); // Rest
            assert_eq!(beat.elements[1].subdivisions, 1); // Note "1"
            
            match &beat.elements[0].element_type {
                BeatElementType::Rest => {},
                _ => panic!("Expected rest"),
            }
        } else {
            panic!("Expected beat");
        }
    }
}