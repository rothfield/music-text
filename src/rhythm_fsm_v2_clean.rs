// Clean room V2 FSM - reimplemented based on V1 behavior
// Goal: Process ParsedElements and calculate duration fractions for rhythm

use crate::models_v2::ParsedElement;

/// Process ParsedElements through FSM and calculate durations
pub fn process_rhythm_v2_clean(elements: Vec<ParsedElement>) -> Vec<ParsedElement> {
    let processor = RhythmProcessorClean::new();
    processor.process(elements)
}

struct RhythmProcessorClean {
    output: Vec<ParsedElement>,
    current_beat: Option<BeatBuilder>,
}

#[derive(Debug)]
struct BeatBuilder {
    elements: Vec<ElementWithSubdivisions>,
    total_divisions: usize,
}

#[derive(Debug)]
struct ElementWithSubdivisions {
    element: ParsedElement,
    subdivisions: usize,
}

impl RhythmProcessorClean {
    fn new() -> Self {
        Self {
            output: Vec::new(),
            current_beat: None,
        }
    }

    fn process(mut self, elements: Vec<ParsedElement>) -> Vec<ParsedElement> {
        for element in elements {
            match &element {
                ParsedElement::Note { .. } => {
                    if self.current_beat.is_some() {
                        self.add_note_to_beat(element);
                    } else {
                        self.start_new_beat(element);
                    }
                }
                ParsedElement::Rest { .. } => {
                    if self.current_beat.is_some() {
                        self.add_note_to_beat(element);
                    } else {
                        self.start_new_beat(element);
                    }
                }
                ParsedElement::Dash { .. } => {
                    if self.current_beat.is_some() {
                        self.extend_current_beat();
                    } else {
                        // Dash without active beat - skip for now
                        // TODO: Handle leading dashes if needed
                    }
                }
                ParsedElement::Barline { .. } |
                ParsedElement::Newline { .. } => {
                    self.finish_current_beat();
                    self.output.push(element);
                }
                _ => {
                    // Other elements (whitespace, slurs, etc.) - pass through
                    self.output.push(element);
                }
            }
        }
        
        // Finish any remaining beat
        self.finish_current_beat();
        
        self.output
    }

    fn start_new_beat(&mut self, element: ParsedElement) {
        let beat = BeatBuilder {
            elements: vec![ElementWithSubdivisions {
                element,
                subdivisions: 1,
            }],
            total_divisions: 1,
        };
        self.current_beat = Some(beat);
    }

    fn add_note_to_beat(&mut self, element: ParsedElement) {
        if let Some(beat) = &mut self.current_beat {
            beat.total_divisions += 1;
            beat.elements.push(ElementWithSubdivisions {
                element,
                subdivisions: 1,
            });
        }
    }

    fn extend_current_beat(&mut self) {
        if let Some(beat) = &mut self.current_beat {
            beat.total_divisions += 1;
            // Extend the last element's subdivisions
            if let Some(last) = beat.elements.last_mut() {
                last.subdivisions += 1;
            }
        }
    }

    fn finish_current_beat(&mut self) {
        if let Some(beat) = self.current_beat.take() {
            // Calculate duration fractions and emit elements with durations
            for elem_with_subdivisions in beat.elements {
                let (reduced_num, reduced_denom) = reduce_fraction(
                    elem_with_subdivisions.subdivisions, 
                    beat.total_divisions
                );
                
                eprintln!("CLEAN FSM: Element {} gets duration {}/{}", 
                    elem_with_subdivisions.element.value(), 
                    reduced_num,
                    reduced_denom);
                
                // Create updated element with duration
                let updated_element = match elem_with_subdivisions.element {
                    ParsedElement::Note { degree, octave, value, position, children, .. } => {
                        ParsedElement::Note {
                            degree,
                            octave,
                            value,
                            position,
                            children,
                            duration: Some((reduced_num, reduced_denom)),
                        }
                    }
                    ParsedElement::Rest { value, position, .. } => {
                        ParsedElement::Rest {
                            value,
                            position,
                            duration: Some((reduced_num, reduced_denom)),
                        }
                    }
                    ParsedElement::Dash { degree, octave, position, .. } => {
                        ParsedElement::Dash {
                            degree,
                            octave,
                            position,
                            duration: Some((reduced_num, reduced_denom)),
                        }
                    }
                    other => other, // Pass through other elements unchanged
                };
                
                self.output.push(updated_element);
            }
        }
    }
}

// Helper function to reduce fractions (like V1)
fn gcd(a: usize, b: usize) -> usize {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn reduce_fraction(numerator: usize, denominator: usize) -> (usize, usize) {
    let g = gcd(numerator, denominator);
    (numerator / g, denominator / g)
}