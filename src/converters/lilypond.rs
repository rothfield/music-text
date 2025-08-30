// LilyPond Source Code Generator - Works directly with ParsedElement, no conversion needed
use crate::models::{Metadata}; // Keep using existing metadata
use crate::pitch::{Degree};
use crate::lilypond_templates::{TemplateContext, render_lilypond};
use crate::rhythm_fsm::{Item, Beat};
use super::transposition::transpose_degree_with_octave;

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
    let mut pending_slur_start = false;
    let mut pending_slur_end = false;
    
    for (element_index, item) in elements.iter().enumerate() {
        match item {
            Item::Tonic(tonic_degree) => {
                // Store the tonic for transposition
                current_tonic = Some(*tonic_degree);
                // Could optionally add a key signature command here
            },
            Item::Beat(beat) => {
                let mut beat_notes = convert_beat_to_lilypond(beat, current_tonic)?;
                
                // Apply pending slur end to the last note of this beat
                if pending_slur_end && !beat_notes.is_empty() {
                    if beat.is_tuplet {
                        // For tuplets, insert slur after the last note inside the braces
                        beat_notes[0] = add_slur_end_to_tuplet(&beat_notes[0]);
                    } else {
                        beat_notes.last_mut().unwrap().push(')');
                    }
                    pending_slur_end = false;
                }
                
                // Apply pending slur start to the first note of this beat
                if pending_slur_start && !beat_notes.is_empty() {
                    if beat.is_tuplet {
                        // For tuplets, insert slur after the first actual note inside the braces
                        beat_notes[0] = add_slur_start_to_tuplet(&beat_notes[0]);
                    } else {
                        beat_notes[0].push('(');
                    }
                    pending_slur_start = false;
                }
                
                // Handle ties: if this beat is tied to previous, add tie to last note of previous beat
                if beat.tied_to_previous && !previous_beat_notes.is_empty() && !beat_notes.is_empty() {
                    // ✅ SAFE: Find last actual note, not just last item
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
            Item::Barline(style) => {
                lilypond_notes.push(format!("\\bar \"{}\"", style));
            },
            Item::Breathmark => {
                lilypond_notes.push("\\breathe".to_string());
            },
            Item::SlurStart => {
                pending_slur_start = true;
            },
            Item::SlurEnd => {
                // Check the context: what was the last processed item and what's next?
                let has_next_beat = elements.iter().skip(element_index + 1).any(|item| matches!(item, Item::Beat(_)));
                let last_was_tuplet = if let Some(last_note_index) = find_last_note_index(&lilypond_notes) {
                    lilypond_notes[last_note_index].contains("\\tuplet")
                } else { 
                    false 
                };
                
                if has_next_beat && last_was_tuplet {
                    // SlurEnd after tuplet with next beat - likely cross-beat slur, defer to next beat
                    pending_slur_end = true;
                } else {
                    // SlurEnd after regular beat or at end - apply to last note immediately  
                    if !lilypond_notes.is_empty() {
                        if let Some(last_note_index) = find_last_note_index(&lilypond_notes) {
                            let last_note = &lilypond_notes[last_note_index];
                            if last_note.contains("\\tuplet") {
                                lilypond_notes[last_note_index] = add_slur_end_to_tuplet(last_note);
                            } else {
                                lilypond_notes[last_note_index].push(')');
                            }
                        }
                    }
                }
            },
        }
    }
    
    // Handle any remaining pending slur end at the end of the song
    if pending_slur_end && !lilypond_notes.is_empty() {
        if let Some(last_note_index) = find_last_note_index(&lilypond_notes) {
            let last_note = &lilypond_notes[last_note_index];
            if last_note.contains("\\tuplet") {
                lilypond_notes[last_note_index] = add_slur_end_to_tuplet(last_note);
            } else {
                lilypond_notes[last_note_index].push(')');
            }
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

fn convert_beat_to_lilypond(beat: &Beat, current_tonic: Option<Degree>) -> Result<Vec<String>, String> {
    let mut notes = Vec::new();
    
    for beat_element in &beat.elements {
        // Use FSM-calculated tuplet_duration for notation display
        let duration_string = fraction_to_lilypond_note(beat_element.tuplet_duration);
        
        if beat_element.is_note() {
            let lily_note = degree_to_lilypond(beat_element.degree.unwrap(), beat_element.octave.unwrap(), current_tonic)?;
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

/// Add slur start marker to the first note inside a tuplet
/// \tuplet 3/2 { c4 d8 } -> \tuplet 3/2 { c4( d8 }
fn add_slur_start_to_tuplet(tuplet_str: &str) -> String {
    // Find the first note after the opening brace
    if let Some(brace_pos) = tuplet_str.find("{ ") {
        let before_brace = &tuplet_str[..brace_pos + 2]; // Include "{ "
        let after_brace = &tuplet_str[brace_pos + 2..];
        
        // Find the end of the first note (look for space or closing brace)
        if let Some(first_note_end) = after_brace.find(|c: char| c == ' ' || c == '}') {
            let first_note = &after_brace[..first_note_end];
            let rest = &after_brace[first_note_end..];
            return format!("{}{}({}", before_brace, first_note, rest);
        }
    }
    // Fallback - just append to the end if parsing fails
    format!("{}(", tuplet_str)
}

/// Add slur end marker to the last note inside a tuplet
/// \tuplet 3/2 { c4 d8 } -> \tuplet 3/2 { c4 d8) }
fn add_slur_end_to_tuplet(tuplet_str: &str) -> String {
    // Find the closing brace
    if let Some(brace_pos) = tuplet_str.rfind(" }") {
        let before_brace = &tuplet_str[..brace_pos];
        let after_brace = &tuplet_str[brace_pos..]; // " }"
        
        // Find the last note before the closing brace
        // Work backwards from the brace position to find the last note
        let content_before_brace = before_brace.trim_end();
        if let Some(last_space) = content_before_brace.rfind(' ') {
            let before_last_note = &content_before_brace[..last_space + 1];
            let last_note = &content_before_brace[last_space + 1..];
            return format!("{}{}){}", before_last_note, last_note, after_brace);
        } else {
            // Only one note in tuplet
            return format!("{}){}", before_brace, after_brace);
        }
    }
    // Fallback - just append to the end if parsing fails  
    format!("{})", tuplet_str)
}

// Transposition functions moved to shared module: src/converters/transposition.rs

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