// V2 LilyPond Converter - Works directly with ParsedElement, no conversion needed
use crate::models_v2::{DocumentV2, ParsedElement, ParsedChild};
use crate::models::{Metadata}; // Keep using existing metadata
use crate::pitch::{PitchCode, LilyPondNoteNames};
use crate::lilypond_templates::{TemplateContext, render_lilypond};
use crate::rhythm_fsm_v2::{OutputItemV2, BeatV2};

pub fn convert_fsm_output_to_lilypond(
    fsm_output: &Vec<OutputItemV2>,
    metadata: &Metadata,
    note_names: LilyPondNoteNames,
    source: Option<&str>
) -> Result<String, String> {
    eprintln!("V2 LILYPOND CONVERTER: Processing FSM output with beats");
    
    let mut lilypond_notes: Vec<String> = Vec::new();
    let mut previous_beat_notes: Vec<String> = Vec::new();
    
    for (beat_index, item) in fsm_output.iter().enumerate() {
        match item {
            OutputItemV2::Beat(beat) => {
                let mut beat_notes = convert_beat_to_lilypond(beat, &note_names)?;
                
                // Handle ties: if this beat is tied to previous, add tie to last note of previous beat
                if beat.tied_to_previous && !previous_beat_notes.is_empty() && !beat_notes.is_empty() {
                    // Add tie marker to the last note of the previous beat
                    if let Some(last_note) = lilypond_notes.last_mut() {
                        if !last_note.ends_with('~') {
                            *last_note = format!("{}~", last_note);
                        }
                    }
                }
                
                lilypond_notes.extend(beat_notes.clone());
                previous_beat_notes = beat_notes;
            },
            OutputItemV2::Barline(style) => {
                lilypond_notes.push(format!("\\bar \"{}\"", style));
            },
            OutputItemV2::Breathmark => {
                lilypond_notes.push("\\breathe".to_string());
            },
            OutputItemV2::SlurStart => {
                // Handle slur start if needed
            },
            OutputItemV2::SlurEnd => {
                // Handle slur end if needed
            },
        }
    }
    
    let staves = lilypond_notes.join(" ");
    
    // Build template context
    let mut context = TemplateContext::builder()
        .staves(staves);
    
    if let Some(title) = &metadata.title {
        context = context.title(&title.text);
    }
    
    if let Some(source) = source {
        context = context.source_comment(source);
    }
    
    let context = context.build();
    
    // Auto-select template based on document complexity
    let template = crate::lilypond_templates::auto_select_template_for_metadata(metadata);
    
    // Render template
    render_lilypond(template, &context)
        .map_err(|e| format!("Template render error: {}", e))
}

fn convert_beat_to_lilypond(beat: &BeatV2, note_names: &LilyPondNoteNames) -> Result<Vec<String>, String> {
    let mut notes = Vec::new();
    
    for beat_element in &beat.elements {
        // Use FSM-calculated tuplet_duration for notation display
        let duration_string = fraction_to_lilypond_note(beat_element.tuplet_duration);
        
        if beat_element.is_note() {
            let lily_note = pitch_code_to_lilypond(beat_element.pitch_code.unwrap(), beat_element.octave.unwrap(), note_names)?;
            eprintln!("V2 LILYPOND: Note {} with tuplet_duration {} -> {}{}", 
                beat_element.value, beat_element.tuplet_duration, lily_note, duration_string);
            
            notes.push(format!("{}{}", lily_note, duration_string));
        } else if beat_element.is_rest() {
            notes.push(format!("r{}", duration_string));
        } // Skip other element types within beats
    }
    
    // Use FSM-provided tuplet information
    if beat.is_tuplet {
        let (tuplet_num, tuplet_den) = beat.tuplet_ratio.unwrap();
        let tuplet_content = notes.join(" ");
        Ok(vec![format!("\\tuplet {}/{} {{ {} }}", tuplet_num, tuplet_den, tuplet_content)])
    } else {
        Ok(notes)
    }
}

fn calculate_tuplet_duration(subdivisions: usize, divisions: usize) -> String {
    use fraction::Fraction;
    use crate::rhythm::RhythmConverter;
    
    // For tuplets, choose duration based on tuplet size, not just subdivisions
    // Larger tuplets need shorter base durations
    let base_duration = if divisions <= 3 {
        // Small tuplets (3-tuplet) use eighth notes
        match subdivisions {
            1 => Fraction::new(1u64, 8u64),   // Eighth note
            2 => Fraction::new(1u64, 4u64),   // Quarter note
            3 => Fraction::new(3u64, 8u64),   // Dotted quarter
            4 => Fraction::new(1u64, 2u64),   // Half note
            _ => Fraction::new(1u64, 8u64),
        }
    } else if divisions <= 7 {
        // Medium tuplets (4-7 notes) use sixteenth notes  
        match subdivisions {
            1 => Fraction::new(1u64, 16u64),  // Sixteenth note
            2 => Fraction::new(1u64, 8u64),   // Eighth note
            3 => Fraction::new(3u64, 16u64),  // Dotted sixteenth
            4 => Fraction::new(1u64, 4u64),   // Quarter note
            _ => Fraction::new(1u64, 16u64),
        }
    } else if divisions <= 15 {
        // Large tuplets (8-15 notes) use thirty-second notes
        match subdivisions {
            1 => Fraction::new(1u64, 32u64),  // Thirty-second note
            2 => Fraction::new(1u64, 16u64),  // Sixteenth note  
            3 => Fraction::new(3u64, 32u64),  // Dotted thirty-second
            4 => Fraction::new(1u64, 8u64),   // Eighth note
            _ => Fraction::new(1u64, 32u64),
        }
    } else {
        // Very large tuplets (16+ notes) use sixty-fourth notes
        match subdivisions {
            1 => Fraction::new(1u64, 64u64),  // Sixty-fourth note
            2 => Fraction::new(1u64, 32u64),  // Thirty-second note
            3 => Fraction::new(3u64, 64u64),  // Dotted sixty-fourth
            4 => Fraction::new(1u64, 16u64),  // Sixteenth note
            _ => Fraction::new(1u64, 64u64),
        }
    };
    
    // Convert fraction to LilyPond duration using existing converter
    let durations = RhythmConverter::fraction_to_lilypond(base_duration);
    
    // For tuplets, take the first duration (no ties needed within tuplet)
    durations.into_iter()
        .filter(|d| d != "~")
        .next()
        .unwrap_or_else(|| "64".to_string())
}

fn calculate_lilypond_duration(subdivisions: usize, divisions: usize) -> String {
    use fraction::Fraction;
    use crate::rhythm::RhythmConverter;
    
    // Calculate the fraction of the beat this note gets  
    let note_fraction = Fraction::new(subdivisions as u64, divisions as u64);
    
    // Each beat is a quarter note (1/4), so the actual duration is:
    // note_duration = note_fraction * beat_duration = note_fraction * 1/4
    let quarter_beat = Fraction::new(1u64, 4u64);
    let actual_duration = note_fraction * quarter_beat;
    
    eprintln!("DURATION DEBUG: {}/{} of beat -> {} * 1/4 = {}", 
        subdivisions, divisions, note_fraction, actual_duration);
    
    // Use the existing robust fraction-to-lilypond converter
    let durations = RhythmConverter::fraction_to_lilypond(actual_duration);
    
    eprintln!("DURATION DEBUG: {} -> {:?}", actual_duration, durations);
    
    // For now, just take the first duration (ignoring ties)
    // TODO: Handle complex tied durations properly
    durations.into_iter()
        .filter(|d| d != "~")  // Skip tie markers
        .next()
        .unwrap_or_else(|| "8".to_string()) // Fallback
}

// Keep the original function for compatibility but have it use FSM output properly
pub fn convert_document_v2_to_lilypond(
    document: &DocumentV2, 
    note_names: LilyPondNoteNames, 
    source: Option<&str>
) -> Result<String, String> {
    eprintln!("V2 LILYPOND CONVERTER: Processing {} elements", document.elements.len());
    
    // Convert elements to LilyPond notation
    let mut lilypond_notes = Vec::new();
    
    // Need to process elements in beats to calculate proper durations
    // For now, implement simple rhythm calculation
    // TODO: Use proper FSM beat information for accurate durations
    
    let total_notes = document.elements.iter()
        .filter(|e| matches!(e, ParsedElement::Note { .. }))
        .count();
    
    for element in &document.elements {
        match element {
            ParsedElement::Note { pitch_code, octave, value, position: _, children: _, .. } => {
                let lily_note = pitch_code_to_lilypond(*pitch_code, *octave, &note_names)?;
                
                // Simple rhythm calculation: if multiple notes, make them shorter
                let duration = match total_notes {
                    1 => "4",  // Single note = quarter note
                    2 => "8",  // Two notes = eighth notes
                    3 => "8",  // Three notes = eighth notes (for now)
                    4 => "16", // Four notes = sixteenth notes
                    _ => "16", // More notes = sixteenth notes
                };
                
                eprintln!("V2 LILYPOND: Note {} octave {} -> {} duration {}", value, octave, lily_note, duration);
                lilypond_notes.push(format!("{}{}", lily_note, duration));
            },
            
            ParsedElement::Rest { value: _, position: _, .. } => {
                lilypond_notes.push("r4".to_string()); // Quarter rest
            },
            
            ParsedElement::Barline { style, position: _ } => {
                lilypond_notes.push(format!("\\bar \"{}\"", style));
            },
            
            // Skip other elements for now
            ParsedElement::Dash { .. } |
            ParsedElement::SlurStart { .. } |
            ParsedElement::SlurEnd { .. } |
            ParsedElement::Whitespace { .. } |
            ParsedElement::Newline { .. } |
            ParsedElement::Word { .. } |
            ParsedElement::Symbol { .. } |
            ParsedElement::Unknown { .. } => {
                // Skip these elements
            }
        }
    }
    
    let staves = lilypond_notes.join(" ");
    
    // Auto-select template based on document complexity
    let template = crate::lilypond_templates::auto_select_template_v2(document);
    
    // Build template context
    let mut context = TemplateContext::builder()
        .staves(staves);
    
    if let Some(title) = &document.metadata.title {
        context = context.title(&title.text);
    }
    
    if let Some(source) = source {
        context = context.source_comment(source);
    }
    
    let context = context.build();
    
    // Render template
    render_lilypond(template, &context)
        .map_err(|e| format!("Template render error: {}", e))
}

fn pitch_code_to_lilypond(pitch_code: PitchCode, octave: i8, _note_names: &LilyPondNoteNames) -> Result<String, String> {
    // Convert PitchCode to LilyPond note name - handle all variants
    let base_note = match pitch_code {
        // 1 series (Do/Sa/C)
        PitchCode::N1bb => "cff",   PitchCode::N1b => "cf",     PitchCode::N1 => "c",
        PitchCode::N1s => "cs",     PitchCode::N1ss => "css",
        // 2 series (Re/D)  
        PitchCode::N2bb => "dff",   PitchCode::N2b => "df",     PitchCode::N2 => "d",
        PitchCode::N2s => "ds",     PitchCode::N2ss => "dss",
        // 3 series (Mi/Ga/E)
        PitchCode::N3bb => "eff",   PitchCode::N3b => "ef",     PitchCode::N3 => "e",
        PitchCode::N3s => "es",     PitchCode::N3ss => "ess",
        // 4 series (Fa/Ma/F)  
        PitchCode::N4bb => "fff",   PitchCode::N4b => "ff",     PitchCode::N4 => "f",
        PitchCode::N4s => "fs",     PitchCode::N4ss => "fss",
        // 5 series (Sol/Pa/G)
        PitchCode::N5bb => "gff",   PitchCode::N5b => "gf",     PitchCode::N5 => "g",
        PitchCode::N5s => "gs",     PitchCode::N5ss => "gss",
        // 6 series (La/Dha/A)
        PitchCode::N6bb => "aff",   PitchCode::N6b => "af",     PitchCode::N6 => "a",
        PitchCode::N6s => "as",     PitchCode::N6ss => "ass",
        // 7 series (Ti/Ni/B)
        PitchCode::N7bb => "bff",   PitchCode::N7b => "bf",     PitchCode::N7 => "b",
        PitchCode::N7s => "bs",     PitchCode::N7ss => "bss",
    };
    
    // Handle octave modifications
    let octave_marks = match octave {
        -2 => ",,",
        -1 => ",",
        0 => "",        // Middle octave
        1 => "'",
        2 => "''",
        _ => "",        // Default to middle for extreme octaves
    };
    
    Ok(format!("{}{}", base_note, octave_marks))
}

/// Convert a Fraction duration to LilyPond note duration string
fn fraction_to_lilypond_note(duration: fraction::Fraction) -> String {
    use crate::rhythm::RhythmConverter;
    
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