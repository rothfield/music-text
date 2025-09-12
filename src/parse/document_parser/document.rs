use crate::parse::model::{Document, Directive, Stave, Source, Position, UpperElement, LowerElement, UpperLine, LowerLine};
use crate::rhythm::types::{ParsedElement, ParsedChild, SlurRole, BeatGroupRole};
use super::error::ParseError;
use super::stave::parse_stave_from_paragraph;
use super::content_line::{parse_content_line, count_musical_elements, detect_line_notation_system};

/// Result of parsing a paragraph
#[derive(Debug)]
enum ParagraphContent {
    Directives(Vec<Directive>),
    Stave(Stave),
}

/// Hand-written recursive descent parser for music notation
pub fn parse_document(input: &str) -> Result<Document, ParseError> {
    if input.trim().is_empty() {
        return Ok(Document {
            directives: Vec::new(),
            staves: Vec::new(),
            source: Source {
                value: input.to_string(),
                position: Position { line: 1, column: 1 },
            },
        });
    }

    // Check for single-line document special case first
    if let Some(doc) = try_parse_single_line_document(input)? {
        return Ok(doc);
    }

    // Parse input directly without preprocessing
    
    // Split into paragraphs by blank lines
    let paragraphs = split_into_paragraphs(input);
    let mut directives = Vec::new();
    let mut staves = Vec::new();

    for (para_index, paragraph) in paragraphs.iter().enumerate() {
        if !paragraph.trim().is_empty() {
            match parse_paragraph(paragraph, para_index + 1) {
                Ok(ParagraphContent::Directives(mut paragraph_directives)) => {
                    directives.append(&mut paragraph_directives);
                }
                Ok(ParagraphContent::Stave(stave)) => {
                    staves.push(stave);
                }
                Err(e) => return Err(e),
            }
        }
    }

    // Allow documents with only directives (no staves required)
    let mut document = Document {
        directives,
        staves,
        source: Source {
            value: input.to_string(),
            position: Position { line: 1, column: 1 },
        },
    };

    // Apply spatial analysis - assign octave markers to notes
    assign_octave_markers_to_document(&mut document);
    
    // Apply spatial analysis - assign slurs to notes
    assign_slurs_to_document(&mut document);
    
    // Apply spatial analysis - assign lyrics to notes (must be after slurs)
    assign_lyrics_to_document(&mut document);
    
    // Apply spatial analysis - assign beat groups to notes
    assign_beat_groups_to_document(&mut document);
    
    // Apply spatial analysis - splice unknown tokens into content lines
    splice_unknown_tokens_to_document(&mut document);

    Ok(document)
}

/// Split input into paragraphs separated by blank lines
fn split_into_paragraphs(input: &str) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut current_paragraph = String::new();
    
    for line in input.lines() {
        if line.trim().is_empty() {
            // Blank line - end current paragraph
            if !current_paragraph.trim().is_empty() {
                paragraphs.push(current_paragraph.clone());
                current_paragraph.clear();
            }
        } else {
            // Non-blank line - add to current paragraph
            if !current_paragraph.is_empty() {
                current_paragraph.push('\n');
            }
            current_paragraph.push_str(line);
        }
    }
    
    // Don't forget the last paragraph
    if !current_paragraph.trim().is_empty() {
        paragraphs.push(current_paragraph);
    }
    
    paragraphs
}

/// Parse a paragraph using functional parser chain
fn parse_paragraph(paragraph: &str, line_number: usize) -> Result<ParagraphContent, ParseError> {
    try_parse_directives(paragraph, line_number)
        .or_else(|_| try_parse_stave(paragraph, line_number))
}

/// Try to parse paragraph as directives (single or multi-line)
fn try_parse_directives(paragraph: &str, line_number: usize) -> Result<ParagraphContent, ParseError> {
    let lines: Vec<&str> = paragraph.lines().collect();
    let mut directives = Vec::new();
    
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            match parse_single_directive(trimmed, line_number + i) {
                Ok(directive) => directives.push(directive),
                Err(_) => {
                    // If any line fails as directive, whole paragraph fails as directives
                    return Err(ParseError {
                        message: "Not a directive paragraph".to_string(),
                        line: line_number + i,
                        column: 1,
                    });
                }
            }
        }
    }
    
    if directives.is_empty() {
        Err(ParseError {
            message: "No directives found".to_string(),
            line: line_number,
            column: 1,
        })
    } else {
        Ok(ParagraphContent::Directives(directives))
    }
}

/// Try to parse paragraph as a stave
fn try_parse_stave(paragraph: &str, line_number: usize) -> Result<ParagraphContent, ParseError> {
    parse_stave_from_paragraph(paragraph, line_number)
        .map(ParagraphContent::Stave)
}

/// Parse a single directive line in the format "key:value" or "key: value"
fn parse_single_directive(line: &str, line_number: usize) -> Result<Directive, ParseError> {
    // Look for key:value pattern
    if let Some(colon_pos) = line.find(':') {
        let key = line[..colon_pos].trim().to_string();
        let value = line[colon_pos + 1..].trim().to_string();
        
        // Validate key is not empty and doesn't contain musical content
        if key.is_empty() {
            return Err(ParseError {
                message: "Directive key cannot be empty".to_string(),
                line: line_number,
                column: 1,
            });
        }
        
        // Reject if key contains barlines or looks like musical content
        if key.contains('|') || is_likely_musical_content(&key) {
            return Err(ParseError {
                message: "Not a valid directive".to_string(),
                line: line_number,
                column: 1,
            });
        }
        
        Ok(Directive {
            key,
            value,
            source: Source {
                value: line.to_string(),
                position: Position { line: line_number, column: 1 },
            },
        })
    } else {
        Err(ParseError {
            message: "Directive must contain colon (:)".to_string(),
            line: line_number,
            column: 1,
        })
    }
}

/// Check if a string looks like musical content
fn is_likely_musical_content(s: &str) -> bool {
    // Check for obvious musical patterns
    if s.contains('|') {  // Barlines are strong musical indicators
        return true;
    }
    
    // Count distinctive musical patterns
    let musical_patterns: usize = s.split_whitespace()
        .filter(|word| {
            // Look for sequences of musical notes
            word.len() >= 2 && 
            word.chars().all(|c| matches!(c, 
                '1'..='7' |                                    // Numbers
                'C' | 'D' | 'E' | 'F' | 'G' | 'A' | 'B' |     // Western
                'S' | 'R' | 'M' | 'P' | 'N' |                 // Sargam uppercase
                's' | 'r' | 'g' | 'm' | 'p' | 'd' | 'n' |     // Sargam lowercase
                '-' | '#' | 'b'                                // Musical symbols
            ))
        })
        .count();
    
    // If multiple musical pattern words, likely musical content
    musical_patterns >= 2
}

/// Assign octave markers from upper and lower lines to notes spatially
fn assign_octave_markers_to_document(document: &mut Document) {
    for stave in &mut document.staves {
        assign_octave_markers_to_stave(stave);
    }
}

/// Splice unknown tokens from upper and lower lines into content line based on column position
fn splice_unknown_tokens_to_document(document: &mut Document) {
    for stave in &mut document.staves {
        splice_unknown_tokens_to_stave(stave);
    }
}

/// Assign octave markers to notes in a single stave
fn assign_octave_markers_to_stave(stave: &mut Stave) {
    // Collect octave markers from upper lines with their column positions
    let mut upper_markers: Vec<(usize, i8)> = Vec::new();
    
    for upper_line in &stave.upper_lines {
        for element in &upper_line.elements {
            if let UpperElement::UpperOctaveMarker { marker, source } = element {
                let octave_value = octave_marker_to_value(marker, true);
                // Use actual source column position (1-based)
                upper_markers.push((source.position.column, octave_value));
            }
        }
    }
    
    // Collect octave markers from lower lines with their column positions
    let mut lower_markers: Vec<(usize, i8)> = Vec::new();
    
    for lower_line in &stave.lower_lines {
        for element in &lower_line.elements {
            if let LowerElement::LowerOctaveMarker { marker, source } = element {
                let octave_value = octave_marker_to_value(marker, false);
                // Use actual source column position (1-based)
                lower_markers.push((source.position.column, octave_value));
            }
        }
    }
    
    // Combine all octave markers
    let mut all_markers = upper_markers;
    all_markers.extend(lower_markers);
    
    if all_markers.is_empty() {
        return;
    }
    
    // Assign markers to notes in content line based on column positions
    for element in &mut stave.content_line {
        if let ParsedElement::Note { octave, position, .. } = element {
            let note_col = position.col;
            
            // Find octave marker at the same column position (both use 1-based indexing)
            if let Some(&(_, marker_octave)) = all_markers.iter().find(|(marker_col, _)| *marker_col == note_col) {
                *octave = marker_octave;
            }
        }
    }
}

/// Convert octave marker string to numeric octave value
fn octave_marker_to_value(marker: &str, is_upper: bool) -> i8 {
    let base_value = match marker {
        "." => 1,
        ":" => 2,
        "*" => 3,
        "'" => 4,
        _ => 0,
    };
    
    if is_upper {
        base_value  // Upper markers are positive (higher octaves)
    } else {
        -base_value // Lower markers are negative (lower octaves)
    }
}

/// Assign slurs from upper lines to notes spatially
fn assign_slurs_to_document(document: &mut Document) {
    for stave in &mut document.staves {
        assign_slurs_to_stave(stave);
    }
}

/// Assign slurs to notes in a single stave
fn assign_slurs_to_stave(stave: &mut Stave) {
    // Find slur segments in upper lines
    let slur_segments = find_slur_segments(&stave.upper_lines);
    
    if slur_segments.is_empty() {
        return;
    }
    
    // Collect visual column positions of all notes in the content line
    // We need to track the actual column position from the parsed elements
    let mut note_positions = Vec::new();
    let mut note_indices = Vec::new();
    
    for (index, element) in stave.content_line.iter().enumerate() {
        if let ParsedElement::Note { position, .. } = element {
            // Use column position directly (1-based, same as underscore groups)
            note_positions.push(position.col);
            note_indices.push(index);
        }
    }
    
    // Apply slur markings to notes based on spatial overlap
    for (slur_start, slur_end) in slur_segments {
        // Find all notes that fall within this slur span based on visual position
        let mut notes_in_slur: Vec<(usize, usize)> = Vec::new(); // (visual_pos, index)
        
        for (&visual_pos, &index) in note_positions.iter().zip(note_indices.iter()) {
            if visual_pos >= slur_start && visual_pos <= slur_end {
                notes_in_slur.push((visual_pos, index));
            }
        }
        
        // Skip if slur covers fewer than 2 notes (not a valid slur)
        // Single underscores may have other meanings (e.g., flat in Bhatkhande notation)
        if notes_in_slur.len() < 2 {
            if notes_in_slur.len() == 1 {
                // Warn about single-note slur
                eprintln!("Warning: Slur at columns {}-{} only covers one note and will be ignored (slurs require 2+ notes)", 
                         slur_start, slur_end);  // Already 1-based column positions
            } else {
                // Warn about orphaned slur (no notes)
                eprintln!("Warning: Slur at columns {}-{} doesn't align with any notes", 
                         slur_start, slur_end);  // Already 1-based column positions
            }
            continue;
        }
        
        // Assign SlurRole based on position in slur (2+ notes guaranteed)
        for (i, &(_, element_index)) in notes_in_slur.iter().enumerate() {
            if let ParsedElement::Note { slur, in_slur, .. } = &mut stave.content_line[element_index] {
                *slur = Some(if i == 0 {
                    SlurRole::Start     // First note
                } else if i == notes_in_slur.len() - 1 {
                    SlurRole::End       // Last note
                } else {
                    SlurRole::Middle    // Middle note
                });
                *in_slur = true; // Set convenience flag
            }
        }
    }
}

/// Find slur segments (start, end positions) from upper lines
fn find_slur_segments(upper_lines: &[UpperLine]) -> Vec<(usize, usize)> {
    let mut segments = Vec::new();
    
    for upper_line in upper_lines {
        for element in &upper_line.elements {
            match element {
                UpperElement::UpperUnderscores { value, source } => {
                    let slur_len = value.len();
                    if slur_len >= 2 {
                        // Use actual source position (1-based) - keep as 1-based to match note positions
                        let start_pos = source.position.column; 
                        let end_pos = start_pos + slur_len - 1;
                        segments.push((start_pos, end_pos));
                    }
                }
                // No need to track column positions since we use actual source positions
                _ => {}
            }
        }
    }
    
    segments
}

/// Assign beat groups from lower lines to notes spatially
fn assign_beat_groups_to_document(document: &mut Document) {
    for stave in &mut document.staves {
        assign_beat_groups_to_stave(stave);
    }
}

/// Assign beat groups to notes in a single stave
fn assign_beat_groups_to_stave(stave: &mut Stave) {
    // Find underscore group segments in lower lines
    let underscore_group_segments = find_underscore_group_segments(&stave.lower_lines);
    
    if underscore_group_segments.is_empty() {
        return;
    }
    
    // Collect visual column positions of all notes in the content line
    let mut note_positions = Vec::new();
    let mut note_indices = Vec::new();
    
    for (index, element) in stave.content_line.iter().enumerate() {
        if let ParsedElement::Note { position, .. } = element {
            // Use column position directly (1-based, same as underscore groups)
            note_positions.push(position.col);
            note_indices.push(index);
        }
    }
    
    // Apply beat group markings to notes based on spatial overlap
    for (group_start, group_end) in underscore_group_segments {
        // Find all notes that fall within this beat group span based on visual position
        let mut notes_in_group: Vec<(usize, usize)> = Vec::new(); // (visual_pos, index)
        
        for (&visual_pos, &index) in note_positions.iter().zip(note_indices.iter()) {
            if visual_pos >= group_start && visual_pos <= group_end {
                notes_in_group.push((visual_pos, index));
            }
        }
        
        // Skip if beat group covers fewer than 2 notes (not meaningful)
        if notes_in_group.len() < 2 {
            continue;
        }
        
        // Assign BeatGroupRole based on position in group (2+ notes guaranteed)
        for (i, &(_, element_index)) in notes_in_group.iter().enumerate() {
            if let ParsedElement::Note { beat_group, in_beat_group, .. } = &mut stave.content_line[element_index] {
                *beat_group = Some(if i == 0 {
                    BeatGroupRole::Start     // First note
                } else if i == notes_in_group.len() - 1 {
                    BeatGroupRole::End       // Last note
                } else {
                    BeatGroupRole::Middle    // Middle note
                });
                *in_beat_group = true; // Set convenience flag
            }
        }
    }
}

/// Find underscore group segments (start, end positions) from lower lines using actual source positions
fn find_underscore_group_segments(lower_lines: &[LowerLine]) -> Vec<(usize, usize)> {
    let mut segments = Vec::new();
    
    for lower_line in lower_lines {
        for element in &lower_line.elements {
            if let LowerElement::LowerUnderscores { value, source } = element {
                let group_len = value.len();
                if group_len >= 2 {
                    // Use actual source position from token
                    let group_start = source.position.column; // 1-based from source
                    let group_end = group_start + group_len - 1;
                    segments.push((group_start, group_end));
                }
            }
        }
    }
    
    segments
}

/// Check if input qualifies as single-line document
/// Returns true if exactly one non-empty line after trimming whitespace
fn is_single_line_document(input: &str) -> bool {
    let non_empty_lines: Vec<&str> = input.lines()
        .map(|line| line.trim())           // Trim each line
        .filter(|line| !line.is_empty())   // Keep non-empty
        .collect();
    
    non_empty_lines.len() == 1
}

/// Calculate percentage of musical characters in a line
/// Uses existing count_musical_elements function for consistency
fn calculate_musical_percentage(line: &str) -> f32 {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return 0.0;
    }
    
    let musical_count = count_musical_elements(trimmed) as f32;
    let total_count = trimmed.chars().filter(|c| !c.is_whitespace()).count() as f32;
    
    if total_count == 0.0 {
        0.0
    } else {
        (musical_count / total_count) * 100.0
    }
}

/// Try to parse single-line input as musical document
/// Returns Some(Document) if successful, None if not applicable, Err if parsing fails
fn try_parse_single_line_document(input: &str) -> Result<Option<Document>, ParseError> {
    // Check if input qualifies as single-line document
    if !is_single_line_document(input) {
        return Ok(None);
    }
    
    // Get the single non-empty line
    let line = input.lines()
        .map(|line| line.trim())
        .find(|line| !line.is_empty())
        .unwrap(); // Safe because is_single_line_document returned true
    
    // Check if line meets musical content threshold (25%)
    if calculate_musical_percentage(line) < 25.0 {
        // Return empty document for non-musical single-line input
        return Ok(Some(Document {
            directives: Vec::new(),
            staves: Vec::new(),
            source: Source {
                value: input.to_string(),
                position: Position { line: 1, column: 1 },
            },
        }));
    }
    
    // Detect notation system first
    let notation_system = detect_line_notation_system(line);
    
    // Parse the line as a content line
    let elements = parse_content_line(line, 1, notation_system)?;
    
    // Create a simple stave with the parsed content
    let stave = Stave {
        content_line: elements,
        upper_lines: Vec::new(),
        lower_lines: Vec::new(),
        lyrics_lines: Vec::new(),
        text_lines_before: Vec::new(),
        text_lines_after: Vec::new(),
        notation_system,
        source: Source {
            value: line.to_string(),
            position: Position { line: 1, column: 1 },
        },
        begin_multi_stave: false,
        end_multi_stave: false,
    };
    
    // Create document with the single stave
    let document = Document {
        directives: Vec::new(),
        staves: vec![stave],
        source: Source {
            value: input.to_string(),
            position: Position { line: 1, column: 1 },
        },
    };
    
    Ok(Some(document))
}

/// Splice unknown tokens from upper lines, lower lines, and text lines into content line based on column position  
fn splice_unknown_tokens_to_stave(stave: &mut Stave) {
    let mut unknown_tokens: Vec<(usize, ParsedElement)> = Vec::new();
    
    // Collect unknown tokens from upper lines
    for upper_line in &stave.upper_lines {
        for element in &upper_line.elements {
            if let UpperElement::Unknown { value, source } = element {
                let unknown_element = ParsedElement::Unknown {
                    value: value.clone(),
                    position: crate::rhythm::types::Position { 
                        row: source.position.line, 
                        col: source.position.column 
                    },
                };
                unknown_tokens.push((source.position.column, unknown_element));
            }
        }
    }
    
    // Collect unknown tokens from lower lines
    for lower_line in &stave.lower_lines {
        for element in &lower_line.elements {
            if let LowerElement::Unknown { value, source } = element {
                let unknown_element = ParsedElement::Unknown {
                    value: value.clone(),
                    position: crate::rhythm::types::Position { 
                        row: source.position.line, 
                        col: source.position.column 
                    },
                };
                unknown_tokens.push((source.position.column, unknown_element));
            }
        }
    }
    
    // Collect unknown tokens from text lines before content
    for text_line in &stave.text_lines_before {
        let unknown_element = ParsedElement::Unknown {
            value: text_line.content.clone(),
            position: crate::rhythm::types::Position { 
                row: text_line.source.position.line, 
                col: text_line.source.position.column 
            },
        };
        unknown_tokens.push((text_line.source.position.column, unknown_element));
    }
    
    // If no unknown tokens, nothing to do
    if unknown_tokens.is_empty() {
        return;
    }
    
    // Create a new content line with unknown tokens spliced in by column position
    let mut all_elements: Vec<(usize, ParsedElement)> = Vec::new();
    
    // Add existing content line elements
    for element in stave.content_line.drain(..) {
        let col = element.position().col;
        all_elements.push((col, element));
    }
    
    // Add unknown tokens
    all_elements.extend(unknown_tokens);
    
    // Sort all elements by column position
    all_elements.sort_by_key(|(col, _)| *col);
    
    // Rebuild content line with spliced unknown tokens
    stave.content_line = all_elements.into_iter().map(|(_, element)| element).collect();
}

/// Assign lyrics to notes in the document (must be after slurs are assigned)
fn assign_lyrics_to_document(document: &mut Document) {
    for stave in &mut document.staves {
        assign_lyrics_to_stave(stave);
    }
}

/// Assign lyrics to notes in a single stave, respecting slurs
fn assign_lyrics_to_stave(stave: &mut Stave) {
    // Extract and split all syllables from lyrics lines
    let mut syllables = Vec::new();
    for lyrics_line in &stave.lyrics_lines {
        for syllable in &lyrics_line.syllables {
            // Split hyphenated syllables (like "he-llo" -> ["he", "llo"])
            let split_syllables: Vec<String> = syllable.content
                .split('-')
                .map(|part| part.trim().to_string())
                .filter(|part| !part.is_empty())
                .collect();
            syllables.extend(split_syllables);
        }
    }
    
    if syllables.is_empty() {
        return;
    }
    
    let mut syllable_iter = syllables.into_iter();
    
    // Assign syllables to singable notes, respecting slurs
    for element in &mut stave.content_line {
        if let ParsedElement::Note { children, slur, position, .. } = element {
            // Only assign syllables to notes that can sing:
            // - Notes not in slur (slur = None)  
            // - Start of slur (slur = Some(Start))
            // - Skip middle and end of slur (they extend the syllable from start)
            let should_get_syllable = match slur {
                Some(SlurRole::Middle) | Some(SlurRole::End) => false,
                _ => true, // None or Start gets a syllable
            };
            
            if should_get_syllable {
                if let Some(syllable_text) = syllable_iter.next() {
                    // Add syllable as a child of this note (distance 0 = same level)
                    children.push(ParsedChild::Syllable {
                        text: syllable_text,
                        distance: 0, // Lyrics are typically at the same level as notes
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_paragraphs() {
        let input = "line1\nline2\n\nline3\nline4\n\n\nline5";
        let paragraphs = split_into_paragraphs(input);
        assert_eq!(paragraphs, vec!["line1\nline2", "line3\nline4", "line5"]);
    }

    // Single-line document parsing tests
    
    #[test]
    fn test_single_note() {
        let result = parse_document("1");
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.staves.len(), 1);
        assert!(doc.staves[0].content_line.len() > 0);
    }

    #[test]
    fn test_single_line_western_notation() {
        let result = parse_document("C");
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.staves.len(), 1);
    }

    #[test]
    fn test_single_line_sargam_notation() {
        let result = parse_document("S");
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.staves.len(), 1);
    }

    #[test]
    fn test_single_line_with_threshold() {
        // 50% musical content (1 out of 2 chars)
        let result = parse_document("1x");
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.staves.len(), 1);

        // 28.5% musical content (2 out of 7 chars) - just above 25%
        let result = parse_document("12hello");
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.staves.len(), 1);
        
        // 0% musical content - should return empty document
        let result = parse_document("hello");
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.staves.len(), 0);
    }

    #[test]
    fn test_single_line_with_blanks() {
        // Trailing blank lines
        let result = parse_document("1\n\n\n");
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.staves.len(), 1);

        // Leading/trailing spaces
        let result = parse_document("  1  ");
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.staves.len(), 1);

        // Mixed whitespace
        let result = parse_document("   1  \n  \n\n ");
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.staves.len(), 1);
    }

    #[test]
    fn test_single_line_document_detection() {
        assert!(is_single_line_document("1"));
        assert!(is_single_line_document("  1  "));
        assert!(is_single_line_document("1\n\n"));
        assert!(is_single_line_document("   1  \n  \n\n "));
        
        assert!(!is_single_line_document("1\n2"));
        assert!(!is_single_line_document("1\n  2  \n"));
        assert!(!is_single_line_document("   \n  \n  "));
    }

    #[test]
    fn test_calculate_musical_percentage() {
        // 100% musical
        assert_eq!(calculate_musical_percentage("123"), 100.0);
        assert_eq!(calculate_musical_percentage("SRG"), 100.0);
        assert_eq!(calculate_musical_percentage("CDE"), 100.0);
        
        // 50% musical
        assert_eq!(calculate_musical_percentage("1x"), 50.0);
        
        // ~28.5% musical (2/7)
        let percentage = calculate_musical_percentage("12hello");
        assert!((percentage - 28.57).abs() < 0.1); // Allow small floating point variance
        
        // 0% musical
        assert_eq!(calculate_musical_percentage("hello"), 0.0);
        assert_eq!(calculate_musical_percentage("xyz"), 0.0);
        
        // Empty
        assert_eq!(calculate_musical_percentage(""), 0.0);
        assert_eq!(calculate_musical_percentage("   "), 0.0);
    }

    #[test]
    fn test_multiline_uses_normal_parsing() {
        // Multi-line input should NOT trigger single-line parsing
        let result = parse_document("title: test\n\n123");
        assert!(result.is_ok());
        let doc = result.unwrap();
        // Should parse as directive + stave via normal parsing
        assert_eq!(doc.directives.len(), 1);
        assert_eq!(doc.staves.len(), 1);
    }

    #[test] 
    fn test_notation_system_detection_integration() {
        // Test that notation systems are properly detected
        let number_result = parse_document("123");
        assert!(number_result.is_ok());
        
        let western_result = parse_document("CDE");
        assert!(western_result.is_ok());
        
        let sargam_result = parse_document("SRG");
        assert!(sargam_result.is_ok());
        
        // All should produce staves with content
        assert!(number_result.unwrap().staves[0].content_line.len() > 0);
        assert!(western_result.unwrap().staves[0].content_line.len() > 0);
        assert!(sargam_result.unwrap().staves[0].content_line.len() > 0);
    }

    #[test]
    fn test_beat_group_assignment() {
        // Test case: two notes with beat group underneath
        let input = "|1 3\n ___";
        let result = parse_document(input);
        assert!(result.is_ok(), "Failed to parse beat group test case");
        
        let doc = result.unwrap();
        assert_eq!(doc.staves.len(), 1, "Should have exactly one stave");
        
        let stave = &doc.staves[0];
        
        // Check that beat group was parsed
        assert_eq!(stave.lower_lines.len(), 1, "Should have 1 lower line");
        
        // Check lower line has beat group
        let beat_group_line = &stave.lower_lines[0];
        let has_beat_group = beat_group_line.elements.iter().any(|elem| {
            matches!(elem, crate::parse::model::LowerElement::LowerUnderscores { .. })
        });
        assert!(has_beat_group, "Lower line should contain beat group");
        
        // Check content line has notes
        let notes: Vec<_> = stave.content_line.iter()
            .filter_map(|elem| match elem {
                crate::rhythm::types::ParsedElement::Note { degree, octave, beat_group, in_beat_group, .. } => {
                    Some((degree, octave, beat_group, in_beat_group))
                }
                _ => None
            })
            .collect();
        
        assert_eq!(notes.len(), 2, "Should have exactly 2 notes");
        
        // Beat group assignment should work now
        assert!(notes.iter().any(|(_, _, beat_group, in_beat_group)| 
            beat_group.is_some() && **in_beat_group), 
            "At least one note should be assigned to beat group");
        
        println!("Beat group test - Notes found: {:?}", notes);
        println!("Beat group test - Lower lines: {}", stave.lower_lines.len());
    }
}