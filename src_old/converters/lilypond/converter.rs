// LilyPond Source Code Generator - Works directly with ParsedElement, no conversion needed
use crate::models::{Metadata}; // Keep using existing metadata
use crate::models::{Degree};
use crate::converters::lilypond::templates::{TemplateContext, render_lilypond};
use crate::parser_v2_fsm::{Item, Beat};
use crate::converters::transposition::transpose_degree_with_octave;

/// Find the index of the last actual note (not barline, breathmark, etc.) in lilypond_notes
fn find_last_note_index(lilypond_notes: &[String]) -> Option<usize> {
    // Search backwards for the last actual note (not barline, breathmark, etc.)
    for (i, note) in lilypond_notes.iter().enumerate().rev() {
        if !note.starts_with("\\bar") && !note.starts_with("\\breathe") {
            // Include tuplets and regular notes
            return Some(i);
        }
    }
    None
}

pub fn convert_elements_to_lilypond_src(
    elements: &Vec<Item>,
    metadata: &Metadata,
    source: Option<&str>
) -> Result<String, String> {
    eprintln!("V2 LILYPOND CONVERTER: Processing FSM output with beats");
    
    let mut lilypond_notes: Vec<String> = Vec::new();
    let mut previous_beat_notes: Vec<String> = Vec::new();
    let mut current_tonic: Option<Degree> = None;
    
    for (_element_index, item) in elements.iter().enumerate() {
        match item {
            Item::Tonic(tonic_degree) => {
                // Store the tonic for transposition
                current_tonic = Some(*tonic_degree);
                // Could optionally add a key signature command here
            },
            Item::Beat(beat) => {
                let beat_notes = convert_beat_to_lilypond(beat, current_tonic)?;
                
                // Handle ties: if this beat is tied to previous, add tie to last note of previous beat
                if beat.tied_to_previous && !previous_beat_notes.is_empty() && !beat_notes.is_empty() {
                    if let Some(last_note_index) = find_last_note_index(&lilypond_notes) {
                        let last_note = &mut lilypond_notes[last_note_index];
                        if !last_note.ends_with('~') && !last_note.ends_with(')') {
                            *last_note = format!("{}~", last_note);
                        } else if last_note.ends_with(')') {
                            // Insert tie before the closing slur
                            let len = last_note.len();
                            last_note.insert(len - 1, '~');
                        }
                    }
                }
                
                lilypond_notes.extend(beat_notes.clone());
                previous_beat_notes = beat_notes;
            },
            Item::Barline(barline_type, tala) => {
                // Add tala marker to the last note before adding the barline
                if let Some(tala_num) = tala {
                    let tala_display = if *tala_num == 255 { "+" } else { &tala_num.to_string() };
                    let tala_text = &format!(r#"^\markup {{ "{}" }}"#, tala_display);
                    
                    // Attach tala markup to the last note
                    if let Some(last_note) = lilypond_notes.last_mut() {
                        *last_note = format!("{}{}", last_note, tala_text);
                    }
                }
                
                let lily_barline = barline_type_to_lilypond(barline_type, _element_index == 0);
                lilypond_notes.push(lily_barline);
            },
            Item::Breathmark => {
                lilypond_notes.push("\\breathe".to_string());
            },
        }
    }
    
    let staves = lilypond_notes.join(" ");
    
    // Extract lyrics from beat elements
    let mut lyrics_parts: Vec<String> = Vec::new();
    for item in elements.iter() {
        if let Item::Beat(beat) = item {
            for beat_element in &beat.elements {
                if let Some(syllable) = beat_element.syl() {
                    lyrics_parts.push(syllable);
                } else if beat_element.is_note() || beat_element.is_rest() {
                    // Add placeholder for notes/rests without syllables
                    lyrics_parts.push("_".to_string());
                }
            }
        }
    }
    
    // Build template context
    let mut context = TemplateContext::builder()
        .staves(staves);
    
    // Add lyrics if any syllables were found
    if !lyrics_parts.is_empty() && lyrics_parts.iter().any(|s| s != "_") {
        let lyrics_string = lyrics_parts.join(" ");
        context = context.lyrics(lyrics_string);
    }
    
    if let Some(title) = &metadata.title {
        context = context.title(&title.text);
    }
    
    if let Some(source) = source {
        context = context.source_comment(source);
    }
    
    let context = context.build();
    
    // Auto-select template based on document complexity
    let template = crate::converters::lilypond::templates::auto_select_template_for_metadata(metadata);
    
    // Render template
    render_lilypond(template, &context)
        .map_err(|e| format!("Template render error: {}", e))
}

fn convert_beat_to_lilypond(beat: &Beat, current_tonic: Option<Degree>) -> Result<Vec<String>, String> {
    let mut notes = Vec::new();
    
    for beat_element in &beat.elements {
        // Use FSM-calculated tuplet_duration for notation display
        let duration_string = fraction_to_lilypond_note(beat_element.tuplet_duration);
        
        if beat_element.is_note() {
            let (degree, octave, _, _) = beat_element.as_note().unwrap();
            let lily_note = degree_to_lilypond(*degree, octave, current_tonic)?;
            eprintln!("V2 LILYPOND: Note {} with tuplet_duration {} -> {}{}", 
                beat_element.value, beat_element.tuplet_duration, lily_note, duration_string);
            
            let mut note_str = format!("{}{}", lily_note, duration_string);
            
            // Add slur markers based on note's slur attribute
            use crate::models::SlurRole;
            let slur = if let Some((_, _, _, slur_role)) = beat_element.as_note() {
                slur_role
            } else {
                &None
            };
            match slur {
                Some(SlurRole::Start) => note_str.push('('),
                Some(SlurRole::End) => note_str.push(')'),
                Some(SlurRole::StartEnd) => note_str.push_str("()"),
                Some(SlurRole::Middle) => {}, // No marker for middle notes
                None => {},
            }
            
            // Add ornament markers
            use crate::models::OrnamentType;
            for ornament in &beat_element.ornaments() {
                match ornament {
                    OrnamentType::Mordent => note_str.push_str("\\mordent"),
                    OrnamentType::Trill => note_str.push_str("\\trill"),
                    OrnamentType::Turn => note_str.push_str("\\turn"),
                    OrnamentType::Grace => {}, // Grace notes handled differently
                }
            }
            
            notes.push(note_str);
        } else if beat_element.is_rest() {
            notes.push(format!("r{}", duration_string));
        } // Skip other element types within beats
    }
    
    // Add manual beaming for eighth notes and shorter
    add_manual_beaming(&mut notes)?;
    
    // Use FSM-provided tuplet information
    if beat.is_tuplet {
        let (tuplet_num, tuplet_den) = beat.tuplet_ratio.unwrap();
        let tuplet_content = notes.join(" ");
        Ok(vec![format!("\\tuplet {}/{} {{ {} }}", tuplet_num, tuplet_den, tuplet_content)])
    } else {
        Ok(notes)
    }
}

/// Add manual beam brackets to notes in a beat - if beat has more than one note, add [ to first and ] to last
fn add_manual_beaming(notes: &mut Vec<String>) -> Result<(), String> {
    if notes.len() > 1 {
        notes[0].push('[');
        let last_idx = notes.len() - 1;
        notes[last_idx].push(']');
    }
    Ok(())
}

// Removed unused heuristic functions:
// - calculate_tuplet_duration: Used hardcoded duration mappings instead of trusting FSM 
// - calculate_lilypond_duration: Did fractional calculations that FSM already handles
// - convert_document_v2_to_lilypond: Used simple note-count heuristics instead of proper rhythm analysis
//
// The V2 architecture correctly uses FSM-calculated tuplet_duration values directly.


fn degree_to_lilypond(degree: Degree, octave: i8, current_tonic: Option<Degree>) -> Result<String, String> {
    // Transpose the degree and octave based on tonic
    let (transposed_degree, adjusted_octave) = if let Some(tonic) = current_tonic {
        transpose_degree_with_octave(degree, octave, tonic)
    } else {
        (degree, octave)
    };
    
    // Convert transposed Degree to LilyPond note name - handle all variants
    let base_note = match transposed_degree {
        // 1 series (Do/Sa/C)
        Degree::N1bb => "cff",   Degree::N1b => "cf",     Degree::N1 => "c",
        Degree::N1s => "cs",     Degree::N1ss => "css",
        // 2 series (Re/D)  
        Degree::N2bb => "dff",   Degree::N2b => "df",     Degree::N2 => "d",
        Degree::N2s => "ds",     Degree::N2ss => "dss",
        // 3 series (Mi/Ga/E)
        Degree::N3bb => "eff",   Degree::N3b => "ef",     Degree::N3 => "e",
        Degree::N3s => "es",     Degree::N3ss => "ess",
        // 4 series (Fa/Ma/F)  
        Degree::N4bb => "fff",   Degree::N4b => "ff",     Degree::N4 => "f",
        Degree::N4s => "fs",     Degree::N4ss => "fss",
        // 5 series (Sol/Pa/G)
        Degree::N5bb => "gff",   Degree::N5b => "gf",     Degree::N5 => "g",
        Degree::N5s => "gs",     Degree::N5ss => "gss",
        // 6 series (La/Dha/A)
        Degree::N6bb => "aff",   Degree::N6b => "af",     Degree::N6 => "a",
        Degree::N6s => "as",     Degree::N6ss => "ass",
        // 7 series (Ti/Ni/B)
        Degree::N7bb => "bff",   Degree::N7b => "bf",     Degree::N7 => "b",
        Degree::N7s => "bs",     Degree::N7ss => "bss",
    };
    
    // Handle octave modifications (use adjusted_octave from transposition)
    let octave_marks = match adjusted_octave {
        -2 => ",,",
        -1 => ",",
        0 => "",        // Middle octave
        1 => "'",
        2 => "''",
        _ => "",        // Default to middle for extreme octaves
    };
    
    Ok(format!("{}{}", base_note, octave_marks))
}


/// Convert barline type to proper LilyPond syntax
fn barline_type_to_lilypond(barline_type: &crate::models::BarlineType, is_at_beginning: bool) -> String {
    use crate::models::BarlineType;
    match (barline_type, is_at_beginning) {
        (BarlineType::RepeatStart, true) => "\\bar \".|:\"".to_string(),
        (BarlineType::RepeatStart, false) => "\\bar \"|:\"".to_string(),
        (BarlineType::RepeatEnd, _) => "\\bar \":|.\"".to_string(),
        (BarlineType::Double, _) => "\\bar \"||\"".to_string(),
        (BarlineType::Final, _) => "\\bar \"|.\"".to_string(),
        (BarlineType::RepeatBoth, _) => "\\bar \":|:\"".to_string(),
        (BarlineType::Single, _) => "\\bar \"|\"".to_string(),
    }
}

// Transposition functions moved to shared module: src/converters/transposition.rs

/// Convert a Fraction duration to LilyPond note duration string
fn fraction_to_lilypond_note(duration: fraction::Fraction) -> String {
    use crate::models::RhythmConverter;
    
    // Use the existing rhythm converter to get VexFlow durations, then map to LilyPond
    let vexflow_durations = RhythmConverter::fraction_to_vexflow(duration);
    
    if let Some((vexflow_duration, dots)) = vexflow_durations.first() {
        // Convert VexFlow duration to LilyPond duration
        let lily_duration = match vexflow_duration.as_str() {
            "w" => "1",     // whole note
            "h" => "2",     // half note
            "q" => "4",     // quarter note
            "8" => "8",     // eighth note
            "16" => "16",   // sixteenth note
            "32" => "32",   // thirty-second note
            "64" => "64",   // sixty-fourth note
            _ => "4",       // default to quarter note
        };
        
        // Add dots if needed
        let dot_string = ".".repeat(*dots as usize);
        format!("{}{}", lily_duration, dot_string)
    } else {
        // Fallback to quarter note
        "4".to_string()
    }
}