// V2 LilyPond Converter - Works directly with ParsedElement, no conversion needed
use crate::models::{Metadata}; // Keep using existing metadata
use crate::pitch::{Degree, LilyPondNoteNames};
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
    
    for (_beat_index, item) in fsm_output.iter().enumerate() {
        match item {
            OutputItemV2::Beat(beat) => {
                let beat_notes = convert_beat_to_lilypond(beat, &note_names)?;
                
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
            let lily_note = degree_to_lilypond(beat_element.degree.unwrap(), beat_element.octave.unwrap(), note_names)?;
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

// Removed unused heuristic functions:
// - calculate_tuplet_duration: Used hardcoded duration mappings instead of trusting FSM 
// - calculate_lilypond_duration: Did fractional calculations that FSM already handles
// - convert_document_v2_to_lilypond: Used simple note-count heuristics instead of proper rhythm analysis
//
// The V2 architecture correctly uses FSM-calculated tuplet_duration values directly.


fn degree_to_lilypond(degree: Degree, octave: i8, _note_names: &LilyPondNoteNames) -> Result<String, String> {
    // Convert Degree to LilyPond note name - handle all variants
    let base_note = match degree {
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