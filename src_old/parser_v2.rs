// Fresh grammar-inspired parser - built from scratch
// Based on doremi-script's successful stave-centric approach

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::models::{ParsedElement, Position, Degree};

#[derive(Debug, Clone, PartialEq)]
enum MusicalToken {
    Note { pitch: String, accidental: Option<Accidental> },
    Dash,
    Barline(String),
    Whitespace,
}

// =============================================================================
// CORE DATA TYPES - Grammar-inspired structure
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub directives: HashMap<String, String>,  // key: C, time: 4/4, etc.
    pub staves: Vec<Stave>,                   // Complete musical units
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stave {
    pub upper_annotations: Vec<String>,       // Lines above content
    pub content: ContentLine,                 // The actual music
    pub lower_annotations: Vec<String>,       // Lines below content  
    lyrics: Vec<LyricsLine>,                 // Private - consumed during parsing
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentLine {
    pub line_number: Option<u32>,            // Optional 1), 2), etc.
    pub elements: Vec<MusicalElement>,       // Notes, rests, barlines
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MusicalElement {
    Note { 
        pitch: String,      // S, R, G, 1, 2, 3, C, D, E, etc.
        accidental: Option<Accidental>, // #, b
        octave: i8,         // Calculated from markers
    },
    Rest,
    Dash,                   // Note extension
    Barline(String),        // |, ||, :|, etc.
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Accidental {
    Sharp,    // #
    Flat,     // b
}

// =============================================================================
// FSM-ENHANCED DATA TYPES - With rhythm analysis
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentWithFSM {
    pub directives: HashMap<String, String>,
    pub staves: Vec<StaveWithFSM>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]  
pub struct StaveWithFSM {
    pub upper_annotations: Vec<String>,
    pub lower_annotations: Vec<String>,
    pub lyrics: Vec<LyricsLine>,
    #[serde(skip)] // Skip serialization of FSM output for now
    pub fsm_output: Vec<crate::parser_v2_fsm::Item>, // FSM-processed musical elements with rhythm
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LyricsLine {
    pub syllables: Vec<String>,
}

// =============================================================================
// MAIN PARSER FUNCTION
// =============================================================================

pub fn parse(input: &str) -> Result<Document, ParseError> {
    let mut parser = Parser::new(input);
    parser.parse_document()
}

/// Parse input text and process through FSM for rhythm analysis
pub fn parse_with_fsm(input: &str) -> Result<DocumentWithFSM, ParseError> {
    // First do the normal V2 parsing
    let document = parse(input)?;
    
    // Convert each stave's musical elements to ParsedElements and run through FSM
    let mut staves_with_fsm = Vec::new();
    
    for (stave_idx, stave) in document.staves.iter().enumerate() {
        // Get syllables before conversion (they were assigned to notes during parsing)  
        let syllables_for_conversion = stave.get_syllables_for_conversion();
        
        // Convert V2 MusicalElements to ParsedElements with syllable assignment
        let parsed_elements = convert_musical_elements_to_parsed_with_syllables(
            &stave.content.elements, 
            stave_idx, 
            &syllables_for_conversion
        );
        
        // Run through FSM for rhythm analysis  
        let fsm_items = crate::parser_v2_fsm::group_elements_with_fsm_full(&parsed_elements, &[0]);
        
        let stave_with_fsm = StaveWithFSM {
            upper_annotations: stave.upper_annotations.clone(),
            lower_annotations: stave.lower_annotations.clone(),
            lyrics: vec![], // Empty - lyrics were consumed during parsing
            fsm_output: fsm_items,
        };
        
        staves_with_fsm.push(stave_with_fsm);
    }
    
    Ok(DocumentWithFSM {
        directives: document.directives,
        staves: staves_with_fsm,
    })
}

/// Convert V2 MusicalElements to ParsedElements with syllable assignment for FSM processing
fn convert_musical_elements_to_parsed_with_syllables(
    elements: &[MusicalElement], 
    line_num: usize, 
    syllables: &[String]
) -> Vec<ParsedElement> {
    let mut parsed_elements = Vec::new();
    let mut syllable_idx = 0;
    
    for (col, element) in elements.iter().enumerate() {
        let position = Position { row: line_num, col };
        
        let parsed_element = match element {
            MusicalElement::Note { pitch, accidental: _, octave } => {
                // Convert pitch string to Degree
                let degree = pitch_to_degree(pitch);
                
                // Assign syllable if available
                let mut children = vec![];
                if syllable_idx < syllables.len() {
                    children.push(crate::models::parsed::ParsedChild::Syllable {
                        text: syllables[syllable_idx].clone(),
                        distance: 1, // Below the note
                    });
                    syllable_idx += 1;
                }
                
                ParsedElement::Note {
                    degree,
                    octave: *octave,
                    value: pitch.clone(),
                    position,
                    children,         // Now includes syllables from lyrics lines
                    duration: None,   // FSM will calculate this
                    slur: None,       // No slur support in V2 parser for now
                }
            },
            
            MusicalElement::Rest => {
                ParsedElement::Rest {
                    value: "-".to_string(),
                    position,
                    duration: None,
                }
            },
            
            MusicalElement::Dash => {
                ParsedElement::Dash {
                    degree: None,     // FSM will inherit from previous note
                    octave: None,     // FSM will inherit from previous note
                    position,
                    duration: None,
                }
            },
            
            MusicalElement::Barline(style) => {
                ParsedElement::Barline {
                    style: style.clone(),
                    position,
                    tala: None,
                }
            },
        };
        
        parsed_elements.push(parsed_element);
    }
    
    parsed_elements
}

/// Convert pitch string to Degree enum
fn pitch_to_degree(pitch: &str) -> Degree {
    match pitch.to_uppercase().as_str() {
        // Number notation
        "1" => Degree::N1,
        "2" => Degree::N2,
        "3" => Degree::N3,
        "4" => Degree::N4,
        "5" => Degree::N5,
        "6" => Degree::N6,
        "7" => Degree::N7,
        // Sargam notation
        "S" => Degree::N1,
        "R" => Degree::N2,
        "G" => Degree::N3,
        "M" => Degree::N4,
        "P" => Degree::N5,
        "D" => Degree::N6, // Dha in sargam
        "N" => Degree::N7,
        // Western notation (basic mapping to C major)
        "C" => Degree::N1,
        "E" => Degree::N3,
        "F" => Degree::N4,
        "B" => Degree::N7,
        _ => Degree::N1, // Default fallback
    }
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error at line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ParseError {}

// =============================================================================
// PARSER IMPLEMENTATION
// =============================================================================

struct Parser {
    lines: Vec<String>,
    position: usize,
}

impl Parser {
    fn new(input: &str) -> Self {
        Self {
            lines: input.lines().map(|s| s.to_string()).collect(),
            position: 0,
        }
    }

    fn parse_document(&mut self) -> Result<Document, ParseError> {
        let mut document = Document {
            directives: HashMap::new(),
            staves: Vec::new(),
        };

        // Phase 1: Parse directives at the beginning
        self.parse_directives(&mut document)?;

        // Phase 2: Parse staves
        while !self.is_at_end() {
            self.skip_empty_lines();
            
            if !self.is_at_end() {
                let stave = self.parse_stave()?;
                document.staves.push(stave);
            }
        }

        Ok(document)
    }

    fn parse_directives(&mut self, document: &mut Document) -> Result<(), ParseError> {
        while !self.is_at_end() {
            let line = self.current_line();
            
            if self.is_directive(line) {
                let (key, value) = self.parse_directive(line)?;
                document.directives.insert(key, value);
                self.advance();
            } else if line.trim().is_empty() {
                self.advance();  // Skip empty lines
            } else {
                break;  // Hit non-directive content, stop parsing directives
            }
        }
        Ok(())
    }

    fn parse_stave(&mut self) -> Result<Stave, ParseError> {
        let mut upper_annotations = Vec::new();
        let mut lower_annotations = Vec::new();
        let mut lyrics = Vec::new();

        // Phase 1: Collect all lines for this stave (until empty line or EOF)
        let start_pos = self.position;
        let stave_lines = self.collect_stave_lines();
        
        if stave_lines.is_empty() {
            return Err(ParseError {
                message: "Empty stave".to_string(),
                line: start_pos,
            });
        }

        // Phase 2: Find the content line (contains musical notation)
        let content_idx = self.find_content_line(&stave_lines)?;
        
        // Phase 3: Parse each section
        // Lines above content = upper annotations
        for i in 0..content_idx {
            let line = &stave_lines[i];
            if !line.trim().is_empty() {
                upper_annotations.push(line.clone());
            }
        }

        // Parse the content line itself
        let content = self.parse_content_line(&stave_lines[content_idx], start_pos + content_idx)?;

        // Lines below content = lower annotations and lyrics
        for i in (content_idx + 1)..stave_lines.len() {
            let line = &stave_lines[i];
            if !line.trim().is_empty() {
                if self.is_lyrics_line(line) {
                    lyrics.push(self.parse_lyrics_line(line));
                } else {
                    lower_annotations.push(line.clone());
                }
            }
        }

        // Create stave with raw data
        let mut stave = Stave {
            upper_annotations,
            content,
            lower_annotations,
            lyrics,
        };

        // CRITICAL: Assign lyrics to notes and consume lyrics field
        stave.assign_lyrics();

        Ok(stave)
    }

    fn collect_stave_lines(&mut self) -> Vec<String> {
        let mut lines = Vec::new();
        
        while !self.is_at_end() {
            let line = self.current_line();
            
            // Stop at empty line (stave boundary) 
            if line.trim().is_empty() {
                self.advance(); // Consume the empty line
                break;
            }
            
            lines.push(line.to_string());
            self.advance();
        }
        
        lines
    }

    fn find_content_line(&self, lines: &[String]) -> Result<usize, ParseError> {
        for (i, line) in lines.iter().enumerate() {
            if self.is_content_line(line) {
                return Ok(i);
            }
        }
        
        Err(ParseError {
            message: "No musical content line found in stave".to_string(),
            line: self.position,
        })
    }

    fn parse_content_line(&self, line: &str, line_num: usize) -> Result<ContentLine, ParseError> {
        let trimmed = line.trim();
        let mut chars = trimmed.chars().peekable();
        let mut elements = Vec::new();
        let mut line_number = None;

        // Check for line number at the beginning (like "1) ")
        if let Some(num) = self.try_parse_line_number(&mut chars) {
            line_number = Some(num);
        }

        // Parse musical elements using handwritten musical tokenizer
        let remaining_text = chars.collect::<String>();
        let tokens = self.tokenize_musical_content(&remaining_text);
        
        for token in tokens {
            match token {
                MusicalToken::Note { pitch, accidental } => {
                    let octave = self.calculate_octave(line, &pitch);
                    elements.push(MusicalElement::Note { pitch, accidental, octave });
                },
                MusicalToken::Dash => {
                    elements.push(MusicalElement::Dash);
                },
                MusicalToken::Barline(barline_str) => {
                    elements.push(MusicalElement::Barline(barline_str));
                },
                MusicalToken::Whitespace => {
                    // Skip whitespace tokens
                },
            }
        }

        if elements.is_empty() {
            return Err(ParseError {
                message: "No musical elements found in content line".to_string(),
                line: line_num,
            });
        }

        Ok(ContentLine {
            line_number,
            elements,
        })
    }

    fn parse_lyrics_line(&self, line: &str) -> LyricsLine {
        let syllables = line
            .trim()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        LyricsLine { syllables }
    }

    // =============================================================================
    // HELPER FUNCTIONS - Grammar-inspired classification
    // =============================================================================

    fn is_directive(&self, line: &str) -> bool {
        let trimmed = line.trim();
        // Look for "word: value" pattern
        if let Some(colon_pos) = trimmed.find(':') {
            let before_colon = trimmed[..colon_pos].trim();
            // Check if it's a single word followed by colon
            !before_colon.is_empty() && !before_colon.contains(' ')
        } else {
            false
        }
    }

    fn parse_directive(&self, line: &str) -> Result<(String, String), ParseError> {
        let trimmed = line.trim();
        if let Some(colon_pos) = trimmed.find(':') {
            let key = trimmed[..colon_pos].trim().to_string();
            let value = trimmed[colon_pos + 1..].trim().to_string();
            Ok((key, value))
        } else {
            Err(ParseError {
                message: format!("Invalid directive format: {}", line),
                line: self.position,
            })
        }
    }

    fn is_content_line(&self, line: &str) -> bool {
        let trimmed = line.trim();
        
        // Must contain musical elements (pitches or barlines)
        // Look for pitches: S R G M P D N (sargam) or 1-7 (numbers) or C D E F G A B (western)
        let has_pitches = trimmed.chars().any(|c| self.is_pitch_character(c));
        let has_barlines = trimmed.contains('|');
        
        has_pitches || has_barlines
    }

    fn is_pitch_character(&self, ch: char) -> bool {
        matches!(ch, 
            // Sargam notation
            'S' | 'R' | 'G' | 'M' | 'P' | 'D' | 'N' |
            // Number notation  
            '1' | '2' | '3' | '4' | '5' | '6' | '7' |
            // Western notation (avoiding overlap with sargam)
            'C' | 'E' | 'F' | 'A' | 'B'
        )
    }

    fn is_lyrics_line(&self, line: &str) -> bool {
        let trimmed = line.trim();
        // Simple heuristic: contains alphabetic characters but no musical symbols
        !trimmed.is_empty() && 
        trimmed.chars().any(|c| c.is_alphabetic()) &&
        !trimmed.contains('|') &&
        !trimmed.chars().any(|c| self.is_pitch_character(c))
    }

    fn try_parse_line_number(&self, chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<u32> {
        let mut num_str = String::new();
        
        // Collect digits
        while let Some(&ch) = chars.peek() {
            if ch.is_ascii_digit() {
                num_str.push(chars.next().unwrap());
            } else {
                break;
            }
        }

        // Check for closing parenthesis
        if !num_str.is_empty() {
            if let Some(&')') = chars.peek() {
                chars.next(); // consume ')'
                // Skip any following whitespace
                while let Some(&ch) = chars.peek() {
                    if ch.is_whitespace() {
                        chars.next();
                    } else {
                        break;
                    }
                }
                return num_str.parse().ok();
            }
        }

        None
    }

    fn calculate_octave(&self, line: &str, pitch: &str) -> i8 {
        // Find the position of this pitch in the line
        let pitch_pos = if let Some(pos) = line.find(pitch) {
            pos
        } else {
            return 0; // Default octave if pitch not found
        };

        let line_chars: Vec<char> = line.chars().collect();
        let mut octave_adjustment = 0i8;

        // Look for octave markers around this pitch position
        // Check positions before and after the pitch
        let search_range = std::cmp::max(0, pitch_pos as i32 - 3) as usize..
                          std::cmp::min(line_chars.len(), pitch_pos + 3);

        for i in search_range {
            if i < line_chars.len() {
                match line_chars[i] {
                    '\'' => octave_adjustment += 1,  // Apostrophe = higher octave
                    '.' => octave_adjustment -= 1,   // Dot = lower octave  
                    '*' => octave_adjustment += 1,   // Asterisk = higher octave
                    _ => continue,
                }
            }
        }

        octave_adjustment
    }

    // =============================================================================
    // UTILITY FUNCTIONS
    // =============================================================================

    fn current_line(&self) -> &str {
        if self.position < self.lines.len() {
            &self.lines[self.position]
        } else {
            ""
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.lines.len()
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.position += 1;
        }
    }

    fn skip_empty_lines(&mut self) {
        while !self.is_at_end() && self.current_line().trim().is_empty() {
            self.advance();
        }
    }

    // =============================================================================
    // MUSICAL TOKENIZER - Grammar-inspired implementation  
    // =============================================================================
    
    /// Tokenize musical content using grammar-inspired patterns
    /// Handles complex cases like "CBb" → ["C", "Bb"], proper whitespace separation
    fn tokenize_musical_content(&self, input: &str) -> Vec<MusicalToken> {
        let mut tokens = Vec::new();
        let mut chars = input.chars().peekable();
        
        while let Some(ch) = chars.peek() {
            match ch {
                // Whitespace - preserve as structural separator
                ' ' | '\t' => {
                    self.consume_whitespace(&mut chars);
                    tokens.push(MusicalToken::Whitespace);
                }
                // Barlines - must be handled before pitch patterns
                '|' => {
                    if let Some(barline) = self.consume_barline(&mut chars) {
                        tokens.push(MusicalToken::Barline(barline));
                    }
                }
                // Dash - note extension
                '-' => {
                    chars.next(); // consume
                    tokens.push(MusicalToken::Dash);
                }
                // Musical pitches - complex pattern matching
                _ if self.is_pitch_character(*ch) => {
                    if let Some(pitch_token) = self.consume_pitch_token(&mut chars) {
                        tokens.push(pitch_token);
                    }
                }
                // Skip unknown characters for now
                _ => {
                    chars.next();
                }
            }
        }
        
        tokens
    }
    
    fn consume_whitespace(&self, chars: &mut std::iter::Peekable<std::str::Chars>) {
        while let Some(' ') | Some('\t') = chars.peek() {
            chars.next();
        }
    }
    
    fn consume_barline(&self, chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<String> {
        let mut barline = String::new();
        
        // First |
        if chars.peek() == Some(&'|') {
            barline.push(chars.next().unwrap());
        } else {
            return None;
        }
        
        // Possible second | for ||
        if chars.peek() == Some(&'|') {
            barline.push(chars.next().unwrap());
        }
        
        // Possible : for |: or :|
        if chars.peek() == Some(&':') {
            barline.push(chars.next().unwrap());
        } else if barline == "|" && chars.peek() == Some(&'.') {
            // |. final barline  
            barline.push(chars.next().unwrap());
        }
        
        Some(barline)
    }
    
    fn consume_pitch_token(&self, chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<MusicalToken> {
        let mut pitch = String::new();
        let mut accidental = None;
        
        // Step 1: Try to consume a base pitch character
        if let Some(&ch) = chars.peek() {
            if self.is_base_pitch_character(ch) {
                pitch.push(chars.next().unwrap());
            } else {
                return None;
            }
        }
        
        // Step 2: Look for accidentals (# or b) - grammar shows accidentals can be single or double
        let mut accidental_str = String::new();
        
        // Check for first accidental
        if let Some(&'#') = chars.peek() {
            accidental_str.push(chars.next().unwrap());
            // Check for second # (double sharp)
            if let Some(&'#') = chars.peek() {
                accidental_str.push(chars.next().unwrap());
            }
        } else if let Some(&'b') = chars.peek() {
            accidental_str.push(chars.next().unwrap());
            // Check for second b (double flat)  
            if let Some(&'b') = chars.peek() {
                accidental_str.push(chars.next().unwrap());
            }
        }
        
        // Convert accidental string to enum
        if !accidental_str.is_empty() {
            accidental = match accidental_str.as_str() {
                "#" | "##" => Some(Accidental::Sharp),  // Single or double sharp
                "b" | "bb" => Some(Accidental::Flat),   // Single or double flat
                _ => None,
            };
        }
        
        Some(MusicalToken::Note { pitch, accidental })
    }
    
    fn is_base_pitch_character(&self, ch: char) -> bool {
        // All possible base pitch characters from grammar analysis
        matches!(ch, 
            // Number notation
            '1' | '2' | '3' | '4' | '5' | '6' | '7' |
            // Sargam notation (both cases) - includes overlapping Western letters
            'S' | 's' | 'R' | 'r' | 'G' | 'g' | 'M' | 'm' | 
            'P' | 'p' | 'D' | 'd' | 'N' | 'n' |
            // Western notation (non-overlapping with sargam)
            'C' | 'E' | 'F' | 'A' | 'B'
        )
    }
}

// =============================================================================
// CONVENIENCE FUNCTIONS
// =============================================================================

impl Document {
    pub fn new() -> Self {
        Self {
            directives: HashMap::new(),
            staves: Vec::new(),
        }
    }

    pub fn get_directive(&self, key: &str) -> Option<&String> {
        self.directives.get(key)
    }
}

impl Stave {
    /// Create a new stave with processed content (lyrics already assigned)
    pub fn new_processed(
        upper_annotations: Vec<String>,
        content: ContentLine,
        lower_annotations: Vec<String>,
    ) -> Self {
        Stave {
            upper_annotations,
            content,
            lower_annotations,
            lyrics: vec![], // Empty - lyrics were consumed during parsing
        }
    }

    /// Assign syllables from lyrics lines to notes, then consume/clear lyrics
    fn assign_lyrics(&mut self) {
        if self.lyrics.is_empty() {
            return;
        }

        // Get all syllables from first lyrics line (for now)
        let syllables = if let Some(first_line) = self.lyrics.first() {
            first_line.syllables.clone()
        } else {
            return;
        };

        let mut syllable_idx = 0;

        // Assign syllables to notes in content line
        for element in &mut self.content.elements {
            if let MusicalElement::Note { pitch: _, accidental: _, octave: _ } = element {
                if syllable_idx < syllables.len() {
                    // TODO: Attach syllable to note as ParsedChild::Syllable when converting to ParsedElement
                    // For now, this is a placeholder - actual syllable attachment happens during conversion
                    syllable_idx += 1;
                }
            }
        }

        // CRITICAL: Clear lyrics field after processing - they are now consumed
        self.lyrics.clear();
    }

    /// Get current syllables for conversion (temporary method until proper attachment)
    pub fn get_syllables_for_conversion(&self) -> Vec<String> {
        if let Some(first_line) = self.lyrics.first() {
            first_line.syllables.clone()
        } else {
            Vec::new()
        }
    }

    /// Get count of singable notes (for lyrics assignment)
    pub fn singable_note_count(&self) -> usize {
        self.content.elements.iter()
            .filter(|e| matches!(e, MusicalElement::Note { .. }))
            .count()
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_document() {
        let input = r#"
key: C
time: 4/4

| S R G M |
  do re mi fa
"#;

        let doc = parse(input).unwrap();
        
        assert_eq!(doc.directives.get("key"), Some(&"C".to_string()));
        assert_eq!(doc.directives.get("time"), Some(&"4/4".to_string()));
        assert_eq!(doc.staves.len(), 1);
        
        let stave = &doc.staves[0];
        assert_eq!(stave.content.elements.len(), 6); // | S R G M |
        assert_eq!(stave.lyrics.len(), 1);
        assert_eq!(stave.lyrics[0].syllables, vec!["do", "re", "mi", "fa"]);
    }

    #[test]
    fn test_parse_content_line() {
        let mut parser = Parser::new("");
        let result = parser.parse_content_line("| S R G M |", 0).unwrap();
        
        assert_eq!(result.elements.len(), 6);
        assert!(matches!(result.elements[0], MusicalElement::Barline(_)));
        assert!(matches!(result.elements[1], MusicalElement::Note { .. }));
        assert!(matches!(result.elements[5], MusicalElement::Barline(_)));
    }

    #[test]
    fn test_parse_with_line_number() {
        let mut parser = Parser::new("");
        let result = parser.parse_content_line("1) | S R G M |", 0).unwrap();
        
        assert_eq!(result.line_number, Some(1));
        assert_eq!(result.elements.len(), 6);
    }

    #[test]
    fn test_directive_parsing() {
        let input = r#"
key: D  
time: 3/4
title: Simple Song

| S R G |
"#;

        let doc = parse(input).unwrap();
        
        assert_eq!(doc.directives.len(), 3);
        assert_eq!(doc.directives.get("key"), Some(&"D".to_string()));
        assert_eq!(doc.directives.get("time"), Some(&"3/4".to_string()));
        assert_eq!(doc.directives.get("title"), Some(&"Simple Song".to_string()));
    }

    #[test]
    fn test_multiple_staves() {
        let input = r#"
| S R |
  do re

| G M |
  mi fa
"#;

        let doc = parse(input).unwrap();
        assert_eq!(doc.staves.len(), 2);
        assert_eq!(doc.staves[0].lyrics[0].syllables, vec!["do", "re"]);
        assert_eq!(doc.staves[1].lyrics[0].syllables, vec!["mi", "fa"]);
    }
}