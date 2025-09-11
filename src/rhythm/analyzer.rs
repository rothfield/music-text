/// Sophisticated Rhythm FSM - Copied from old working system
/// Converts flat element lists into beat-grouped structures with proper fraction-based durations

// ContentElement import removed - no longer needed with direct ParsedElement architecture
use crate::rhythm::types::*;
use crate::rhythm::converters::BarlineType;
use fraction::Fraction;

// FSM output structures for rhythm processing
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
    CollectingPitch,    // Extending current note with dashes
    CollectingRests,    // Processing leading dashes as rests
    Halt,               // End of processing
}

struct FSM {
    state: State,
    extension_chain_active: bool,
    last_note_across_beats: Option<Degree>,
    pending_tie: bool,  // Track when a dash has created a tie intention
    current_beat: Option<Beat>,
    output: Vec<Item>,
}

impl FSM {
    fn new() -> Self {
        Self {
            state: State::S0,
            extension_chain_active: false,
            last_note_across_beats: None,
            pending_tie: false,
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
                        self.pending_tie = true;  // Mark that we have a tie intention
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
        if matches!(self.state, State::S0 | State::CollectingPitch | State::CollectingRests) {
            self.finish_beat();
        }
        self.state = State::Halt;
    }

    fn start_beat_with_note(&mut self, element: &ParsedElement) {
        // Finish any existing beat first
        if self.current_beat.is_some() {
            self.finish_beat();
        }
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
            let tied_note = ParsedElement::new_note(
                degree,
                0, // Default octave - could be improved
                format!("{:?}", degree),
                dash_element.position().clone(),
            );

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
            if self.pending_tie {
                if let Some(pending_pitch) = self.last_note_across_beats {
                    if *degree == pending_pitch {
                        self.pending_tie = false;  // Clear tie intention after using
                        self.last_note_across_beats = None; // Clear after using
                        return true;
                    }
                }
            }
        }
        // Clear any unused tie intention
        self.pending_tie = false;
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
        self.pending_tie = false;  // Clear any pending ties
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


/// Convert ParsedElements to FSM output using hierarchical state design
pub fn process_rhythm(elements: &[ParsedElement]) -> Vec<Item> {
    let mut fsm = FSM::new();
    fsm.process(elements);
    fsm.output
}

/// Batch rhythm processing - processes all staves together for better context
/// This allows the FSM to track extension chains and ties across stave boundaries
pub fn process_rhythm_batch(all_stave_content_lines: &[&Vec<ParsedElement>]) -> Vec<Vec<Item>> {
    let mut all_results = Vec::new();
    
    for content_line in all_stave_content_lines {
        // For now, process each stave individually (same as before)
        // Future enhancement: could track extension chains across staves
        let rhythm_items = process_rhythm(content_line);
        all_results.push(rhythm_items);
    }
    
    all_results
}

// ContentElement conversion functions removed - no longer needed with direct ParsedElement architecture