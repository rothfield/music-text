// Rhythm FSM - Hierarchical state design based on doremi-script
use fraction::Fraction;
use crate::models::{ParsedElement, Degree, Position};
use crate::parser_v2_fsm::{Beat as FsmBeat, BeatElement as FsmBeatElement, Event, Item};

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
    extension_chain_active: bool,        // Can dash extend across beats?
    last_note_across_beats: Option<Degree>, // For cross-beat ties
    current_beat: Option<FsmBeat>,       // Current beat being built
    output: Vec<Item>,                   // Final output items
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
                    // This shouldn't happen - we transition directly from S0 to CollectingPitch/CollectingRests
                    // But handle it as a safety fallback
                    if self.current_beat.as_ref().map_or(true, |b| b.elements.is_empty()) {
                        self.add_rest_to_beat(element);
                        self.state = State::CollectingRests;
                    } else {
                        self.extend_last_element();
                        self.state = State::CollectingPitch;
                    }
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
                (State::InBeat, ParsedElement::Symbol { value, .. }) if value == "'" => {
                    self.handle_breathmark();
                    // Stay in InBeat - breathmark doesn't end beat
                },

                // CollectingPitch State - Extending current note
                (State::CollectingPitch, ParsedElement::Dash { .. }) => {
                    self.extend_last_element();
                    // Stay in CollectingPitch
                },
                (State::CollectingPitch, ParsedElement::Note { .. }) => {
                    self.add_note_to_beat(element);
                    // Stay in CollectingPitch - now collecting this new note
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
                (State::CollectingPitch, ParsedElement::Symbol { value, .. }) if value == "'" => {
                    self.handle_breathmark();
                    self.state = State::InBeat; // Can't collect more pitches after breath
                },

                // CollectingRests State - Processing rest extensions
                (State::CollectingRests, ParsedElement::Dash { .. }) => {
                    self.extend_last_element();
                    // Stay in CollectingRests
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
                (State::CollectingRests, ParsedElement::Symbol { value, .. }) if value == "'" => {
                    self.handle_breathmark();
                    self.state = State::InBeat;
                },

                // Halt State - No more transitions
                (State::Halt, _) => break,

                // Unhandled combinations - log and ignore
                _ => {
                    eprintln!("Unhandled state transition: {:?} with {:?}", self.state, element);
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
        
        let mut beat = FsmBeat {
            divisions: 1,
            elements: vec![],
            tied_to_previous,
            is_tuplet: false,
            tuplet_ratio: None,
        };
        
        beat.elements.push(FsmBeatElement::from(element.clone()).with_subdivisions(1));
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

            let mut beat = FsmBeat {
                divisions: 1,
                elements: vec![],
                tied_to_previous: true,
                is_tuplet: false,
                tuplet_ratio: None,
            };
            
            beat.elements.push(FsmBeatElement::from(tied_note).with_subdivisions(1));
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

        let mut beat = FsmBeat {
            divisions: 1,
            elements: vec![],
            tied_to_previous: false,
            is_tuplet: false,
            tuplet_ratio: None,
        };
        
        beat.elements.push(FsmBeatElement::from(rest_element).with_subdivisions(1));
        self.current_beat = Some(beat);
        self.state = State::CollectingRests;
    }

    fn add_note_to_beat(&mut self, element: &ParsedElement) {
        if let Some(beat) = &mut self.current_beat {
            beat.divisions += 1;
            beat.elements.push(FsmBeatElement::from(element.clone()).with_subdivisions(1));
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
            beat.elements.push(FsmBeatElement::from(rest_element).with_subdivisions(1));
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
            // A single element always fills the beat, regardless of subdivisions
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
            match crate::models::BarlineType::from_str(style) {
                Ok(barline_type) => self.output.push(Item::Barline(barline_type, *tala)),
                Err(err) => eprintln!("Warning: {}", err),
            }
        }
    }
}

/// Convert ParsedElements to FSM output using hierarchical state design
pub fn convert_parsed_to_fsm_output(elements: &[ParsedElement]) -> Vec<Item> {
    let mut fsm = FSM::new();
    fsm.process(elements);
    fsm.output
}