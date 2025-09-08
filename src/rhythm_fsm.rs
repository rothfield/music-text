/// Sophisticated Rhythm FSM - Copied from old working system
/// Converts flat element lists into beat-grouped structures with proper fraction-based durations

use crate::document::model::{MusicalElement, PitchCode};
use crate::old_models::*;
use fraction::Fraction;

// FSM output structures (from old parser_v2_fsm.rs)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Event {
    Note {
        degree: Degree,
        octave: i8,
        children: Vec<ParsedChild>,  // syllables, ornaments, octave markers
        slur: Option<SlurRole>,
    },
    Rest,  // Rests have no additional data
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BeatElement {
    pub event: Event,
    pub subdivisions: usize,
    pub duration: Fraction,               // Actual beat fraction: subdivisions/divisions  
    pub tuplet_duration: Fraction,        // Mathematical tuplet duration (1/6, 1/3, etc.)
    pub tuplet_display_duration: Option<Fraction>, // Display duration for tuplets (1/16, 1/8, etc.), None for regular notes
    pub value: String,                    // Original text value
    pub position: Position,               // Source position
}

impl BeatElement {
    pub fn with_subdivisions(mut self, subdivisions: usize) -> Self {
        self.subdivisions = subdivisions;
        self
    }
    
    pub fn extend_subdivision(&mut self) {
        self.subdivisions += 1;
    }
    
    pub fn is_note(&self) -> bool {
        matches!(self.event, Event::Note { .. })
    }
    
    pub fn is_rest(&self) -> bool {
        matches!(self.event, Event::Rest)
    }
    
    pub fn as_note(&self) -> Option<(&Degree, &i8, &Vec<ParsedChild>, &Option<SlurRole>)> {
        if let Event::Note { degree, octave, children, slur } = &self.event {
            Some((degree, octave, children, slur))
        } else {
            None
        }
    }
    
    pub fn syl(&self) -> Option<String> {
        // Extract syllable from ParsedChild::Syllable in children
        if let Event::Note { children, .. } = &self.event {
            children.iter().rev().find_map(|child| match child {
                ParsedChild::Syllable { text, .. } => Some(text.clone()),
                _ => None
            })
        } else {
            None
        }
    }
    
    pub fn ornaments(&self) -> Vec<OrnamentType> {
        if let Event::Note { children, .. } = &self.event {
            children.iter().filter_map(|child| {
                if let ParsedChild::Ornament { kind, .. } = child {
                    Some(kind.clone())
                } else {
                    None
                }
            }).collect()
        } else {
            Vec::new()
        }
    }
}

impl From<ParsedElement> for BeatElement {
    fn from(element: ParsedElement) -> Self {
        let (event, value, position) = match element {
            ParsedElement::Note { degree, octave, value, position, children, slur, .. } => {
                let event = Event::Note { degree, octave, children, slur };
                (event, value, position)
            },
            ParsedElement::Rest { value, position, .. } => {
                (Event::Rest, value, position)
            },
            ParsedElement::Dash { degree, octave, position, .. } => {
                // Dash creates a tied note if it has degree/octave, otherwise it's handled as rest
                if let (Some(deg), Some(oct)) = (degree, octave) {
                    let event = Event::Note { 
                        degree: deg, 
                        octave: oct, 
                        children: vec![], 
                        slur: None 
                    };
                    (event, "-".to_string(), position)
                } else {
                    (Event::Rest, "-".to_string(), position)
                }
            },
            _ => {
                // Other elements (Barline, Whitespace, etc.) shouldn't reach here
                // but we'll handle them as rests for safety
                return BeatElement {
                    event: Event::Rest,
                    subdivisions: 1,
                    duration: Fraction::new(0u64, 1u64),
                    tuplet_duration: Fraction::new(0u64, 1u64),
                    tuplet_display_duration: None,
                    value: "".to_string(),
                    position: Position { row: 0, col: 0 },
                };
            }
        };
        
        BeatElement {
            event,
            subdivisions: 1,
            duration: Fraction::new(1u64, 4u64),
            tuplet_duration: Fraction::new(1u64, 4u64),
            tuplet_display_duration: None,
            value,
            position,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Beat {
    pub divisions: usize,
    pub elements: Vec<BeatElement>,
    pub tied_to_previous: bool,
    pub is_tuplet: bool,                      
    pub tuplet_ratio: Option<(usize, usize)>, 
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Item {
    Beat(Beat),
    Barline(BarlineType, Option<u8>), // BarlineType and optional tala (0-6)
    Breathmark,
    Tonic(Degree), // Tonic/Key declaration (e.g., "key: D" -> Degree::N2)
}

// FSM State Machine (simplified version of old rhythm_fsm.rs)
#[derive(Debug, PartialEq)]
enum State {
    S0,                  // Initial/Between elements
    InBeat,             // Processing beat elements
    CollectingPitch,    // Extending current note with dashes
    CollectingRests,    // Processing leading dashes as rests
    Halt,               // End of processing
}

struct FSM {
    state: State,
    extension_chain_active: bool,
    last_note_across_beats: Option<Degree>,
    current_beat: Option<Beat>,
    output: Vec<Item>,
}

impl FSM {
    fn new() -> Self {
        Self {
            state: State::S0,
            extension_chain_active: false,
            last_note_across_beats: None,
            current_beat: None,
            output: vec![],
        }
    }

    fn process(&mut self, elements: &[ParsedElement]) {
        for element in elements {
            match (&self.state, element) {
                // S0 State - Initial/Between elements
                (State::S0, ParsedElement::Note { .. }) => {
                    self.start_beat_with_note(element);
                },
                (State::S0, ParsedElement::Dash { .. }) => {
                    if self.extension_chain_active && self.last_note_across_beats.is_some() {
                        self.start_beat_with_tied_note(element);
                    } else {
                        self.start_beat_with_rest(element);
                    }
                },
                (State::S0, ParsedElement::Barline { .. }) => {
                    self.emit_barline(element);
                },
                (State::S0, ParsedElement::Symbol { value, .. }) if value == "'" => {
                    self.handle_breathmark();
                },
                (State::S0, ParsedElement::Whitespace { .. }) => {
                    // Whitespace in S0 is ignored (beat separators handled elsewhere)
                },

                // InBeat State - Processing beat elements
                (State::InBeat, ParsedElement::Note { .. }) => {
                    self.add_note_to_beat(element);
                    self.state = State::CollectingPitch;
                },
                (State::InBeat, ParsedElement::Dash { .. }) => {
                    self.add_rest_to_beat(element);
                    self.state = State::CollectingRests;
                },
                (State::InBeat, ParsedElement::Whitespace { .. }) => {
                    self.finish_beat();
                    self.state = State::S0;
                },
                (State::InBeat, ParsedElement::Barline { .. }) => {
                    self.finish_beat();
                    self.emit_barline(element);
                    self.state = State::S0;
                },

                // CollectingPitch State - Extending current note
                (State::CollectingPitch, ParsedElement::Dash { .. }) => {
                    self.extend_last_element();
                },
                (State::CollectingPitch, ParsedElement::Note { .. }) => {
                    self.add_note_to_beat(element);
                },
                (State::CollectingPitch, ParsedElement::Whitespace { .. }) => {
                    self.finish_beat();
                    self.state = State::S0;
                },
                (State::CollectingPitch, ParsedElement::Barline { .. }) => {
                    self.finish_beat();
                    self.emit_barline(element);
                    self.state = State::S0;
                },

                // CollectingRests State - Processing rest extensions
                (State::CollectingRests, ParsedElement::Dash { .. }) => {
                    self.extend_last_element();
                },
                (State::CollectingRests, ParsedElement::Note { .. }) => {
                    self.add_note_to_beat(element);
                    self.state = State::CollectingPitch;
                },
                (State::CollectingRests, ParsedElement::Whitespace { .. }) => {
                    self.finish_beat();
                    self.state = State::S0;
                },
                (State::CollectingRests, ParsedElement::Barline { .. }) => {
                    self.finish_beat();
                    self.emit_barline(element);
                    self.state = State::S0;
                },

                // Unhandled combinations
                _ => {
                    // Skip unhandled transitions
                }
            }
        }

        // Finish any pending beat
        if matches!(self.state, State::InBeat | State::CollectingPitch | State::CollectingRests) {
            self.finish_beat();
        }
        self.state = State::Halt;
    }

    fn start_beat_with_note(&mut self, element: &ParsedElement) {
        let tied_to_previous = self.check_for_tie(element);
        
        let mut beat = Beat {
            divisions: 1,
            elements: vec![],
            tied_to_previous,
            is_tuplet: false,
            tuplet_ratio: None,
        };
        
        beat.elements.push(BeatElement::from(element.clone()).with_subdivisions(1));
        self.current_beat = Some(beat);
        self.update_extension_chain(element);
        self.state = State::CollectingPitch;
    }

    fn start_beat_with_tied_note(&mut self, dash_element: &ParsedElement) {
        if let Some(degree) = self.last_note_across_beats {
            // Create tied note with same pitch as last note
            let tied_note = ParsedElement::Note {
                degree,
                octave: 0, // Default octave - could be improved
                value: format!("{:?}", degree),
                position: dash_element.position().clone(),
                children: vec![],
                duration: None,
                slur: None,
            };

            let mut beat = Beat {
                divisions: 1,
                elements: vec![],
                tied_to_previous: true,
                is_tuplet: false,
                tuplet_ratio: None,
            };
            
            beat.elements.push(BeatElement::from(tied_note).with_subdivisions(1));
            self.current_beat = Some(beat);
            self.state = State::CollectingPitch;
        } else {
            // Fallback to rest if no previous note
            self.start_beat_with_rest(dash_element);
        }
    }

    fn start_beat_with_rest(&mut self, dash_element: &ParsedElement) {
        let rest_element = ParsedElement::Rest {
            value: "r".to_string(),
            position: dash_element.position().clone(),
            duration: None,
        };

        let mut beat = Beat {
            divisions: 1,
            elements: vec![],
            tied_to_previous: false,
            is_tuplet: false,
            tuplet_ratio: None,
        };
        
        beat.elements.push(BeatElement::from(rest_element).with_subdivisions(1));
        self.current_beat = Some(beat);
        self.state = State::CollectingRests;
    }

    fn add_note_to_beat(&mut self, element: &ParsedElement) {
        if let Some(beat) = &mut self.current_beat {
            beat.divisions += 1;
            beat.elements.push(BeatElement::from(element.clone()).with_subdivisions(1));
            self.update_extension_chain(element);
        }
    }

    fn add_rest_to_beat(&mut self, dash_element: &ParsedElement) {
        let rest_element = ParsedElement::Rest {
            value: "r".to_string(),
            position: dash_element.position().clone(),
            duration: None,
        };

        if let Some(beat) = &mut self.current_beat {
            beat.divisions += 1;
            beat.elements.push(BeatElement::from(rest_element).with_subdivisions(1));
        }
    }

    fn extend_last_element(&mut self) {
        if let Some(beat) = &mut self.current_beat {
            beat.divisions += 1;
            if let Some(last) = beat.elements.last_mut() {
                last.extend_subdivision();
            }
        }
    }

    fn check_for_tie(&mut self, element: &ParsedElement) -> bool {
        if let ParsedElement::Note { degree, .. } = element {
            if let Some(pending_pitch) = self.last_note_across_beats {
                if *degree == pending_pitch {
                    self.last_note_across_beats = None; // Clear after using
                    return true;
                }
            }
        }
        false
    }

    fn update_extension_chain(&mut self, element: &ParsedElement) {
        if let ParsedElement::Note { degree, .. } = element {
            self.last_note_across_beats = Some(*degree);
            self.extension_chain_active = true;
        }
    }

    fn handle_breathmark(&mut self) {
        self.extension_chain_active = false;
        self.last_note_across_beats = None;
        self.output.push(Item::Breathmark);
    }

    fn finish_beat(&mut self) {
        if let Some(mut beat) = self.current_beat.take() {
            // Tuplet detection: not a power of 2 AND more than one element
            beat.is_tuplet = beat.elements.len() > 1 && 
                           beat.divisions > 1 && 
                           (beat.divisions & (beat.divisions - 1)) != 0;
            
            if beat.is_tuplet {
                let power_of_2 = Self::find_next_lower_power_of_2(beat.divisions);
                beat.tuplet_ratio = Some((beat.divisions, power_of_2));
                
                // Calculate duration for tuplet elements
                let each_unit = Fraction::new(1u64, 4u64) / power_of_2;
                for beat_element in &mut beat.elements {
                    // Actual duration (subdivision fraction of the beat)
                    beat_element.duration = Fraction::new(beat_element.subdivisions as u64, beat.divisions as u64);
                    // Tuplet duration for display
                    beat_element.tuplet_duration = each_unit * beat_element.subdivisions;
                    beat_element.tuplet_display_duration = Some(each_unit * beat_element.subdivisions);
                }
            } else {
                // Regular beat processing
                for beat_element in &mut beat.elements {
                    let base_duration = Fraction::new(beat_element.subdivisions as u64, beat.divisions as u64) 
                                      * Fraction::new(1u64, 4u64);
                    beat_element.duration = base_duration;
                    beat_element.tuplet_duration = base_duration;
                }
            }
            
            // Set pending tie for next beat if last element can be extended
            if let Some(last_element) = beat.elements.last() {
                if last_element.is_note() && self.extension_chain_active {
                    if let Some((degree, _, _, _)) = last_element.as_note() {
                        self.last_note_across_beats = Some(*degree);
                    }
                }
            }
            
            self.output.push(Item::Beat(beat));
        }
    }

    fn find_next_lower_power_of_2(n: usize) -> usize {
        let mut power = 1;
        while power * 2 < n {
            power *= 2;
        }
        power.max(2)
    }

    fn emit_barline(&mut self, element: &ParsedElement) {
        if let ParsedElement::Barline { style, tala, .. } = element {
            match BarlineType::from_str(style) {
                Ok(barline_type) => self.output.push(Item::Barline(barline_type, *tala)),
                Err(_err) => {} // Skip invalid barlines
            }
        }
    }
}

impl ParsedElement {
    fn position(&self) -> &Position {
        match self {
            ParsedElement::Note { position, .. } => position,
            ParsedElement::Rest { position, .. } => position,
            ParsedElement::Dash { position, .. } => position,
            ParsedElement::Barline { position, .. } => position,
            ParsedElement::Whitespace { position, .. } => position,
            ParsedElement::Symbol { position, .. } => position,
        }
    }
}

/// Convert MusicalElements to FSM output using sophisticated rhythm processing
pub fn process_rhythm(elements: &[MusicalElement]) -> Vec<Item> {
    // First convert current elements to old ParsedElement format
    let parsed_elements = convert_musical_elements_to_parsed_elements(elements);
    
    // Then process with sophisticated FSM
    let mut fsm = FSM::new();
    fsm.process(&parsed_elements);
    fsm.output
}

/// Convert current MusicalElement to old ParsedElement format
fn convert_musical_elements_to_parsed_elements(elements: &[MusicalElement]) -> Vec<ParsedElement> {
    let mut result = Vec::new();
    
    for element in elements {
        match element {
            MusicalElement::Note(note) => {
                let degree = convert_pitchcode_to_degree(note.pitch_code);
                
                // Convert syllable to ParsedChild only for tabla notation
                let children = if note.notation_system == crate::document::model::NotationSystem::Tabla && !note.syllable.is_empty() {
                    vec![ParsedChild::Syllable { 
                        text: note.syllable.clone(),
                        distance: 1 // Below the note 
                    }]
                } else {
                    vec![]
                };
                
                result.push(ParsedElement::Note {
                    degree,
                    octave: note.octave,
                    value: note.source.value.clone(), // Use original source value
                    position: Position {
                        row: note.source.position.line,
                        col: note.source.position.column,
                    },
                    children,
                    duration: None,
                    slur: if note.in_slur { Some(SlurRole::Middle) } else { None }, // Basic slur handling
                });
            },
            MusicalElement::Dash { source, .. } => {
                result.push(ParsedElement::Dash {
                    degree: None, // Will be inherited by FSM
                    octave: None,
                    position: Position {
                        row: source.position.line,
                        col: source.position.column,
                    },
                    duration: None,
                });
            },
            MusicalElement::Space { source, .. } => {
                result.push(ParsedElement::Whitespace {
                    value: " ".to_string(),
                    position: Position {
                        row: source.position.line,
                        col: source.position.column,
                    },
                });
            },
            MusicalElement::Barline { source, .. } => {
                result.push(ParsedElement::Barline {
                    style: "|".to_string(),
                    position: Position {
                        row: source.position.line,
                        col: source.position.column,
                    },
                    tala: None,
                });
            },
            _ => {
                // Skip other element types for now
            }
        }
    }
    
    result
}

/// Convert current PitchCode to old Degree format
fn convert_pitchcode_to_degree(pitch_code: crate::document::model::PitchCode) -> Degree {
    use crate::document::model::PitchCode;
    match pitch_code {
        PitchCode::N1bb => Degree::N1bb,
        PitchCode::N1b => Degree::N1b,
        PitchCode::N1 => Degree::N1,
        PitchCode::N1s => Degree::N1s,
        PitchCode::N1ss => Degree::N1ss,
        PitchCode::N2bb => Degree::N2bb,
        PitchCode::N2b => Degree::N2b,
        PitchCode::N2 => Degree::N2,
        PitchCode::N2s => Degree::N2s,
        PitchCode::N2ss => Degree::N2ss,
        PitchCode::N3bb => Degree::N3bb,
        PitchCode::N3b => Degree::N3b,
        PitchCode::N3 => Degree::N3,
        PitchCode::N3s => Degree::N3s,
        PitchCode::N3ss => Degree::N3ss,
        PitchCode::N4bb => Degree::N4bb,
        PitchCode::N4b => Degree::N4b,
        PitchCode::N4 => Degree::N4,
        PitchCode::N4s => Degree::N4s,
        PitchCode::N4ss => Degree::N4ss,
        PitchCode::N5bb => Degree::N5bb,
        PitchCode::N5b => Degree::N5b,
        PitchCode::N5 => Degree::N5,
        PitchCode::N5s => Degree::N5s,
        PitchCode::N5ss => Degree::N5ss,
        PitchCode::N6bb => Degree::N6bb,
        PitchCode::N6b => Degree::N6b,
        PitchCode::N6 => Degree::N6,
        PitchCode::N6s => Degree::N6s,
        PitchCode::N6ss => Degree::N6ss,
        PitchCode::N7bb => Degree::N7bb,
        PitchCode::N7b => Degree::N7b,
        PitchCode::N7 => Degree::N7,
        PitchCode::N7s => Degree::N7s,
        PitchCode::N7ss => Degree::N7ss,
    }
}