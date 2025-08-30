// Staff Notation Converter - Works directly with FSM Item, clean architecture
// Generates VexFlow-compatible JSON for staff notation rendering
use crate::models::Metadata;
use crate::rhythm_fsm::{Item, Beat};
use crate::pitch::{Degree};
use crate::rhythm::RhythmConverter;
use super::transposition::transpose_degree_with_octave;
use serde::{Deserialize, Serialize};

/// Staff notation output structures for rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffNotationStave {
    pub notes: Vec<StaffNotationElement>,
    pub key_signature: Option<String>, // Key signature like "C", "G", "F", "Bb", etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StaffNotationElement {
    Note {
        keys: Vec<String>,
        duration: String,
        dots: u8,
        accidentals: Vec<StaffNotationAccidental>,
        tied: bool,
        original_duration: Option<String>, // Store FSM duration like "1/12" for triplet detection
        beam_start: bool,
        beam_end: bool,
        syl: Option<String>, // Syllable/lyric text
    },
    Rest {
        duration: String,
        dots: u8,
        original_duration: Option<String>,
    },
    BarLine {
        bar_type: String,
    },
    Breathe,
    SlurStart {
        // Empty struct to ensure proper serialization as {type: "SlurStart"}
    },
    SlurEnd {
        // Empty struct to ensure proper serialization as {type: "SlurEnd"}
    },
    Tuplet {
        notes: Vec<StaffNotationElement>,
        ratio: (u8, u8), // (3, 2) for triplets, (5, 4) for quintuplets
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffNotationAccidental {
    pub index: usize,
    pub accidental: String,
}

/// Key signature handler for scale degree mapping
#[derive(Debug, Clone)]
struct KeyTransposer {
    /// Root note of the key (scale degree 1 maps to this)
    tonic_note: &'static str,
    /// Key signature accidentals that apply to scale degrees
    _key_signature: Vec<(usize, bool)>, // (note_index_0_to_6, is_sharp) - for future use
}

impl KeyTransposer {
    fn new(key: Option<&String>) -> Self {
        match key.as_ref().map(|k| k.to_uppercase()).as_deref() {
            Some("G") => KeyTransposer { tonic_note: "g", _key_signature: vec![(3, true)] }, // F#
            Some("D") => KeyTransposer { tonic_note: "d", _key_signature: vec![(3, true), (0, true)] }, // F#, C#  
            Some("A") => KeyTransposer { tonic_note: "a", _key_signature: vec![(3, true), (0, true), (4, true)] }, // F#, C#, G#
            Some("E") => KeyTransposer { tonic_note: "e", _key_signature: vec![(3, true), (0, true), (4, true), (1, true)] }, // F#, C#, G#, D#
            Some("B") => KeyTransposer { tonic_note: "b", _key_signature: vec![(3, true), (0, true), (4, true), (1, true), (5, true)] }, // F#, C#, G#, D#, A#
            Some("F") => KeyTransposer { tonic_note: "f", _key_signature: vec![(6, false)] }, // Bb
            Some("BB") | Some("B♭") => KeyTransposer { tonic_note: "bb", _key_signature: vec![(6, false), (2, false)] }, // Bb, Eb
            Some("EB") | Some("E♭") => KeyTransposer { tonic_note: "eb", _key_signature: vec![(6, false), (2, false), (5, false)] }, // Bb, Eb, Ab
            Some("AB") | Some("A♭") => KeyTransposer { tonic_note: "ab", _key_signature: vec![(6, false), (2, false), (5, false), (1, false)] }, // Bb, Eb, Ab, Db
            _ => KeyTransposer { tonic_note: "c", _key_signature: vec![] }, // C major (no accidentals)
        }
    }

    /// Convert Degree to VexFlow key using same transposition as LilyPond
    fn transpose_pitch(&self, degree: Degree, octave: i8) -> (String, Vec<StaffNotationAccidental>) {
        // Use the same transposition logic as LilyPond
        let (transposed_degree, adjusted_octave) = transpose_degree_with_octave(degree, octave, self.get_tonic_degree());
        
        // Convert transposed degree to VexFlow note name
        let note_name = degree_to_vexflow_note_name(transposed_degree);
        
        // VexFlow octave: 4 = middle C
        let vexflow_octave = adjusted_octave + 4;
        
        // Create VexFlow key
        let key = format!("{}/{}", note_name, vexflow_octave);
        
        // No accidentals needed - they're built into the note name
        (key, vec![])
    }
    
    /// Get the tonic degree for this key
    fn get_tonic_degree(&self) -> Degree {
        match self.tonic_note {
            "c" => Degree::N1,
            "d" => Degree::N2, 
            "e" => Degree::N3,
            "f" => Degree::N4,
            "g" => Degree::N5,
            "a" => Degree::N6,
            "b" => Degree::N7,
            "bb" => Degree::N7b,
            "eb" => Degree::N3b,
            "ab" => Degree::N6b,
            _ => Degree::N1, // Default to C
        }
    }
    
    /// Get transposition table that maps scale degrees to (note, octave_adjustment)
    /// The octave_adjustment is added to the input octave to handle wrapping
    fn get_transposition_table(&self) -> [(&'static str, i8); 7] {
        match self.tonic_note {
            "c" => [
                ("c", 0), ("d", 0), ("e", 0), ("f", 0), ("g", 0), ("a", 0), ("b", 0)
            ],
            "g" => [
                ("g", 0), ("a", 0), ("b", 0), ("c", 1), ("d", 1), ("e", 1), ("fs", 1)
            ],
            "d" => [
                ("d", 0), ("e", 0), ("fs", 0), ("g", 0), ("a", 0), ("b", 0), ("cs", 1)
            ],
            "a" => [
                ("a", 0), ("b", 0), ("cs", 1), ("d", 1), ("e", 1), ("fs", 1), ("gs", 1)
            ],
            "e" => [
                ("e", 0), ("fs", 0), ("gs", 0), ("a", 0), ("b", 0), ("cs", 1), ("ds", 1)
            ],
            "b" => [
                ("b", 0), ("cs", 1), ("ds", 1), ("e", 1), ("fs", 1), ("gs", 1), ("as", 1)
            ],
            "f" => [
                ("f", 0), ("g", 0), ("a", 0), ("bb", 0), ("c", 1), ("d", 1), ("e", 1)
            ],
            "bb" => [
                ("bb", 0), ("c", 1), ("d", 1), ("eb", 1), ("f", 1), ("g", 1), ("a", 1)
            ],
            "eb" => [
                ("eb", 0), ("f", 0), ("g", 0), ("ab", 0), ("bb", 0), ("c", 1), ("d", 1)
            ],
            "ab" => [
                ("ab", 0), ("bb", 0), ("c", 1), ("db", 1), ("eb", 1), ("f", 1), ("g", 1)
            ],
            _ => [ // Default to C major
                ("c", 0), ("d", 0), ("e", 0), ("f", 0), ("g", 0), ("a", 0), ("b", 0)
            ],
        }
    }
    
    /// Get the scale for the current key (scale degrees 1-7 mapped to actual notes)
    fn get_scale_for_key(&self) -> [&'static str; 7] {
        // For now, simplified mapping. In reality, you'd calculate the proper scale.
        match self.tonic_note {
            "c" => ["c", "d", "e", "f", "g", "a", "b"],
            "g" => ["g", "a", "b", "c", "d", "e", "fs"], // G major has F#
            "d" => ["d", "e", "fs", "g", "a", "b", "cs"], // D major has F#, C#
            "f" => ["f", "g", "a", "bb", "c", "d", "e"], // F major has Bb
            _ => ["c", "d", "e", "f", "g", "a", "b"], // Default to C major
        }
    }
}

/// Convert Degree to VexFlow note name (includes accidentals)
fn degree_to_vexflow_note_name(degree: Degree) -> String {
    use Degree::*;
    match degree {
        // Scale degree 1 (Do/Sa/C)
        N1bb => "cbb".to_string(), N1b => "cb".to_string(), N1 => "c".to_string(),
        N1s => "cs".to_string(), N1ss => "css".to_string(),
        // Scale degree 2 (Re/D)  
        N2bb => "dbb".to_string(), N2b => "db".to_string(), N2 => "d".to_string(),
        N2s => "ds".to_string(), N2ss => "dss".to_string(),
        // Scale degree 3 (Mi/Ga/E)
        N3bb => "ebb".to_string(), N3b => "eb".to_string(), N3 => "e".to_string(),
        N3s => "es".to_string(), N3ss => "ess".to_string(),
        // Scale degree 4 (Fa/Ma/F)
        N4bb => "fbb".to_string(), N4b => "fb".to_string(), N4 => "f".to_string(),
        N4s => "fs".to_string(), N4ss => "fss".to_string(),
        // Scale degree 5 (Sol/Pa/G)
        N5bb => "gbb".to_string(), N5b => "gb".to_string(), N5 => "g".to_string(),
        N5s => "gs".to_string(), N5ss => "gss".to_string(),
        // Scale degree 6 (La/Dha/A)
        N6bb => "abb".to_string(), N6b => "ab".to_string(), N6 => "a".to_string(),
        N6s => "as".to_string(), N6ss => "ass".to_string(),
        // Scale degree 7 (Ti/Ni/B)
        N7bb => "bbb".to_string(), N7b => "bb".to_string(), N7 => "b".to_string(),
        N7s => "bs".to_string(), N7ss => "bss".to_string(),
    }
}

/// Convert Degree to scale degree (note letter + accidental offset)
fn degree_to_scale_degree(degree: Degree) -> (&'static str, i8) {
    use Degree::*;
    match degree {
        // Scale degree 1 (Do/Sa/C)
        N1bb => ("c", -2), N1b => ("c", -1), N1 => ("c", 0), N1s => ("c", 1), N1ss => ("c", 2),
        // Scale degree 2 (Re/D)  
        N2bb => ("d", -2), N2b => ("d", -1), N2 => ("d", 0), N2s => ("d", 1), N2ss => ("d", 2),
        // Scale degree 3 (Mi/Ga/E)
        N3bb => ("e", -2), N3b => ("e", -1), N3 => ("e", 0), N3s => ("e", 1), N3ss => ("e", 2),
        // Scale degree 4 (Fa/Ma/F)
        N4bb => ("f", -2), N4b => ("f", -1), N4 => ("f", 0), N4s => ("f", 1), N4ss => ("f", 2),
        // Scale degree 5 (Sol/Pa/G)
        N5bb => ("g", -2), N5b => ("g", -1), N5 => ("g", 0), N5s => ("g", 1), N5ss => ("g", 2),
        // Scale degree 6 (La/Dha/A)
        N6bb => ("a", -2), N6b => ("a", -1), N6 => ("a", 0), N6s => ("a", 1), N6ss => ("a", 2),
        // Scale degree 7 (Ti/Ni/B)
        N7bb => ("b", -2), N7b => ("b", -1), N7 => ("b", 0), N7s => ("b", 1), N7ss => ("b", 2),
    }
}



/// Generate JavaScript code for VexFlow rendering
/// This actually converts to staff notation JSON for the web interface
pub fn convert_elements_to_vexflow_js(
    elements: &Vec<Item>,
    metadata: &Metadata
) -> Result<String, String> {
    // Convert to staff notation structure first
    let staves = convert_elements_to_staff_notation(elements, metadata)?;
    
    // Serialize to JSON for the web interface
    serde_json::to_string(&staves)
        .map_err(|e| format!("Failed to serialize staff notation to JSON: {}", e))
}

/// Main conversion function from V2 FSM output to staff notation JSON
pub fn convert_elements_to_staff_notation(
    elements: &Vec<Item>,
    metadata: &Metadata
) -> Result<Vec<StaffNotationStave>, String> {
    // Try both "Key" and "key" for case-insensitive match
    let transpose_key = metadata.attributes.get("Key")
        .or_else(|| metadata.attributes.get("key"));
    let transposer = KeyTransposer::new(transpose_key);
    
    let mut staves = Vec::new();
    let mut current_stave = StaffNotationStave { 
        notes: Vec::new(),
        key_signature: transpose_key.cloned()
    };
    
    // First pass: collect all beats to handle ties properly
    let mut beats = Vec::new();
    for item in elements {
        match item {
            Item::Beat(beat) => {
                beats.push(beat);
            },
            _ => {
                // Process non-beat items immediately after processing all beats
            }
        }
    }
    
    // Second pass: process beats with tie information
    for (i, beat) in beats.iter().enumerate() {
        let next_beat_tied = beats.get(i + 1).map_or(false, |next| next.tied_to_previous);
        process_beat_v2(beat, &mut current_stave, &transposer, next_beat_tied)?;
    }
    
    // Third pass: process remaining non-beat items
    for item in elements {
        match item {
            Item::Beat(_) => {
                // Already processed
            },
            Item::Tonic(_tonic_degree) => {
                // Tonic is handled through the transposer, not displayed directly
            },
            Item::Barline(style) => {
                current_stave.notes.push(StaffNotationElement::BarLine {
                    bar_type: map_barline_style(style),
                });
            },
            Item::Breathmark => {
                current_stave.notes.push(StaffNotationElement::Breathe);
            },
            Item::SlurStart => {
                current_stave.notes.push(StaffNotationElement::SlurStart {});
            },
            Item::SlurEnd => {
                current_stave.notes.push(StaffNotationElement::SlurEnd {});
            },
        }
    }
    
    // Add the completed stave
    if !current_stave.notes.is_empty() {
        staves.push(current_stave);
    }
    
    // If no beats were processed, create empty stave
    if staves.is_empty() {
        staves.push(StaffNotationStave {
            notes: Vec::new(),
            key_signature: transpose_key.cloned()
        });
    }
    
    Ok(staves)
}

/// Extract duration calculation logic to avoid duplication
fn calculate_vexflow_durations(beat_element: &crate::rhythm_fsm::BeatElement) -> Vec<(String, u8)> {
    let duration_to_use = beat_element.tuplet_display_duration.unwrap_or(beat_element.tuplet_duration);
    RhythmConverter::fraction_to_vexflow(duration_to_use)
}

/// Create note elements for a beat element
fn create_note_elements(
    beat_element: &crate::rhythm_fsm::BeatElement,
    transposer: &KeyTransposer
) -> Vec<StaffNotationElement> {
    let vexflow_durations = calculate_vexflow_durations(beat_element);
    let (key, accidentals) = transposer.transpose_pitch(beat_element.degree.unwrap(), beat_element.octave.unwrap());
    
    vexflow_durations.iter().enumerate().map(|(j, (vexflow_duration, dots))| {
        let should_tie = j < vexflow_durations.len() - 1; // Tie if more durations follow
        
        StaffNotationElement::Note {
            keys: vec![key.clone()],
            duration: vexflow_duration.clone(),
            dots: *dots,
            accidentals: accidentals.clone(),
            tied: should_tie,
            original_duration: Some(format!("{}", beat_element.tuplet_display_duration.unwrap_or(beat_element.tuplet_duration))),
            beam_start: false,
            beam_end: false,
            syl: None,
        }
    }).collect()
}

/// Create rest elements for a beat element
fn create_rest_elements(
    beat_element: &crate::rhythm_fsm::BeatElement
) -> Vec<StaffNotationElement> {
    let vexflow_durations = calculate_vexflow_durations(beat_element);
    
    vexflow_durations.into_iter().map(|(vexflow_duration, dots)| {
        StaffNotationElement::Rest {
            duration: vexflow_duration,
            dots,
            original_duration: Some(format!("{}", beat_element.tuplet_display_duration.unwrap_or(beat_element.tuplet_duration))),
        }
    }).collect()
}

fn process_beat_v2(
    beat: &Beat, 
    stave: &mut StaffNotationStave,
    transposer: &KeyTransposer,
    next_beat_tied: bool
) -> Result<(), String> {
    let mut beat_notes = Vec::new();
    
    for (_i, beat_element) in beat.elements.iter().enumerate() {
        if beat_element.is_note() {
            beat_notes.extend(create_note_elements(beat_element, transposer));
        } else if beat_element.is_rest() {
            beat_notes.extend(create_rest_elements(beat_element));
        } 
        // Skip other element types within beats (they're handled elsewhere)
    }
    
    // Handle ties to next beat: if the next beat is tied to this beat,
    // mark the last note in this beat as tied
    if next_beat_tied {
        if let Some(last_note) = beat_notes.iter_mut().rev().find(|note| matches!(note, StaffNotationElement::Note { .. })) {
            if let StaffNotationElement::Note { tied, .. } = last_note {
                *tied = true;
            }
        }
    }
    
    // Apply beaming to beat notes
    beat_notes = apply_beaming_v2(beat_notes, beat.is_tuplet);
    
    // Use FSM-provided tuplet information
    if beat.is_tuplet {
        let (tuplet_num, tuplet_den) = beat.tuplet_ratio.unwrap();
        stave.notes.push(StaffNotationElement::Tuplet {
            notes: beat_notes,
            ratio: (tuplet_num as u8, tuplet_den as u8),
        });
    } else {
        stave.notes.extend(beat_notes);
    }
    
    Ok(())
}

fn apply_beaming_v2(mut notes: Vec<StaffNotationElement>, is_tuplet: bool) -> Vec<StaffNotationElement> {
    if notes.len() < 2 {
        return notes;
    }
    
    // Find sequences of beamable notes first
    let mut beam_groups = Vec::new();
    let mut beam_start: Option<usize> = None;
    let mut beamable_count = 0;
    
    for (i, note) in notes.iter().enumerate() {
        let is_beamable = match note {
            StaffNotationElement::Note { duration, .. } => {
                if is_tuplet {
                    // In tuplets, beam quarter notes and shorter
                    matches!(duration.as_str(), "q" | "8" | "16" | "32")
                } else {
                    // Normal beaming: eighth notes and shorter
                    matches!(duration.as_str(), "8" | "16" | "32") 
                }
            },
            _ => false,
        };
        
        if is_beamable {
            if beam_start.is_none() {
                beam_start = Some(i);
                beamable_count = 1;
            } else {
                beamable_count += 1;
            }
        } else {
            // End current beam if it has 2+ notes
            if let Some(start) = beam_start {
                if beamable_count >= 2 {
                    beam_groups.push((start, i - 1));
                }
            }
            beam_start = None;
            beamable_count = 0;
        }
    }
    
    // Handle beam at end
    if let Some(start) = beam_start {
        if beamable_count >= 2 {
            beam_groups.push((start, notes.len() - 1));
        }
    }
    
    // Apply beam groups
    for (start, end) in beam_groups {
        apply_beam_markers(&mut notes, start, end);
    }
    
    notes
}

fn apply_beam_markers(notes: &mut Vec<StaffNotationElement>, start: usize, end: usize) {
    for (i, note) in notes.iter_mut().enumerate() {
        if i >= start && i <= end {
            if let StaffNotationElement::Note { ref mut beam_start, ref mut beam_end, .. } = note {
                *beam_start = i == start;
                *beam_end = i == end;
            }
        }
    }
}


fn map_barline_style(style: &str) -> String {
    match style {
        "|:" => "repeat-begin".to_string(),
        ":|" => "repeat-end".to_string(),
        "||" => "double".to_string(),
        "|." => "final".to_string(),
        "||:" => "double-repeat-begin".to_string(),
        ":||" => "double-repeat-end".to_string(),
        "::" => "double-repeat".to_string(),
        "[:" => "repeat-begin".to_string(),
        ":]" => "repeat-end".to_string(),
        "|" => "single".to_string(),
        _ => "single".to_string(),
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_key_transposer_creation() {
        let transposer = KeyTransposer::new(Some(&"G".to_string()));
        assert_eq!(transposer.tonic_note, "g");
        assert_eq!(transposer._key_signature.len(), 1);
        assert_eq!(transposer._key_signature[0], (3, true)); // F# 
    }
    
    #[test]
    fn test_degree_to_scale_degree() {
        assert_eq!(degree_to_scale_degree(Degree::N1), ("c", 0));   // Scale degree 1 = C natural
        assert_eq!(degree_to_scale_degree(Degree::N2), ("d", 0));   // Scale degree 2 = D natural 
        assert_eq!(degree_to_scale_degree(Degree::N1s), ("c", 1));  // Scale degree 1 sharp = C#
        assert_eq!(degree_to_scale_degree(Degree::N2b), ("d", -1)); // Scale degree 2 flat = Db
    }
    
    #[test]
    fn test_slur_serialization() {
        let elements = vec![
            StaffNotationElement::SlurStart {},
            StaffNotationElement::SlurEnd {},
        ];
        
        let json = serde_json::to_string(&elements).unwrap();
        println!("Serialized SlurStart/SlurEnd: {}", json);
        
        // Should serialize as objects with type field
        assert!(json.contains(r#""type":"SlurStart""#));
        assert!(json.contains(r#""type":"SlurEnd""#));
    }
    
}