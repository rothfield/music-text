use crate::models::{Document, Node};
use crate::pitch::LilyPondNoteNames;
use crate::pitch::{PitchCode, pitchcode_to_dutch_lilypond, pitchcode_to_english_lilypond};
use crate::lilypond_templates::{LilyPondTemplate, TemplateContext, render_lilypond, auto_select_template};
use fraction::Fraction;
use std::str::FromStr;
use std::collections::HashMap;

fn transpose_snippet(key: Option<&String>) -> String {
    match key {
        None => String::new(),
        Some(k) if k.to_lowercase() == "c" => String::new(),
        Some(key_str) => {
            // Map all common keys to LilyPond pitch names (20+ mappings for comprehensive coverage)
            let pitch_map: HashMap<&str, &str> = [
                // Natural keys
                ("C", "c"), ("D", "d"), ("E", "e"), ("F", "f"), ("G", "g"), ("A", "a"), ("B", "b"),
                // Sharp keys
                ("C#", "cs"), ("D#", "ds"), ("F#", "fs"), ("G#", "gs"), ("A#", "as"),
                // Flat keys  
                ("Db", "df"), ("Eb", "ef"), ("Gb", "gf"), ("Ab", "af"), ("Bb", "bf"),
                // Less common enharmonics
                ("Cb", "cf"), ("E#", "es"), ("Fb", "ff"), ("B#", "bs"),
                // Lowercase variants (user convenience)
                ("c", "c"), ("d", "d"), ("e", "e"), ("f", "f"), ("g", "g"), ("a", "a"), ("b", "b"),
                ("c#", "cs"), ("d#", "ds"), ("f#", "fs"), ("g#", "gs"), ("a#", "as"),
                ("db", "df"), ("eb", "ef"), ("gb", "gf"), ("ab", "af"), ("bb", "bf")
            ].iter().cloned().collect();
            
            if let Some(&lily_pitch) = pitch_map.get(key_str.as_str()) {
                format!("\\transpose c' {}'", lily_pitch)
            } else {
                String::new()
            }
        }
    }
}

fn key_signature_snippet(key: Option<&String>) -> String {
    match key {
        None => String::new(),
        Some(k) if k.to_lowercase() == "c" => "\\key c \\major".to_string(),
        Some(_) => {
            // When there's a key specified, use C major as the base key
            // The transpose directive will handle the actual transposition
            "\\key c \\major".to_string()
        }
    }
}

// Helper function to parse more complex key signature notations
fn parse_complex_key_signature(key_str: &str) -> Option<String> {
    let key_lower = key_str.to_lowercase();
    
    // Handle patterns like "key of C major", "C maj", "A min", etc.
    if key_lower.contains("key of") {
        let parts: Vec<&str> = key_lower.split_whitespace().collect();
        if parts.len() >= 4 {
            return parse_complex_key_signature(&format!("{} {}", parts[2], parts[3]));
        }
    }
    
    // Handle abbreviated forms
    if key_lower.ends_with(" maj") || key_lower.ends_with(" major") {
        let note_part = key_lower.replace(" maj", "").replace(" major", "");
        return Some(format!("{} \\major", convert_note_to_lilypond(&note_part)));
    }
    
    if key_lower.ends_with(" min") || key_lower.ends_with(" minor") {
        let note_part = key_lower.replace(" min", "").replace(" minor", "");
        return Some(format!("{} \\minor", convert_note_to_lilypond(&note_part)));
    }
    
    // Handle single letter with accidentals
    if key_str.len() <= 3 {
        // Try to interpret as major key
        return Some(format!("{} \\major", convert_note_to_lilypond(key_str)));
    }
    
    None
}

// Helper function to convert note names to LilyPond format
fn convert_note_to_lilypond(note: &str) -> String {
    let note_map: HashMap<&str, &str> = [
        ("c", "c"), ("c#", "cs"), ("db", "df"), ("d", "d"), ("d#", "ds"), ("eb", "ef"),
        ("e", "e"), ("f", "f"), ("f#", "fs"), ("gb", "gf"), ("g", "g"), ("g#", "gs"),
        ("ab", "af"), ("a", "a"), ("a#", "as"), ("bb", "bf"), ("b", "b"),
        // Handle some uppercase variants
        ("C", "c"), ("C#", "cs"), ("Db", "df"), ("D", "d"), ("D#", "ds"), ("Eb", "ef"),
        ("E", "e"), ("F", "f"), ("F#", "fs"), ("Gb", "gf"), ("G", "g"), ("G#", "gs"),
        ("Ab", "af"), ("A", "a"), ("A#", "as"), ("Bb", "bf"), ("B", "b"),
    ].iter().cloned().collect();
    
    note_map.get(note).unwrap_or(&note).to_string()
}

fn fraction_to_lilypond_proper(frac: Fraction) -> Vec<String> {
    // Create a lookup table for common fractions to LilyPond durations
    let lookup = [
        (Fraction::new(1u64, 1u64), vec!["1".to_string()]),
        (Fraction::new(1u64, 2u64), vec!["2".to_string()]),
        (Fraction::new(1u64, 4u64), vec!["4".to_string()]),
        (Fraction::new(1u64, 8u64), vec!["8".to_string()]),
        (Fraction::new(1u64, 16u64), vec!["16".to_string()]),
        (Fraction::new(1u64, 32u64), vec!["32".to_string()]),
        (Fraction::new(3u64, 8u64), vec!["4.".to_string()]),
        (Fraction::new(3u64, 16u64), vec!["8.".to_string()]),
        (Fraction::new(3u64, 32u64), vec!["16.".to_string()]),
    ];
    
    // Check for direct match first
    for (lookup_frac, durations) in &lookup {
        if frac == *lookup_frac {
            return durations.clone();
        }
    }
    
    // Use shared rhythm decomposition logic
    let fraction_parts = crate::rhythm::RhythmConverter::decompose_fraction_to_standard_durations(frac);
    
    // Convert each fraction to LilyPond duration string
    fraction_parts.iter().map(|f| {
        match *f {
            f if f == Fraction::new(1u64, 1u64) => "1".to_string(),
            f if f == Fraction::new(1u64, 2u64) => "2".to_string(),
            f if f == Fraction::new(1u64, 4u64) => "4".to_string(),
            f if f == Fraction::new(1u64, 8u64) => "8".to_string(),
            f if f == Fraction::new(1u64, 16u64) => "16".to_string(),
            f if f == Fraction::new(1u64, 32u64) => "32".to_string(),
            _ => "32".to_string(), // Fallback
        }
    }).collect()
}

fn pitchcode_to_lilypond(pitch_code: PitchCode, octave: Option<i8>, note_names: LilyPondNoteNames) -> String {
    let base_note = match note_names {
        LilyPondNoteNames::Dutch => pitchcode_to_dutch_lilypond(pitch_code),
        LilyPondNoteNames::English => pitchcode_to_english_lilypond(pitch_code),
    };
    
    // Convert octave to LilyPond notation (shifted down one octave)
    // 0 = middle octave (c) - one octave lower than before
    // 1 = upper octave (c')
    // -1 = lower octave (c,)
    // -2 = lowest octave (c,,)
    let octave_suffix = match octave.unwrap_or(0) {
        -2 => ",,,",   // triple comma for lowest octave
        -1 => ",,",    // double comma for lower octave
        0 => "",       // no suffix for middle octave (middle C, one octave lower)
        1 => "'",      // single quote for upper octave
        2 => "''",     // double quote for highest octave
        _ => "",       // fallback to middle octave
    };
    
    format!("{}{}", base_note, octave_suffix)
}

fn add_slur_markers_to_line(mut line_notes: Vec<String>, elements: &[Node]) -> Vec<String> {
    // Collect all slur events with their associated note positions
    let mut slur_events = Vec::new();
    let mut current_note_idx = 0;
    
    for element in elements {
        match element.node_type.as_str() {
            "BEAT" => {
                // Check if this beat contains slur start/end
                let has_slur_start = element.nodes.iter().any(|n| n.node_type == "SLUR_START");
                let has_slur_end = element.nodes.iter().any(|n| n.node_type == "SLUR_END");
                
                // Count pitches in this beat
                let pitch_count = element.nodes.iter().filter(|n| n.node_type == "PITCH").count();
                
                if pitch_count > 0 {
                    // Record slur start on first note of this beat
                    if has_slur_start {
                        slur_events.push((current_note_idx, "start"));
                    }
                    
                    // Record slur end on last note of this beat
                    if has_slur_end {
                        slur_events.push((current_note_idx + pitch_count - 1, "end"));
                    }
                    
                    current_note_idx += pitch_count;
                }
            }
            "SLUR_START" => {
                // Standalone slur start - apply to next note
                slur_events.push((current_note_idx, "start"));
            }
            "SLUR_END" => {
                // Standalone slur end - apply to previous note (current_note_idx - 1)
                if current_note_idx > 0 {
                    slur_events.push((current_note_idx - 1, "end"));
                }
            }
            _ => {}
        }
    }
    
    // Now apply slur markers to the appropriate notes
    // We need to track which musical note we're at, accounting for tuplets
    let mut musical_note_idx = 0;
    
    for (line_idx, note) in line_notes.iter_mut().enumerate() {
        // Skip non-musical elements
        if note.starts_with("\\bar") || note.starts_with("\\breathe") {
            continue;
        }
        
        if note.starts_with("\\tuplet") {
            // Handle tuplet: extract and count notes
            let tuplet_notes_count = if let Some(start) = note.find('{') {
                if let Some(end) = note.rfind('}') {
                    let content = &note[start+1..end];
                    let tuplet_notes: Vec<&str> = content.split_whitespace()
                        .filter(|s| !s.is_empty() && !s.contains('~'))
                        .collect();
                    tuplet_notes.len()
                } else {
                    0
                }
            } else {
                0
            };
            
            // Apply slur markers to notes within this tuplet
            for (i, slur_event) in slur_events.iter() {
                if *i >= musical_note_idx && *i < musical_note_idx + tuplet_notes_count {
                    let tuplet_note_position = *i - musical_note_idx;
                    let marker = if *slur_event == "start" { "(" } else { ")" };
                    let updated_note = add_slur_to_tuplet_note(note, tuplet_note_position, marker);
                    *note = updated_note;
                }
            }
            
            musical_note_idx += tuplet_notes_count;
        } else {
            // Handle single note
            for (i, slur_event) in slur_events.iter() {
                if *i == musical_note_idx {
                    let marker = if *slur_event == "start" { "(" } else { ")" };
                    *note = format!("{}{}", note, marker);
                }
            }
            musical_note_idx += 1;
        }
    }
    
    line_notes
}

// Helper function to add slur markers to specific notes within a tuplet
fn add_slur_to_tuplet_note(tuplet_str: &str, note_index: usize, marker: &str) -> String {
    if let Some(start) = tuplet_str.find('{') {
        if let Some(end) = tuplet_str.rfind('}') {
            let prefix = &tuplet_str[..start+1];
            let content = &tuplet_str[start+1..end];
            let suffix = &tuplet_str[end..];
            
            // Split content into notes, filtering out ties
            let mut notes: Vec<String> = content.split_whitespace()
                .filter(|s| !s.is_empty() && !s.contains('~'))
                .map(|s| s.to_string())
                .collect();
            
            // Add marker to the specified note
            if note_index < notes.len() {
                notes[note_index] = format!("{}{}", notes[note_index], marker);
            }
            
            // Reconstruct the tuplet
            format!("{} {} {}", prefix, notes.join(" "), suffix)
        } else {
            tuplet_str.to_string()
        }
    } else {
        tuplet_str.to_string()
    }
}

fn convert_fsm_document_to_lilypond(document: &Document, note_names: LilyPondNoteNames) -> Result<(String, String), String> {
    let mut all_notes = Vec::new();
    let mut all_syllables = Vec::new();
    let mut previous_pitch: Option<(PitchCode, Option<i8>)> = None;
    
    // Process each line
    for line_node in &document.nodes {
        if line_node.node_type == "LINE" {
            let mut line_notes: Vec<String> = Vec::new();
            let mut line_syllables: Vec<String> = Vec::new();
            
            // Process each element in the line
            for element in &line_node.nodes {
                match element.node_type.as_str() {
                    "BEAT" => {
                        // Check if this beat should be a tuplet
                        let total_subdivisions = element.divisions;
                        let is_tuplet = total_subdivisions > 1 && (total_subdivisions & (total_subdivisions - 1)) != 0;
                        
                        let mut beat_notes: Vec<String> = Vec::new();
                        
                        // Process notes in the beat
                        for note_node in &element.nodes {
                            if note_node.node_type == "PITCH" {
                                // Extract duration from the value (e.g., "S[1/1]" -> "1/1")
                                let duration_str = if let Some(start) = note_node.value.find('[') {
                                    if let Some(end) = note_node.value.find(']') {
                                        &note_node.value[start+1..end]
                                    } else {
                                        "1/1"
                                    }
                                } else {
                                    "1/1"
                                };
                                
                                // Convert fraction to LilyPond duration
                                let lily_durations = if let Ok(frac) = fraction::Fraction::from_str(duration_str) {
                                    fraction_to_lilypond_proper(frac)
                                } else {
                                    vec!["4".to_string()]
                                };
                                
                                // Convert pitch code to note
                                if let Some(pitch_code) = note_node.pitch_code {
                                    let pitch_str = pitchcode_to_lilypond(pitch_code, note_node.octave, note_names);
                                    
                                    // Check if this note should be tied to the previous one
                                    // Only tie notes of the same pitch within the same beat or when explicitly marked for tie
                                    // Do not automatically tie across different beats/tuplets
                                    let should_tie = false; // Disable automatic tying for now - this was causing incorrect ties between tuplets
                                    
                                    // Add each duration (for tied notes within this note)
                                    for (i, duration) in lily_durations.iter().enumerate() {
                                        let note_str = format!("{}{}", pitch_str, duration);
                                        
                                        // Add tie symbol after all but the last note
                                        if i < lily_durations.len() - 1 {
                                            beat_notes.push(format!("{}~", note_str));
                                            // For tied notes within same pitch, use "_" in lyrics
                                            if i == 0 {
                                                // First note gets the syllable or "_"
                                                line_syllables.push(note_node.syl.as_deref().unwrap_or("_").to_string());
                                            } else {
                                                // Continuation notes get "_"
                                                line_syllables.push("_".to_string());
                                            }
                                        } else {
                                            beat_notes.push(note_str);
                                            // Last note of this pitch
                                            if lily_durations.len() == 1 {
                                                // Single duration, use the syllable or "_"
                                                line_syllables.push(note_node.syl.as_deref().unwrap_or("_").to_string());
                                            } else {
                                                // Multiple durations, this is the last one
                                                line_syllables.push("_".to_string());
                                            }
                                        }
                                    }
                                    
                                    // Update previous pitch for next iteration
                                    previous_pitch = Some((pitch_code, note_node.octave));
                                }
                            }
                        }
                        
                        // Wrap in tuplet if needed
                        if is_tuplet && beat_notes.len() > 1 {
                            // Calculate tuplet ratio (e.g., 5/4 for quintuplet)
                            let tuplet_num = total_subdivisions;
                            let tuplet_den = match total_subdivisions {
                                3 => 2,   // Triplet: 3 notes in place of 2
                                5 => 4,   // Quintuplet: 5 notes in place of 4
                                6 => 4,   // Sextuplet: 6 notes in place of 4  
                                7 => 4,   // Septuplet: 7 notes in place of 4
                                9 => 8,   // 9-tuplet: 9 notes in place of 8
                                10 => 8,  // 10-tuplet: 10 notes in place of 8
                                11 => 8,  // 11-tuplet: 11 notes in place of 8
                                12 => 8,  // 12-tuplet: 12 notes in place of 8
                                _ => (total_subdivisions / 2).max(2), // Fallback
                            };
                            
                            let tuplet_content = beat_notes.join(" ");
                            line_notes.push(format!("\\tuplet {}/{} {{ {} }}", tuplet_num, tuplet_den, tuplet_content));
                        } else {
                            // Add notes directly
                            if !beat_notes.is_empty() {
                                line_notes.extend(beat_notes);
                            }
                        }
                    }
                    "BARLINE" => {
                        // Convert barline types to LilyPond format
                        let barline_str = match element.value.as_str() {
                            "|" => "\\bar \"|\"",
                            "||" => "\\bar \"||\"",
                            ":|" => "\\bar \":|.\"",
                            "|:" => "\\bar \".|:\"",
                            ":|:" => "\\bar \":|.:\"",
                            _ => "\\bar \"|\"",  // Default to simple barline
                        };
                        line_notes.push(barline_str.to_string());
                        // Reset tie tracking at barlines
                        previous_pitch = None;
                    }
                    "BREATHMARK" => {
                        line_notes.push("\\breathe".to_string());
                    }
                    _ => {}
                }
            }
            
            // Apply slur markers to the line notes based on SLUR_START/SLUR_END elements
            if !line_notes.is_empty() {
                let processed_notes = add_slur_markers_to_line(line_notes, &line_node.nodes);
                all_notes.extend(processed_notes);
                all_syllables.extend(line_syllables);
            }
        }
    }
    
    // If no notes found, return a rest
    if all_notes.is_empty() {
        Ok(("r1".to_string(), "".to_string()))
    } else {
        let notes_str = all_notes.join(" ");
        let lyrics_str = if all_syllables.iter().any(|s| s != "_") {
            all_syllables.join(" ")
        } else {
            "".to_string() // No real syllables, only underscores
        };
        Ok((notes_str, lyrics_str))
    }
}

pub fn convert_to_lilypond(document: &Document, note_names: LilyPondNoteNames, source_notation: Option<&str>) -> Result<String, String> {
    // Convert the FSM-structured document to LilyPond notation
    let (rhythmic_notation, lyrics) = convert_fsm_document_to_lilypond(document, note_names)?;
    
    // Build template context
    let mut context_builder = TemplateContext::builder()
        .staves(rhythmic_notation);
        
    // Add title if present
    if let Some(title) = &document.metadata.title {
        context_builder = context_builder.title(&title.text);
    }
    
    // Add composer if present
    if let Some(composer) = document.metadata.directives.iter().find(|d| d.key == "Author") {
        context_builder = context_builder.composer(&composer.value);
    }
    
    // Add source comment if provided
    if let Some(source) = source_notation {
        context_builder = context_builder.source_comment(source);
    }
    
    // Add key signature if present
    if let Some(key) = document.metadata.attributes.get("Key") {
        let key_sig = key_signature_snippet(Some(key));
        if !key_sig.is_empty() {
            context_builder = context_builder.key_signature(key_sig);
        }
    }
    
    // Add lyrics if present
    if !lyrics.is_empty() {
        context_builder = context_builder.lyrics(lyrics);
    }
    
    let context = context_builder.build();
    
    // Auto-select appropriate template
    let template_type = auto_select_template(document);
    
    // Render the template
    render_lilypond(template_type, &context).map_err(|e| e.to_string())
}

/// Convert to LilyPond using a specific template
pub fn convert_to_lilypond_with_template(
    document: &Document, 
    note_names: LilyPondNoteNames, 
    source_notation: Option<&str>, 
    template_type: LilyPondTemplate
) -> Result<String, String> {
    // Convert the FSM-structured document to LilyPond notation
    let (rhythmic_notation, lyrics) = convert_fsm_document_to_lilypond(document, note_names)?;
    
    // Build template context (same logic as main function)
    let mut context_builder = TemplateContext::builder()
        .staves(rhythmic_notation);
        
    if let Some(title) = &document.metadata.title {
        context_builder = context_builder.title(&title.text);
    }
    
    if let Some(composer) = document.metadata.directives.iter().find(|d| d.key == "Author") {
        context_builder = context_builder.composer(&composer.value);
    }
    
    if let Some(source) = source_notation {
        context_builder = context_builder.source_comment(source);
    }
    
    if let Some(key) = document.metadata.attributes.get("Key") {
        let key_sig = key_signature_snippet(Some(key));
        if !key_sig.is_empty() {
            context_builder = context_builder.key_signature(key_sig);
        }
    }
    
    if !lyrics.is_empty() {
        context_builder = context_builder.lyrics(lyrics);
    }
    
    let context = context_builder.build();
    
    // Render with the specified template
    render_lilypond(template_type, &context).map_err(|e| e.to_string())
}