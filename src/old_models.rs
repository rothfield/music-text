// Old model types needed for sophisticated FSM
// Copied from old.music-text codebase

use serde::{Deserialize, Serialize};
use fraction::Fraction;

// From old models/pitch.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Degree {
    // 1 series (Do/Sa/C)
    N1bb, N1b, N1, N1s, N1ss,
    // 2 series (Re/D)
    N2bb, N2b, N2, N2s, N2ss,
    // 3 series (Mi/Ga/E)
    N3bb, N3b, N3, N3s, N3ss,
    // 4 series (Fa/Ma/F)
    N4bb, N4b, N4, N4s, N4ss,
    // 5 series (Sol/Pa/G)
    N5bb, N5b, N5, N5s, N5ss,
    // 6 series (La/Dha/A)
    N6bb, N6b, N6, N6s, N6ss,
    // 7 series (Ti/Ni/B)
    N7bb, N7b, N7, N7s, N7ss,
}

// From old models/parsed.rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SlurRole {
    Start,
    Middle,
    End,
    StartEnd,
}

/// Shared position information for all parsed elements
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

/// Types of musical ornaments that can be attached to notes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrnamentType {
    Mordent,
    Trill,
    Turn,
    Grace,
}

/// Child elements that can be attached to notes (vertical spatial relationships)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParsedChild {
    /// Octave markers like dots, colons, apostrophes
    OctaveMarker { 
        symbol: String,
        distance: i8, // Vertical distance from parent note (-1 = above, +1 = below)
    },
    /// Musical ornaments like mordents, trills
    Ornament { 
        kind: OrnamentType,
        distance: i8,
    },
    /// Syllable/lyric text
    Syllable { 
        text: String,
        distance: i8,
    },
}

/// Parsed elements - what the parser extracts from raw text (flat structure)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParsedElement {
    /// Musical note with pitch
    Note { 
        degree: Degree,
        octave: i8, // Calculated from octave markers
        value: String, // Original text value (e.g. "G", "S", "1")
        position: Position,
        children: Vec<ParsedChild>, // Attached ornaments, octave markers, lyrics
        duration: Option<(usize, usize)>, // Duration fraction (numerator, denominator) from FSM
        slur: Option<SlurRole>, // Assigned by vertical_parser
    },
    
    /// Rest note
    Rest { 
        value: String, // Original text value
        position: Position,
        duration: Option<(usize, usize)>, // Duration fraction (numerator, denominator) from FSM
    },
    
    /// Note extension (dash) - inherits pitch from preceding note
    Dash { 
        degree: Option<Degree>, // Inherited from preceding note
        octave: Option<i8>, // Inherited or calculated from local octave markers
        position: Position,
        duration: Option<(usize, usize)>, // Duration fraction (numerator, denominator) from FSM
    },
    
    /// Bar line separator
    Barline { 
        style: String, // "|", "||", etc.
        position: Position,
        tala: Option<u8>, // Associated tala marker (0-6)
    },
    
    /// Whitespace/beat separator
    Whitespace { 
        value: String,
        position: Position,
    },
    
    /// Breath mark
    Symbol { 
        value: String,
        position: Position,
    },
}

// From old models/domain.rs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BarlineType {
    Single,
    Double,
    RepeatStart,
    RepeatEnd,
    RepeatBoth,
}

impl BarlineType {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "|" => Ok(BarlineType::Single),
            "||" => Ok(BarlineType::Double),
            "|:" => Ok(BarlineType::RepeatStart),
            ":|" => Ok(BarlineType::RepeatEnd),
            "|:|" => Ok(BarlineType::RepeatBoth),
            _ => Err(format!("Unknown barline type: {}", s)),
        }
    }
}

// From old models/rhythm.rs
#[derive(Debug, Clone)]
pub struct RhythmConverter;

impl RhythmConverter {
    /// Convert fraction to VexFlow duration notation (duration, dots)
    pub fn fraction_to_vexflow(frac: Fraction) -> Vec<(String, u8)> {
        // Lookup table for common fractions to VexFlow durations
        let lookup = [
            (Fraction::new(1u64, 1u64), vec![("w".to_string(), 0)]),    // whole note
            (Fraction::new(1u64, 2u64), vec![("h".to_string(), 0)]),    // half note
            (Fraction::new(1u64, 4u64), vec![("q".to_string(), 0)]),    // quarter note
            (Fraction::new(1u64, 8u64), vec![("8".to_string(), 0)]),    // eighth note
            (Fraction::new(1u64, 16u64), vec![("16".to_string(), 0)]),  // sixteenth note
            (Fraction::new(1u64, 32u64), vec![("32".to_string(), 0)]),  // thirty-second note
            (Fraction::new(3u64, 8u64), vec![("q".to_string(), 1)]),    // dotted quarter
            (Fraction::new(3u64, 16u64), vec![("8".to_string(), 1)]),   // dotted eighth
            (Fraction::new(3u64, 32u64), vec![("16".to_string(), 1)]),  // dotted sixteenth
        ];
        
        for (ref_frac, result) in lookup.iter() {
            if *ref_frac == frac {
                return result.clone();
            }
        }
        
        // Fallback - decompose into standard durations
        vec![("q".to_string(), 0)]
    }
}