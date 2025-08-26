// V2 LilyPond Converter - Works directly with ParsedElement, no conversion needed
use crate::models_v2::{DocumentV2, ParsedElement, ParsedChild};
use crate::models::{Metadata}; // Keep using existing metadata
use crate::pitch::{PitchCode, LilyPondNoteNames};
use crate::lilypond_templates::{TemplateContext, render_lilypond};

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
            ParsedElement::Note { pitch_code, octave, value, position: _, children: _ } => {
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
            
            ParsedElement::Rest { value: _, position: _ } => {
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