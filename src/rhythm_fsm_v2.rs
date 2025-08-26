// Rhythm FSM V2 - Works with ParsedElement instead of Node
use crate::models_v2::{ParsedElement, Position};
use crate::pitch::PitchCode;

fn previous_power_of_two(n: usize) -> usize {
    if n <= 1 {
        return 1;
    }
    let mut power = 1;
    while power * 2 < n {
        power *= 2;
    }
    power
}

fn gcd(a: usize, b: usize) -> usize {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

fn reduce_fraction(numerator: usize, denominator: usize) -> (usize, usize) {
    let g = gcd(numerator, denominator);
    (numerator / g, denominator / g)
}

#[derive(Debug, Clone)]
struct ElementV2 {
    element: ParsedElement,
    subdivisions: usize,
    duration: Option<String>,
}

#[derive(Debug)]
struct BeatV2 {
    divisions: usize,
    elements: Vec<ElementV2>,
    tied_to_previous: bool,
}

#[derive(Debug)]
enum OutputItemV2 {
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
        let mut beat = BeatV2 { divisions: 1, elements: vec![], tied_to_previous: false };
        beat.elements.push(ElementV2 { element: element.clone(), subdivisions: 1, duration: None });
        self.current_beat = Some(beat);
        self.state = State::InBeat;
    }

    fn start_beat_dash(&mut self, dash_element: &ParsedElement) {
        let last_element = self.find_last_non_dash_element();
        if let Some(prev_element) = last_element {
            // Found previous pitch - create a tied note
            if let ParsedElement::Note { pitch_code, octave, position, .. } = prev_element {
                let tied_note = ParsedElement::Note {
                    pitch_code: *pitch_code,
                    octave: *octave,
                    value: format!("{:?}", pitch_code), // Convert pitch_code to debug representation
                    position: dash_element.position().clone(),
                    children: vec![], // No children for tied notes
                };

                let mut beat = BeatV2 { 
                    divisions: 1, 
                    elements: vec![], 
                    tied_to_previous: true
                };
                beat.elements.push(ElementV2 { element: tied_note, subdivisions: 1, duration: None });
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
        };
        let mut beat = BeatV2 { divisions: 1, elements: vec![], tied_to_previous: false };
        beat.elements.push(ElementV2 { element: rest_element, subdivisions: 1, duration: None });
        self.current_beat = Some(beat);
        self.state = State::InBeat;
    }

    fn extend_last_element(&mut self) {
        if let Some(beat) = &mut self.current_beat {
            beat.divisions += 1;
            if let Some(last) = beat.elements.last_mut() {
                last.subdivisions += 1;
            }
        }
    }

    fn add_pitch_to_beat(&mut self, element: &ParsedElement) {
        if let Some(beat) = &mut self.current_beat {
            beat.divisions += 1;
            beat.elements.push(ElementV2 { element: element.clone(), subdivisions: 1, duration: None });
        }
    }

    fn find_last_non_dash_element(&self) -> Option<&ParsedElement> {
        // Look through the output to find the last non-dash element
        for output_item in self.output.iter().rev() {
            if let OutputItemV2::Beat(beat) = output_item {
                for element_wrapper in beat.elements.iter().rev() {
                    if !self.is_dash(&element_wrapper.element) {
                        return Some(&element_wrapper.element);
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
        if let Some(beat) = self.current_beat.take() {
            self.output.push(OutputItemV2::Beat(beat));
        }
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
fn convert_fsm_output_to_elements(output: Vec<OutputItemV2>) -> Vec<ParsedElement> {
    let mut result = Vec::new();
    
    for item in output {
        match item {
            OutputItemV2::Beat(beat) => {
                // Create a beat container or just add elements directly
                for element_wrapper in beat.elements {
                    result.push(element_wrapper.element);
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