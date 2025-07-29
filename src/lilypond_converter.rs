use crate::models::Document;
use crate::pitch::PitchCode;
use fraction::Fraction;
use std::fs;

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
    
    // For complex fractions, decompose into tied notes
    // Start with largest possible note and work down
    let mut remaining = frac;
    let mut result = Vec::new();
    
    let standard_durations = [
        (Fraction::new(1u64, 1u64), "1"),
        (Fraction::new(1u64, 2u64), "2"), 
        (Fraction::new(1u64, 4u64), "4"),
        (Fraction::new(1u64, 8u64), "8"),
        (Fraction::new(1u64, 16u64), "16"),
        (Fraction::new(1u64, 32u64), "32"),
    ];
    
    for (dur_frac, dur_str) in &standard_durations {
        while remaining >= *dur_frac {
            result.push(dur_str.to_string());
            remaining = remaining - *dur_frac;
        }
    }
    
    if result.is_empty() {
        vec!["32".to_string()] // Fallback to thirty-second note
    } else {
        result
    }
}

fn pitchcode_to_lilypond(pitch_code: PitchCode, octave: Option<i8>) -> String {
    let base_note = match pitch_code {
        PitchCode::C => "c",
        PitchCode::Db => "df",
        PitchCode::D => "d",
        PitchCode::Eb => "ef",
        PitchCode::E => "e",
        PitchCode::F => "f",
        PitchCode::Fs => "fs",
        PitchCode::G => "g",
        PitchCode::Ab => "af",
        PitchCode::A => "a",
        PitchCode::Bb => "bf",
        PitchCode::B => "b",
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


fn is_dash(value: &str) -> bool {
    value.chars().all(|c| c == '-')
}

pub fn convert_to_lilypond(document: &Document) -> Result<String, String> {
    let template = fs::read_to_string("src/lilypond_template.ly")
        .or_else(|_| fs::read_to_string("lilypond_template.ly"))
        .unwrap_or_else(|_| include_str!("lilypond_template.ly").to_string());

    // Build header
    let mut header = String::new();
    if let Some(title) = &document.metadata.title {
        header.push_str(&format!(r#"\header {{ title = "{}" "#, title.text));
        if let Some(composer) = document.metadata.directives.iter().find(|d| d.key == "Author") {
            header.push_str(&format!(r#"composer = "{}" "#, composer.value));
        }
        header.push_str("}");
    }

    // First pass: collect all pitch elements in order
    let mut all_pitches = Vec::new();
    for line_node in document.nodes.iter()
        .filter(|n| n.node_type == "LINE" && n.value.starts_with("music-line-")) {
        
        for node in &line_node.nodes {
            if node.node_type == "BEAT" {
                for beat_element in &node.nodes {
                    if beat_element.node_type == "PITCH" {
                        all_pitches.push(beat_element);
                    }
                }
            } else if node.node_type == "PITCH" {
                // Handle individual PITCH nodes (when no barlines/beats)
                all_pitches.push(node);
            }
        }
    }

    // Build notes and lyrics with tie tracking
    let mut all_notes = Vec::new();
    let mut all_lyrics = Vec::new();
    let mut last_note_pitch: Option<String> = None;
    let mut needs_tie = false;
    let mut pitch_index = 0;

    for (line_index, line_node) in document.nodes.iter()
        .filter(|n| n.node_type == "LINE" && n.value.starts_with("music-line-"))
        .enumerate() {
        
        let mut measure_notes = Vec::new();
        let mut measure_lyrics = Vec::new();

        for node in &line_node.nodes {
            if node.node_type == "BEAT" {
                let total_subdivisions = node.divisions;
                let is_tuplet = total_subdivisions > 1 && (total_subdivisions & (total_subdivisions - 1)) != 0;

                let (tuplet_ratio_num, tuplet_ratio_den) = if is_tuplet {
                    let den = 1 << ((total_subdivisions as f64).log2().floor() as u64);
                    (total_subdivisions, den)
                } else {
                    (0, 0) // Not a tuplet
                };

                let mut beat_notes = Vec::new();
                let mut beat_lyrics = Vec::new();
                
                for beat_element in &node.nodes {
                    if beat_element.node_type == "PITCH" {
                        if is_dash(&beat_element.value) && beat_element.dash_consumed {
                            pitch_index += 1;
                            continue;
                        }
                        
                        let subdivisions = beat_element.divisions;
                        
                        let element_fraction = if is_tuplet {
                            let beat_duration_in_whole_notes = Fraction::new(1u64, 4u64);
                            let note_duration_in_beat = Fraction::new(subdivisions as u64, tuplet_ratio_den as u64);
                            note_duration_in_beat * beat_duration_in_whole_notes
                        } else {
                            Fraction::new(subdivisions as u64, total_subdivisions as u64) * Fraction::new(1u64, 4u64)
                        };

                        let durations = fraction_to_lilypond_proper(element_fraction);
                        
                        // Check if next pitch is a dash
                        let next_is_dash = pitch_index + 1 < all_pitches.len() 
                            && is_dash(&all_pitches[pitch_index + 1].value);
                        
                        if is_dash(&beat_element.value) {
                            // This is a dash - tie it to the previous note if we have one
                            if let Some(ref prev_pitch) = last_note_pitch {
                                if !durations.is_empty() {
                                    let note_str = if durations.len() == 1 {
                                        format!("{}{}", prev_pitch, durations[0])
                                    } else {
                                        durations.iter()
                                            .map(|dur| format!("{}{}", prev_pitch, dur))
                                            .collect::<Vec<_>>()
                                            .join("~ ")
                                    };
                                    
                                    if needs_tie {
                                        beat_notes.push(format!("~ {}", note_str));
                                    } else {
                                        beat_notes.push(note_str);
                                    }
                                    
                                    // Add tie if next is also dash
                                    if next_is_dash {
                                        if let Some(last) = beat_notes.last_mut() {
                                            last.push('~');
                                        }
                                    }
                                    needs_tie = next_is_dash;
                                } else {
                                    let note_str = format!("{}8", prev_pitch);
                                    if needs_tie {
                                        beat_notes.push(format!("~ {}", note_str));
                                    } else {
                                        beat_notes.push(note_str);
                                    }
                                    if next_is_dash {
                                        if let Some(last) = beat_notes.last_mut() {
                                            last.push('~');
                                        }
                                    }
                                    needs_tie = next_is_dash;
                                }
                            } else {
                                // No previous note, treat as rest
                                if !durations.is_empty() {
                                    beat_notes.push(format!("r{}", durations[0]));
                                } else {
                                    beat_notes.push("r8".to_string());
                                }
                                needs_tie = false;
                            }
                            beat_lyrics.push("_".to_string());
                        } else {
                            // This is a regular note
                            let pitch_code = beat_element.pitch_code
                                .ok_or_else(|| format!("Missing pitch_code for PITCH node with value '{}'", beat_element.value))?;
                            let lily_note = pitchcode_to_lilypond(pitch_code, beat_element.octave);
                            last_note_pitch = Some(lily_note.clone());
                            
                            if !durations.is_empty() {
                                let note_str = if durations.len() == 1 {
                                    format!("{}{}", lily_note, durations[0])
                                } else {
                                    durations.iter()
                                        .map(|dur| format!("{}{}", lily_note, dur))
                                        .collect::<Vec<_>>()
                                        .join("~ ")
                                };
                                
                                let mut final_note = if needs_tie {
                                    format!("~ {}", note_str)
                                } else {
                                    note_str
                                };
                                
                                // Add tie if next is dash
                                if next_is_dash {
                                    final_note.push('~');
                                }
                                
                                beat_notes.push(final_note);
                            } else {
                                let mut note_str = format!("{}8", lily_note);
                                if needs_tie {
                                    note_str = format!("~ {}", note_str);
                                }
                                if next_is_dash {
                                    note_str.push('~');
                                }
                                beat_notes.push(note_str);
                            }
                            needs_tie = false;
                            
                            // Handle lyrics
                            let mut has_lyrics = false;
                            for child in &beat_element.nodes {
                                if child.node_type == "WORD" && !child.value.contains(")") && !child.value.contains(":") {
                                    let clean_lyric = child.value.replace("/", r"\/");
                                    beat_lyrics.push(format!("\"{}\"", clean_lyric));
                                    has_lyrics = true;
                                    break;
                                }
                            }
                            if !has_lyrics {
                                beat_lyrics.push("_".to_string());
                            }
                        }
                        pitch_index += 1;
                    }
                }
                
                if !beat_notes.is_empty() {
                    let final_beat_notes = beat_notes.join(" ");
                    if is_tuplet {
                        measure_notes.push(format!(r#"\tuplet {}/{} {{ {} }}"#, tuplet_ratio_num, tuplet_ratio_den, final_beat_notes));
                    } else {
                        measure_notes.push(final_beat_notes);
                    }
                    measure_lyrics.extend(beat_lyrics);
                }
            } else if node.node_type == "PITCH" {
                // Handle individual PITCH nodes (when no barlines/beats)
                if !is_dash(&node.value) {
                    let pitch_code = node.pitch_code
                        .ok_or_else(|| format!("Missing pitch_code for PITCH node with value '{}'", node.value))?;
                    let lily_note = pitchcode_to_lilypond(pitch_code, node.octave);
                    last_note_pitch = Some(lily_note.clone());
                    
                    // Default to quarter notes for individual pitches
                    let note_str = format!("{}4", lily_note);
                    measure_notes.push(note_str);
                    measure_lyrics.push("_".to_string());
                }
            } else if node.node_type == "BARLINE" {
                if !measure_notes.is_empty() {
                    let barline_type = match node.value.as_str() {
                        "|:" => ".|:",
                        ":|" => ":|.",
                        "||" => "||",
                        "|." => "|.",
                        ":|:" => ":|:",
                        _ => "|",
                    };
                    all_notes.push(measure_notes.join(" "));
                    all_lyrics.push(measure_lyrics.join(" "));
                    all_notes.push(format!(r#"\bar "{}""#, barline_type));
                    measure_notes.clear();
                    measure_lyrics.clear();
                }
            }
        }
        
        if !measure_notes.is_empty() {
            all_notes.push(measure_notes.join(" "));
            all_lyrics.push(measure_lyrics.join(" "));
        }

        if line_index < document.nodes.len() - 1 {
            all_notes.push(r"\break".to_string());
        }
    }

    // Substitute into template
    Ok(template
        .replace("{{header}}", &header)
        .replace("{{notes}}", &all_notes.join(" "))
        .replace("{{lyrics}}", &all_lyrics.join(" ")))
}