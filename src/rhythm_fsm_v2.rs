// Rhythm FSM V2 - Works with ParsedElement instead of Node
use crate::models_v2::{ParsedElement, ParsedChild, OrnamentType, Position};
use crate::pitch::Degree;
use fraction::Fraction;


#[derive(Debug, Clone, PartialEq)]
pub enum ParsedElementType {
    Note,
    Rest, 
    Dash,
    Barline,
    SlurStart,
    SlurEnd,
    Whitespace,
    Newline,
    Word,
    Symbol,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct BeatElement {
    // Beat-specific fields
    pub subdivisions: usize,
    pub duration: Fraction,        // Actual beat fraction: subdivisions/divisions  
    pub tuplet_duration: Fraction, // Simple rule duration: subdivisions * (1/4 ÷ power_of_2)
    
    // All ParsedElement fields copied directly (bit copy approach)
    pub degree: Option<Degree>, // Note/Dash: Some(code), Others: None
    pub octave: Option<i8>,            // Note/Dash: Some(octave), Others: None  
    pub value: String,                 // Original text value from all elements
    pub position: Position,            // Position from all elements
    pub children: Vec<ParsedChild>,    // Note: actual children, Others: empty vec
    pub element_duration: Option<(usize, usize)>, // Original ParsedElement duration (replaced by Fraction durations)
    
    // Extracted convenience fields
    pub syl: Option<String>,           // Extracted from children
    pub ornaments: Vec<OrnamentType>,  // Extracted from children  
    pub octave_markers: Vec<String>,   // Extracted from children
    
    // Element type (instead of multiple boolean flags)
    pub element_type: ParsedElementType,
}

impl From<ParsedElement> for BeatElement {
    fn from(element: ParsedElement) -> Self {
        let (degree, octave, value, position, children, element_duration, element_type) = match element {
            ParsedElement::Note { degree, octave, value, position, children, duration } => 
                (Some(degree), Some(octave), value, position, children, duration, ParsedElementType::Note),
            ParsedElement::Rest { value, position, duration } => 
                (None, None, value, position, vec![], duration, ParsedElementType::Rest),
            ParsedElement::Dash { degree, octave, position, duration } => 
                (degree, octave, "-".to_string(), position, vec![], duration, ParsedElementType::Dash),
            ParsedElement::Barline { style, position } => 
                (None, None, style, position, vec![], None, ParsedElementType::Barline),
            ParsedElement::SlurStart { position } => 
                (None, None, "(".to_string(), position, vec![], None, ParsedElementType::SlurStart),
            ParsedElement::SlurEnd { position } => 
                (None, None, ")".to_string(), position, vec![], None, ParsedElementType::SlurEnd),
            ParsedElement::Whitespace { width: _, position } => 
                (None, None, " ".to_string(), position, vec![], None, ParsedElementType::Whitespace),
            ParsedElement::Newline { position } => 
                (None, None, "\n".to_string(), position, vec![], None, ParsedElementType::Newline),
            ParsedElement::Word { text, position } => 
                (None, None, text, position, vec![], None, ParsedElementType::Word),
            ParsedElement::Symbol { value, position } => 
                (None, None, value, position, vec![], None, ParsedElementType::Symbol),
            ParsedElement::Unknown { value, position } => 
                (None, None, value, position, vec![], None, ParsedElementType::Unknown),
        };
        
        // Extract convenience fields from children
        let syl = children.iter().find_map(|child| match child {
            ParsedChild::Syllable { text, .. } => Some(text.clone()),
            _ => None
        });
        
        let ornaments = children.iter().filter_map(|child| match child {
            ParsedChild::Ornament { kind, .. } => Some(kind.clone()),
            _ => None
        }).collect();
        
        let octave_markers = children.iter().filter_map(|child| match child {
            ParsedChild::OctaveMarker { symbol, .. } => Some(symbol.clone()),
            _ => None
        }).collect();
        
        Self {
            subdivisions: 1, // Default, will be set by FSM
            duration: Fraction::new(0u64, 1u64), // Default, will be calculated in finish_beat
            tuplet_duration: Fraction::new(0u64, 1u64), // Default, will be calculated in finish_beat
            degree,
            octave,
            value,
            position,
            children,
            element_duration,
            syl,
            ornaments,
            octave_markers,
            element_type,
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
    pub fn is_note(&self) -> bool { self.element_type == ParsedElementType::Note }
    pub fn is_rest(&self) -> bool { self.element_type == ParsedElementType::Rest }
    pub fn is_dash(&self) -> bool { self.element_type == ParsedElementType::Dash }
    pub fn is_barline(&self) -> bool { self.element_type == ParsedElementType::Barline }
    pub fn is_slur_start(&self) -> bool { self.element_type == ParsedElementType::SlurStart }
    pub fn is_slur_end(&self) -> bool { self.element_type == ParsedElementType::SlurEnd }
}

#[derive(Debug, Clone)]
pub struct BeatV2 {
    pub divisions: usize,
    pub elements: Vec<BeatElement>,           // RENAMED: ElementV2 → BeatElement
    pub tied_to_previous: bool,
    pub is_tuplet: bool,                      // NEW: Fast boolean check  
    pub tuplet_ratio: Option<(usize, usize)>, // NEW: (divisions, power_of_2) for tuplets
}

#[derive(Debug, Clone)]
pub enum OutputItemV2 {
    Beat(BeatV2),
    Barline(String),
    Breathmark,
    SlurStart,
    SlurEnd,
}

#[derive(Debug, PartialEq)]
enum State {
    S0,
    InBeat,
    Halt,
}

struct FSMV2 {
    state: State,
    output: Vec<OutputItemV2>,
    current_beat: Option<BeatV2>,
    inside_beat_bracket: bool,
}

impl FSMV2 {
    fn new() -> Self {
        Self {
            state: State::S0,
            output: vec![],
            current_beat: None,
            inside_beat_bracket: false,
        }
    }

    fn process(&mut self, elements: Vec<&ParsedElement>) {
        let mut iter = elements.into_iter().peekable();
        while let Some(element) = iter.next() {
            match self.state {
                State::S0 => {
                    if self.is_barline(element) {
                        self.emit_barline(element.value());
                    } else if self.is_beat_separator(element) {
                        // beat_separator, no-op
                    } else if self.is_breathmark(element) {
                        self.emit_breathmark();
                    } else if self.is_slur_start(element) {
                        self.emit_slur_start();
                    } else if self.is_slur_end(element) {
                        self.emit_slur_end();
                    } else if self.is_dash(element) {
                        self.start_beat_dash(element);
                    } else if self.is_pitch(element) {
                        self.start_beat_pitch(element);
                        self.update_beat_bracket_state(element);
                    }
                    // Unknown tokens stay in same state (S0)
                },
                State::InBeat => {
                    if self.is_barline(element) || self.is_beat_separator(element) {
                        self.finish_beat();
                        if self.is_barline(element) {
                            self.emit_barline(element.value());
                        }
                        self.state = State::S0;
                    } else if self.is_breathmark(element) {
                        self.finish_beat();
                        self.emit_breathmark();
                        self.state = State::S0;
                    } else if self.is_slur_start(element) {
                        self.emit_slur_start();
                    } else if self.is_slur_end(element) {
                        self.emit_slur_end();
                    } else if self.is_dash(element) {
                        self.extend_last_element();
                    } else if self.is_pitch(element) {
                        self.add_pitch_to_beat(element);
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
        let mut beat = BeatV2 { divisions: 1, elements: vec![], tied_to_previous: false, is_tuplet: false, tuplet_ratio: None };
        beat.elements.push(BeatElement::from(element.clone()).with_subdivisions(1));
        self.current_beat = Some(beat);
        self.state = State::InBeat;
    }

    fn start_beat_dash(&mut self, dash_element: &ParsedElement) {
        let last_element = self.find_last_non_dash_element();
        if let Some(prev_beat_element) = last_element {
            // Found previous pitch - create a tied note
            if prev_beat_element.is_note() && prev_beat_element.degree.is_some() {
                let tied_note = ParsedElement::Note {
                    degree: prev_beat_element.degree.unwrap(),
                    octave: prev_beat_element.octave.unwrap(),
                    value: format!("{:?}", prev_beat_element.degree.unwrap()), // Convert degree to debug representation
                    position: dash_element.position().clone(),
                    children: vec![], // No children for tied notes
                    duration: None,
                };

                let mut beat = BeatV2 { 
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
        let mut beat = BeatV2 { divisions: 1, elements: vec![], tied_to_previous: false, is_tuplet: false, tuplet_ratio: None };
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
        // Look through the output to find the last non-dash element
        for output_item in self.output.iter().rev() {
            if let OutputItemV2::Beat(beat) = output_item {
                for beat_element in beat.elements.iter().rev() {
                    if !beat_element.is_dash() {
                        return Some(beat_element);
                    }
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

    fn is_slur_start(&self, element: &ParsedElement) -> bool {
        matches!(element, ParsedElement::SlurStart { .. })
    }

    fn is_slur_end(&self, element: &ParsedElement) -> bool {
        matches!(element, ParsedElement::SlurEnd { .. })
    }

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
                }
            } else {
                // Regular beat processing
                for beat_element in &mut beat.elements {
                    let base_duration = Fraction::new(beat_element.subdivisions as u64, beat.divisions as u64) * Fraction::new(1u64, 4u64);
                    beat_element.duration = base_duration;
                    beat_element.tuplet_duration = base_duration; // Same for regular beats
                }
            }
            
            self.output.push(OutputItemV2::Beat(beat));
        }
    }
    
    fn find_next_lower_power_of_2(n: usize) -> usize {
        let mut power = 1;
        while power * 2 < n {
            power *= 2;
        }
        power.max(2)
    }

    fn emit_barline(&mut self, value: String) {
        self.output.push(OutputItemV2::Barline(value));
    }

    fn emit_breathmark(&mut self) {
        self.output.push(OutputItemV2::Breathmark);
    }

    fn emit_slur_start(&mut self) {
        self.output.push(OutputItemV2::SlurStart);
    }

    fn emit_slur_end(&mut self) {
        self.output.push(OutputItemV2::SlurEnd);
    }
}

// Convert FSM output back to ParsedElements
pub fn convert_fsm_output_to_elements_public(output: Vec<OutputItemV2>) -> Vec<ParsedElement> {
    convert_fsm_output_to_elements(output)
}

fn convert_fsm_output_to_elements(output: Vec<OutputItemV2>) -> Vec<ParsedElement> {
    let mut result = Vec::new();
    
    for item in output {
        match item {
            OutputItemV2::Beat(beat) => {
                // Create a beat container or just add elements directly
                for beat_element in beat.elements {
                    // Reconstruct ParsedElement from BeatElement
                    let reconstructed_element = match beat_element.element_type {
                        ParsedElementType::Note => ParsedElement::Note {
                            degree: beat_element.degree.unwrap(),
                            octave: beat_element.octave.unwrap(),
                            value: beat_element.value,
                            position: beat_element.position,
                            children: beat_element.children,
                            duration: Some((*beat_element.tuplet_duration.numer().unwrap() as usize, *beat_element.tuplet_duration.denom().unwrap() as usize)),
                        },
                        ParsedElementType::Rest => ParsedElement::Rest {
                            value: beat_element.value,
                            position: beat_element.position,
                            duration: Some((*beat_element.tuplet_duration.numer().unwrap() as usize, *beat_element.tuplet_duration.denom().unwrap() as usize)),
                        },
                        ParsedElementType::Dash => ParsedElement::Dash {
                            degree: beat_element.degree,
                            octave: beat_element.octave,
                            position: beat_element.position,
                            duration: Some((*beat_element.tuplet_duration.numer().unwrap() as usize, *beat_element.tuplet_duration.denom().unwrap() as usize)),
                        },
                        _ => ParsedElement::Symbol {
                            value: beat_element.value,
                            position: beat_element.position,
                        }
                    };
                    result.push(reconstructed_element);
                }
            },
            OutputItemV2::Barline(value) => {
                result.push(ParsedElement::Barline {
                    style: value,
                    position: Position::new(0, 0), // Position will need to be preserved better
                });
            },
            OutputItemV2::Breathmark => {
                result.push(ParsedElement::Symbol {
                    value: "'".to_string(),
                    position: Position::new(0, 0),
                });
            },
            OutputItemV2::SlurStart => {
                result.push(ParsedElement::SlurStart {
                    position: Position::new(0, 0),
                });
            },
            OutputItemV2::SlurEnd => {
                result.push(ParsedElement::SlurEnd {
                    position: Position::new(0, 0),
                });
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
    
    let result = convert_fsm_output_to_elements(fsm.output);
    eprintln!("V2 FSM DEBUG: Final result count: {}", result.len());
    for (i, element) in result.iter().enumerate() {
        eprintln!("V2 FSM DEBUG: Result {}: {:?}", i, element);
    }
    
    result
}

// New function that returns the full FSM output with beat information
pub fn group_elements_with_fsm_full(elements: &[ParsedElement], _lines_of_music: &[usize]) -> Vec<OutputItemV2> {
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