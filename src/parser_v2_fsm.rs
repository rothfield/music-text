// Rhythm FSM V2 - Works with ParsedElement instead of Node
use crate::models::{ParsedElement, ParsedChild, OrnamentType, Position, SlurRole};
use crate::models::Degree;
use fraction::Fraction;

#[derive(Debug, Clone, serde::Serialize)]
pub enum Event {
    Note {
        degree: Degree,
        octave: i8,
        children: Vec<ParsedChild>,  // syllables, ornaments, octave markers
        slur: Option<SlurRole>,
    },
    Rest,  // Rests have no additional data
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BeatElement {
    pub event: Event,
    pub subdivisions: usize,
    pub duration: Fraction,               // Actual beat fraction: subdivisions/divisions  
    pub tuplet_duration: Fraction,        // Mathematical tuplet duration (1/6, 1/3, etc.)
    pub tuplet_display_duration: Option<Fraction>, // Display duration for tuplets (1/16, 1/8, etc.), None for regular notes
    pub value: String,                    // Original text value
    pub position: Position,               // Source position
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
        
        Self {
            event,
            subdivisions: 1, // Default, will be set by FSM
            duration: Fraction::new(0u64, 1u64), // Default, will be calculated in finish_beat
            tuplet_duration: Fraction::new(0u64, 1u64), // Default, will be calculated in finish_beat
            tuplet_display_duration: None, // None for regular notes, Some() for tuplets
            value,
            position,
        }
    }
}

impl BeatElement {
    pub fn with_subdivisions(mut self, subdivisions: usize) -> Self {
        self.subdivisions = subdivisions;
        self
    }
    
    pub fn extend_subdivision(&mut self) {
        self.subdivisions += 1;
    }
    
    // Helper methods for element type checking
    pub fn is_note(&self) -> bool { 
        matches!(self.event, Event::Note { .. })
    }
    
    pub fn is_rest(&self) -> bool { 
        matches!(self.event, Event::Rest)
    }
    
    // Get note data if this is a note
    pub fn as_note(&self) -> Option<(&Degree, i8, &Vec<ParsedChild>, &Option<SlurRole>)> {
        match &self.event {
            Event::Note { degree, octave, children, slur } => Some((degree, *octave, children, slur)),
            Event::Rest => None,
        }
    }
    
    // Extract convenience fields from children (for compatibility)
    pub fn syl(&self) -> Option<String> {
        if let Event::Note { children, .. } = &self.event {
            // Return the LAST syllable (which is the corrected/split one if lyrics were processed)
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
            children.iter().filter_map(|child| match child {
                ParsedChild::Ornament { kind, .. } => Some(kind.clone()),
                _ => None
            }).collect()
        } else {
            vec![]
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Beat {
    pub divisions: usize,
    pub elements: Vec<BeatElement>,           // RENAMED: ElementV2 → BeatElement
    pub tied_to_previous: bool,
    pub is_tuplet: bool,                      // NEW: Fast boolean check  
    pub tuplet_ratio: Option<(usize, usize)>, // NEW: (divisions, power_of_2) for tuplets
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum Item {
    Beat(Beat),
    Barline(crate::models::BarlineType, Option<u8>), // BarlineType and optional tala (0-6)
    Breathmark,
    Tonic(Degree), // Tonic/Key declaration (e.g., "key: D" -> Degree::N2)
}

#[derive(Debug, PartialEq)]
enum State {
    S0,
    InBeat,
    Halt,
}

struct FSMV2 {
    state: State,
    output: Vec<Item>,
    current_beat: Option<Beat>,
    inside_beat_bracket: bool,
    pending_tie_pitch: Option<Degree>, // Track pitch that needs tying across barlines
    last_was_dash: bool, // Track if last processed element was a dash/extender
}

impl FSMV2 {
    fn new() -> Self {
        Self {
            state: State::S0,
            output: vec![],
            current_beat: None,
            inside_beat_bracket: false,
            pending_tie_pitch: None,
            last_was_dash: false,
        }
    }

    // /// Pre-process elements to detect ~note~ patterns and convert them to notes with mordent ornaments
    // fn preprocess_mordents(&self, elements: Vec<&ParsedElement>) -> Vec<ParsedElement> {
    //     eprintln!("🎵 MORDENT DEBUG: preprocess_mordents called with {} elements", elements.len());
    //     for (idx, elem) in elements.iter().enumerate() {
    //         eprintln!("🎵 MORDENT DEBUG: Element {}: {:?}", idx, elem);
    //     }
    //     
    //     let mut result = Vec::new();
    //     let mut i = 0;
    //     
    //     while i < elements.len() {
    //         // Check for ~note~ pattern
    //         if i + 2 < elements.len() {
    //             if let (
    //                 ParsedElement::Symbol { value: tilde1, .. },
    //                 note @ ParsedElement::Note { .. },
    //                 ParsedElement::Symbol { value: tilde2, .. }
    //             ) = (&elements[i], &elements[i + 1], &elements[i + 2]) {
    //                 if tilde1 == "~" && tilde2 == "~" {
    //                     eprintln!("🎵 MORDENT DEBUG: Found ~note~ pattern at indices {}, {}, {}", i, i+1, i+2);
    //                     // Create a new note with a mordent ornament
    //                     if let ParsedElement::Note { degree, octave, value, position, children, duration, slur } = note {
    //                         let mut new_children = children.clone();
    //                         new_children.push(crate::parsed_models::ParsedChild::Ornament {
    //                             kind: crate::parsed_models::OrnamentType::Mordent,
    //                             distance: 0,
    //                         });
    //                         
    //                         let mordent_note = ParsedElement::Note {
    //                             degree: *degree,
    //                             octave: *octave,
    //                             value: value.clone(),
    //                             position: position.clone(),
    //                             children: new_children,
    //                             duration: *duration,
    //                             slur: slur.clone(),
    //                         };
    //                         eprintln!("🎵 MORDENT DEBUG: Created mordent note: {:?}", mordent_note);
    //                         result.push(mordent_note);
    //                         
    //                         i += 3; // Skip the ~ note ~ pattern
    //                         continue;
    //                     }
    //                 }
    //             }
    //         }
    //         
    //         // No pattern matched, copy element as-is
    //         result.push(elements[i].clone());
    //         i += 1;
    //     }
    //     
    //     result
    // }

    fn process(&mut self, elements: Vec<&ParsedElement>) {
        // Note: Ornament processing now handled by vertical_parser.rs region processor
        // No need for preprocessing here as ornaments are attached to notes before FSM
        let processed_elements: Vec<ParsedElement> = elements.iter().map(|e| (*e).clone()).collect();
        let mut iter = processed_elements.iter().peekable();
        while let Some(element) = iter.next() {
            match self.state {
                State::S0 => {
                    if self.is_barline(element) {
                        self.emit_barline(element);
                    } else if self.is_beat_separator(element) {
                        // beat_separator, no-op
                    } else if self.is_breathmark(element) {
                        self.emit_breathmark();
                        self.last_was_dash = false;
                    } else if self.is_dash(element) {
                        self.start_beat_dash(element);
                        self.last_was_dash = true;
                    } else if self.is_pitch(element) {
                        self.start_beat_pitch(element);
                        self.last_was_dash = false;
                        self.update_beat_bracket_state(element);
                    }
                    // Unknown tokens stay in same state (S0)
                },
                State::InBeat => {
                    if self.is_barline(element) || self.is_beat_separator(element) {
                        self.finish_beat();
                        if self.is_barline(element) {
                            self.emit_barline(element);
                        }
                        self.state = State::S0;
                    } else if self.is_breathmark(element) {
                        self.finish_beat();
                        self.emit_breathmark();
                        self.last_was_dash = false;
                        self.state = State::S0;
                    } else if self.is_dash(element) {
                        self.extend_last_element();
                        self.last_was_dash = true;
                    } else if self.is_pitch(element) {
                        self.add_pitch_to_beat(element);
                        self.last_was_dash = false;
                        self.update_beat_bracket_state(element);
                    }
                    // Unknown tokens stay in same state (InBeat)
                },
                State::Halt => break,
            }
        }

        if self.state == State::InBeat {
            self.finish_beat();
        }
        self.state = State::Halt;
    }

    fn start_beat_pitch(&mut self, element: &ParsedElement) {
        // Check if this note should be tied to previous beat
        let tied_to_previous = if let Some(pending_pitch) = self.pending_tie_pitch {
            // Check if this note matches the pending tie pitch
            match element {
                ParsedElement::Note { degree, .. } => *degree == pending_pitch,
                _ => false,
            }
        } else {
            false
        };
        
        let mut beat = Beat { 
            divisions: 1, 
            elements: vec![], 
            tied_to_previous, 
            is_tuplet: false, 
            tuplet_ratio: None 
        };
        beat.elements.push(BeatElement::from(element.clone()).with_subdivisions(1));
        self.current_beat = Some(beat);
        self.state = State::InBeat;
        
        // Clear pending tie after processing
        if tied_to_previous {
            self.pending_tie_pitch = None;
        }
    }

    fn start_beat_dash(&mut self, dash_element: &ParsedElement) {
        let last_element = self.find_last_non_dash_element();
        if let Some(prev_beat_element) = last_element {
            // Found previous element - check if it's a note
            if let Some((degree, octave, _, _)) = prev_beat_element.as_note() {
                // Create a tied note with same pitch
                let tied_note = ParsedElement::Note {
                    degree: *degree,
                    octave: octave,
                    value: format!("{:?}", degree), // Convert degree to debug representation
                    position: dash_element.position().clone(),
                    children: vec![], // No children for tied notes
                    duration: None,
                    slur: None, // Tied notes don't inherit slur
                };

                let mut beat = Beat { 
                    divisions: 1, 
                    elements: vec![], 
                    tied_to_previous: true,
                    is_tuplet: false,
                    tuplet_ratio: None
                };
                beat.elements.push(BeatElement::from(tied_note).with_subdivisions(1));
                self.current_beat = Some(beat);
                self.state = State::InBeat;
            } else {
                // Previous wasn't a note - create rest
                self.create_rest_beat(dash_element);
            }
        } else {
            // No previous element - create rest
            self.create_rest_beat(dash_element);
        }
    }

    fn create_rest_beat(&mut self, dash_element: &ParsedElement) {
        let rest_element = ParsedElement::Rest {
            value: "r".to_string(),
            position: dash_element.position().clone(),
            duration: None,
        };
        let mut beat = Beat { divisions: 1, elements: vec![], tied_to_previous: false, is_tuplet: false, tuplet_ratio: None };
        beat.elements.push(BeatElement::from(rest_element).with_subdivisions(1));
        self.current_beat = Some(beat);
        self.state = State::InBeat;
    }

    fn extend_last_element(&mut self) {
        if let Some(beat) = &mut self.current_beat {
            beat.divisions += 1;
            if let Some(last) = beat.elements.last_mut() {
                last.extend_subdivision();
            }
        }
    }

    fn add_pitch_to_beat(&mut self, element: &ParsedElement) {
        if let Some(beat) = &mut self.current_beat {
            beat.divisions += 1;
            beat.elements.push(BeatElement::from(element.clone()).with_subdivisions(1));
        }
    }

    fn find_last_non_dash_element(&self) -> Option<&BeatElement> {
        // Look through the output to find the last element
        // (We no longer have "dash" elements - dashes create notes or rests)
        for output_item in self.output.iter().rev() {
            if let Item::Beat(beat) = output_item {
                if let Some(last) = beat.elements.last() {
                    return Some(last);
                }
            }
        }
        None
    }

    // Helper methods to identify element types
    fn is_barline(&self, element: &ParsedElement) -> bool {
        matches!(element, ParsedElement::Barline { .. })
    }

    fn is_beat_separator(&self, element: &ParsedElement) -> bool {
        let is_whitespace = matches!(element, 
            ParsedElement::Whitespace { .. } | 
            ParsedElement::Newline { .. }
        );
        
        if is_whitespace && self.inside_beat_bracket {
            false  // Don't treat as beat separator when inside beat bracket
        } else {
            is_whitespace
        }
    }

    fn is_breathmark(&self, element: &ParsedElement) -> bool {
        matches!(element, ParsedElement::Symbol { value, .. } if value == "'")
    }

    fn is_dash(&self, element: &ParsedElement) -> bool {
        matches!(element, ParsedElement::Dash { .. })
    }

    fn is_pitch(&self, element: &ParsedElement) -> bool {
        matches!(element, ParsedElement::Note { .. })
    }

    // fn is_slur_start(&self, element: &ParsedElement) -> bool {
    //     matches!(element, ParsedElement::SlurStart { .. })
    // }

    // fn is_slur_end(&self, element: &ParsedElement) -> bool {
    //     matches!(element, ParsedElement::SlurEnd { .. })
    // }

    fn update_beat_bracket_state(&mut self, _element: &ParsedElement) {
        // TODO: Beat bracket logic needs to be implemented
        // For now, we don't have beat bracket attributes in ParsedElement
        // This would need to be added to the Note variant or handled differently
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
                
                // Calculate both duration types for tuplets
                let each_unit = Fraction::new(1u64, 4u64) / power_of_2;
                for beat_element in &mut beat.elements {
                    // Actual duration (subdivision fraction of the beat)
                    beat_element.duration = Fraction::new(beat_element.subdivisions as u64, beat.divisions as u64);
                    // Simple rule duration (for notation display)
                    beat_element.tuplet_duration = each_unit * beat_element.subdivisions;
                    // Display duration specifically for tuplet notation
                    beat_element.tuplet_display_duration = Some(each_unit * beat_element.subdivisions);
                }
            } else {
                // Regular beat processing
                for beat_element in &mut beat.elements {
                    let base_duration = Fraction::new(beat_element.subdivisions as u64, beat.divisions as u64) * Fraction::new(1u64, 4u64);
                    eprintln!("FSM: Regular beat element {} subdivisions={} divisions={} base_duration={}", 
                        beat_element.value, beat_element.subdivisions, beat.divisions, base_duration);
                    beat_element.duration = base_duration;
                    beat_element.tuplet_duration = base_duration; // Same for regular beats
                }
            }
            
            // Check if this beat should create a pending tie (last element was dash/extender)
            if let Some(last_element) = beat.elements.last() {
                if last_element.is_note() && self.last_was_dash {
                    // Last element was a dash, so this note should tie to next same pitch
                    if let Some((degree, _, _, _)) = last_element.as_note() {
                        self.pending_tie_pitch = Some(*degree);
                    }
                }
            }
            
            self.output.push(Item::Beat(beat));
            
            // Reset dash tracking when finishing beat
            self.last_was_dash = false;
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
                Err(err) => eprintln!("Warning: {}", err), // Log warning but continue
            }
        }
    }

    fn emit_breathmark(&mut self) {
        // Breath marks break tie chains
        self.pending_tie_pitch = None;
        self.output.push(Item::Breathmark);
    }

}

// Convert FSM output back to ParsedElements
pub fn convert_elements_to_elements_public(output: Vec<Item>) -> Vec<ParsedElement> {
    convert_elements_to_elements(output)
}

fn convert_elements_to_elements(output: Vec<Item>) -> Vec<ParsedElement> {
    let mut result = Vec::new();
    
    for item in output {
        match item {
            Item::Beat(beat) => {
                // Create a beat container or just add elements directly
                for beat_element in beat.elements {
                    // Reconstruct ParsedElement from BeatElement
                    let reconstructed_element = match &beat_element.event {
                        Event::Note { degree, octave, children, slur } => {
                            ParsedElement::Note {
                                degree: *degree,
                                octave: *octave,
                                value: beat_element.value,
                                position: beat_element.position,
                                children: children.clone(),
                                duration: Some((*beat_element.tuplet_duration.numer().unwrap() as usize, *beat_element.tuplet_duration.denom().unwrap() as usize)),
                                slur: slur.clone(),
                            }
                        },
                        Event::Rest => {
                            ParsedElement::Rest {
                                value: beat_element.value,
                                position: beat_element.position,
                                duration: Some((*beat_element.tuplet_duration.numer().unwrap() as usize, *beat_element.tuplet_duration.denom().unwrap() as usize)),
                            }
                        }
                    };
                    result.push(reconstructed_element);
                }
            },
            Item::Barline(value, tala) => {
                result.push(ParsedElement::Barline {
                    style: value.to_str().to_string(),
                    position: Position::new(0, 0), // Position will need to be preserved better
                    tala: tala.clone(),
                });
            },
            Item::Breathmark => {
                result.push(ParsedElement::Symbol {
                    value: "'".to_string(),
                    position: Position::new(0, 0),
                });
            },
            Item::Tonic(_tonic_degree) => {
                // Tonic is for internal transposition, not displayed in parsed elements
                // Could optionally create a Symbol or Comment element if we want to show it
            },
        }
    }
    
    result
}

pub fn group_elements_with_fsm(elements: &[ParsedElement], _lines_of_music: &[usize]) -> Vec<ParsedElement> {
    eprintln!("V2 FSM DEBUG: Input elements count: {}", elements.len());
    for (i, element) in elements.iter().enumerate() {
        eprintln!("V2 FSM DEBUG: Element {}: {:?}", i, element);
    }
    
    let mut fsm = FSMV2::new();
    let element_refs: Vec<&ParsedElement> = elements.iter().collect();
    fsm.process(element_refs);
    
    eprintln!("V2 FSM DEBUG: Output items count: {}", fsm.output.len());
    for (i, item) in fsm.output.iter().enumerate() {
        eprintln!("V2 FSM DEBUG: Output {}: {:?}", i, item);
    }
    
    let result = convert_elements_to_elements(fsm.output);
    eprintln!("V2 FSM DEBUG: Final result count: {}", result.len());
    for (i, element) in result.iter().enumerate() {
        eprintln!("V2 FSM DEBUG: Result {}: {:?}", i, element);
    }
    
    result
}

// New function that returns the full FSM output with beat information
pub fn group_elements_with_fsm_full(elements: &[ParsedElement], _lines_of_music: &[usize]) -> Vec<Item> {
    eprintln!("V2 FSM DEBUG: Input elements count: {}", elements.len());
    for (i, element) in elements.iter().enumerate() {
        eprintln!("V2 FSM DEBUG: Element {}: {:?}", i, element);
    }
    
    let mut fsm = FSMV2::new();
    let element_refs: Vec<&ParsedElement> = elements.iter().collect();
    fsm.process(element_refs);
    
    eprintln!("V2 FSM DEBUG: Output items count: {}", fsm.output.len());
    for (i, item) in fsm.output.iter().enumerate() {
        eprintln!("V2 FSM DEBUG: Output {}: {:?}", i, item);
    }
    
    fsm.output
}