use crate::models::Document;
use crate::pitch::PitchCode;
use crate::rhythm::RhythmConverter;
use fraction::Fraction;

#[derive(Debug, Clone)]
enum RhythmState {
    InBeat,
    CollectingPitch { current_pitch: String, subdivisions: usize },
    CollectingDashes { pitch: String, subdivisions: usize },
}

#[derive(Debug, Clone)]
struct RhythmicContext {
    state: RhythmState,
    last_pitch: Option<String>,
    needs_tie: bool,
    output: Vec<String>,
}

impl RhythmicContext {
    fn new() -> Self {
        Self {
            state: RhythmState::InBeat,
            last_pitch: None,
            needs_tie: false,
            output: Vec::new(),
        }
    }

    fn process_pitch_element(&mut self, element: &crate::models::Node, total_subdivisions: usize, note_names: LilyPondNoteNames, pitch_to_note: &PitchToNoteFn, fraction_to_duration: &FractionToDurationFn) {
        let is_dash_element = is_dash(&element.value);
        
        match (&self.state, is_dash_element) {
            // Starting a new pitch in a beat
            (RhythmState::InBeat, false) => {
                let pitch_code = element.pitch_code.unwrap_or(PitchCode::N1);
                let octave = element.octave.unwrap_or(0);
                let pitch_str = pitch_to_note(pitch_code, Some(octave), note_names);
                self.state = RhythmState::CollectingPitch {
                    current_pitch: pitch_str.clone(),
                    subdivisions: element.divisions,
                };
                self.last_pitch = Some(pitch_str);
            }
            
            // Dash at beginning of beat - convert to previous pitch and tie
            (RhythmState::InBeat, true) => {
                if let Some(ref prev_pitch) = self.last_pitch {
                    self.state = RhythmState::CollectingDashes {
                        pitch: prev_pitch.clone(),
                        subdivisions: element.divisions,
                    };
                    self.needs_tie = true;
                } else {
                    // No previous pitch - create rest
                    let element_fraction = Fraction::new(element.divisions as u64, total_subdivisions as u64) * Fraction::new(1u64, 4u64);
                    let durations = fraction_to_duration(element_fraction);
                    for duration in durations {
                        self.output.push(format!("r{}", duration));
                    }
                }
            }
            
            // Dash after pitch - extend current pitch
            (RhythmState::CollectingPitch { current_pitch, subdivisions }, true) => {
                self.state = RhythmState::CollectingDashes {
                    pitch: current_pitch.clone(),
                    subdivisions: subdivisions + element.divisions,
                };
            }
            
            // Another dash - keep extending
            (RhythmState::CollectingDashes { pitch, subdivisions }, true) => {
                self.state = RhythmState::CollectingDashes {
                    pitch: pitch.clone(),
                    subdivisions: subdivisions + element.divisions,
                };
            }
            
            // New pitch after collecting - finish previous note and start new one
            (RhythmState::CollectingPitch { current_pitch, subdivisions } | 
             RhythmState::CollectingDashes { pitch: current_pitch, subdivisions }, false) => {
                // Finish the previous note
                let pitch_clone = current_pitch.clone();
                let subdivisions_clone = *subdivisions;
                self.finish_note(&pitch_clone, subdivisions_clone, total_subdivisions, fraction_to_duration);
                
                // Start new pitch
                let pitch_code = element.pitch_code.unwrap_or(PitchCode::N1);
                let octave = element.octave.unwrap_or(0);
                let pitch_str = pitch_to_note(pitch_code, Some(octave), note_names);
                self.state = RhythmState::CollectingPitch {
                    current_pitch: pitch_str.clone(),
                    subdivisions: element.divisions,
                };
                self.last_pitch = Some(pitch_str);
            }
        }
    }
    
    fn finish_note(&mut self, pitch: &str, subdivisions: usize, total_subdivisions: usize, fraction_to_duration: &FractionToDurationFn) {
        let element_fraction = Fraction::new(subdivisions as u64, total_subdivisions as u64) * Fraction::new(1u64, 4u64);
        let durations = fraction_to_duration(element_fraction);
        
        for (i, duration) in durations.iter().enumerate() {
            let note_str = if i == 0 && self.needs_tie {
                format!("~ {}{}", pitch, duration)
            } else if i == 0 {
                format!("{}{}", pitch, duration)
            } else {
                format!("~ {}{}", pitch, duration)
            };
            self.output.push(note_str);
        }
        
        self.needs_tie = false;
    }
    
    fn finish_beat(&mut self, total_subdivisions: usize, fraction_to_duration: &FractionToDurationFn) {
        let (current_pitch, subdivisions) = match &self.state {
            RhythmState::CollectingPitch { current_pitch, subdivisions } |
            RhythmState::CollectingDashes { pitch: current_pitch, subdivisions } => {
                (current_pitch.clone(), *subdivisions)
            }
            RhythmState::InBeat => {
                return; // Nothing to finish
            }
        };
        
        self.finish_note(&current_pitch, subdivisions, total_subdivisions, fraction_to_duration);
        self.state = RhythmState::InBeat;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LilyPondNoteNames {
    Dutch,
    English,
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
    let fraction_parts = RhythmConverter::decompose_fraction_to_standard_durations(frac);
    
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

// Generic type aliases for callback functions
type FractionToDurationFn = Box<dyn Fn(Fraction) -> Vec<String>>;
type PitchToNoteFn = Box<dyn Fn(PitchCode, Option<i8>, LilyPondNoteNames) -> String>;


fn is_dash(value: &str) -> bool {
    value.chars().all(|c| c == '-')
}

fn extract_octave_from_attached_nodes(node: &crate::models::Node) -> Option<i8> {
    let mut octave_offset = 0i8;
    
    // Look for attached octave marker nodes
    for child in &node.nodes {
        if child.node_type == "OCTAVE_MARKER" {
            match child.value.as_str() {
                "." => octave_offset += 1,  // Upper octave
                ":" => octave_offset -= 1,  // Lower octave
                _ => {}
            }
        }
    }
    
    if octave_offset != 0 {
        Some(octave_offset)
    } else {
        node.octave  // Fall back to direct octave field if no attached markers
    }
}

fn is_beamable_duration(duration: &str) -> bool {
    matches!(duration, "8" | "16" | "32" | "64")
}

fn add_beaming_to_notes(notes: Vec<String>) -> Vec<String> {
    add_beaming_to_notes_with_context(notes, false)
}

fn add_beaming_to_notes_with_context(notes: Vec<String>, is_tuplet_context: bool) -> Vec<String> {
    if notes.len() < 2 {
        return notes;
    }
    
    let mut result = Vec::new();
    let mut beam_group = Vec::new();
    
    for note in notes {
        // Extract duration from note (last part that's a number or number with dot)
        let duration = if let Some(pos) = note.rfind(|c: char| c.is_ascii_digit()) {
            let start = note[..=pos].rfind(|c: char| !c.is_ascii_digit() && c != '.').map(|i| i + 1).unwrap_or(0);
            &note[start..=pos]
        } else {
            ""
        };
        
        let can_beam = if is_tuplet_context {
            // In tuplets, be more aggressive about beaming - beam quarter notes and shorter
            is_beamable_duration_tuplet(duration) && !note.contains('~') && !note.starts_with('r')
        } else {
            // Normal beaming rules
            is_beamable_duration(duration) && !note.contains('~') && !note.starts_with('r')
        };
        
        if can_beam {
            // This note can be beamed
            beam_group.push(note);
        } else {
            // This note cannot be beamed, flush any existing beam group
            flush_beam_group(&mut beam_group, &mut result);
            result.push(note);
        }
    }
    
    // Handle any remaining beam group at the end
    flush_beam_group(&mut beam_group, &mut result);
    
    result
}

fn flush_beam_group(beam_group: &mut Vec<String>, result: &mut Vec<String>) {
    if beam_group.len() > 1 {
        // Add beam brackets to the group
        for (i, beam_note) in beam_group.iter().enumerate() {
            if i == 0 {
                result.push(format!("{}[", beam_note));
            } else if i == beam_group.len() - 1 {
                result.push(format!("{}]", beam_note));
            } else {
                result.push(beam_note.clone());
            }
        }
    } else if beam_group.len() == 1 {
        result.push(beam_group[0].clone());
    }
    beam_group.clear();
}

fn is_beamable_duration_tuplet(duration: &str) -> bool {
    // In tuplets, allow more aggressive beaming including quarter notes
    matches!(duration, "4" | "8" | "16" | "32" | "64")
}

fn convert_line_to_rhythmic_notation(
    line_node: &crate::models::Node, 
    note_names: LilyPondNoteNames,
    fraction_to_duration: &FractionToDurationFn,
    pitch_to_note: &PitchToNoteFn
) -> Result<String, String> {
    // TODO: Fix this implementation - currently broken
    // Using simple fallback for now
    Ok("c4".to_string())
}

/* BROKEN CODE - COMMENTED OUT TO ALLOW COMPILATION
fn _old_broken_implementation() {
    for node in &line_node.nodes {
        if node.node_type == "WHITESPACE" {
            // Add spacing between beats - in LilyPond this means ensuring proper note separation
            // The spacing will be handled by the join at the end
            continue;
        } else if node.node_type == "BEAT" {
            let total_subdivisions = node.divisions;
            let is_tuplet = total_subdivisions > 1 && (total_subdivisions & (total_subdivisions - 1)) != 0;

            let (tuplet_ratio_num, tuplet_ratio_den) = if is_tuplet {
                // Better tuplet ratio calculation for common cases
                match total_subdivisions {
                    3 => (3, 2),  // Triplet: 3 notes in place of 2
                    5 => (5, 4),  // Quintuplet: 5 notes in place of 4
                    6 => (6, 4),  // Sextuplet: 6 notes in place of 4
                    7 => (7, 4),  // Septuplet: 7 notes in place of 4
                    9 => (9, 8),  // Nonuplet: 9 notes in place of 8
                    10 => (10, 8), // Decuplet: 10 notes in place of 8
                    11 => (11, 8), // 11-tuplet: 11 notes in place of 8
                    12 => (12, 8), // 12-tuplet: 12 notes in place of 8
                    _ => {
                        // Fallback: find nearest power of 2
                        let den = 1 << ((total_subdivisions as f64).log2().floor() as u64);
                        (total_subdivisions, den)
                    }
                }
            } else {
                (0, 0)
            };

            let mut beat_notes = Vec::new();
            
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

                    let durations = fraction_to_duration(element_fraction);
                    
                    // Check if next pitch is a dash
                    let next_is_dash = pitch_index + 1 < line_pitches.len() 
                        && is_dash(&line_pitches[pitch_index + 1].value);
                    
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
                    } else {
                        // This is a regular note
                        let pitch_code = beat_element.pitch_code
                            .ok_or_else(|| format!("Missing pitch_code for PITCH node with value '{}'", beat_element.value))?;
                        let octave = extract_octave_from_attached_nodes(beat_element);
                        let lily_note = pitch_to_note(pitch_code, octave, note_names);
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
                    }
                    pitch_index += 1;
                }
            }
            
            if !beat_notes.is_empty() {
                // Apply beaming to the beat notes (tuplet-aware)
                let beamed_notes = add_beaming_to_notes_with_context(beat_notes, is_tuplet);
                let final_beat_notes = beamed_notes.join(" ");
                if is_tuplet {
                    measure_notes.push(format!(r#"\tuplet {}/{} {{ {} }}"#, tuplet_ratio_num, tuplet_ratio_den, final_beat_notes));
                } else {
                    measure_notes.push(final_beat_notes);
                }
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
                measure_notes.push(format!(r#"\bar "{}""#, barline_type));
            }
        }
    }
    
    // If no musical content was found in this line, add a quarter note rest
    if measure_notes.is_empty() {
        measure_notes.push("r4".to_string());
    }
    
    Ok(measure_notes.join(" "))
}

pub fn convert_to_rhythmic_notation(
    document: &Document,
    note_names: LilyPondNoteNames,
    fraction_to_duration: FractionToDurationFn,
    pitch_to_note: PitchToNoteFn
) -> Result<String, String> {
    convert_to_rhythmic_notation_with_names(document, note_names, fraction_to_duration, pitch_to_note)
}

pub fn convert_to_rhythmic_notation_with_names(
    document: &Document, 
    note_names: LilyPondNoteNames,
    fraction_to_duration: FractionToDurationFn,
    pitch_to_note: PitchToNoteFn
) -> Result<String, String> {
    // Process each line separately
    let mut all_staves = Vec::new();
    
    for line_node in document.nodes.iter()
        .filter(|n| n.node_type == "MUSICAL_LINE" || n.node_type == "LINE") {
        
        let line_notes = convert_line_to_rhythmic_notation(line_node, note_names, &fraction_to_duration, &pitch_to_note)?;
        if !line_notes.trim().is_empty() {
            all_staves.push(line_notes);
        }
    }

    // Handle case where no musical content was found
    if all_staves.is_empty() {
        // Generate a whole note rest if no musical content
        // This allows titles and metadata to still be displayed
        all_staves.push("r1".to_string());
    }

    // Join staves with breaks, like doremi-script does
    let staves_output = if all_staves.len() == 1 {
        // Single line - no breaks needed
        all_staves.join("")
    } else {
        // Multiple lines - use breaks
        all_staves.join(" \\break ")
    };

    // Return just the musical content, format-specific wrappers handle templating
    Ok(staves_output)
}
*/

