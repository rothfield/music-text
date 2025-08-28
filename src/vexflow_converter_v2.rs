// V2 VexFlow Converter - Works directly with FSM OutputItemV2, clean architecture
use crate::models::Metadata;
use crate::rhythm_fsm_v2::{OutputItemV2, BeatV2};
use crate::pitch::{Degree};
use crate::rhythm::RhythmConverter;
use serde::{Deserialize, Serialize};

/// VexFlow output structures - moved from V1 vexflow_fsm_converter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VexFlowStave {
    pub notes: Vec<VexFlowElement>,
    pub key_signature: Option<String>, // Key signature like "C", "G", "F", "Bb", etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VexFlowElement {
    Note {
        keys: Vec<String>,
        duration: String,
        dots: u8,
        accidentals: Vec<VexFlowAccidental>,
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
    SlurStart,
    SlurEnd,
    Tuplet {
        notes: Vec<VexFlowElement>,
        ratio: (u8, u8), // (3, 2) for triplets, (5, 4) for quintuplets
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VexFlowAccidental {
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

    /// Convert Degree to VexFlow key using scale degree mapping
    fn transpose_pitch(&self, degree: Degree, octave: i8) -> (String, Vec<VexFlowAccidental>) {
        // Map scale degree to note in the current key
        let (scale_degree_letter, chromatic_alteration) = degree_to_scale_degree(degree);
        
        // Map scale degrees to the current key
        // For now, simple approach: C major scale degrees → target key
        let c_major_scale = ["c", "d", "e", "f", "g", "a", "b"];
        let target_key_scale = self.get_scale_for_key();
        
        let scale_degree_index = c_major_scale.iter().position(|&note| note == scale_degree_letter)
            .unwrap_or(0);
        let target_note = target_key_scale[scale_degree_index];
        
        // Adjust octave for VexFlow (4 = middle C)
        let vexflow_octave = octave + 4;
        
        // Create VexFlow key
        let key = format!("{}/{}", target_note, vexflow_octave);
        
        // Handle chromatic alterations (sharps/flats beyond key signature)
        let mut accidentals = Vec::new();
        if chromatic_alteration != 0 {
            let accidental_symbol = if chromatic_alteration > 0 {
                "#".repeat(chromatic_alteration as usize)
            } else {
                "b".repeat((-chromatic_alteration) as usize)
            };
            
            accidentals.push(VexFlowAccidental {
                index: 0,
                accidental: accidental_symbol,
            });
        }
        
        (key, accidentals)
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



/// Main conversion function from V2 FSM output to VexFlow JSON
pub fn convert_fsm_output_to_vexflow(
    fsm_output: &Vec<OutputItemV2>,
    metadata: &Metadata
) -> Result<Vec<VexFlowStave>, String> {
    let transpose_key = metadata.attributes.get("Key");
    let transposer = KeyTransposer::new(transpose_key);
    
    let mut staves = Vec::new();
    let mut current_stave = VexFlowStave { 
        notes: Vec::new(),
        key_signature: transpose_key.cloned()
    };
    
    // First pass: collect all beats to handle ties properly
    let mut beats = Vec::new();
    for item in fsm_output {
        match item {
            OutputItemV2::Beat(beat) => {
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
    for item in fsm_output {
        match item {
            OutputItemV2::Beat(_) => {
                // Already processed
            },
            OutputItemV2::Barline(style) => {
                current_stave.notes.push(VexFlowElement::BarLine {
                    bar_type: map_barline_style(style),
                });
            },
            OutputItemV2::Breathmark => {
                current_stave.notes.push(VexFlowElement::Breathe);
            },
            OutputItemV2::SlurStart => {
                current_stave.notes.push(VexFlowElement::SlurStart);
            },
            OutputItemV2::SlurEnd => {
                current_stave.notes.push(VexFlowElement::SlurEnd);
            },
        }
    }
    
    // Add the completed stave
    if !current_stave.notes.is_empty() {
        staves.push(current_stave);
    }
    
    // If no beats were processed, create empty stave
    if staves.is_empty() {
        staves.push(VexFlowStave {
            notes: Vec::new(),
            key_signature: transpose_key.cloned()
        });
    }
    
    Ok(staves)
}

fn process_beat_v2(
    beat: &BeatV2, 
    stave: &mut VexFlowStave,
    transposer: &KeyTransposer,
    next_beat_tied: bool
) -> Result<(), String> {
    let mut beat_notes = Vec::new();
    
    for (_i, beat_element) in beat.elements.iter().enumerate() {
        if beat_element.is_note() {
                // Use FSM-calculated tuplet_duration for VexFlow
                let vexflow_durations = RhythmConverter::fraction_to_vexflow(beat_element.tuplet_duration);
                
                // Transpose pitch
                let (key, accidentals) = transposer.transpose_pitch(beat_element.degree.unwrap(), beat_element.octave.unwrap());
                
                // Handle tied notes (if this element spans multiple durations)
                for (j, (vexflow_duration, dots)) in vexflow_durations.iter().enumerate() {
                    let should_tie = j < vexflow_durations.len() - 1; // Tie if more durations follow
                    
                    beat_notes.push(VexFlowElement::Note {
                        keys: vec![key.clone()],
                        duration: vexflow_duration.clone(),
                        dots: *dots,
                        accidentals: accidentals.clone(),
                        tied: should_tie,
                        original_duration: Some(format!("{}", beat_element.tuplet_duration)),
                        beam_start: false,
                        beam_end: false,
                        syl: None,
                    });
                }
        } else if beat_element.is_rest() {
                // Use FSM-calculated tuplet_duration for rests
                let vexflow_durations = RhythmConverter::fraction_to_vexflow(beat_element.tuplet_duration);
                
                for (vexflow_duration, dots) in vexflow_durations {
                    beat_notes.push(VexFlowElement::Rest {
                        duration: vexflow_duration,
                        dots,
                        original_duration: Some(format!("{}", beat_element.tuplet_duration)),
                    });
                }
        } 
        // Skip other element types within beats (they're handled elsewhere)
    }
    
    // Handle ties to next beat: if the next beat is tied to this beat,
    // mark the last note in this beat as tied
    if next_beat_tied {
        if let Some(last_note) = beat_notes.iter_mut().rev().find(|note| matches!(note, VexFlowElement::Note { .. })) {
            if let VexFlowElement::Note { tied, .. } = last_note {
                *tied = true;
            }
        }
    }
    
    // Apply beaming to beat notes
    apply_beaming_v2(&mut beat_notes, beat.is_tuplet);
    
    // Use FSM-provided tuplet information
    if beat.is_tuplet {
        let (tuplet_num, tuplet_den) = beat.tuplet_ratio.unwrap();
        stave.notes.push(VexFlowElement::Tuplet {
            notes: beat_notes,
            ratio: (tuplet_num as u8, tuplet_den as u8),
        });
    } else {
        stave.notes.extend(beat_notes);
    }
    
    Ok(())
}

fn apply_beaming_v2(notes: &mut Vec<VexFlowElement>, is_tuplet: bool) {
    if notes.len() < 2 {
        return;
    }
    
    // Find sequences of beamable notes first
    let mut beam_groups = Vec::new();
    let mut beam_start: Option<usize> = None;
    let mut beamable_count = 0;
    
    for (i, note) in notes.iter().enumerate() {
        let is_beamable = match note {
            VexFlowElement::Note { duration, .. } => {
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
        apply_beam_markers(notes, start, end);
    }
}

fn apply_beam_markers(notes: &mut Vec<VexFlowElement>, start: usize, end: usize) {
    for (i, note) in notes.iter_mut().enumerate() {
        if i >= start && i <= end {
            if let VexFlowElement::Note { beam_start, beam_end, .. } = note {
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
    
}