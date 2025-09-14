use crate::parse::model::{Document, Stave, TextLine, Source, NotationSystem, Position as ModelPosition};
use crate::rhythm::types::{ParsedElement, Degree, Position};

/// Parse error for recursive descent parser
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Parse error at line {}, column {}: {}", self.line, self.column, self.message)
    }
}

impl std::error::Error for ParseError {}

impl From<ParseError> for String {
    fn from(error: ParseError) -> Self {
        error.to_string()
    }
}

/// Result type for recursive descent parsing - returns parsed value and new position
type ParseResult<T> = Result<(T, usize), ParseError>;

/// Detect notation system from a line of text
fn detect_line_notation_system(line: &str) -> NotationSystem {
    // Check for Western notation (C D E F G A B)
    if line.chars().any(|c| matches!(c, 'C' | 'D' | 'E' | 'F' | 'G' | 'A' | 'B')) {
        return NotationSystem::Western;
    }
    
    // Check for Sargam notation (S R G M P D N)
    if line.chars().any(|c| matches!(c, 'S' | 'R' | 'G' | 'M' | 'P' | 'D' | 'N')) {
        return NotationSystem::Sargam;
    }
    
    // Default to Number notation
    NotationSystem::Number
}

/// Check if a line contains musical content (has barlines or 3+ musical elements)
fn is_content_line(line: &str) -> bool {
    let trimmed = line.trim();
    eprintln!("DEBUG: is_content_line checking: '{}'", trimmed);

    // Must contain barlines or have significant musical content
    if trimmed.contains('|') {
        eprintln!("DEBUG: Found barline, treating as content line");
        return true;
    }
    
    // Count musical characters
    let musical_chars = trimmed.chars().filter(|&c| {
        c.is_ascii_digit() || // Numbers 1-7
        matches!(c, 'S' | 'R' | 'G' | 'M' | 'P' | 'D' | 'N') || // Sargam  
        matches!(c, 'C' | 'D' | 'E' | 'F' | 'G' | 'A' | 'B') || // Western
        c == '-' // Dashes
    }).count();
    
    musical_chars >= 3
}

/// Parse entire document using recursive descent
pub fn parse_document(input: &str) -> Result<Document, ParseError> {
    let (document, final_pos) = parse_document_internal(input, 0)?;
    
    // Ensure we consumed entire input
    if final_pos != input.len() {
        return Err(ParseError {
            message: format!("Unexpected content at position {}", final_pos),
            line: 1, // TODO: calculate actual line
            column: final_pos,
        });
    }
    
    Ok(document)
}

/// Internal document parsing function
fn parse_document_internal(input: &str, mut pos: usize) -> ParseResult<Document> {
    let mut staves = Vec::new();
    let mut document_lines = Vec::new();
    
    while pos < input.len() {
        // Try to parse different document elements
        if let Ok((stave, new_pos)) = parse_stave(input, pos) {
            staves.push(stave);
            pos = new_pos;
        } else if let Ok((empty_line, new_pos)) = parse_empty_line(input, pos) {
            document_lines.push(crate::parse::model::DocumentLine::EmptyLine(empty_line));
            pos = new_pos;
        } else {
            // Skip unknown character for now (TODO: proper error handling)
            pos += 1;
        }
    }
    
    Ok((Document {
        directives: Vec::new(),
        staves,
        document_lines,
        lines: Vec::new(), // TODO: populate if needed
        source: Source {
            value: Some(input.to_string()),
            position: ModelPosition { line: 1, column: 1 },
        },
    }, pos))
}

/// Parse a stave (group of lines with one content line)
pub fn parse_stave(input: &str, pos: usize) -> ParseResult<Stave> {
    let start_pos = pos;
    let mut current_pos = pos;
    
    // Look for content line by scanning ahead
    let content_line_start = find_content_line_position(input, pos)?;
    
    // Detect notation system from content line
    let content_line_text = extract_line_content(input, content_line_start);
    let notation_system = detect_line_notation_system(&content_line_text);
    
    // Parse content line
    let (content_line, new_pos) = parse_content_line(input, content_line_start)?;
    
    // For now, create minimal stave structure
    Ok((Stave {
        text_lines_before: Vec::new(),
        content_line,
        rhythm_items: None,
        upper_lines: Vec::new(),
        lower_lines: Vec::new(),
        lyrics_lines: Vec::new(),
        text_lines_after: Vec::new(),
        notation_system,
        source: Source {
            value: Some(input[start_pos..new_pos].to_string()),
            position: ModelPosition { line: 1, column: start_pos + 1 }, // TODO: proper line tracking
        },
        begin_multi_stave: false,
        end_multi_stave: false,
    }, new_pos))
}

/// Parse a content line (musical content ending with newline or EOI)
pub fn parse_content_line(input: &str, pos: usize) -> ParseResult<Vec<ParsedElement>> {
    let mut elements = Vec::new();

    // Detect notation system for this line
    let line_content = extract_line_content(input, pos);
    let notation_system = detect_line_notation_system(&line_content);

    let chars: Vec<char> = input.chars().collect();
    let mut char_pos = pos;
    
    while char_pos < chars.len() {
        let ch = chars[char_pos];
        eprintln!("DEBUG: Processing character '{}' at position {}", ch, char_pos);

        match ch {
            '\n' => {
                // Found newline terminator
                elements.push(ParsedElement::Newline {
                    value: "\n".to_string(),
                    position: Position { row: 1, col: char_pos + 1 }, // TODO: proper position
                });
                return Ok((elements, char_pos + 1));
            }
            '|' => {
                elements.push(ParsedElement::Barline {
                    style: "|".to_string(),
                    position: Position { row: 1, col: char_pos + 1 },
                    tala: None,
                });
                char_pos += 1;
            }
            ' ' => {
                elements.push(ParsedElement::Whitespace {
                    value: " ".to_string(),
                    position: Position { row: 1, col: char_pos + 1 },
                });
                char_pos += 1;
            }
            '-' => {
                elements.push(ParsedElement::Dash {
                    degree: None,
                    octave: None,
                    position: Position { row: 1, col: char_pos + 1 },
                    duration: None,
                });
                char_pos += 1;
            }
            '1'..='7' => {
                // Parse number note
                let degree = match ch {
                    '1' => Degree::N1,
                    '2' => Degree::N2,
                    '3' => Degree::N3,
                    '4' => Degree::N4,
                    '5' => Degree::N5,
                    '6' => Degree::N6,
                    '7' => Degree::N7,
                    _ => unreachable!(),
                };
                elements.push(ParsedElement::new_note(
                    degree,
                    0,
                    ch.to_string(),
                    Position { row: 1, col: char_pos + 1 },
                ));
                char_pos += 1;
            }
            'S' | 'R' | 'G' | 'M' | 'P' | 'D' | 'N' => {
                // Parse Sargam note
                let degree = match ch {
                    'S' => Degree::N1,
                    'R' => Degree::N2,
                    'G' => Degree::N3,
                    'M' => Degree::N4,
                    'P' => Degree::N5,
                    'D' => Degree::N6,
                    'N' => Degree::N7,
                    _ => unreachable!(),
                };
                elements.push(ParsedElement::new_note(
                    degree,
                    0,
                    ch.to_string(),
                    Position { row: 1, col: char_pos + 1 },
                ));
                char_pos += 1;
            }
            _ => {
                // Accumulate consecutive unknown characters
                let start_pos = char_pos;
                let mut unknown_text = String::new();
                unknown_text.push(ch);
                char_pos += 1;

                // Collect consecutive unknown characters
                while char_pos < chars.len() {
                    let next_ch = chars[char_pos];

                    // Stop if we hit a recognized character
                    if matches!(next_ch, '\n' | '|' | ' ' | '-' | '1'..='7' | 'S' | 'R' | 'G' | 'M' | 'P' | 'D' | 'N') {
                        break;
                    }

                    unknown_text.push(next_ch);
                    char_pos += 1;
                }

                eprintln!("DEBUG: Creating Unknown token '{}' at position {}", unknown_text, start_pos + 1);
                elements.push(ParsedElement::Unknown {
                    value: unknown_text,
                    position: Position { row: 1, col: start_pos + 1 },
                });
                // char_pos is already at the right position from the inner loop
            }
        }
    }
    
    // Reached end of input without finding newline - add EndOfInput token
    elements.push(ParsedElement::EndOfInput {
        position: Position { row: 1, col: char_pos + 1 },
    });

    Ok((elements, char_pos))
}

/// Parse an empty line (whitespace followed by newline or EOI)
pub fn parse_empty_line(input: &str, pos: usize) -> ParseResult<crate::parse::model::EmptyLine> {
    let mut current_pos = pos;
    let mut elements = Vec::new();
    
    // Convert input to chars for consistent indexing
    let chars: Vec<char> = input.chars().collect();
    
    // Consume whitespace
    while current_pos < chars.len() {
        let ch = chars[current_pos];
        
        if ch == ' ' || ch == '\t' {
            elements.push(crate::parse::model::EmptyLineElement::Whitespace {
                value: ch.to_string(),
                position: ModelPosition { line: 1, column: current_pos + 1 },
            });
            current_pos += 1;
        } else if ch == '\n' {
            elements.push(crate::parse::model::EmptyLineElement::Newline {
                value: "\n".to_string(),
                position: ModelPosition { line: 1, column: current_pos + 1 },
            });
            current_pos += 1;
            break;
        } else {
            // Not an empty line - this is content
            return Err(ParseError {
                message: "Not an empty line".to_string(),
                line: 1,
                column: current_pos + 1,
            });
        }
    }
    
    // If we reached end of input without newline, add EOI
    if current_pos >= input.len() && !elements.is_empty() {
        elements.push(crate::parse::model::EmptyLineElement::EndOfInput {
            position: ModelPosition { line: 1, column: current_pos + 1 },
        });
    }
    
    Ok((crate::parse::model::EmptyLine {
        elements,
        source: Source {
            value: Some(input[pos..current_pos].to_string()),
            position: ModelPosition { line: 1, column: pos + 1 },
        },
    }, current_pos))
}

/// Helper: Find position of next content line
fn find_content_line_position(input: &str, start_pos: usize) -> Result<usize, ParseError> {
    let mut pos = start_pos;
    
    while pos < input.len() {
        // Extract line starting at this position
        let line_content = extract_line_content(input, pos);
        
        if is_content_line(&line_content) {
            return Ok(pos);
        }
        
        // Skip to next line
        pos = skip_to_next_line(input, pos);
    }
    
    Err(ParseError {
        message: "No content line found".to_string(),
        line: 1,
        column: start_pos + 1,
    })
}

/// Helper: Extract line content from position to next newline or EOI
fn extract_line_content(input: &str, start_pos: usize) -> String {
    let mut content = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut pos = start_pos;
    
    while pos < chars.len() {
        let ch = chars[pos];
        if ch == '\n' {
            break;
        }
        content.push(ch);
        pos += 1;
    }
    
    content
}

/// Helper: Skip to start of next line
fn skip_to_next_line(input: &str, start_pos: usize) -> usize {
    let chars: Vec<char> = input.chars().collect();
    let mut pos = start_pos;
    
    while pos < chars.len() {
        if chars[pos] == '\n' {
            return pos + 1; // Start of next line
        }
        pos += 1;
    }
    
    pos // End of input
}