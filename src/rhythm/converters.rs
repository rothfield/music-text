// Rhythm conversion utilities - extracted from old_models.rs
// Handles conversion between different rhythm representations

use serde::{Deserialize, Serialize};
use fraction::Fraction;

/// Barline type enumeration for document structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BarlineType {
    Single,      // |
    Double,      // ||
    Final,       // |.
    RepeatStart, // |:
    RepeatEnd,   // :|
    RepeatBoth,  // :|: or |:|
}

impl BarlineType {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "|" => Ok(BarlineType::Single),
            "||" => Ok(BarlineType::Double),
            "|." => Ok(BarlineType::Final),
            "|:" => Ok(BarlineType::RepeatStart),
            ":|" => Ok(BarlineType::RepeatEnd),
            "|:|" | ":|:" => Ok(BarlineType::RepeatBoth),
            _ => Err(format!("Unknown barline type: {}", s)),
        }
    }
}

/// Converter for rhythm-related operations
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