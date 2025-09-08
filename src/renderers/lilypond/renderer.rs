use crate::stave_parser::ProcessedStave;
use crate::rhythm_fsm::{Item, Event};
use crate::old_models::Degree;
use super::formatters::{MinimalFormatter, FullFormatter, WebFastFormatter};
use serde::Serialize;

/// Main LilyPond rendering orchestrator
pub struct LilyPondRenderer {
    minimal_formatter: MinimalFormatter,
    full_formatter: FullFormatter,
    web_fast_formatter: WebFastFormatter,
}

impl LilyPondRenderer {
    pub fn new() -> Self {
        Self {
            minimal_formatter: MinimalFormatter::new(),
            full_formatter: FullFormatter::new(),
            web_fast_formatter: WebFastFormatter::new(),
        }
    }
    
    /// Convert staves to minimal LilyPond notation
    pub fn render_minimal(&self, staves: &[ProcessedStave]) -> String {
        let notes_content = self.convert_staves_to_notes(staves);
        self.minimal_formatter.format(&notes_content)
    }
    
    /// Convert staves to full LilyPond score using template
    pub fn render_full(&self, staves: &[ProcessedStave]) -> String {
        let notes_content = self.convert_staves_to_notes(staves);
        self.full_formatter.format(&notes_content)
    }
    
    /// Convert staves to fast web-optimized LilyPond for SVG generation
    pub fn render_web_fast(&self, staves: &[ProcessedStave]) -> String {
        let notes_content = self.convert_staves_to_notes(staves);
        self.web_fast_formatter.format(&notes_content)
    }
    
    /// Core conversion logic: ProcessedStaves with beats -> LilyPond note content
    fn convert_staves_to_notes(&self, staves: &[ProcessedStave]) -> String {
        let mut result = String::new();
        
        for (i, stave) in staves.iter().enumerate() {
            if i > 0 {
                result.push_str(" | ");
            }
            
            for rhythm_item in &stave.rhythm_items {
                match rhythm_item {
                    Item::Beat(beat) => {
                        if beat.is_tuplet {
                            if let Some((num, den)) = beat.tuplet_ratio {
                                result.push_str(&format!("\\tuplet {}/{} {{ ", num, den));
                                
                                for beat_element in &beat.elements {
                                    let lily_note = self.convert_beat_element_to_lilypond(beat_element);
                                    result.push_str(&lily_note);
                                }
                                
                                result.push_str("} ");
                            }
                        } else {
                            // Regular beat - no tuplet wrapper
                            for beat_element in &beat.elements {
                                let lily_note = self.convert_beat_element_to_lilypond(beat_element);
                                result.push_str(&lily_note);
                            }
                        }
                    }
                    Item::Barline(_, _) => {
                        result.push_str("| ");
                    }
                    Item::Breathmark => {
                        result.push_str("\\breathe ");
                    }
                    Item::Tonic(_) => {
                        // Tonic declarations don't generate output notation - they set context
                        // Skip for now
                    }
                }
            }
        }
        
        result
    }
    
    /// Convert a beat element to LilyPond notation with proper duration
    fn convert_beat_element_to_lilypond(&self, beat_element: &crate::rhythm_fsm::BeatElement) -> String {
        match &beat_element.event {
            Event::Note { degree, octave, .. } => {
                let lily_pitch = degree_to_lilypond_simple(*degree);
                // Use the sophisticated FSM-calculated tuplet_duration instead of simple subdivision mapping
                let duration = fraction_to_lilypond_duration(beat_element.tuplet_duration);
                format!("{}{} ", lily_pitch, duration)
            }
            Event::Rest => {
                // Use the sophisticated FSM-calculated tuplet_duration for rests too
                let duration = fraction_to_lilypond_duration(beat_element.tuplet_duration);
                format!("r{} ", duration)
            }
        }
    }
    
    /// Convert subdivisions to LilyPond duration (simplified for now)
    fn subdivisions_to_lilypond_duration(&self, subdivisions: usize) -> String {
        match subdivisions {
            1 => "8".to_string(),   // eighth note
            2 => "4".to_string(),   // quarter note  
            3 => "4.".to_string(),  // dotted quarter
            4 => "2".to_string(),   // half note
            _ => "4".to_string(),   // fallback to quarter
        }
    }
}

/// Convert fraction duration to LilyPond duration notation
fn fraction_to_lilypond_duration(duration: fraction::Fraction) -> String {
    // Convert fraction like 1/4 to LilyPond duration like "4"
    // This is the sophisticated approach from the old system
    
    let numerator = *duration.numer().unwrap();
    let denominator = *duration.denom().unwrap();
    
    // For simple fractions like 1/4, 1/8, 1/2, etc.
    if numerator == 1 {
        match denominator {
            1 => "1".to_string(),    // whole note
            2 => "2".to_string(),    // half note  
            4 => "4".to_string(),    // quarter note
            8 => "8".to_string(),    // eighth note
            16 => "16".to_string(),  // sixteenth note
            32 => "32".to_string(),  // thirty-second note
            64 => "64".to_string(),  // sixty-fourth note
            _ => "4".to_string(),    // fallback to quarter
        }
    }
    // For dotted notes like 3/8 (dotted quarter), 3/16 (dotted eighth), etc.
    else if numerator == 3 {
        match denominator {
            4 => "2.".to_string(),   // dotted half (3/4 = 1/2 with dot)
            8 => "4.".to_string(),   // dotted quarter (3/8 = 1/4 with dot)  
            16 => "8.".to_string(),  // dotted eighth (3/16 = 1/8 with dot)
            32 => "16.".to_string(), // dotted sixteenth (3/32 = 1/16 with dot)
            _ => "4.".to_string(),   // fallback to dotted quarter
        }
    }
    // For double-dotted notes like 7/16, etc.
    else if numerator == 7 {
        match denominator {
            8 => "2..".to_string(),  // double-dotted half
            16 => "4..".to_string(), // double-dotted quarter
            32 => "8..".to_string(), // double-dotted eighth
            _ => "4..".to_string(),  // fallback to double-dotted quarter
        }
    }
    // For other fractions, try to find the closest standard duration
    else {
        // Convert to decimal and find closest match
        let decimal_value = numerator as f64 / denominator as f64;
        
        if decimal_value >= 0.75 { "1".to_string() }      // whole note
        else if decimal_value >= 0.375 { "2".to_string() } // half note
        else if decimal_value >= 0.1875 { "4".to_string() } // quarter note
        else if decimal_value >= 0.09375 { "8".to_string() } // eighth note
        else if decimal_value >= 0.046875 { "16".to_string() } // sixteenth note
        else { "32".to_string() } // thirty-second note or shorter
    }
}

/// Convert degree to simple LilyPond pitch notation
fn degree_to_lilypond_simple(degree: Degree) -> String {
    match degree {
        Degree::N1bb => "bff".to_string(),
        Degree::N1b => "bf".to_string(),
        Degree::N1 => "c".to_string(),
        Degree::N1s => "cs".to_string(),
        Degree::N1ss => "css".to_string(),
        
        Degree::N2bb => "dff".to_string(),
        Degree::N2b => "df".to_string(),
        Degree::N2 => "d".to_string(),
        Degree::N2s => "ds".to_string(),
        Degree::N2ss => "dss".to_string(),
        
        Degree::N3bb => "eff".to_string(),
        Degree::N3b => "ef".to_string(),
        Degree::N3 => "e".to_string(),
        Degree::N3s => "es".to_string(),
        Degree::N3ss => "ess".to_string(),
        
        Degree::N4bb => "fff".to_string(),
        Degree::N4b => "ff".to_string(),
        Degree::N4 => "f".to_string(),
        Degree::N4s => "fs".to_string(),
        Degree::N4ss => "fss".to_string(),
        
        Degree::N5bb => "gff".to_string(),
        Degree::N5b => "gf".to_string(),
        Degree::N5 => "g".to_string(),
        Degree::N5s => "gs".to_string(),
        Degree::N5ss => "gss".to_string(),
        
        Degree::N6bb => "aff".to_string(),
        Degree::N6b => "af".to_string(),
        Degree::N6 => "a".to_string(),
        Degree::N6s => "as".to_string(),
        Degree::N6ss => "ass".to_string(),
        
        Degree::N7bb => "bff".to_string(),
        Degree::N7b => "bf".to_string(),
        Degree::N7 => "b".to_string(),
        Degree::N7s => "bs".to_string(),
        Degree::N7ss => "bss".to_string(),
    }
}

impl Default for LilyPondRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize)]
pub struct TemplateContext {
    pub version: String,
    pub staves: String,
    pub source_comment: Option<String>,
    pub title: Option<String>,
    pub composer: Option<String>,
    pub time_signature: Option<String>,
    pub key_signature: Option<String>,
    pub lyrics: Option<String>,
    pub midi: bool,
    pub tempo: Option<String>,
}