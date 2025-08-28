// V2 VexFlow Converter - Works directly with FSM OutputItemV2, clean architecture
use crate::models_v2::{ParsedElement, DocumentV2};
use crate::models::Metadata;
use crate::rhythm_fsm_v2::{OutputItemV2, BeatV2};
use crate::pitch::{PitchCode};
use crate::rhythm::RhythmConverter;
use serde::{Deserialize, Serialize};
use fraction::Fraction;
use std::collections::HashMap;

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

/// Clean key transposition using structured data instead of hardcoded mappings
#[derive(Debug, Clone)]
struct KeyTransposer {
    /// Semitone offset from C (e.g., G = 7, F = -5 = 7 in mod 12)
    semitone_offset: i8,
    /// Key signature accidentals (F# = true, Bb = false for flats)
    key_signature: Vec<(usize, bool)>, // (note_index_0_to_6, is_sharp)
}

impl KeyTransposer {
    fn new(key: Option<&String>) -> Self {
        match key.as_ref().map(|k| k.to_uppercase()).as_deref() {
            Some("G") => KeyTransposer { semitone_offset: 7, key_signature: vec![(3, true)] }, // F#
            Some("D") => KeyTransposer { semitone_offset: 2, key_signature: vec![(3, true), (0, true)] }, // F#, C#  
            Some("A") => KeyTransposer { semitone_offset: 9, key_signature: vec![(3, true), (0, true), (4, true)] }, // F#, C#, G#
            Some("E") => KeyTransposer { semitone_offset: 4, key_signature: vec![(3, true), (0, true), (4, true), (1, true)] }, // F#, C#, G#, D#
            Some("B") => KeyTransposer { semitone_offset: 11, key_signature: vec![(3, true), (0, true), (4, true), (1, true), (5, true)] }, // F#, C#, G#, D#, A#
            Some("F") => KeyTransposer { semitone_offset: 5, key_signature: vec![(6, false)] }, // Bb
            Some("BB") | Some("B♭") => KeyTransposer { semitone_offset: 10, key_signature: vec![(6, false), (2, false)] }, // Bb, Eb
            Some("EB") | Some("E♭") => KeyTransposer { semitone_offset: 3, key_signature: vec![(6, false), (2, false), (5, false)] }, // Bb, Eb, Ab
            Some("AB") | Some("A♭") => KeyTransposer { semitone_offset: 8, key_signature: vec![(6, false), (2, false), (5, false), (1, false)] }, // Bb, Eb, Ab, Db
            _ => KeyTransposer { semitone_offset: 0, key_signature: vec![] }, // C major (no accidentals)
        }
    }

    /// Convert PitchCode to VexFlow key with proper transposition and accidentals
    fn transpose_pitch(&self, pitch_code: PitchCode, octave: i8) -> (String, Vec<VexFlowAccidental>) {
        // Convert PitchCode to base semitone (C=0, D=2, E=4, etc.)
        let base_semitone = pitch_code_to_semitone(pitch_code);
        
        // Apply transposition
        let transposed_semitone = (base_semitone + self.semitone_offset) % 12;
        
        // Convert back to note name
        let (note_letter, base_accidental) = semitone_to_note_letter(transposed_semitone);
        
        // Adjust octave for VexFlow (4 = middle C)
        let vexflow_octave = octave + 4; // Convert from relative to absolute
        
        // Create VexFlow key
        let key = format!("{}/{}", note_letter, vexflow_octave);
        
        // Determine if accidental needs to be shown based on key signature
        let mut accidentals = Vec::new();
        if let Some(accidental_symbol) = base_accidental {
            let note_index = note_letter_to_index(&note_letter);
            let key_has_accidental = self.key_signature.iter()
                .any(|(idx, _)| *idx == note_index);
                
            // Show accidental if it's not in key signature or contradicts it
            if !key_has_accidental || self.accidental_contradicts_key_signature(&accidental_symbol, note_index) {
                accidentals.push(VexFlowAccidental {
                    index: 0,
                    accidental: accidental_symbol,
                });
            }
        }
        
        (key, accidentals)
    }
    
    fn accidental_contradicts_key_signature(&self, accidental: &str, note_index: usize) -> bool {
        self.key_signature.iter()
            .find(|(idx, _)| *idx == note_index)
            .map_or(false, |(_, is_sharp)| {
                (*is_sharp && accidental.contains('b')) || (!*is_sharp && accidental.contains('#'))
            })
    }
}

/// Convert PitchCode enum to semitone offset from C
fn pitch_code_to_semitone(pitch_code: PitchCode) -> i8 {
    use PitchCode::*;
    match pitch_code {
        // 1 series (C)
        N1bb => -2, N1b => -1, N1 => 0, N1s => 1, N1ss => 2,
        // 2 series (D)  
        N2bb => 0, N2b => 1, N2 => 2, N2s => 3, N2ss => 4,
        // 3 series (E)
        N3bb => 2, N3b => 3, N3 => 4, N3s => 5, N3ss => 6,
        // 4 series (F)
        N4bb => 3, N4b => 4, N4 => 5, N4s => 6, N4ss => 7,
        // 5 series (G)
        N5bb => 5, N5b => 6, N5 => 7, N5s => 8, N5ss => 9,
        // 6 series (A)
        N6bb => 7, N6b => 8, N6 => 9, N6s => 10, N6ss => 11,
        // 7 series (B)
        N7bb => 9, N7b => 10, N7 => 11, N7s => 0, N7ss => 1,
    }
}

/// Convert semitone back to note letter and accidental
fn semitone_to_note_letter(semitone: i8) -> (String, Option<String>) {
    let normalized = ((semitone % 12) + 12) % 12; // Handle negative numbers
    match normalized {
        0 => ("c".to_string(), None),
        1 => ("c".to_string(), Some("#".to_string())),
        2 => ("d".to_string(), None),
        3 => ("d".to_string(), Some("#".to_string())),
        4 => ("e".to_string(), None),
        5 => ("f".to_string(), None),
        6 => ("f".to_string(), Some("#".to_string())),
        7 => ("g".to_string(), None),
        8 => ("g".to_string(), Some("#".to_string())),
        9 => ("a".to_string(), None),
        10 => ("a".to_string(), Some("#".to_string())),
        11 => ("b".to_string(), None),
        _ => ("c".to_string(), None), // Fallback
    }
}

fn note_letter_to_index(note: &str) -> usize {
    match note.chars().next().unwrap_or('c') {
        'c' => 0, 'd' => 1, 'e' => 2, 'f' => 3, 'g' => 4, 'a' => 5, 'b' => 6,
        _ => 0,
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
    
    for item in fsm_output {
        match item {
            OutputItemV2::Beat(beat) => {
                process_beat_v2(beat, &mut current_stave, &transposer)?;
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
    transposer: &KeyTransposer
) -> Result<(), String> {
    let mut beat_notes = Vec::new();
    
    for (_i, beat_element) in beat.elements.iter().enumerate() {
        if beat_element.is_note() {
                // Use FSM-calculated tuplet_duration for VexFlow
                let vexflow_durations = RhythmConverter::fraction_to_vexflow(beat_element.tuplet_duration);
                
                // Transpose pitch
                let (key, accidentals) = transposer.transpose_pitch(beat_element.pitch_code.unwrap(), beat_element.octave.unwrap());
                
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
                        syl: None, // TODO: Extract syllables from children
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

fn convert_tuplet_duration_to_vexflow_v2(subdivisions: usize, divisions: usize) -> Vec<(String, u8)> {
    // For tuplets, map subdivisions to appropriate durations like LilyPond converter
    // This respects the subdivision proportions instead of using uniform durations
    
    let base_duration = if divisions <= 3 {
        // Small tuplets (3-tuplet) use eighth/quarter notes
        match subdivisions {
            1 => "8".to_string(),   // Eighth note
            2 => "q".to_string(),   // Quarter note
            3 => "q".to_string(),   // Quarter note (dotted would be "q" with dots=1, but keep simple)
            4 => "h".to_string(),   // Half note
            _ => "8".to_string(),
        }
    } else if divisions <= 7 {
        // Medium tuplets (4-7 notes) use sixteenth/eighth notes  
        match subdivisions {
            1 => "16".to_string(),  // Sixteenth note
            2 => "8".to_string(),   // Eighth note
            3 => "8".to_string(),   // Eighth note
            4 => "q".to_string(),   // Quarter note
            _ => "16".to_string(),
        }
    } else if divisions <= 15 {
        // Large tuplets (8-15 notes) use thirty-second/sixteenth notes
        match subdivisions {
            1 => "32".to_string(),  // Thirty-second note
            2 => "16".to_string(),  // Sixteenth note  
            3 => "16".to_string(),  // Sixteenth note
            4 => "8".to_string(),   // Eighth note
            _ => "32".to_string(),
        }
    } else {
        // Very large tuplets (16+ notes) use sixty-fourth/thirty-second notes
        match subdivisions {
            1 => "64".to_string(),  // Sixty-fourth note
            2 => "32".to_string(),  // Thirty-second note
            3 => "32".to_string(),  // Thirty-second note
            4 => "16".to_string(),  // Sixteenth note
            _ => "64".to_string(),
        }
    };
    
    vec![(base_duration, 0)]
}

// Keep old function for compatibility but mark as deprecated
fn convert_tuplet_duration_to_vexflow(_subdivisions: usize, total_divisions: usize) -> Vec<(String, u8)> {
    // DEPRECATED: Use convert_tuplet_duration_to_vexflow_v2 instead
    let base_duration = if total_divisions <= 3 {
        "8".to_string()    // Triplets and smaller get eighth notes
    } else if total_divisions <= 6 {
        "16".to_string()   // Larger tuplets get sixteenth notes  
    } else {
        "32".to_string()   // Very large tuplets get thirty-second notes
    };
    
    vec![(base_duration, 0)]
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

// Legacy compatibility function - delegates to FSM-based converter
pub fn convert_document_v2_to_vexflow(
    _document: &DocumentV2
) -> Result<Vec<VexFlowStave>, String> {
    // V2 documents should use the FSM output path, not direct conversion
    // Return empty result for now - this function shouldn't be called in V2 flow
    Ok(vec![VexFlowStave {
        notes: Vec::new(),
        key_signature: None,
    }])
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_key_transposer_creation() {
        let transposer = KeyTransposer::new(Some(&"G".to_string()));
        assert_eq!(transposer.semitone_offset, 7);
        assert_eq!(transposer.key_signature.len(), 1);
        assert_eq!(transposer.key_signature[0], (3, true)); // F# 
    }
    
    #[test]
    fn test_pitch_code_to_semitone() {
        assert_eq!(pitch_code_to_semitone(PitchCode::N1), 0);  // C
        assert_eq!(pitch_code_to_semitone(PitchCode::N2), 2);  // D 
        assert_eq!(pitch_code_to_semitone(PitchCode::N1s), 1); // C#
    }
    
    #[test]
    fn test_semitone_to_note_letter() {
        let (note, acc) = semitone_to_note_letter(0);
        assert_eq!(note, "c");
        assert_eq!(acc, None);
        
        let (note, acc) = semitone_to_note_letter(1);
        assert_eq!(note, "c");
        assert_eq!(acc, Some("#".to_string()));
    }
}