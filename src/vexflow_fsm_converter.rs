// VexFlow converter that operates directly on FSM output
use crate::models::{Document, Node};
use crate::pitch::{PitchCode, pitchcode_to_english_lilypond};
use serde::{Deserialize, Serialize};
use fraction::Fraction;
use std::str::FromStr;
use std::collections::HashMap;

fn get_transposed_vexflow_pitch(pitch_code: PitchCode, key: Option<&String>) -> (String, Option<String>) {
    use crate::pitch::PitchCode::*;
    
    let key_str = match key {
        None => return pitch_to_vexflow_components(pitch_code),
        Some(k) => k.to_uppercase(),
    };
    
    if key_str == "C" {
        return pitch_to_vexflow_components(pitch_code);
    }
    
    // Direct mapping: (key, input_pitch) -> (vexflow_note, accidental)
    let mappings: HashMap<(&str, PitchCode), (&str, Option<&str>)> = [
        // Key D mappings (N1 in key D = D)
        // In D major, the key signature has F# and C#
        (("D", N1), ("d", None)),       // C -> D
        (("D", N1s), ("d", Some("#"))), // C# -> D#
        (("D", N1b), ("d", Some("b"))), // Cb -> Db
        (("D", N2), ("e", None)),       // D -> E
        (("D", N2s), ("f", None)),      // D# -> F (which is F# in key sig, so no accidental needed)
        (("D", N2b), ("e", Some("b"))), // Db -> Eb
        (("D", N3), ("f", None)),       // E -> F# (key sig has F#, so just "f")
        (("D", N3s), ("g", None)),      // E# -> G
        (("D", N3b), ("f", Some("n"))), // Eb -> F natural (needs natural since key sig has F#)
        (("D", N4), ("g", None)),       // F -> G
        (("D", N4s), ("a", None)),      // F# -> A
        (("D", N4b), ("g", Some("b"))), // Fb -> Gb
        (("D", N5), ("a", None)),       // G -> A
        (("D", N5s), ("a", Some("#"))), // G# -> A#
        (("D", N5b), ("a", Some("b"))), // Gb -> Ab
        (("D", N6), ("b", None)),       // A -> B
        (("D", N6s), ("c", None)),      // A# -> C# (key sig has C#, so just "c")
        (("D", N6b), ("b", Some("b"))), // Ab -> Bb
        (("D", N7), ("c", None)),       // B -> C# (key sig has C#, so just "c")
        (("D", N7s), ("d", None)),      // B# -> D
        (("D", N7b), ("c", Some("n"))), // Bb -> C natural (needs natural since key sig has C#)
        
        // Key G mappings (N1 in key G = G)
        (("G", N1), ("g", None)),      // C -> G
        (("G", N1s), ("g", Some("#"))), // C# -> G#
        (("G", N2), ("a", None)),      // D -> A
        (("G", N2b), ("a", Some("b"))), // Db -> Ab
        (("G", N3), ("b", None)),      // E -> B
        (("G", N4), ("c", None)),      // F -> C
        (("G", N4s), ("c", Some("#"))), // F# -> C#
        (("G", N5), ("d", None)),      // G -> D
        (("G", N6), ("e", None)),      // A -> E
        (("G", N7), ("f", Some("#"))), // B -> F#
        (("G", N7b), ("f", None)),     // Bb -> F
        
        // Key F mappings (N1 in key F = F)
        (("F", N1), ("f", None)),      // C -> F
        (("F", N2), ("g", None)),      // D -> G
        (("F", N3), ("a", None)),      // E -> A
        (("F", N4), ("b", Some("b"))), // F -> Bb
        (("F", N4s), ("b", None)),     // F# -> B
        (("F", N5), ("c", None)),      // G -> C
        (("F", N6), ("d", None)),      // A -> D
        (("F", N7), ("e", None)),      // B -> E
        (("F", N7b), ("e", Some("b"))), // Bb -> Eb
        
        // Key Bb mappings (N1 in key Bb = Bb)
        (("BB", N1), ("b", Some("b"))), // C -> Bb
        (("BB", N2), ("c", None)),      // D -> C
        (("BB", N3), ("d", None)),      // E -> D
        (("BB", N4), ("e", Some("b"))), // F -> Eb
        (("BB", N4s), ("e", None)),     // F# -> E
        (("BB", N5), ("f", None)),      // G -> F
        (("BB", N6), ("g", None)),      // A -> G
        (("BB", N7), ("a", None)),      // B -> A
        
        // Key A mappings (N1 in key A = A)
        (("A", N1), ("a", None)),      // C -> A
        (("A", N2), ("b", None)),      // D -> B
        (("A", N3), ("c", Some("#"))), // E -> C#
        (("A", N4), ("d", None)),      // F -> D
        (("A", N4s), ("d", Some("#"))), // F# -> D# (Eb)
        (("A", N5), ("e", None)),      // G -> E
        (("A", N6), ("f", Some("#"))), // A -> F#
        (("A", N7), ("g", Some("#"))), // B -> G#
        
        // Key E mappings (N1 in key E = E)
        (("E", N1), ("e", None)),      // C -> E
        (("E", N2), ("f", Some("#"))), // D -> F#
        (("E", N3), ("g", Some("#"))), // E -> G#
        (("E", N4), ("a", None)),      // F -> A
        (("E", N4s), ("a", Some("#"))), // F# -> A#
        (("E", N5), ("b", None)),      // G -> B
        (("E", N6), ("c", Some("#"))), // A -> C#
        (("E", N7), ("d", Some("#"))), // B -> D#
        
        // Key B mappings (N1 in key B = B)
        (("B", N1), ("b", None)),      // C -> B
        (("B", N2), ("c", Some("#"))), // D -> C#
        (("B", N3), ("d", Some("#"))), // E -> D#
        (("B", N4), ("e", None)),      // F -> E
        (("B", N4s), ("f", None)),     // F# -> F (E#)
        (("B", N5), ("f", Some("#"))), // G -> F#
        (("B", N6), ("g", Some("#"))), // A -> G#
        (("B", N7), ("a", Some("#"))), // B -> A#
        
        // Key Eb mappings (N1 in key Eb = Eb)
        (("EB", N1), ("e", Some("b"))), // C -> Eb
        (("EB", N2), ("f", None)),      // D -> F
        (("EB", N3), ("g", None)),      // E -> G
        (("EB", N4), ("a", Some("b"))), // F -> Ab
        (("EB", N4s), ("a", None)),     // F# -> A
        (("EB", N5), ("b", Some("b"))), // G -> Bb
        (("EB", N6), ("c", None)),      // A -> C
        (("EB", N7), ("d", None)),      // B -> D
        
        // Key Ab mappings (N1 in key Ab = Ab)
        (("AB", N1), ("a", Some("b"))), // C -> Ab
        (("AB", N2), ("b", Some("b"))), // D -> Bb
        (("AB", N3), ("c", None)),      // E -> C
        (("AB", N4), ("d", Some("b"))), // F -> Db
        (("AB", N4s), ("d", None)),     // F# -> D
        (("AB", N5), ("e", Some("b"))), // G -> Eb
        (("AB", N6), ("f", None)),      // A -> F
        (("AB", N7), ("g", None)),      // B -> G
    ].iter().cloned().collect();
    
    mappings.get(&(key_str.as_str(), pitch_code))
        .map(|(note, acc)| (note.to_string(), acc.map(|a| a.to_string())))
        .unwrap_or_else(|| pitch_to_vexflow_components(pitch_code))
}

fn pitch_to_vexflow_components(pitch_code: PitchCode) -> (String, Option<String>) {
    // Convert pitch code to base note name without octave
    let base_note = pitchcode_to_english_lilypond(pitch_code);
    let (note_letter, accidental_str) = parse_lilypond_note(&base_note);
    
    let accidental = match accidental_str.as_str() {
        "s" => Some("#".to_string()),
        "ss" => Some("##".to_string()),
        "f" => Some("b".to_string()),
        "ff" => Some("bb".to_string()),
        _ => None,
    };
    
    (note_letter, accidental)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VexFlowStave {
    pub notes: Vec<VexFlowElement>,
    pub key_signature: Option<String>, // Key signature like "C", "G", "F", "Bb", etc.
}

/// Simple note table for tracking accidentals: C D E F G A B
/// Each position stores the current accidental state for that note
#[derive(Debug, Clone)]
pub struct NoteAccidentalTable {
    /// Index 0=C, 1=D, 2=E, 3=F, 4=G, 5=A, 6=B
    /// Values: None=natural, Some("#")=sharp, Some("b")=flat, Some("##")=double sharp, etc.
    notes: [Option<String>; 7],
}

impl NoteAccidentalTable {
    /// Create new table initialized with key signature
    pub fn new(key_signature: Option<&String>) -> Self {
        let mut table = Self {
            notes: [None, None, None, None, None, None, None], // All natural initially
        };
        
        // Initialize with key signature accidentals
        if let Some(key) = key_signature {
            match key.to_uppercase().as_str() {
                "G" => { table.notes[3] = Some("#".to_string()); } // F#
                "D" => { 
                    table.notes[3] = Some("#".to_string()); // F#
                    table.notes[0] = Some("#".to_string()); // C#
                }
                "A" => {
                    table.notes[3] = Some("#".to_string()); // F#
                    table.notes[0] = Some("#".to_string()); // C#
                    table.notes[4] = Some("#".to_string()); // G#
                }
                "E" => {
                    table.notes[3] = Some("#".to_string()); // F#
                    table.notes[0] = Some("#".to_string()); // C#
                    table.notes[4] = Some("#".to_string()); // G#
                    table.notes[1] = Some("#".to_string()); // D#
                }
                "B" => {
                    table.notes[3] = Some("#".to_string()); // F#
                    table.notes[0] = Some("#".to_string()); // C#
                    table.notes[4] = Some("#".to_string()); // G#
                    table.notes[1] = Some("#".to_string()); // D#
                    table.notes[5] = Some("#".to_string()); // A#
                }
                "F#" => {
                    table.notes[3] = Some("#".to_string()); // F#
                    table.notes[0] = Some("#".to_string()); // C#
                    table.notes[4] = Some("#".to_string()); // G#
                    table.notes[1] = Some("#".to_string()); // D#
                    table.notes[5] = Some("#".to_string()); // A#
                    table.notes[2] = Some("#".to_string()); // E#
                }
                "C#" => {
                    table.notes[3] = Some("#".to_string()); // F#
                    table.notes[0] = Some("#".to_string()); // C#
                    table.notes[4] = Some("#".to_string()); // G#
                    table.notes[1] = Some("#".to_string()); // D#
                    table.notes[5] = Some("#".to_string()); // A#
                    table.notes[2] = Some("#".to_string()); // E#
                    table.notes[6] = Some("#".to_string()); // B#
                }
                "F" => { table.notes[6] = Some("b".to_string()); } // Bb
                "BB" => {
                    table.notes[6] = Some("b".to_string()); // Bb
                    table.notes[2] = Some("b".to_string()); // Eb
                }
                "EB" => {
                    table.notes[6] = Some("b".to_string()); // Bb
                    table.notes[2] = Some("b".to_string()); // Eb
                    table.notes[5] = Some("b".to_string()); // Ab
                }
                "AB" => {
                    table.notes[6] = Some("b".to_string()); // Bb
                    table.notes[2] = Some("b".to_string()); // Eb
                    table.notes[5] = Some("b".to_string()); // Ab
                    table.notes[1] = Some("b".to_string()); // Db
                }
                "DB" => {
                    table.notes[6] = Some("b".to_string()); // Bb
                    table.notes[2] = Some("b".to_string()); // Eb
                    table.notes[5] = Some("b".to_string()); // Ab
                    table.notes[1] = Some("b".to_string()); // Db
                    table.notes[4] = Some("b".to_string()); // Gb
                }
                "GB" => {
                    table.notes[6] = Some("b".to_string()); // Bb
                    table.notes[2] = Some("b".to_string()); // Eb
                    table.notes[5] = Some("b".to_string()); // Ab
                    table.notes[1] = Some("b".to_string()); // Db
                    table.notes[4] = Some("b".to_string()); // Gb
                    table.notes[0] = Some("b".to_string()); // Cb
                }
                "CB" => {
                    table.notes[6] = Some("b".to_string()); // Bb
                    table.notes[2] = Some("b".to_string()); // Eb
                    table.notes[5] = Some("b".to_string()); // Ab
                    table.notes[1] = Some("b".to_string()); // Db
                    table.notes[4] = Some("b".to_string()); // Gb
                    table.notes[0] = Some("b".to_string()); // Cb
                    table.notes[3] = Some("b".to_string()); // Fb
                }
                _ => {} // C major or unknown - all natural
            }
        }
        
        table
    }
    
    /// Reset table for new bar (restore to key signature state)
    pub fn reset_for_new_bar(&mut self, key_signature: Option<&String>) {
        *self = Self::new(key_signature);
    }
    
    /// Check if we need to show an accidental for this note
    /// Returns (show_accidental, accidental_symbol)
    pub fn check_and_update_note(&mut self, note_letter: &str, desired_accidental: Option<&str>) -> (bool, Option<String>) {
        let note_index = match note_letter.to_lowercase().as_str() {
            "c" => 0, "d" => 1, "e" => 2, "f" => 3, 
            "g" => 4, "a" => 5, "b" => 6,
            _ => return (false, None), // Invalid note
        };
        
        let current_accidental = &self.notes[note_index];
        let desired = desired_accidental.map(|s| s.to_string());
        
        // If current state matches desired, no accidental needed
        if *current_accidental == desired {
            return (false, None);
        }
        
        // Update table with new accidental
        self.notes[note_index] = desired.clone();
        
        // Return what accidental to show
        match desired {
            None => (true, Some("n".to_string())), // Natural symbol
            Some(acc) => (true, Some(acc)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
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
    Tuplet {
        notes: Vec<VexFlowElement>,
        divisions: u8, // Original beat divisions (e.g., 3 for triplet, 5 for quintuplet)
    },
    SlurStart,
    SlurEnd,
    Mordent {
        note_index: usize, // Index of the note this mordent applies to
        mordent_type: String, // "upper" or "lower"
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VexFlowAccidental {
    pub index: usize,
    pub accidental: String,
}

pub fn convert_fsm_to_vexflow(document: &Document) -> Result<Vec<VexFlowStave>, String> {
    let mut staves = Vec::new();
    
    // Get transpose key from document metadata
    let transpose_key = document.metadata.attributes.get("Key");
    
    // Process each line as a stave
    for line_node in &document.nodes {
        if line_node.node_type == "LINE" {
            let mut stave = VexFlowStave { 
                notes: Vec::new(),
                key_signature: transpose_key.map(|k| k.clone())
            };
            
            // Initialize accidental table for this stave
            let mut accidental_table = NoteAccidentalTable::new(transpose_key);
            
            // Process each element in the line
            for (beat_idx, element) in line_node.nodes.iter().enumerate() {
                match element.node_type.as_str() {
                    "BEAT" => {
                        process_beat(&element, &mut stave, transpose_key, &mut accidental_table, &line_node.nodes, beat_idx)?;
                    }
                    "BARLINE" => {
                        // Reset accidental table for new bar
                        accidental_table.reset_for_new_bar(transpose_key);
                        
                        // Determine barline type based on the actual barline value
                        let bar_type = match element.value.as_str() {
                            "|:" => "repeat-begin",
                            ":|" => "repeat-end", 
                            "||" => "double",
                            "|." => "final",  // Single bar with dot (end of piece)
                            "||:" => "double-repeat-begin",
                            ":||" => "double-repeat-end", 
                            "::" => "double-repeat", // Both begin and end repeat
                            "[:" => "repeat-begin", // Alternative repeat begin notation
                            ":]" => "repeat-end",   // Alternative repeat end notation  
                            "|" => "single",
                            _ => "single" // Default fallback
                        };
                        
                        // Add barlines (they will be handled by the frontend renderer)
                        stave.notes.push(VexFlowElement::BarLine {
                            bar_type: bar_type.to_string(),
                        });
                    }
                    "BREATHMARK" => {
                        stave.notes.push(VexFlowElement::Breathe);
                    }
                    "SLUR_START" => {
                        stave.notes.push(VexFlowElement::SlurStart);
                    }
                    "SLUR_END" => {
                        stave.notes.push(VexFlowElement::SlurEnd);
                    }
                    _ => {}
                }
            }
            
            if !stave.notes.is_empty() {
                // NOTE: Tie detection is already handled correctly during note creation (lines 487-527)
                // The apply_stave_tie_detection function incorrectly ties ALL consecutive same-pitch notes
                // Commenting out to fix issue where GG becomes tied instead of two separate notes
                // apply_stave_tie_detection(&mut stave.notes);
                staves.push(stave);
            }
        }
    }
    
    Ok(staves)
}

fn process_beat(beat_node: &Node, stave: &mut VexFlowStave, transpose_key: Option<&String>, accidental_table: &mut NoteAccidentalTable, line_nodes: &Vec<Node>, beat_idx: usize) -> Result<(), String> {
    eprintln!("DEBUG: Processing beat with {} child nodes", beat_node.nodes.len());
    let total_subdivisions = beat_node.divisions;
    // Only create tuplets for non-power-of-2 subdivisions
    let is_tuplet = total_subdivisions > 1 && (total_subdivisions & (total_subdivisions - 1)) != 0;
    
    let mut beat_elements = Vec::new();
    
    // Process notes in the beat
    for (i, note_node) in beat_node.nodes.iter().enumerate() {
        eprintln!("DEBUG: Processing node type: {}", note_node.node_type);
        if note_node.node_type == "SLUR_START" {
            eprintln!("DEBUG: Adding SlurStart");
            beat_elements.push(VexFlowElement::SlurStart);
        } else if note_node.node_type == "SLUR_END" {
            eprintln!("DEBUG: Adding SlurEnd");
            beat_elements.push(VexFlowElement::SlurEnd);
        } else if note_node.node_type == "PITCH" {
            eprintln!("DEBUG: Processing PITCH node with value: {}", note_node.value);
            // Extract duration from the value (e.g., "S[1/16]" -> "1/16") 
            let duration_str = if let Some(start) = note_node.value.find('[') {
                if let Some(end) = note_node.value.find(']') {
                    let full_tag = &note_node.value[start+1..end];
                    // Use the fraction directly
                    full_tag
                } else {
                    "1/4"
                }
            } else {
                "1/4"
            };
            
            // Convert fraction to VexFlow duration using same logic as LilyPond
            let vexflow_durations = if let Ok(frac) = Fraction::from_str(duration_str) {
                fraction_to_vexflow_duration_proper(frac)
            } else {
                vec![("q".to_string(), 0)]
            };
            
            if let Some(pitch_code) = note_node.pitch_code {
                // Get transposed pitch components directly
                let (transposed_note, desired_accidental) = get_transposed_vexflow_pitch(pitch_code, transpose_key);
                
                // Check if we need to display an accidental using our table
                let (show_accidental, accidental_symbol) = accidental_table.check_and_update_note(
                    &transposed_note, 
                    desired_accidental.as_deref()
                );
                
                // VexFlow octave (4 = middle C)
                let vexflow_octave = match note_node.octave.unwrap_or(0) {
                    -2 => 2,
                    -1 => 3,
                    0 => 4,   // middle octave
                    1 => 5,
                    2 => 6,
                    _ => 4,
                };
                
                let key = format!("{}/{}", transposed_note, vexflow_octave);
                
                // Only add accidental if the table says we need to show one
                let mut accidentals = Vec::new();
                if show_accidental {
                    if let Some(acc_symbol) = accidental_symbol {
                        accidentals.push(VexFlowAccidental {
                            index: 0,
                            accidental: acc_symbol,
                        });
                    }
                }
                
                // Track the first note index for mordent attachment
                let first_note_index = beat_elements.len();
                
                // Add each duration (for tied notes) - SAME AS LILYPOND
                for (j, (vexflow_duration, dots)) in vexflow_durations.iter().enumerate() {
                    // Check if this note should tie to the next one
                    // Look ahead to see if the next pitch node has dash_consumed=true
                    let should_tie_to_next = if j == 0 {
                        // First check within this beat
                        let tie_within_beat = beat_node.nodes.iter()
                            .skip(i + 1)
                            .find(|n| n.node_type == "PITCH")
                            .map_or(false, |next_pitch| {
                                next_pitch.dash_consumed && 
                                next_pitch.pitch_code == note_node.pitch_code // Only tie same pitches
                            });
                        
                        // If no tie within beat, check across beats
                        if !tie_within_beat {
                            // Look for the next BEAT node and check if its first PITCH has dash_consumed AND same pitch
                            line_nodes.iter()
                                .skip(beat_idx + 1)
                                .find(|n| n.node_type == "BEAT")
                                .and_then(|next_beat| next_beat.nodes.iter().find(|n| n.node_type == "PITCH"))
                                .map_or(false, |first_pitch| {
                                    first_pitch.dash_consumed && 
                                    first_pitch.pitch_code == note_node.pitch_code // Only tie same pitches
                                })
                        } else {
                            tie_within_beat
                        }
                    } else {
                        false
                    };
                    // Only the first note should have lyrics - don't duplicate lyrics on tied continuations
                    // Extract syllable from SYL child nodes or use the syl attribute
                    let note_syl = if j == 0 { 
                        extract_syllable_from_node(note_node).or(note_node.syl.clone())
                    } else { None };
                    
                    beat_elements.push(VexFlowElement::Note {
                        keys: vec![key.clone()],
                        duration: vexflow_duration.clone(),
                        dots: *dots,
                        accidentals: accidentals.clone(),
                        tied: should_tie_to_next,
                        original_duration: Some(duration_str.to_string()), // Preserve FSM duration
                        beam_start: false, // Will be set by beaming logic
                        beam_end: false,
                        syl: note_syl, // Only first note gets lyrics, not tied continuations
                    });
                }
                
                // Check for mordent ornaments attached to this pitch
                // Attach mordent to the first note of this pitch (even if tied)
                for child in &note_node.nodes {
                    if child.node_type == "MORDENT" {
                        // Determine if it's an upper or lower mordent
                        // For now, treat ~ as upper mordent (standard convention)
                        let mordent_type = if child.value == "~" { "upper" } else { "lower" };
                        
                        beat_elements.push(VexFlowElement::Mordent {
                            note_index: first_note_index, // Index of the first note for this pitch
                            mordent_type: mordent_type.to_string(),
                        });
                    }
                }
            }
        } else if note_node.node_type == "REST" {
            // Handle rest nodes
            let duration_str = if let Some(start) = note_node.value.find('[') {
                if let Some(end) = note_node.value.find(']') {
                    let full_tag = &note_node.value[start+1..end];
                    // Use the fraction directly
                    full_tag
                } else {
                    "1/4"
                }
            } else {
                "1/4"
            };
            
            let vexflow_durations = if let Ok(frac) = Fraction::from_str(duration_str) {
                fraction_to_vexflow_duration_proper(frac)
            } else {
                vec![("q".to_string(), 0)]
            };
            
            for (vexflow_duration, dots) in vexflow_durations {
                beat_elements.push(VexFlowElement::Rest {
                    duration: vexflow_duration,
                    dots,
                    original_duration: Some(duration_str.to_string()), // Preserve FSM duration
                });
            }
        }
    }
    
    // Apply beaming to beat elements
    apply_vexflow_beaming(&mut beat_elements, is_tuplet);
    
    // Wrap in tuplet if needed
    if is_tuplet && beat_elements.len() > 1 {
        stave.notes.push(VexFlowElement::Tuplet {
            notes: beat_elements,
            divisions: total_subdivisions as u8,
        });
    } else {
        // Add elements directly to stave
        eprintln!("DEBUG: Adding {} beat elements to stave", beat_elements.len());
        stave.notes.extend(beat_elements);
    }
    
    Ok(())
}

fn apply_vexflow_beaming(elements: &mut Vec<VexFlowElement>, is_tuplet: bool) {
    if elements.len() < 2 {
        return;
    }
    
    // First pass: identify beam groups
    let mut beam_groups = Vec::new();
    let mut beam_start_idx = None;
    let mut beamable_count = 0;
    
    for (i, element) in elements.iter().enumerate() {
        let is_beamable = match element {
            VexFlowElement::Note { duration, .. } => {
                if is_tuplet {
                    // In tuplets, beam quarter notes and shorter
                    matches!(duration.as_str(), "q" | "8" | "16" | "32")
                } else {
                    // Normal beaming: eighth notes and shorter
                    matches!(duration.as_str(), "8" | "16" | "32")
                }
            }
            _ => false,
        };
        
        if is_beamable {
            if beam_start_idx.is_none() {
                beam_start_idx = Some(i);
            }
            beamable_count += 1;
        } else {
            // End current beam group if it has 2+ notes
            if let Some(start_idx) = beam_start_idx {
                if beamable_count >= 2 {
                    beam_groups.push((start_idx, i - 1));
                }
            }
            beam_start_idx = None;
            beamable_count = 0;
        }
    }
    
    // Handle beam group at end
    if let Some(start_idx) = beam_start_idx {
        if beamable_count >= 2 {
            beam_groups.push((start_idx, elements.len() - 1));
        }
    }
    
    // Second pass: apply beaming
    for (start, end) in beam_groups {
        apply_beam_to_range(elements, start, end);
    }
}

fn apply_beam_to_range(elements: &mut Vec<VexFlowElement>, start: usize, end: usize) {
    for (i, element) in elements.iter_mut().enumerate() {
        if i >= start && i <= end {
            if let VexFlowElement::Note { beam_start, beam_end, .. } = element {
                *beam_start = i == start;
                *beam_end = i == end;
            }
        }
    }
}

fn apply_tie_detection(elements: &mut Vec<VexFlowElement>) {
    if elements.len() < 2 {
        return;
    }
    
    // Look for consecutive notes with the same pitch
    for i in 0..elements.len() - 1 {
        // First, check if both elements are notes and have same keys
        let should_tie = match (&elements[i], &elements[i + 1]) {
            (VexFlowElement::Note { keys: keys1, .. }, 
             VexFlowElement::Note { keys: keys2, .. }) => {
                keys1 == keys2
            }
            _ => false
        };
        
        // If they should be tied, modify the first note
        if should_tie {
            if let VexFlowElement::Note { tied, .. } = &mut elements[i] {
                *tied = true;
            }
        }
    }
}

fn get_note_duration(element: &VexFlowElement, note_idx: usize) -> Option<String> {
    match element {
        VexFlowElement::Note { duration, .. } => Some(duration.clone()),
        VexFlowElement::Tuplet { notes, .. } => {
            if let Some(VexFlowElement::Note { duration, .. }) = notes.get(note_idx) {
                Some(duration.clone())
            } else {
                None
            }
        }
        _ => None,
    }
}

fn apply_stave_tie_detection(stave_elements: &mut Vec<VexFlowElement>) {
    if stave_elements.len() < 2 {
        return;
    }
    
    // Get all notes from the stave, flattening tuplets
    let mut all_notes: Vec<(usize, usize, Vec<String>)> = Vec::new(); // (element_idx, note_idx, keys)
    
    for (elem_idx, element) in stave_elements.iter().enumerate() {
        match element {
            VexFlowElement::Note { keys, .. } => {
                all_notes.push((elem_idx, 0, keys.clone()));
            }
            VexFlowElement::Tuplet { notes, .. } => {
                for (note_idx, tuplet_note) in notes.iter().enumerate() {
                    if let VexFlowElement::Note { keys, .. } = tuplet_note {
                        all_notes.push((elem_idx, note_idx, keys.clone()));
                    }
                }
            }
            _ => {} // Skip other element types
        }
    }
    
    // Check for consecutive notes with same pitch and duration for ties
    for i in 0..all_notes.len().saturating_sub(1) {
        let (elem_idx1, note_idx1, keys1) = &all_notes[i];
        let (elem_idx2, note_idx2, keys2) = &all_notes[i + 1];
        
        // Get durations of both notes to compare
        let duration1 = get_note_duration(&stave_elements[*elem_idx1], *note_idx1);
        let duration2 = get_note_duration(&stave_elements[*elem_idx2], *note_idx2);
        
        // Only tie notes with same pitch (correct musical definition of ties)
        if keys1 == keys2 {
            // Mark the first note as tied (source of tie)
            // This follows LilyPond convention where the tie is after the first note
            match &mut stave_elements[*elem_idx1] {
                VexFlowElement::Note { tied, .. } => {
                    *tied = true;
                }
                VexFlowElement::Tuplet { notes, .. } => {
                    if let Some(VexFlowElement::Note { tied, .. }) = notes.get_mut(*note_idx1) {
                        *tied = true;
                    }
                }
                _ => {}
            }
        }
    }
}

fn extract_duration_from_value(value: &str) -> &str {
    if let Some(start) = value.find('[') {
        if let Some(end) = value.find(']') {
            return &value[start+1..end];
        }
    }
    "1/1"
}

fn fraction_to_vexflow_duration_proper(frac: Fraction) -> Vec<(String, u8)> {
    // Create a lookup table for common fractions to VexFlow durations - MIRRORS LILYPOND
    let lookup = [
        (Fraction::new(1u64, 1u64), vec![("w".to_string(), 0)]),    // whole note
        (Fraction::new(1u64, 2u64), vec![("h".to_string(), 0)]),    // half note
        (Fraction::new(1u64, 4u64), vec![("q".to_string(), 0)]),    // quarter note
        (Fraction::new(1u64, 8u64), vec![("8".to_string(), 0)]),    // eighth note
        (Fraction::new(1u64, 16u64), vec![("16".to_string(), 0)]),  // sixteenth note
        (Fraction::new(1u64, 32u64), vec![("32".to_string(), 0)]),  // thirty-second note
        (Fraction::new(3u64, 8u64), vec![("q".to_string(), 1)]),    // dotted quarter
        (Fraction::new(3u64, 16u64), vec![("8".to_string(), 1)]),   // dotted eighth
        (Fraction::new(3u64, 32u64), vec![("16".to_string(), 1)]),  // dotted sixteenth
        (Fraction::new(7u64, 8u64), vec![("h".to_string(), 2)]),    // double dotted half
        (Fraction::new(7u64, 16u64), vec![("q".to_string(), 2)]),   // double dotted quarter
        (Fraction::new(7u64, 32u64), vec![("8".to_string(), 2)]),   // double dotted eighth
    ];
    
    // Check for direct match first
    for (lookup_frac, durations) in &lookup {
        if frac == *lookup_frac {
            return durations.clone();
        }
    }
    
    // Use shared rhythm decomposition logic - SAME AS LILYPOND
    let fraction_parts = crate::rhythm::RhythmConverter::decompose_fraction_to_standard_durations(frac);
    
    // Convert each fraction to VexFlow duration string
    fraction_parts.iter().map(|f| {
        match *f {
            f if f == Fraction::new(1u64, 1u64) => ("w".to_string(), 0),
            f if f == Fraction::new(1u64, 2u64) => ("h".to_string(), 0),
            f if f == Fraction::new(1u64, 4u64) => ("q".to_string(), 0),
            f if f == Fraction::new(1u64, 8u64) => ("8".to_string(), 0),
            f if f == Fraction::new(1u64, 16u64) => ("16".to_string(), 0),
            f if f == Fraction::new(1u64, 32u64) => ("32".to_string(), 0),
            _ => ("32".to_string(), 0), // Fallback
        }
    }).collect()
}

fn fraction_to_vexflow_duration(fraction_str: &str, _beat_divisions: usize) -> Result<(String, u8), String> {
    // Parse the fraction
    let frac = match Fraction::from_str(fraction_str) {
        Ok(f) => f,
        Err(_) => return Err(format!("Invalid fraction: {}", fraction_str)),
    };
    
    // Use the proper function and return first result
    let durations = fraction_to_vexflow_duration_proper(frac);
    if let Some((duration, dots)) = durations.first() {
        Ok((duration.clone(), *dots))
    } else {
        Ok(("q".to_string(), 0))
    }
}

fn find_closest_duration<'a>(target: Fraction, durations: &'a [(Fraction, &'a str, u8)]) -> &'a (Fraction, &'a str, u8) {
    durations.iter()
        .min_by_key(|(frac, _, _)| {
            let diff = if *frac > target {
                *frac - target
            } else {
                target - *frac
            };
            // Convert to a comparable integer by finding common denominator
            diff.numer().unwrap_or(&0) * 1000 / diff.denom().unwrap_or(&1)
        })
        .unwrap_or(&durations[4]) // Default to quarter note
}

fn pitch_to_vexflow_key(pitch_code: PitchCode, octave: Option<i8>) -> Result<(String, Option<String>), String> {
    // Get base note name
    let base_note = pitchcode_to_english_lilypond(pitch_code);
    
    // Extract note letter and accidental
    let (note_letter, accidental) = parse_lilypond_note(&base_note);
    
    // VexFlow octave (4 = middle C)
    let vexflow_octave = match octave.unwrap_or(0) {
        -2 => 2,
        -1 => 3,
        0 => 4,   // middle octave
        1 => 5,
        2 => 6,
        _ => 4,
    };
    
    let key = format!("{}/{}", note_letter, vexflow_octave);
    
    let vexflow_accidental = match accidental.as_str() {
        "s" => Some("#".to_string()),      // sharp
        "ss" => Some("##".to_string()),    // double sharp  
        "f" => Some("b".to_string()),      // flat
        "ff" => Some("bb".to_string()),    // double flat
        _ => None,
    };
    
    Ok((key, vexflow_accidental))
}

fn parse_lilypond_note(note: &str) -> (String, String) {
    // Extract base note letter and accidentals
    if note.is_empty() {
        return ("c".to_string(), String::new());
    }
    
    let first_char = note.chars().next().unwrap();
    let rest = &note[1..];
    
    (first_char.to_string(), rest.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accidental_table_f_major() {
        // F major has Bb in key signature
        let mut table = NoteAccidentalTable::new(Some(&"F".to_string()));
        
        // Bb should not need accidental (it's in key signature)
        let (show, symbol) = table.check_and_update_note("b", Some("b"));
        assert_eq!(show, false);
        assert_eq!(symbol, None);
        
        // B natural should need natural symbol
        let (show, symbol) = table.check_and_update_note("b", None);
        assert_eq!(show, true);
        assert_eq!(symbol, Some("n".to_string()));
        
        // Another B natural in same bar should not need accidental
        let (show, symbol) = table.check_and_update_note("b", None);
        assert_eq!(show, false);
        assert_eq!(symbol, None);
    }

    #[test]
    fn test_accidental_table_g_major() {
        // G major has F# in key signature
        let mut table = NoteAccidentalTable::new(Some(&"G".to_string()));
        
        // F# should not need accidental (it's in key signature)
        let (show, symbol) = table.check_and_update_note("f", Some("#"));
        assert_eq!(show, false);
        assert_eq!(symbol, None);
        
        // F natural should need natural symbol
        let (show, symbol) = table.check_and_update_note("f", None);
        assert_eq!(show, true);
        assert_eq!(symbol, Some("n".to_string()));
    }

    #[test]
    fn test_accidental_table_bar_reset() {
        let mut table = NoteAccidentalTable::new(Some(&"G".to_string()));
        
        // Modify F to natural in first bar
        let (show, symbol) = table.check_and_update_note("f", None);
        assert_eq!(show, true);
        assert_eq!(symbol, Some("n".to_string()));
        
        // Reset for new bar
        table.reset_for_new_bar(Some(&"G".to_string()));
        
        // F natural should need accidental again after bar reset
        let (show, symbol) = table.check_and_update_note("f", None);
        assert_eq!(show, true);
        assert_eq!(symbol, Some("n".to_string()));
    }

    #[test]
    fn test_accidental_table_c_major() {
        // C major has no accidentals in key signature
        let mut table = NoteAccidentalTable::new(Some(&"C".to_string()));
        
        // C natural should not need accidental
        let (show, symbol) = table.check_and_update_note("c", None);
        assert_eq!(show, false);
        assert_eq!(symbol, None);
        
        // C# should need sharp
        let (show, symbol) = table.check_and_update_note("c", Some("#"));
        assert_eq!(show, true);
        assert_eq!(symbol, Some("#".to_string()));
        
        // Another C# in same bar should not need accidental
        let (show, symbol) = table.check_and_update_note("c", Some("#"));
        assert_eq!(show, false);
        assert_eq!(symbol, None);
        
        // C natural should now need natural symbol
        let (show, symbol) = table.check_and_update_note("c", None);
        assert_eq!(show, true);
        assert_eq!(symbol, Some("n".to_string()));
    }
}

/// Extract the first syllable from SYL child nodes
fn extract_syllable_from_node(node: &Node) -> Option<String> {
    // Look for the first SYL child node
    for child in &node.nodes {
        if child.node_type == "SYL" && !child.value.trim().is_empty() {
            return Some(child.value.clone());
        }
    }
    None
}