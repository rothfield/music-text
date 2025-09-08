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
        let (notes_content, lyrics_content) = self.convert_staves_to_raw_notes_and_lyrics(staves);
        self.web_fast_formatter.format_with_lyrics(&notes_content, &lyrics_content)
    }
    
    /// Core conversion logic: ProcessedStaves with beats -> LilyPond note content
    fn convert_staves_to_notes(&self, staves: &[ProcessedStave]) -> String {
        let (notes_content, _) = self.convert_staves_to_notes_and_lyrics(staves);
        notes_content
    }
    
    /// Convert staves to raw notes and lyrics content for web-fast template (no Staff wrappers)
    fn convert_staves_to_raw_notes_and_lyrics(&self, staves: &[ProcessedStave]) -> (String, String) {
        // For simplicity, just use the first stave for web-fast template
        // Multi-stave support in web-fast template would need additional template logic
        if let Some(first_stave) = staves.first() {
            let (stave_notes, stave_lyrics, _) = self.convert_single_stave_to_notes_and_lyrics(first_stave);
            (stave_notes, stave_lyrics)
        } else {
            (String::new(), String::new())
        }
    }

    /// Convert staves to both notes and lyrics content for addLyrics pattern (with Staff wrappers)
    fn convert_staves_to_notes_and_lyrics(&self, staves: &[ProcessedStave]) -> (String, String) {
        let mut notes_result = String::new();
        let mut lyrics_result = String::new();
        let mut has_lyrics = false;
        
        // For multiple staves, wrap everything in simultaneous music
        if staves.len() > 1 {
            notes_result.push_str("<<\n");
        }
        
        let mut in_multi_stave_group = false;
        
        for (i, stave) in staves.iter().enumerate() {
            // Check if this stave begins a multi-stave group
            if stave.begin_multi_stave && !in_multi_stave_group {
                notes_result.push_str("  \\new StaffGroup <<\n");
                in_multi_stave_group = true;
            }
            
            // Convert single stave to notes and lyrics
            let (stave_notes, stave_lyrics, stave_has_lyrics) = self.convert_single_stave_to_notes_and_lyrics(stave);
            
            // Wrap stave in \new Staff context with proper indentation
            let indent = if staves.len() > 1 && in_multi_stave_group { "    " } else if staves.len() > 1 { "  " } else { "" };
            let wrapped_notes = format!(
                "{}\\new Staff {{\n{}  \\fixed c' {{\n{}    \\autoBeamOff\n{}    {}\n{}  }}\n{}}}",
                indent, indent, indent, indent, stave_notes.trim(), indent, indent
            );
            
            notes_result.push_str(&wrapped_notes);
            if i + 1 < staves.len() {
                notes_result.push('\n');
            }
            
            lyrics_result.push_str(&stave_lyrics);
            if stave_has_lyrics {
                has_lyrics = true;
            }
            
            // Check if this stave ends a multi-stave group
            if stave.end_multi_stave && in_multi_stave_group {
                notes_result.push_str("  >>\n");
                in_multi_stave_group = false;
            }
        }
        
        // Close the simultaneous music block
        if staves.len() > 1 {
            notes_result.push_str(">>");
        }
        
        // Return empty lyrics if no actual syllables were found
        let final_lyrics = if has_lyrics {
            lyrics_result.trim().to_string()
        } else {
            String::new()
        };
        
        (notes_result, final_lyrics)
    }
    
    /// Convert a single stave to LilyPond notation and extract lyrics
    fn convert_single_stave_to_notes_and_lyrics(&self, stave: &ProcessedStave) -> (String, String, bool) {
        let mut notes_result = String::new();
        let mut lyrics_result = String::new();
        let mut has_lyrics = false;
        let mut is_first_item = true;
        
        for rhythm_item in &stave.rhythm_items {
            match rhythm_item {
                Item::Beat(beat) => {
                    is_first_item = false;
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
                    // Skip leading barlines as they create invalid LilyPond syntax
                    if !is_first_item {
                        notes_result.push_str("| ");
                    }
                    is_first_item = false;
                    // No lyrics entry needed for barlines
                }
                Item::Breathmark => {
                    notes_result.push_str("\\breathe ");
                    is_first_item = false;
                    // No lyrics entry needed for breath marks
                }
                Item::Tonic(_) => {
                    // Tonic declarations don't generate output notation - they set context
                    // Skip for now
                }
            }
        }
        
        // Return empty lyrics if no actual syllables were found
        let final_lyrics = if has_lyrics {
            lyrics_result.trim().to_string()
        } else {
            String::new()
        };
        
        (notes_result, final_lyrics, has_lyrics)
    }
    
    /// Convert a beat element to LilyPond notation and extract syllable for lyrics
    fn convert_beat_element_to_lilypond_with_lyrics(&self, beat_element: &crate::rhythm_fsm::BeatElement) -> (String, Option<String>) {
        match &beat_element.event {
            Event::Note { degree, .. } => {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stave_parser::ProcessedStave;
    use crate::document::model::{NotationSystem, Source, Position};
    use crate::rhythm_fsm::{Item, Beat, BeatElement, Event};
    use crate::old_models::{Degree, BarlineType};

    #[test]
    fn test_row_row_row_multi_stave() {
        let renderer = LilyPondRenderer::new();
        
        // Create Row, Row, Row Your Boat test with actual musical content
        let source = Source {
            value: "test".to_string(),
            position: Position { line: 1, column: 1 },
        };

        // Create the three parts of Row, Row, Row Your Boat
        // Part 1: |1 1 1-2 3|3-2 3-4 5 -|
        let beat1_1 = create_test_beat(vec![
            (Event::Note { degree: Degree::N1, octave: 4, children: Vec::new(), slur: None }, 1),
            (Event::Note { degree: Degree::N1, octave: 4, children: Vec::new(), slur: None }, 1),
            (Event::Note { degree: Degree::N1, octave: 4, children: Vec::new(), slur: None }, 2), // 1-2 (extended)
            (Event::Note { degree: Degree::N3, octave: 4, children: Vec::new(), slur: None }, 1),
        ]);
        let beat1_2 = create_test_beat(vec![
            (Event::Note { degree: Degree::N3, octave: 4, children: Vec::new(), slur: None }, 2), // 3-2 (extended)
            (Event::Note { degree: Degree::N3, octave: 4, children: Vec::new(), slur: None }, 1),
            (Event::Note { degree: Degree::N4, octave: 4, children: Vec::new(), slur: None }, 1),
            (Event::Note { degree: Degree::N5, octave: 4, children: Vec::new(), slur: None }, 1),
            (Event::Rest, 1),
        ]);

        // Part 2: |5 5 5-5 5|5-5 5-5 5 -|
        let beat2_1 = create_test_beat(vec![
            (Event::Note { degree: Degree::N5, octave: 4, children: Vec::new(), slur: None }, 1),
            (Event::Note { degree: Degree::N5, octave: 4, children: Vec::new(), slur: None }, 1),
            (Event::Note { degree: Degree::N5, octave: 4, children: Vec::new(), slur: None }, 2), // 5-5 (extended)
            (Event::Note { degree: Degree::N5, octave: 4, children: Vec::new(), slur: None }, 1),
        ]);
        let beat2_2 = create_test_beat(vec![
            (Event::Note { degree: Degree::N5, octave: 4, children: Vec::new(), slur: None }, 2), // 5-5 (extended)
            (Event::Note { degree: Degree::N5, octave: 4, children: Vec::new(), slur: None }, 2), // 5-5 (extended)
            (Event::Note { degree: Degree::N5, octave: 4, children: Vec::new(), slur: None }, 1),
            (Event::Rest, 1),
        ]);

        // Part 3: |1 1 1-1 1|1-1 1-1 1 -|
        let beat3_1 = create_test_beat(vec![
            (Event::Note { degree: Degree::N1, octave: 4, children: Vec::new(), slur: None }, 1),
            (Event::Note { degree: Degree::N1, octave: 4, children: Vec::new(), slur: None }, 1),
            (Event::Note { degree: Degree::N1, octave: 4, children: Vec::new(), slur: None }, 2), // 1-1 (extended)
            (Event::Note { degree: Degree::N1, octave: 4, children: Vec::new(), slur: None }, 1),
        ]);
        let beat3_2 = create_test_beat(vec![
            (Event::Note { degree: Degree::N1, octave: 4, children: Vec::new(), slur: None }, 2), // 1-1 (extended)
            (Event::Note { degree: Degree::N1, octave: 4, children: Vec::new(), slur: None }, 2), // 1-1 (extended)
            (Event::Note { degree: Degree::N1, octave: 4, children: Vec::new(), slur: None }, 1),
            (Event::Rest, 1),
        ]);

        let part1 = ProcessedStave {
            text_lines_before: Vec::new(),
            rhythm_items: vec![
                Item::Barline(BarlineType::Single, None),
                Item::Beat(beat1_1),
                Item::Barline(BarlineType::Single, None),
                Item::Beat(beat1_2),
                Item::Barline(BarlineType::Single, None),
            ],
            text_lines_after: Vec::new(),
            notation_system: NotationSystem::Number,
            source: source.clone(),
            begin_multi_stave: true,   // First part begins group
            end_multi_stave: false,
        };

        let part2 = ProcessedStave {
            text_lines_before: Vec::new(),
            rhythm_items: vec![
                Item::Barline(BarlineType::Single, None),
                Item::Beat(beat2_1),
                Item::Barline(BarlineType::Single, None),
                Item::Beat(beat2_2),
                Item::Barline(BarlineType::Single, None),
            ],
            text_lines_after: Vec::new(),
            notation_system: NotationSystem::Number,
            source: source.clone(),
            begin_multi_stave: false,
            end_multi_stave: false,   // Middle part
        };

        let part3 = ProcessedStave {
            text_lines_before: Vec::new(),
            rhythm_items: vec![
                Item::Barline(BarlineType::Single, None),
                Item::Beat(beat3_1),
                Item::Barline(BarlineType::Single, None),
                Item::Beat(beat3_2),
                Item::Barline(BarlineType::Single, None),
            ],
            text_lines_after: Vec::new(),
            notation_system: NotationSystem::Number,
            source: source.clone(),
            begin_multi_stave: false,
            end_multi_stave: true,    // Last part ends group
        };

        let staves = vec![part1, part2, part3];
        let result = renderer.render_minimal(&staves);
        
        println!("Row, Row, Row Your Boat Multi-stave LilyPond output:\n{}", result);
        
        // Check that the output contains StaffGroup markers
        assert!(result.contains("\\new StaffGroup <<"));
        assert!(result.contains(">>"));
        assert!(result.contains("\\new Staff"));
        
        // Check that it contains the musical notes
        assert!(result.contains("c4")); // Note 1
        assert!(result.contains("e4")); // Note 3  
        assert!(result.contains("f4")); // Note 4
        assert!(result.contains("g4")); // Note 5
        assert!(result.contains("r4")); // Rest
    }

    fn create_test_beat(events: Vec<(Event, usize)>) -> Beat {
        let mut elements = Vec::new();
        let mut total_subdivisions = 0;
        
        for (event, subdivisions) in events {
            total_subdivisions += subdivisions;
            elements.push(BeatElement {
                event,
                subdivisions,
                duration: fraction::Fraction::new(subdivisions as u32, total_subdivisions as u32),
                tuplet_duration: fraction::Fraction::new(1u32, 4u32), // Quarter note base
                tuplet_display_duration: None,
                value: "test".to_string(),
                position: crate::old_models::Position { row: 1, col: 1 },
            });
        }
        
        // Determine if this is a tuplet (non-power-of-2 divisions)
        let is_tuplet = total_subdivisions > 1 && (total_subdivisions & (total_subdivisions - 1)) != 0;
        let tuplet_ratio = if is_tuplet && total_subdivisions > 0 {
            // Find next lower power of 2 (with safety check)
            let mut den = 1;
            let mut iteration_count = 0;
            while den * 2 < total_subdivisions && iteration_count < 64 {
                den *= 2;
                iteration_count += 1;
            }
            Some((total_subdivisions, den))
        } else {
            None
        };
        
        Beat {
            divisions: total_subdivisions,
            elements,
            tied_to_previous: false,
            is_tuplet,
            tuplet_ratio,
        }
    }

    #[test]
    fn test_multi_stave_rendering() {
        let renderer = LilyPondRenderer::new();
        
        // Create test data for multi-stave group
        let source = Source {
            value: "test".to_string(),
            position: Position { line: 1, column: 1 },
        };

        // Create simple beat with one note
        let beat_element = BeatElement {
            event: Event::Note { 
                degree: Degree::N1, 
                octave: 4,
                children: Vec::new(),
                slur: None,
            },
            subdivisions: 1,
            duration: fraction::Fraction::new(1u32, 4u32),
            tuplet_duration: fraction::Fraction::new(1u32, 4u32),
            tuplet_display_duration: None,
            value: "1".to_string(),
            position: crate::old_models::Position { row: 1, col: 1 },
        };
        
        let beat = Beat {
            divisions: 1,
            elements: vec![beat_element],
            tied_to_previous: false,
            is_tuplet: false,
            tuplet_ratio: None,
        };

        let stave1 = ProcessedStave {
            text_lines_before: Vec::new(),
            rhythm_items: vec![Item::Beat(beat.clone())],
            text_lines_after: Vec::new(),
            notation_system: NotationSystem::Number,
            source: source.clone(),
            begin_multi_stave: true,   // First stave begins group
            end_multi_stave: false,
        };

        let stave2 = ProcessedStave {
            text_lines_before: Vec::new(),
            rhythm_items: vec![Item::Beat(beat.clone())],
            text_lines_after: Vec::new(),
            notation_system: NotationSystem::Number,
            source: source.clone(),
            begin_multi_stave: false,
            end_multi_stave: true,    // Second stave ends group
        };

        let staves = vec![stave1, stave2];
        let result = renderer.render_minimal(&staves);
        
        println!("Multi-stave LilyPond output:\n{}", result);
        
        // Check that the output contains StaffGroup markers
        assert!(result.contains("\\new StaffGroup <<"));
        assert!(result.contains(">>"));
        assert!(result.contains("\\new Staff"));
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