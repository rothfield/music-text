use crate::parse::model::{Stave, TextLine, Source, Position, UpperLine, LowerLine, LyricsLine, Syllable};
use super::error::ParseError;
use super::content_line::{parse_content_line, is_content_line};
use super::upper_line::parse_upper_line;
use super::lower_line::parse_lower_line;
use super::hash_line::is_hash_line;

/// Phase 3: Parse spatial annotations above content line
/// Returns (upper_lines, remaining_text_lines) 
fn parse_upper_lines(lines: &[&str], start_line: usize) -> Result<(Vec<UpperLine>, Vec<TextLine>), ParseError> {
    let mut upper_lines = Vec::new();
    let mut text_lines_before = Vec::new();
    
    for (i, line) in lines.iter().enumerate() {
        let line_num = start_line + i;
        
        // Spatial classification: lines above content are UpperLines if they contain spatial annotations
        if is_upper_line(line) {
            let upper_line = parse_upper_line(line, line_num)?;
            upper_lines.push(upper_line);
        } else if is_lyrics_line(line) {
            // Single words before content are likely titles/annotations, not lyrics
            // Only treat multi-word lines as problematic lyrics before content
            if line.split_whitespace().count() > 1 {
                return Err(ParseError {
                    message: "Multi-word lyrics lines cannot appear before content line in a stave".to_string(),
                    line: line_num,
                    column: 1,
                });
            } else {
                // Treat single word as generic text line
                text_lines_before.push(TextLine {
                    content: line.to_string(),
                    source: Source {
                        value: Some(line.to_string()),
                        position: Position { line: line_num, column: 1 },
                    },
                });
            }
        } else {
            // Generic text line (title, directives, etc.)
            text_lines_before.push(TextLine {
                content: line.to_string(),
                source: Source {
                    value: Some(line.to_string()),
                    position: Position { line: line_num, column: 1 },
                },
            });
        }
    }
    
    Ok((upper_lines, text_lines_before))
}

/// Phase 4: Parse spatial annotations below content line
/// Returns (lower_lines, lyrics_lines, remaining_text_lines)
fn parse_lower_lines(lines: &[&str], start_line: usize, content_index: usize) -> Result<(Vec<LowerLine>, Vec<LyricsLine>, Vec<TextLine>), ParseError> {
    let mut lower_lines = Vec::new();
    let mut lyrics_lines = Vec::new();
    let mut text_lines_after = Vec::new();
    
    for (i, line) in lines.iter().enumerate() {
        let line_num = start_line + content_index + 1 + i;
        
        // Spatial classification: lines below content are LowerLines if they contain spatial annotations
        if is_lower_line(line) {
            let lower_line = parse_lower_line(line, line_num)?;
            lower_lines.push(lower_line);
        } else if is_lyrics_line(line) {
            // Parse lyrics line (most common position for lyrics)
            let lyrics_line = parse_lyrics_line(line, line_num)?;
            lyrics_lines.push(lyrics_line);
        } else {
            // Generic text line
            text_lines_after.push(TextLine {
                content: line.to_string(),
                source: Source {
                    value: Some(line.to_string()),
                    position: Position { line: line_num, column: 1 },
                },
            });
        }
    }
    
    Ok((lower_lines, lyrics_lines, text_lines_after))
}

/// Phase 1: Identify the content line in a paragraph
/// Must have exactly one content line (identified by barlines or 3+ musical elements)
fn identify_content_line(lines: &[&str], start_line: usize) -> Result<(usize, crate::parse::model::NotationSystem), ParseError> {
    let mut content_line_index = None;
    
    for (i, line) in lines.iter().enumerate() {
        if is_content_line(line) {
            if content_line_index.is_some() {
                return Err(ParseError {
                    message: "Multiple content lines found in stave - only one allowed".to_string(),
                    line: start_line + i,
                    column: 1,
                });
            }
            content_line_index = Some(i);
        }
    }

    match content_line_index {
        Some(i) => {
            let content_line_text = lines[i];
            let notation_system = crate::parse::document_parser::content_line::detect_line_notation_system(content_line_text);
            Ok((i, notation_system))
        },
        None => {
            Err(ParseError {
                message: "No musical content line found in stave".to_string(),
                line: start_line,
                column: 1,
            })
        }
    }
}

/// Parse a single paragraph into a Stave with spatial analysis
/// Classifies lines as: ContentLine, UpperLine, LowerLine, LyricsLine, or generic TextLine
/// Per MUSIC_TEXT_SPECIFICATION.md hierarchical structure
pub fn parse_stave_from_paragraph(paragraph: &str, start_line: usize) -> Result<Stave, ParseError> {
    let lines: Vec<&str> = paragraph.lines().collect();
    if lines.is_empty() {
        return Err(ParseError {
            message: "Empty paragraph".to_string(),
            line: start_line,
            column: 1,
        });
    }

    // Phase 1: Identify content line
    let (content_index, notation_system) = identify_content_line(&lines, start_line)?;

    // Phase 2: Parse content line (tokenization)
    let content_line_text = lines[content_index];
    let content_line = parse_content_line(content_line_text, start_line + content_index, notation_system)?;

    // Phase 3: Parse spatial annotations above content
    let (upper_lines, text_lines_before) = parse_upper_lines(&lines[..content_index], start_line)?;

    // Phase 4: Parse spatial annotations below content
    let (lower_lines, lyrics_lines, text_lines_after) = parse_lower_lines(&lines[content_index + 1..], start_line, content_index)?;

    // Detect multi-stave markers
    let begin_multi_stave = text_lines_before
        .first()
        .map(|line| is_hash_line(&line.content))
        .unwrap_or(false);

    // Check if ANY line after the content line is a hash line
    let end_multi_stave = text_lines_after
        .iter()
        .any(|line| is_hash_line(&line.content));

    // Create stave with consumed source (content has been moved to ParsedElements)  
    let mut stave = Stave {
        text_lines_before,
        content_line,
        rhythm_items: None, // Will be populated by rhythm analysis
        upper_lines,   // ✅ Parsed spatial annotations above content
        lower_lines,   // ✅ Parsed spatial annotations below content  
        lyrics_lines,  // ✅ Parsed syllables for note assignment
        text_lines_after,
        notation_system,
        source: Source {
            value: Some(paragraph.to_string()), // Initially has content
            position: Position {
                line: start_line,
                column: 1,
            },
        },
        begin_multi_stave,
        end_multi_stave,
    };
    
    // Consume the stave source since its content has been parsed into elements
    stave.source.value.take();
    
    Ok(stave)
}

// Line type detection functions per MUSIC_TEXT_SPECIFICATION.md

/// Detect UpperLine: contains octave markers (• typed as .), slurs (_____), ornaments
pub fn is_upper_line(line: &str) -> bool {
    let trimmed = line.trim();
    
    // Check for upper octave markers: dots (.), bullets (•), and colons (:)
    if trimmed.chars().any(|c| c == '.' || c == '•' || c == ':') {
        return true;
    }
    
    // Check for slurs: underscores above content
    // (Note: This is spatial context - we'll determine upper vs lower based on position)
    if trimmed.chars().any(|c| c == '_') {
        return true;
    }
    
    // Check for ornaments: consecutive numbers like "123" or bracketed "<456>"
    // (🚧 planned feature)
    
    false
}

/// Detect LowerLine: contains octave markers below, beat groups (_____), flat markers
pub fn is_lower_line(line: &str) -> bool {
    let trimmed = line.trim();
    
    // Check for beat groups: underscores pattern (mostly underscores and spaces)
    if trimmed.chars().any(|c| c == '_') && 
       trimmed.chars().all(|c| c == '_' || c.is_whitespace()) {
        return true;
    }
    
    // Check for octave markers: dots/colons pattern (mostly dots/colons and spaces)
    if (trimmed.chars().any(|c| c == '.' || c == '•' || c == ':')) && 
       trimmed.chars().all(|c| matches!(c, '.' | '•' | ':' | ' ')) {
        return true;
    }
    
    false
}

/// Detect LyricsLine: based on doremi-script grammar approach
/// A lyrics line contains syllables matching [a-zA-Z'!.,?]+ patterns separated by whitespace
pub fn is_lyrics_line(line: &str) -> bool {
    let trimmed = line.trim();
    
    // Empty lines are not lyrics
    if trimmed.is_empty() {
        return false;
    }
    
    // Check if it looks like a lower annotation pattern first
    if is_lower_annotation_pattern(trimmed) {
        return false;
    }
    
    // A lyrics line should consist of valid syllable patterns
    // Valid syllables are sequences of letters, apostrophes, exclamation marks, and punctuation
    // Split by whitespace and check each token
    for token in trimmed.split_whitespace() {
        // Each token should be a valid syllable (may end with hyphen for continuation)
        let is_valid_syllable = token.chars().all(|c| {
            c.is_alphabetic() || 
            c == '\'' || 
            c == '!' || 
            c == '.' || 
            c == ',' || 
            c == '?' || 
            c == '-'
        });
        
        // Must have at least one letter to be a syllable
        let has_letter = token.chars().any(|c| c.is_alphabetic());
        
        if !is_valid_syllable || !has_letter {
            return false;
        }
    }
    
    // If all tokens are valid syllables, it's a lyrics line
    true
}

/// Check if a line matches lower annotation patterns like _________ or . . . :
fn is_lower_annotation_pattern(line: &str) -> bool {
    let trimmed = line.trim();
    
    // Underscores pattern: _________ (beat grouping)
    if trimmed.chars().all(|c| c == '_' || c.is_whitespace()) && trimmed.contains('_') {
        return true;
    }
    
    // Dots and colons pattern: . . . : (tala markers)
    // Only match if the line is MOSTLY dots/colons/spaces (no letters)
    if trimmed.chars().all(|c| matches!(c, '.' | ':' | ' ')) && (trimmed.contains('.') || trimmed.contains(':')) {
        return true;
    }
    
    false
}

/// Parse a lyrics line into syllables
/// LyricsLine contains syllables like "he-llo world to-day"
fn parse_lyrics_line(line: &str, line_num: usize) -> Result<LyricsLine, ParseError> {
    let trimmed = line.trim();
    let mut syllables = Vec::new();
    
    // Split by spaces to get words/syllables
    for (i, word) in trimmed.split_whitespace().enumerate() {
        syllables.push(Syllable {
            content: word.to_string(),
            source: Source {
                value: Some(word.to_string()),
                position: Position { 
                    line: line_num, 
                    column: 1 + i * (word.len() + 1) // Approximate column position
                },
            },
        });
    }
    
    Ok(LyricsLine {
        syllables,
        source: Source {
            value: Some(line.to_string()),
            position: Position { line: line_num, column: 1 },
        },
    })
}