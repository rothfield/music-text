use serde::Serialize;
use crate::rhythm::{Item, Event, BeatElement};
use crate::rhythm::types::Degree;

#[derive(Serialize, Clone)]
struct Header {
    tagline: Option<String>,
    print_page_number: Option<String>,
    extra: Option<String>,
}

#[derive(Serialize, Clone)]
struct Voice {
    id: String,
    voice_directive: Option<String>,
    fixed: Option<String>,
    music: String,
}

#[derive(Serialize, Clone)]
struct Lyrics {
    voice_id: String,
    syllables: Vec<String>,
}

#[derive(Serialize, Clone)]
struct Staff {
    staff_clef: Option<String>,
    voices: Vec<Voice>,
    lyrics: Vec<Lyrics>,
    begin_stave_group_str: Option<String>,  // "\new StaffGroup <<" when starting group
    end_stave_group_str: Option<String>,    // ">>" when ending group
}

#[derive(Serialize)]
struct TemplateContext {
    version: String,
    language: Option<String>,
    header: Option<Header>,
    paper: Option<String>,
    layout: Option<String>,
    staves: Vec<Staff>,
    midi: Option<String>,
}

/// Full LilyPond formatter using comprehensive mustache template
pub struct FullFormatter;

impl FullFormatter {
    pub fn new() -> Self {
        Self
    }
    
    /// Format staves using comprehensive template  
    pub fn format_staves(&self, staves: &[crate::stave::ProcessedStave]) -> String {
        let context = self.build_template_context(staves);
        
        let template_str = r#"\version "{{version}}"
{{#language}}\language "{{language}}"{{/language}}

{{#header}}
\header {
  {{#tagline}}tagline = {{tagline}}{{/tagline}}
  {{#print_page_number}}print-page-number = {{print_page_number}}{{/print_page_number}}
  {{#extra}}{{{extra}}}{{/extra}}
}
{{/header}}

{{#paper}}
\paper {
  {{{paper}}}
}
{{/paper}}

{{#layout}}
\layout {
  {{{layout}}}
}
{{/layout}}

\score {
  <<
    {{#staves}}
    {{{begin_stave_group_str}}}
    \new Staff <<
      {{#staff_clef}}\clef {{staff_clef}}{{/staff_clef}}
      {{#voices}}
      \new Voice = "{{id}}" { 
        {{#voice_directive}}\{{voice_directive}}{{/voice_directive}}
        {{#fixed}}\fixed {{fixed}} { {{{music}}} }{{/fixed}}
        {{^fixed}}{{{music}}}{{/fixed}}
      }
      {{/voices}}

      {{#lyrics}}
      \new Lyrics \lyricsto "{{voice_id}}" {
        \lyricmode {
          {{#syllables}}{{.}} {{/syllables}}
        }
      }
      {{/lyrics}}
    >>
    {{{end_stave_group_str}}}
    {{/staves}}
  >>
  {{#midi}}\midi { {{{midi}}} }{{/midi}}
}"#;
        
        let template = mustache::compile_str(template_str)
            .expect("Failed to compile comprehensive LilyPond template");
            
        let rendered = template.render_to_string(&context)
            .expect("Failed to render comprehensive LilyPond template");
        
        // Decode HTML entities that mustache might have encoded
        rendered
            .replace("&#39;", "'")
            .replace("&quot;", "\"")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
    }
    
    /// Build TemplateContext from ProcessedStave objects
    fn build_template_context(&self, staves: &[crate::stave::ProcessedStave]) -> TemplateContext {
        let mut output_staves = Vec::new();
        let mut voice_counter = 1;
        
        for processed_stave in staves {
            // Get lyrics from the stave source
            let lyrics_syllables = self.extract_lyrics_from_stave(processed_stave);
            
            // Convert beats to LilyPond notation
            let music_content = self.convert_beats_to_lilypond(processed_stave);
            
            let voice_id = format!("melody{}", voice_counter);
            
            let voice = Voice {
                id: voice_id.clone(),
                voice_directive: None,
                fixed: Some("c'".to_string()),
                music: music_content,
            };
            
            let lyrics = if lyrics_syllables.is_empty() {
                vec![]
            } else {
                vec![Lyrics {
                    voice_id,
                    syllables: lyrics_syllables,
                }]
            };
            
            let staff = Staff {
                staff_clef: Some("treble".to_string()),
                voices: vec![voice],
                lyrics,
                begin_stave_group_str: if processed_stave.begin_multi_stave {
                    Some("\\new StaffGroup <<".to_string())
                } else {
                    None
                },
                end_stave_group_str: if processed_stave.end_multi_stave {
                    Some(">>".to_string())
                } else {
                    None
                },
            };
            
            output_staves.push(staff);
            voice_counter += 1;
        }
        
        TemplateContext {
            version: "2.24.0".to_string(),
            language: Some("english".to_string()),
            header: Some(Header {
                tagline: Some("#f".to_string()),
                print_page_number: Some("#f".to_string()),
                extra: None,
            }),
            paper: None,
            layout: None,
            staves: output_staves,
            midi: None,
        }
    }
    
    /// Extract lyrics from ProcessedStave source
    fn extract_lyrics_from_stave(&self, stave: &crate::stave::ProcessedStave) -> Vec<String> {
        // Extract lyrics from the actual lyrics_lines in the stave
        let mut lyrics_syllables = Vec::new();
        
        for lyrics_line in &stave.lyrics_lines {
            lyrics_syllables.extend(
                lyrics_line.syllables.iter()
                    .map(|syllable| syllable.content.clone())
            );
        }
        
        lyrics_syllables
    }
    
    /// Convert ProcessedStave beats to LilyPond notation
    fn convert_beats_to_lilypond(&self, stave: &crate::stave::ProcessedStave) -> String {
        let (notes_content, _lyrics, _has_lyrics) = self.convert_single_stave_to_notes_and_lyrics(stave);
        notes_content
    }
    
    /// Convert a single stave to LilyPond notation and extract lyrics (copied from working renderer)
    fn convert_single_stave_to_notes_and_lyrics(&self, stave: &crate::stave::ProcessedStave) -> (String, String, bool) {
        let mut notes_result = String::new();
        let mut lyrics_result = String::new();
        let mut has_lyrics = false;
        let mut is_first_item = true;
        let mut previous_beat_last_note: Option<Degree> = None;
        
        for rhythm_item in &stave.rhythm_items {
            match rhythm_item {
                Item::Beat(beat) => {
                    is_first_item = false;
                    
                    // Check for ties to previous beat
                    let need_tie_to_previous = beat.tied_to_previous && 
                        previous_beat_last_note.is_some() &&
                        beat.elements.first().map_or(false, |elem| {
                            if let Event::Note { degree, .. } = &elem.event {
                                Some(*degree) == previous_beat_last_note
                            } else {
                                false
                            }
                        });
                    
                    if beat.is_tuplet {
                        if let Some((num, den)) = beat.tuplet_ratio {
                            notes_result.push_str(&format!("\\tuplet {}/{} {{ ", num, den));
                            
                            for (i, beat_element) in beat.elements.iter().enumerate() {
                                let (lily_note, syllable) = self.convert_beat_element_to_lilypond_with_lyrics(beat_element);
                                
                                // Add tie to first note if needed
                                if i == 0 && need_tie_to_previous {
                                    // Modify the previous notes_result to add tie mark
                                    if let Some(last_space) = notes_result.rfind(' ') {
                                        notes_result.insert_str(last_space, "~");
                                    }
                                }
                                
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
                        for (i, beat_element) in beat.elements.iter().enumerate() {
                            let (lily_note, syllable) = self.convert_beat_element_to_lilypond_with_lyrics(beat_element);
                            
                            // Add tie to first note if needed
                            if i == 0 && need_tie_to_previous {
                                // Modify the previous notes_result to add tie mark
                                if let Some(last_space) = notes_result.rfind(' ') {
                                    notes_result.insert_str(last_space, "~");
                                }
                            }
                            
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
                    
                    // Update previous beat last note for tie detection
                    if let Some(last_element) = beat.elements.last() {
                        if let Event::Note { degree, .. } = &last_element.event {
                            previous_beat_last_note = Some(*degree);
                        }
                    }
                }
                Item::Barline(barline_type, _) => {
                    // Skip leading barlines as they create invalid LilyPond syntax
                    if !is_first_item {
                        use crate::rhythm::converters::BarlineType;
                        match barline_type {
                            BarlineType::Single => notes_result.push_str("| "),
                            BarlineType::Double => notes_result.push_str("\\bar \"||\" "),
                            BarlineType::RepeatStart => notes_result.push_str("\\bar \"|:\" "),
                            BarlineType::RepeatEnd => notes_result.push_str("\\bar \":|\" "),
                            BarlineType::RepeatBoth => notes_result.push_str("\\bar \"|:|\" "),
                        }
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
    fn convert_beat_element_to_lilypond_with_lyrics(&self, beat_element: &BeatElement) -> (String, Option<String>) {
        match &beat_element.event {
            Event::Note { degree, octave, .. } => {
                let lily_pitch = degree_to_lilypond_with_octave(*degree, *octave);
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
        Degree::N1bb => "cff".to_string(),
        Degree::N1b => "cf".to_string(),
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

/// Convert degree and octave to LilyPond pitch notation with octave markers
fn degree_to_lilypond_with_octave(degree: Degree, octave: i8) -> String {
    let base_pitch = degree_to_lilypond_simple(degree);
    let octave_markers = octave_to_lilypond_markers(octave);
    format!("{}{}", base_pitch, octave_markers)
}

/// Convert octave number to LilyPond octave markers
/// Octave 0 = default (c), +1 = higher (c'), -1 = lower (c,)
fn octave_to_lilypond_markers(octave: i8) -> String {
    match octave {
        0 => "".to_string(),  // Default octave
        n if n > 0 => "'".repeat(n as usize),  // Higher octaves: c' c'' c'''
        n if n < 0 => ",".repeat((-n) as usize),  // Lower octaves: c, c,, c,,,
        _ => "".to_string(),  // Fallback
    }
}

impl Default for FullFormatter {
    fn default() -> Self {
        Self::new()
    }
}