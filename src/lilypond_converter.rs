use crate::models::Document;
use crate::rhythmic_converter::LilyPondNoteNames;
use crate::pitch::{PitchCode, pitchcode_to_dutch_lilypond, pitchcode_to_english_lilypond};
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
        Some(key_str) => {
            // Map keys to LilyPond key signatures
            let key_map: HashMap<&str, &str> = [
                // Major keys
                ("C", "c \\major"), ("G", "g \\major"), ("D", "d \\major"), ("A", "a \\major"), ("E", "e \\major"), ("B", "b \\major"),
                ("F#", "fs \\major"), ("C#", "cs \\major"), ("F", "f \\major"), ("Bb", "bf \\major"), ("Eb", "ef \\major"),
                ("Ab", "af \\major"), ("Db", "df \\major"), ("Gb", "gf \\major"), ("Cb", "cf \\major"),
                // Lowercase variants
                ("c", "c \\major"), ("g", "g \\major"), ("d", "d \\major"), ("a", "a \\major"), ("e", "e \\major"), ("b", "b \\major"),
                ("f#", "fs \\major"), ("c#", "cs \\major"), ("f", "f \\major"), ("bb", "bf \\major"), ("eb", "ef \\major"),
                ("ab", "af \\major"), ("db", "df \\major"), ("gb", "gf \\major")
            ].iter().cloned().collect();
            
            if let Some(&key_sig) = key_map.get(key_str.as_str()) {
                format!("\\key {}", key_sig)
            } else {
                "\\key c \\major".to_string() // Default to C major
            }
        }
    }
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

fn convert_fsm_document_to_lilypond(document: &Document, note_names: LilyPondNoteNames) -> Result<String, String> {
    let mut all_notes = Vec::new();
    
    // Process each line
    for line_node in &document.nodes {
        if line_node.node_type == "LINE" {
            let mut line_notes = Vec::new();
            
            // Process each element in the line
            for element in &line_node.nodes {
                match element.node_type.as_str() {
                    "BEAT" => {
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
                                    
                                    // Add each duration (for tied notes)
                                    for duration in lily_durations {
                                        line_notes.push(format!("{}{}", pitch_str, duration));
                                    }
                                }
                            }
                        }
                    }
                    "BARLINE" => {
                        line_notes.push("|".to_string());
                    }
                    "BREATHMARK" => {
                        line_notes.push("\\breathe".to_string());
                    }
                    _ => {}
                }
            }
            
            if !line_notes.is_empty() {
                all_notes.extend(line_notes);
            }
        }
    }
    
    // If no notes found, return a rest
    if all_notes.is_empty() {
        Ok("r1".to_string())
    } else {
        Ok(all_notes.join(" "))
    }
}

pub fn convert_to_lilypond(document: &Document) -> Result<String, String> {
    convert_to_lilypond_with_names(document, LilyPondNoteNames::English)
}

pub fn convert_to_lilypond_with_names(document: &Document, note_names: LilyPondNoteNames) -> Result<String, String> {
    // Convert the FSM-structured document to LilyPond notation
    let rhythmic_notation = convert_fsm_document_to_lilypond(document, note_names)?;
    
    // Wrap in LilyPond template (embedded to work in WASM)
    let template = r#"\version "2.24.0"
\language "english"

{{header}}

\paper {
  tagline = ##f
  print-page-number = ##f
  oddHeaderMarkup = ##f
  evenHeaderMarkup = ##f
  oddFooterMarkup = ##f
  evenFooterMarkup = ##f
  indent = 0\mm
  top-margin = 1\mm
  bottom-margin = 1\mm
  left-margin = 3\mm
  right-margin = 3\mm
  ragged-right = ##t
  system-system-spacing = #'((basic-distance . 2) (minimum-distance . 2) (padding . 0) (stretchability . 0))
  markup-system-spacing = #'((basic-distance . 0) (minimum-distance . 0) (padding . 0) (stretchability . 0))
  score-system-spacing = #'((basic-distance . 0) (minimum-distance . 0) (padding . 0) (stretchability . 0))
  top-system-spacing = #'((basic-distance . 2) (minimum-distance . 2) (padding . 0) (stretchability . 0))
  last-bottom-spacing = #'((basic-distance . 2) (minimum-distance . 2) (padding . 0) (stretchability . 0))
}
\score {
  \new Staff {
    {{transpose}}
    \fixed c' {
      \clef treble
      {{key_signature}}
      \set Score.barCheckSynchronize = ##f
      \set Timing.defaultBarType = #""
      {{notes}}
    }
  }
  \layout {
    \context {
      \Staff
      \remove "Time_signature_engraver"
      \override StaffSymbol.staff-space = #0.7
    }
    \context {
      \Score
      \override SpacingSpanner.base-shortest-duration = #(ly:make-moment 1/32)
      \override SpacingSpanner.shortest-duration-space = #1.0
    }
  }
}

"#;
    
    // Build header for metadata
    let mut header = String::new();
    if let Some(title) = &document.metadata.title {
        header.push_str(&format!(r#"\header {{ title = "{}" "#, title.text));
        if let Some(composer) = document.metadata.directives.iter().find(|d| d.key == "Author") {
            header.push_str(&format!(r#"composer = "{}" "#, composer.value));
        }
        header.push_str("}");
    }
    
    // Generate transpose directive and key signature
    let transpose_directive = transpose_snippet(document.metadata.attributes.get("Key"));
    let key_signature = key_signature_snippet(document.metadata.attributes.get("Key"));
    
    let result = template
        .replace("{{notes}}", &rhythmic_notation)
        .replace("{{header}}", &header)
        .replace("{{transpose}}", &transpose_directive)
        .replace("{{key_signature}}", &key_signature)
        .replace("{{lyrics}}", ""); // No lyrics for now
    Ok(result)
}