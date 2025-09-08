use crate::stave_parser::{ProcessedStave, StaffGroupInfo};
use crate::document::model::StaffGroupType;
use crate::rhythm_fsm::{Item, Event};
use crate::old_models::Degree;
use super::formatters::{MinimalFormatter, FullFormatter, WebFastFormatter};
use serde::Serialize;

/// Helper enum for grouping staves during rendering
#[derive(Debug)]
enum StaveGroup<'a> {
    Single(&'a ProcessedStave),
    Group {
        group_type: StaffGroupType,
        staves: Vec<&'a ProcessedStave>,
    },
}

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
        let (notes_content, lyrics_content) = self.convert_staves_to_notes_and_lyrics(staves);
        self.minimal_formatter.format_with_lyrics(&notes_content, &lyrics_content)
    }
    
    /// Convert staves to full LilyPond score using template
    pub fn render_full(&self, staves: &[ProcessedStave]) -> String {
        let notes_content = self.convert_staves_to_notes(staves);
        self.full_formatter.format(&notes_content)
    }
    
    /// Convert staves to fast web-optimized LilyPond for SVG generation
    pub fn render_web_fast(&self, staves: &[ProcessedStave]) -> String {
        let (notes_content, lyrics_content) = self.convert_staves_to_notes_and_lyrics(staves);
        self.web_fast_formatter.format_with_lyrics(&notes_content, &lyrics_content)
    }
    
    /// Core conversion logic: ProcessedStaves with beats -> LilyPond note content
    fn convert_staves_to_notes(&self, staves: &[ProcessedStave]) -> String {
        let (notes_content, _) = self.convert_staves_to_notes_and_lyrics(staves);
        notes_content
    }
    
    /// Convert staves to both notes and lyrics content for addLyrics pattern
    fn convert_staves_to_notes_and_lyrics(&self, staves: &[ProcessedStave]) -> (String, String) {
        let mut notes_result = String::new();
        let mut lyrics_result = String::new();
        let mut has_lyrics = false;
        
        // Group staves by their staff group context
        let grouped_staves = self.group_staves_by_context(staves);
        
        for (i, stave_group) in grouped_staves.iter().enumerate() {
            if i > 0 {
                notes_result.push_str(" | ");
            }
            
            match stave_group {
                StaveGroup::Single(stave) => {
                    let (stave_notes, stave_lyrics, stave_has_lyrics) = self.convert_single_stave_to_notes_and_lyrics(stave);
                    
                    // Wrap single staves in \new Staff context since template no longer does this
                    let wrapped_notes = format!(
                        "\\new Staff {{\n  \\fixed c' {{\n    \\autoBeamOff\n    {}\n  }}\n}}",
                        stave_notes.trim()
                    );
                    
                    notes_result.push_str(&wrapped_notes);
                    lyrics_result.push_str(&stave_lyrics);
                    if stave_has_lyrics {
                        has_lyrics = true;
                    }
                }
                StaveGroup::Group { group_type, staves: group_staves } => {
                    let (group_notes, group_lyrics, group_has_lyrics) = self.convert_staff_group_to_notes_and_lyrics(*group_type, group_staves);
                    notes_result.push_str(&group_notes);
                    lyrics_result.push_str(&group_lyrics);
                    if group_has_lyrics {
                        has_lyrics = true;
                    }
                }
            }
        }
        
        // Return empty lyrics if no actual syllables were found
        let final_lyrics = if has_lyrics {
            lyrics_result.trim().to_string()
        } else {
            String::new()
        };
        
        (notes_result, final_lyrics)
    }

    /// Group staves by their staff group context
    fn group_staves_by_context<'a>(&self, staves: &'a [ProcessedStave]) -> Vec<StaveGroup<'a>> {
        let mut grouped = Vec::new();
        let mut i = 0;
        
        while i < staves.len() {
            let stave = &staves[i];
            
            if let Some(ref group_info) = stave.staff_group_info {
                // This is part of a staff group - collect all staves in this group
                let group_type = group_info.group_type;
                let group_size = group_info.group_size;
                
                // Collect the group staves (they should be consecutive)
                let mut group_staves = Vec::new();
                for j in 0..group_size {
                    if i + j < staves.len() {
                        group_staves.push(&staves[i + j]);
                    }
                }
                
                grouped.push(StaveGroup::Group { group_type, staves: group_staves });
                i += group_size; // Skip past all the staves in this group
            } else {
                // Single stave
                grouped.push(StaveGroup::Single(stave));
                i += 1;
            }
        }
        
        grouped
    }

    /// Convert a single stave to notes and lyrics
    fn convert_single_stave_to_notes_and_lyrics(&self, stave: &ProcessedStave) -> (String, String, bool) {
        let mut notes_result = String::new();
        let mut lyrics_result = String::new();
        let mut has_lyrics = false;
        
        for rhythm_item in &stave.rhythm_items {
            match rhythm_item {
                Item::Beat(beat) => {
                    if beat.is_tuplet {
                        if let Some((num, den)) = beat.tuplet_ratio {
                            notes_result.push_str(&format!("\\tuplet {}/{} {{ ", num, den));
                            
                            for beat_element in &beat.elements {
                                let (lily_note, syllable) = self.convert_beat_element_to_lilypond_with_lyrics(beat_element);
                                notes_result.push_str(&lily_note);
                                
                                if let Some(syl) = syllable {
                                    lyrics_result.push_str(&syl);
                                    lyrics_result.push(' ');
                                    has_lyrics = true;
                                } else {
                                    // Add placeholder for notes without syllables
                                    lyrics_result.push_str("_ ");
                                }
                            }
                            
                            notes_result.push_str("} ");
                        }
                    } else {
                        // Regular beat - no tuplet wrapper
                        for beat_element in &beat.elements {
                            let (lily_note, syllable) = self.convert_beat_element_to_lilypond_with_lyrics(beat_element);
                            notes_result.push_str(&lily_note);
                            
                            if let Some(syl) = syllable {
                                lyrics_result.push_str(&syl);
                                lyrics_result.push(' ');
                                has_lyrics = true;
                            } else {
                                // Add placeholder for notes without syllables
                                lyrics_result.push_str("_ ");
                            }
                        }
                    }
                }
                Item::Barline(_, _) => {
                    notes_result.push_str("| ");
                    // No lyrics entry needed for barlines
                }
                Item::Breathmark => {
                    notes_result.push_str("\\breathe ");
                    // No lyrics entry needed for breath marks
                }
                Item::Tonic(_) => {
                    // Tonic declarations don't generate output notation - they set context
                    // Skip for now
                }
            }
        }
        
        (notes_result, lyrics_result, has_lyrics)
    }

    /// Convert a staff group to LilyPond notation with staff grouping contexts
    fn convert_staff_group_to_notes_and_lyrics(&self, group_type: StaffGroupType, staves: &[&ProcessedStave]) -> (String, String, bool) {
        let context_name = group_type.to_lilypond_context();
        let mut group_notes = format!("\\new {} <<\n", context_name);
        let mut group_lyrics = String::new();
        let mut has_lyrics = false;
        
        for stave in staves {
            let (stave_notes, stave_lyrics, stave_has_lyrics) = self.convert_single_stave_to_notes_and_lyrics(stave);
            
            // Get staff name and infer clef
            let staff_name = if let Some(ref info) = stave.staff_group_info {
                &info.staff_name
            } else {
                "unnamed"
            };
            
            let clef = self.infer_clef_from_name(staff_name);
            
            group_notes.push_str(&format!("  \\new Staff = \"{}\" {{\n", staff_name));
            
            if let Some(clef) = clef {
                group_notes.push_str(&format!("    \\clef {}\n", clef));
            }
            
            group_notes.push_str("    \\fixed c' {\n");
            group_notes.push_str("      \\autoBeamOff\n");
            group_notes.push_str(&format!("      {}\n", stave_notes.trim()));
            group_notes.push_str("    }\n");
            group_notes.push_str("  }\n");
            
            // Combine lyrics (for now, just concatenate - more sophisticated logic may be needed)
            if stave_has_lyrics {
                if !group_lyrics.is_empty() {
                    group_lyrics.push(' ');
                }
                group_lyrics.push_str(&stave_lyrics);
                has_lyrics = true;
            }
        }
        
        group_notes.push_str(">>\n");
        
        (group_notes, group_lyrics, has_lyrics)
    }

    /// Infer appropriate clef from staff name
    fn infer_clef_from_name(&self, name: &str) -> Option<&'static str> {
        let name_lower = name.to_lowercase();
        match name_lower.as_str() {
            "treble" | "soprano" | "alto" | "violin" | "violin1" | "violin2" | "flute" | "oboe" | "clarinet" => Some("treble"),
            "bass" | "cello" | "contrabass" | "bassoon" | "tuba" => Some("bass"),
            "viola" | "tenor" | "horn" => Some("alto"), 
            _ => None, // Let LilyPond use default treble clef
        }
    }
    
    /// Convert a beat element to LilyPond notation with proper duration
    fn convert_beat_element_to_lilypond(&self, beat_element: &crate::rhythm_fsm::BeatElement) -> String {
        let (note, _) = self.convert_beat_element_to_lilypond_with_lyrics(beat_element);
        note
    }
    
    /// Convert a beat element to LilyPond notation and extract syllable for lyrics
    fn convert_beat_element_to_lilypond_with_lyrics(&self, beat_element: &crate::rhythm_fsm::BeatElement) -> (String, Option<String>) {
        match &beat_element.event {
            Event::Note { degree, octave, .. } => {
                let lily_pitch = degree_to_lilypond_simple(*degree);
                // Use the sophisticated FSM-calculated tuplet_duration instead of simple subdivision mapping
                let duration = fraction_to_lilypond_duration(beat_element.tuplet_duration);
                
                let note = format!("{}{} ", lily_pitch, duration);
                
                // Extract syllable for lyrics - pass through directly from parser
                let syllable = beat_element.syl();
                
                (note, syllable)
            }
            Event::Rest => {
                // Use the sophisticated FSM-calculated tuplet_duration for rests too
                let duration = fraction_to_lilypond_duration(beat_element.tuplet_duration);
                let note = format!("r{} ", duration);
                (note, None) // Rests don't have syllables
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