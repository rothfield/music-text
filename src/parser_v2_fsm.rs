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




