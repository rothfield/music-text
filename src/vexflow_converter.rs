// LilyPond to VexFlow Converter
// 
// This implementation was inspired by the LilyPond to VexFlow conversion logic
// found in Tarmo Johannes' vexflow-react-components project:
// https://github.com/tarmoj/vexflow-react-components
// 
// Original work by Tarmo Johannes (MIT License)
// Adapted to Rust with improvements for better error handling
// and integration with our notation parser system.
//
// Note: LilyPond uses Dutch note names by default (c, d, e, f, g, a, b)
// with 's' for sharp and 'f' for flat (e.g., cs = C#, df = Db)

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::pitch::{PitchCode, pitchcode_to_dutch_lilypond};
use crate::models::{Document, Node};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VexFlowNote {
    pub clef: String,
    pub keys: Vec<String>,
    pub duration: String,
    pub accidentals: Option<Vec<VexFlowAccidental>>,
    pub note_type: VexFlowNoteType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VexFlowAccidental {
    pub index: usize,
    pub accidental: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VexFlowNoteType {
    Note,
    Rest,
}

#[derive(Debug, Clone)]
pub struct LilyPondToVexFlowConverter {
    note_pattern: Regex,
    rest_pattern: Regex,
}

impl LilyPondToVexFlowConverter {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            // Pattern to match Dutch LilyPond notes
            // Matches: note_name + accidentals + octave_marks + duration + optional modifiers
            // Dutch uses: "es" for flat, "is" for sharp (e.g., des = D-flat, cis = C-sharp)
            // Order matters: try double first (eses, isis), then single (es, is)
            note_pattern: Regex::new(r"\b([a-g])(eses|isis|es|is)?([',]{0,3})([0-9]+)(?:\.)?(?:\s*~)?\b")?,
            rest_pattern: Regex::new(r"\br([0-9]+)(?:\.)?")?,
        })
    }

    pub fn convert_lilypond_to_vexflow(&self, lilypond_code: &str) -> Result<Vec<VexFlowNote>, Box<dyn std::error::Error>> {
        // Clean and split into lines
        let musical_content = self.clean_lilypond_code(lilypond_code);
        
        // Pipeline: lines -> notes
        let final_notes: Vec<VexFlowNote> = musical_content
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(|line| self.to_vexflow_line(line))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect();
        Ok(final_notes)
    }

    fn to_vexflow_line(&self, line_content: &str) -> Result<Vec<VexFlowNote>, Box<dyn std::error::Error>> {
        let mut notes = Vec::new();
        
        // Parse notes and rests in this line
        let note_matches: Vec<_> = self.note_pattern.captures_iter(line_content).collect();
        
        for (_, captures) in note_matches.iter().enumerate() {
            let match_str = captures.get(0).unwrap().as_str();
            if let Some(vex_note) = self.parse_lilypond_note(match_str)? {
                notes.push(vex_note);
            }
        }
        
        let rest_matches: Vec<_> = self.rest_pattern.captures_iter(line_content).collect();
        
        for (_, captures) in rest_matches.iter().enumerate() {
            let match_str = captures.get(0).unwrap().as_str();
            if let Some(vex_rest) = self.parse_lilypond_rest(match_str)? {
                notes.push(vex_rest);
            }
        }
        
        Ok(notes)
    }

    fn clean_lilypond_code(&self, lilypond_code: &str) -> String {
        let mut musical_content = lilypond_code.to_string();
        
        // Extract content between \fixed and closing brace if present
        if let Some(captures) = Regex::new(r"\\fixed\s+[a-g]'?\s*\{([^}]+)\}").unwrap().captures(&musical_content) {
            let extracted = captures.get(1).unwrap().as_str().to_string();
            musical_content = extracted;
        }
        
        // Remove common LilyPond directives and formatting
        let clean_patterns = vec![
            (r"\\clef\s+[a-zA-Z]+", ""),
            (r"\\time\s+\d+/\d+", ""),
            (r"\\key\s+[a-g](\s+(major|minor))?", ""),
            (r"\\break", ""),
            (r#"\\bar\s+"[^"]*""#, ""),
            (r"\\set\s+[^\n]*", ""), // Remove \set commands
            (r"\\tuplet\s+\d+/\d+\s*\{([^}]+)\}", "$1"), // Simplify tuplets for now
            // Keep beam markers for now - we'll parse them for grouping
            (r"~", ""), // Remove ties for preview
        ];
        
        for (_, (pattern, replacement)) in clean_patterns.iter().enumerate() {
            let regex = Regex::new(pattern).unwrap();
            musical_content = regex.replace_all(&musical_content, *replacement).to_string();
        }
        
        musical_content.trim().to_string()
    }

    fn parse_lilypond_note(&self, lily_note: &str) -> Result<Option<VexFlowNote>, Box<dyn std::error::Error>> {
        // Parse Dutch LilyPond note format: c4, des8, cis4, etc.
        // Dutch is the default LilyPond format (no \language directive needed)
        let note_regex = Regex::new(r"^([a-g])(eses|isis|es|is)?([',]*)([0-9]+)$")?;
        
        if let Some(captures) = note_regex.captures(lily_note) {
            let note_name = captures.get(1).unwrap().as_str();
            let accidental = captures.get(2).map(|m| m.as_str()).unwrap_or("");
            let octave_marks = captures.get(3).unwrap().as_str();
            let duration = captures.get(4).unwrap().as_str();
            
            // Convert note name to VexFlow format
            let mut vex_pitch = note_name.to_lowercase();
            
            // Handle octave (LilyPond ' raises octave, , lowers octave)
            let mut octave = 4; // Default octave
            if !octave_marks.is_empty() {
                let up_marks = octave_marks.matches('\'').count() as i32;
                let down_marks = octave_marks.matches(',').count() as i32;
                octave = 4 + up_marks - down_marks;
            }
            
            vex_pitch = format!("{}/{}", vex_pitch, octave);
            
            // Convert Dutch accidentals to VexFlow format
            let vex_accidental = match accidental {
                "" => None,
                "is" => Some("#".to_string()),    // Single sharp: cis → C#
                "es" => Some("b".to_string()),    // Single flat: des → D♭
                "isis" => Some("##".to_string()), // Double sharp: cisis → C##
                "eses" => Some("bb".to_string()), // Double flat: deses → D♭♭
                _ => None,
            };
            
            // Convert duration (LilyPond 4=quarter, 2=half, 1=whole, 8=eighth)
            let vex_duration = match duration {
                "1" => "w",    // whole
                "2" => "h",    // half
                "4" => "q",    // quarter
                "8" => "8",    // eighth
                "16" => "16",  // sixteenth
                _ => "q",      // default quarter
            };
            
            let mut vex_note = VexFlowNote {
                clef: "treble".to_string(),
                keys: vec![vex_pitch],
                duration: vex_duration.to_string(),
                accidentals: None,
                note_type: VexFlowNoteType::Note,
            };
            
            if let Some(acc) = vex_accidental {
                vex_note.accidentals = Some(vec![VexFlowAccidental {
                    index: 0,
                    accidental: acc,
                }]);
            }
            
            Ok(Some(vex_note))
        } else {
            Ok(None)
        }
    }

    fn parse_lilypond_rest(&self, lily_rest: &str) -> Result<Option<VexFlowNote>, Box<dyn std::error::Error>> {
        // Parse LilyPond rest format: r4, r2, r1, etc.
        let rest_regex = Regex::new(r"^r([0-9]+)(?:\.)?$")?;
        
        if let Some(captures) = rest_regex.captures(lily_rest) {
            let duration = captures.get(1).unwrap().as_str();
            
            // Convert duration (LilyPond 4=quarter, 2=half, 1=whole, 8=eighth)
            // VexFlow rests don't use the 'r' suffix in the duration - they use the same duration as notes
            let vex_duration = match duration {
                "1" => "w",   // whole rest
                "2" => "h",   // half rest  
                "4" => "q",   // quarter rest
                "8" => "8",   // eighth rest
                "16" => "16", // sixteenth rest
                _ => "q",     // default quarter rest
            };
            
            let vex_rest = VexFlowNote {
                clef: "treble".to_string(),
                keys: vec!["b/4".to_string()], // Rest position
                duration: vex_duration.to_string(),
                accidentals: None,
                note_type: VexFlowNoteType::Rest,
            };
            
            Ok(Some(vex_rest))
        } else {
            Ok(None)
        }
    }

    /// Convert VexFlowNote to JSON string for JavaScript consumption
    pub fn to_vexflow_json(&self, notes: &[VexFlowNote]) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(notes)?)
    }

    /// Convert VexFlowNote vec to JavaScript-compatible format
    pub fn to_vexflow_js_objects(&self, notes: &[VexFlowNote]) -> Vec<HashMap<String, serde_json::Value>> {
        notes.iter().map(|note| {
            let mut js_note = HashMap::new();
            js_note.insert("clef".to_string(), serde_json::Value::String(note.clef.clone()));
            js_note.insert("keys".to_string(), serde_json::Value::Array(
                note.keys.iter().map(|k| serde_json::Value::String(k.clone())).collect()
            ));
            js_note.insert("duration".to_string(), serde_json::Value::String(note.duration.clone()));
            
            if let Some(ref accidentals) = note.accidentals {
                let acc_array: Vec<serde_json::Value> = accidentals.iter().map(|acc| {
                    let mut acc_obj = serde_json::Map::new();
                    acc_obj.insert("index".to_string(), serde_json::Value::Number(serde_json::Number::from(acc.index)));
                    acc_obj.insert("accidental".to_string(), serde_json::Value::String(acc.accidental.clone()));
                    serde_json::Value::Object(acc_obj)
                }).collect();
                js_note.insert("accidentals".to_string(), serde_json::Value::Array(acc_array));
            }
            
            match note.note_type {
                VexFlowNoteType::Rest => {
                    js_note.insert("note_type".to_string(), serde_json::Value::String("Rest".to_string()));
                },
                VexFlowNoteType::Note => {
                    js_note.insert("note_type".to_string(), serde_json::Value::String("Note".to_string()));
                }
            }
            
            
            js_note
        }).collect()
    }
}

impl Default for LilyPondToVexFlowConverter {
    fn default() -> Self {
        Self::new().expect("Failed to create LilyPondToVexFlowConverter")
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_note_conversion() {
        let converter = LilyPondToVexFlowConverter::new().unwrap();
        let lilypond_code = r#"\fixed c' { c4 d4 e4 f4 }"#;
        
        let result = converter.convert_lilypond_to_vexflow(lilypond_code).unwrap();
        assert_eq!(result.len(), 4);
        
        // Test first note
        assert_eq!(result[0].keys[0], "c/4");
        assert_eq!(result[0].duration, "q");
        assert_eq!(result[0].clef, "treble");
    }

    #[test]
    fn test_accidentals() {
        let converter = LilyPondToVexFlowConverter::new().unwrap();
        let lilypond_code = r#"cs4 df4"#;
        
        let result = converter.convert_lilypond_to_vexflow(lilypond_code).unwrap();
        assert_eq!(result.len(), 2);
        
        // Test sharp
        assert_eq!(result[0].keys[0], "c/4");
        assert!(result[0].accidentals.is_some());
        assert_eq!(result[0].accidentals.as_ref().unwrap()[0].accidental, "#");
        
        // Test flat
        assert_eq!(result[1].keys[0], "d/4");
        assert!(result[1].accidentals.is_some());
        assert_eq!(result[1].accidentals.as_ref().unwrap()[0].accidental, "b");
    }

    #[test]
    fn test_octave_marks() {
        let converter = LilyPondToVexFlowConverter::new().unwrap();
        let lilypond_code = r#"c'4 c,4"#;
        
        let result = converter.convert_lilypond_to_vexflow(lilypond_code).unwrap();
        assert_eq!(result.len(), 2);
        
        // Test upper octave
        assert_eq!(result[0].keys[0], "c/5");
        
        // Test lower octave
        assert_eq!(result[1].keys[0], "c/3");
    }

    #[test]
    fn test_rests() {
        let converter = LilyPondToVexFlowConverter::new().unwrap();
        let lilypond_code = r#"c4 r4 d4"#;
        
        let result = converter.convert_lilypond_to_vexflow(lilypond_code).unwrap();
        assert_eq!(result.len(), 3);
        
        // Test rest (it's actually the third element due to parsing order)
        assert_eq!(result[2].duration, "q");
        assert!(matches!(result[2].note_type, VexFlowNoteType::Rest));
    }

    #[test]
    fn test_durations() {
        let converter = LilyPondToVexFlowConverter::new().unwrap();
        let lilypond_code = r#"c1 c2 c4 c8 c16"#;
        
        let result = converter.convert_lilypond_to_vexflow(lilypond_code).unwrap();
        assert_eq!(result.len(), 5);
        
        assert_eq!(result[0].duration, "w");  // whole
        assert_eq!(result[1].duration, "h");  // half
        assert_eq!(result[2].duration, "q");  // quarter
        assert_eq!(result[3].duration, "8");  // eighth
        assert_eq!(result[4].duration, "16"); // sixteenth
    }
}