use crate::parse::model::{Document, Stave, TextLine, WhitespaceLine, NotationSystem};
use crate::parse::document_header;
use crate::rhythm::types::{ParsedElement, Degree, Position};
use std::collections::HashMap;

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


/// Parse document according to formal grammar:
/// document = blank_lines* (multi_stave | stave | single_content_line | header) (blank_lines* stave)*
pub fn parse_document(input: &str) -> Result<Document, ParseError> {
    // Handle empty input
    if input.is_empty() {
        return Ok(Document {
            title: None,
            author: None,
            directives: HashMap::new(),
            elements: Vec::new(),
            value: Some(input.to_string()),
            char_index: 0, // was: line, column, index_in_line, index_in_doc
        });
    }

    let lines: Vec<&str> = input.lines().collect();
    let mut line_idx = 0;
    let mut elements = Vec::new();

    let mut title = None;
    let mut author = None;
    let mut directives = HashMap::new();

    // Skip leading blank lines (blank_lines*)
    while line_idx < lines.len() && lines[line_idx].trim().is_empty() {
        line_idx += 1;
    }

    // Parse first element: (multi_stave | stave | single_content_line | header)
    if line_idx < lines.len() {
        let start_idx = line_idx;

        // Follow grammar: (multi_stave | stave | single_content_line | header)

        // Try multi_stave first (starts with ###)
        if lines[line_idx].trim() == "###" {
            // TODO: implement multi-stave parsing
            // For now, skip to closing ###
            line_idx += 1;
            while line_idx < lines.len() && lines[line_idx].trim() != "###" {
                line_idx += 1;
            }
            if line_idx < lines.len() {
                line_idx += 1; // skip closing ###
            }
        }
        // Try stave (multiple content lines) OR single_content_line OR header
        else if let Some(stave_end) = find_stave_end(&lines, line_idx) {
            if stave_end - line_idx > 1 {
                // Multiple lines - try stave first, then header (following grammar order)
                let stave_lines = &lines[line_idx..stave_end];
                let stave_input = stave_lines.join("\n");

                // Calculate the starting document index
                let start_doc_index: usize = lines[..line_idx].iter()
                    .map(|line| line.len() + 1) // +1 for newline character
                    .sum();

                // First check if this actually looks like musical content
                let looks_like_musical_stave = stave_lines.iter().any(|line| {
                    let trimmed = line.trim();
                    // Must contain barlines or be purely musical notation
                    trimmed.starts_with('|') ||
                    trimmed.starts_with(':') ||
                    trimmed.contains('|') ||
                    // Musical content should be primarily notes/numbers, not words
                    (trimmed.len() <= 20 && trimmed.chars().all(|c|
                        matches!(c, '1'..='7' | 'S' | 'R' | 'G' | 'M' | 'P' | 'D' | 'N' |
                                   's' | 'r' | 'g' | 'm' | 'p' | 'd' | 'n' |
                                   'A'..='G' | 'a'..='g' | ' ' | '-' | '#' | 'b' | '\\')))
                });

                if looks_like_musical_stave {
                    if let Ok(mut stave_elements) = parse_document_body(&stave_input, start_doc_index) {
                        elements.append(&mut stave_elements);
                        line_idx = stave_end;
                    } else {
                        // Failed to parse as stave, try as header
                        line_idx = start_idx;
                        if let Some(header) = document_header::try_parse(&lines, &mut line_idx) {
                            if header.title.is_some() {
                                title = header.title;
                            }
                            if header.author.is_some() {
                                author = header.author;
                            }
                            for (key, value) in header.directives {
                                directives.insert(key, value);
                            }
                        }
                    }
                } else {
                    // Doesn't look like musical content, try as header
                    if let Some(header) = document_header::try_parse(&lines, &mut line_idx) {
                        if header.title.is_some() {
                            title = header.title;
                        }
                        if header.author.is_some() {
                            author = header.author;
                        }
                        for (key, value) in header.directives {
                            directives.insert(key, value);
                        }
                    }
                }
            } else {
                // Single line - follow grammar order: single_content_line | header
                let single_line = lines[line_idx];
                let start_doc_index: usize = lines[..line_idx].iter()
                    .map(|line| line.len() + 1)
                    .sum();

                // Check if this single line looks like musical content
                let trimmed = single_line.trim();
                let looks_like_musical_content = trimmed.starts_with('|') ||
                    trimmed.starts_with(':') ||
                    trimmed.contains('|') ||
                    // Musical content should be primarily notes/numbers, not words
                    (trimmed.len() <= 20 && trimmed.chars().all(|c|
                        matches!(c, '1'..='7' | 'S' | 'R' | 'G' | 'M' | 'P' | 'D' | 'N' |
                                   's' | 'r' | 'g' | 'm' | 'p' | 'd' | 'n' |
                                   'A'..='G' | 'a'..='g' | ' ' | '-' | '#' | 'b' | '\\')));

                if looks_like_musical_content {
                    // Try single_content_line first
                    if let Ok(mut stave_elements) = parse_document_body(single_line, start_doc_index) {
                        elements.append(&mut stave_elements);
                        line_idx += 1;
                    } else {
                        // Failed to parse as content, try as header
                        if let Some(header) = document_header::try_parse(&lines, &mut line_idx) {
                            if header.title.is_some() {
                                title = header.title;
                            }
                            if header.author.is_some() {
                                author = header.author;
                            }
                            for (key, value) in header.directives {
                                directives.insert(key, value);
                            }
                        }
                    }
                } else {
                    // Doesn't look like musical content, try as header first
                    if let Some(header) = document_header::try_parse(&lines, &mut line_idx) {
                        if header.title.is_some() {
                            title = header.title;
                        }
                        if header.author.is_some() {
                            author = header.author;
                        }
                        for (key, value) in header.directives {
                            directives.insert(key, value);
                        }
                    } else {
                        // Failed as header, try as single_content_line anyway
                        if let Ok(mut stave_elements) = parse_document_body(single_line, start_doc_index) {
                            elements.append(&mut stave_elements);
                            line_idx += 1;
                        }
                    }
                }
            }
        }
    }

    // Parse remaining staves: (blank_lines* stave)*
    while line_idx < lines.len() {
        // Skip blank lines
        while line_idx < lines.len() && lines[line_idx].trim().is_empty() {
            line_idx += 1;
        }

        if line_idx >= lines.len() {
            break;
        }

        // Parse stave
        if let Some(stave_end) = find_stave_end(&lines, line_idx) {
            let stave_lines = &lines[line_idx..stave_end];
            let stave_input = stave_lines.join("\n");

            let start_doc_index: usize = lines[..line_idx].iter()
                .map(|line| line.len() + 1)
                .sum();

            if let Ok(mut stave_elements) = parse_document_body(&stave_input, start_doc_index) {
                elements.append(&mut stave_elements);
            }
            line_idx = stave_end;
        } else {
            // Single line stave
            let single_line = lines[line_idx];
            let start_doc_index: usize = lines[..line_idx].iter()
                .map(|line| line.len() + 1)
                .sum();

            if let Ok(mut stave_elements) = parse_document_body(single_line, start_doc_index) {
                elements.append(&mut stave_elements);
            }
            line_idx += 1;
        }
    }

    Ok(Document {
        title,
        author,
        directives,
        elements,
        value: Some(input.to_string()),
        char_index: 0, // was: line, column, index_in_line, index_in_doc
    })
}

/// Find the end of a stave (group of non-blank lines)
fn find_stave_end(lines: &[&str], start: usize) -> Option<usize> {
    let mut idx = start;
    let mut found_content = false;

    while idx < lines.len() && !lines[idx].trim().is_empty() {
        found_content = true;
        idx += 1;
    }

    if found_content {
        Some(idx)
    } else {
        None
    }
}

/// Check if current position starts blank_lines (newline (whitespace* newline)+)
fn is_blank_lines_start(chars: &mut std::iter::Peekable<std::str::Chars>) -> bool {
    matches!(chars.peek(), Some(&'\n'))
}

/// Parse blank_lines (newline (whitespace* newline)+) and return BlankLines element
fn parse_blank_lines_element(chars: &mut std::iter::Peekable<std::str::Chars>, line: &mut usize, column: &mut usize, doc_index: &mut usize) -> Result<crate::parse::model::BlankLines, ParseError> {
    let mut content = String::new();
    let start_line = *line;
    let start_column = *column;
    let start_index_in_line = if *column > 0 { *column - 1 } else { 0 };
    let start_index_in_doc = *doc_index;

    // First newline
    if let Some(&'\n') = chars.peek() {
        chars.next();
        content.push('\n');
        *line += 1;
        *column = 1;
        *doc_index += 1;
    } else {
        return Err(ParseError {
            message: "Expected newline for blank_lines".to_string(),
            line: *line,
            column: *column,
        });
    }

    // (whitespace* newline)+
    while chars.peek().is_some() {
        // Collect whitespace*
        while let Some(&ch) = chars.peek() {
            if ch == ' ' {
                chars.next();
                content.push(ch);
                *column += 1;
                *doc_index += 1;
            } else {
                break;
            }
        }

        // Must have newline
        if let Some(&'\n') = chars.peek() {
            chars.next();
            content.push('\n');
            *line += 1;
            *column = 1;
            *doc_index += 1;
        } else {
            break; // End of blank_lines
        }
    }

    let source_value = content.clone();
    Ok(crate::parse::model::BlankLines {
        content,
        value: Some(source_value),
        line: start_line,
        column: start_column,
        index_in_line: start_index_in_line,
        index_in_doc: start_index_in_doc,
    })
}

/// Parse stave from character stream following grammar (backward compatibility):
/// stave = upper_line* content_line (lower_line | lyrics_line)* (blank_lines | (whitespace* newline)* EOI)
fn parse_stave_from_chars(chars: &mut std::iter::Peekable<std::str::Chars>, line: &mut usize, column: &mut usize, doc_index: &mut usize) -> Result<Stave, ParseError> {
    // For backward compatibility, detect notation system from content in the stave
    let chars_clone = chars.clone();
    let mut temp_stave_content = String::new();
    let mut temp_chars = chars_clone;

    // Peek ahead to find the notation system
    while let Some(&ch) = temp_chars.peek() {
        if ch == '\n' {
            temp_chars.next();
            temp_stave_content.push(ch);
            if is_blank_lines_start(&mut temp_chars.clone()) {
                break;
            }
        } else {
            temp_stave_content.push(temp_chars.next().unwrap());
        }
    }

    let notation_system = detect_notation_system(&temp_stave_content);
    parse_stave_from_chars_with_system(chars, line, column, doc_index, notation_system)
}

/// Parse stave from character stream following grammar (with predetermined notation system):
/// stave = upper_line* content_line (lower_line | lyrics_line)* (blank_lines | (whitespace* newline)* EOI)
fn parse_stave_from_chars_with_system(chars: &mut std::iter::Peekable<std::str::Chars>, line: &mut usize, column: &mut usize, doc_index: &mut usize, notation_system: NotationSystem) -> Result<Stave, ParseError> {
    let start_line = *line;
    let stave_start_doc_index = *doc_index;
    let mut lines = Vec::new();
    let mut stave_content = String::new();

    // Parse upper_line*
    while let Some(&ch) = chars.peek() {
        // Stop if we hit blank_lines (stave boundary)
        if is_blank_lines_start(&mut chars.clone()) {
            break;
        }

        // Collect the current line
        let mut current_line = String::new();
        let this_line_start_doc_index = *doc_index;
        while let Some(&ch) = chars.peek() {
            if ch == '\n' {
                chars.next(); // consume newline
                current_line.push(ch);
                stave_content.push(ch);
                *line += 1;
                *column = 1;
                *doc_index += 1;
                break;
            } else {
                let consumed_char = chars.next().unwrap();
                current_line.push(consumed_char);
                stave_content.push(consumed_char);
                *column += 1;
                *doc_index += 1;
            }
        }

        // Check precedence: upper lines should be checked before content lines
        // because lines like ".123" could match both but should be treated as upper lines
        if is_upper_line(&current_line.trim_end_matches('\n')) {
            // Parse as upper line using our upper_line parser
            if let Ok(parsed_upper) = crate::parse::upper_line_parser::parse_upper_line(&current_line, *line - 1, this_line_start_doc_index) {
                lines.push(crate::parse::model::StaveLine::Upper(parsed_upper));
            } else {
                // Fall back to text line
                let text_line = TextLine {
                    content: current_line.trim_end_matches('\n').to_string(),
                    value: Some(current_line.trim_end_matches('\n').to_string()),
                    char_index: this_line_start_doc_index, // was: line, column, index_in_line, index_in_doc
                };
                lines.push(crate::parse::model::StaveLine::Text(text_line));
            }
        } else if is_content_line(&current_line.trim_end_matches('\n')) {
            // Use the document notation system instead of detecting per line
            // Parse directly to ContentLine using v3 parser
            let content_line = crate::parse::content_line_parser_v3::parse_content_line(
                &current_line,
                *line,
                notation_system,
                this_line_start_doc_index
            ).map_err(|e| ParseError {
                message: format!("Content line parsing failed: {}", e.message),
                line: e.line,
                column: e.column,
            })?;
            lines.push(crate::parse::model::StaveLine::ContentLine(content_line));
            break; // Exit after finding content_line
        } else {
            // Treat as text line
            let text_line = TextLine {
                content: current_line.trim_end_matches('\n').to_string(),
                value: Some(current_line.trim_end_matches('\n').to_string()),
                char_index: this_line_start_doc_index, // was: line, column, index_in_line, index_in_doc
            };
            lines.push(crate::parse::model::StaveLine::Text(text_line));
        }
    }

    // Parse (lower_line | lyrics_line)* after content_line
    while let Some(&ch) = chars.peek() {
        // Stop if we hit blank_lines or EOI - but we'll consume blank_lines as stave termination
        if is_blank_lines_start(&mut chars.clone()) {
            break;
        }

        // Collect the current line
        let mut current_line = String::new();
        let this_line_start_doc_index = *doc_index;
        while let Some(&ch) = chars.peek() {
            if ch == '\n' {
                chars.next(); // consume newline
                current_line.push(ch);
                stave_content.push(ch);
                *line += 1;
                *column = 1;
                *doc_index += 1;
                break;
            } else {
                let consumed_char = chars.next().unwrap();
                current_line.push(consumed_char);
                stave_content.push(consumed_char);
                *column += 1;
                *doc_index += 1;
            }
        }

        if current_line.trim().is_empty() {
            // Handle whitespace* newline pattern - if followed by another newline, it's blank_lines
            if is_blank_lines_start(&mut chars.clone()) {
                break;
            }
            // Otherwise, it's a whitespace line that should be captured as part of the stave
            // Create ParsedElements like Content lines do (consistent with Pattern #1)
            let line_without_newline = current_line.trim_end_matches('\n');
            let mut elements = Vec::new();
            let mut col_position = 0;

            // Parse whitespace characters into ParsedElements
            for ch in line_without_newline.chars() {
                if ch.is_whitespace() {
                    elements.push(ParsedElement::Whitespace {
                        value: ch.to_string(),
                        position: Position { row: *line, col: col_position, char_index: this_line_start_doc_index + col_position },
                    });
                } else {
                    // Non-whitespace in what we thought was a whitespace line - treat as unknown
                    elements.push(ParsedElement::Unknown {
                        value: ch.to_string(),
                        position: Position { row: *line, col: col_position, char_index: this_line_start_doc_index + col_position },
                    });
                }
                col_position += 1;
            }

            // Add newline element if the original line had one
            if current_line.ends_with('\n') {
                elements.push(ParsedElement::Newline {
                    value: "\n".to_string(),
                    position: Position { row: *line, col: col_position, char_index: this_line_start_doc_index + col_position },
                });
            }

            let whitespace_line = WhitespaceLine {
                elements,
                value: Some(current_line.clone()),
                char_index: this_line_start_doc_index, // was: line, column, index_in_line, index_in_doc
            };
            lines.push(crate::parse::model::StaveLine::Whitespace(whitespace_line));
        } else if is_content_line(&current_line.trim_end_matches('\n')) {
            // Additional content lines (multiple content lines in a stave are allowed)
            let content_line = crate::parse::content_line_parser_v3::parse_content_line(
                &current_line,
                *line,
                notation_system,
                this_line_start_doc_index
            ).map_err(|e| ParseError {
                message: format!("Content line parsing failed: {}", e.message),
                line: e.line,
                column: e.column,
            })?;
            lines.push(crate::parse::model::StaveLine::ContentLine(content_line));
        } else if is_lower_line(&current_line.trim_end_matches('\n')) {
            // Parse as lower line using our lower_line parser (include newline)
            if let Ok(parsed_lower) = crate::parse::lower_line_parser::parse_lower_line(&current_line, *line - 1, this_line_start_doc_index) {
                lines.push(crate::parse::model::StaveLine::Lower(parsed_lower));
            } else {
                // Fall back to text line
                let text_line = TextLine {
                    content: current_line.trim_end_matches('\n').to_string(),
                    value: Some(current_line.trim_end_matches('\n').to_string()),
                    char_index: this_line_start_doc_index, // was: line, column, index_in_line, index_in_doc
                };
                lines.push(crate::parse::model::StaveLine::Text(text_line));
            }
        } else {
            // Treat as lyrics/text line
            let text_line = TextLine {
                content: current_line.trim_end_matches('\n').to_string(),
                value: Some(current_line.trim_end_matches('\n').to_string()),
                char_index: this_line_start_doc_index, // was: line, column, index_in_line, index_in_doc
            };
            lines.push(crate::parse::model::StaveLine::Text(text_line));
        }
    }


    if lines.is_empty() {
        return Err(ParseError {
            message: "Empty stave".to_string(),
            line: *line,
            column: *column,
        });
    }

    Ok(Stave {
        lines,
        notation_system,  // Use the document notation system
        value: Some(stave_content),
       line: start_line,
       column: 1,
       index_in_line: 0,
       index_in_doc: stave_start_doc_index,
    })
}

// Helper functions for line classification (used by parse_stave_from_chars)

/// Check if a token is a barline
fn is_barline_token(token: &str) -> bool {
    matches!(token, "|" | "||" | "|." | "|:" | ":|" | ":|:")
}

/// Tokenize a line into basic tokens (split by whitespace and extract barline patterns)
fn tokenize_line(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = line.char_indices().peekable();

    while let Some((i, ch)) = chars.next() {
        match ch {
            ' ' => continue, // Skip whitespace
            '|' => {
                // Handle barline patterns starting with |
                let mut barline = String::from("|");
                while let Some(&(_, next_ch)) = chars.peek() {
                    if matches!(next_ch, '|' | ':' | '.') {
                        barline.push(next_ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(barline);
            }
            ':' => {
                // Handle barline patterns starting with :
                let mut barline = String::from(":");
                if let Some(&(_, '|')) = chars.peek() {
                    barline.push('|');
                    chars.next();
                    // Check for :|:
                    if let Some(&(_, ':')) = chars.peek() {
                        barline.push(':');
                        chars.next();
                    }
                    tokens.push(barline);
                } else {
                    // Single : (not a barline, could be octave marker)
                    tokens.push(barline);
                }
            }
            _ => {
                // Collect other characters into tokens
                let start = i;
                while let Some(&(_, next_ch)) = chars.peek() {
                    if matches!(next_ch, ' ' | '|' | ':') {
                        break;
                    }
                    chars.next();
                }
                let end = chars.peek().map(|(i, _)| *i).unwrap_or(line.len());
                tokens.push(line[start..end].to_string());
            }
        }
    }

    tokens
}

/// Check if a line is a content line (has barline or musical elements)
fn is_content_line(line: &str) -> bool {
    let tokens = tokenize_line(line);

    // Check if any token is a barline
    if tokens.iter().any(|token| is_barline_token(token)) {
        return true;
    }

    // Check for musical content
    line.chars().any(|c| matches!(c, '1'..='7' | 'S' | 'R' | 'G' | 'M' | 'P' | 'D' | 'N' | 's' | 'r' | 'g' | 'm' | 'p' | 'd' | 'n'))
}

/// Check if a line is an upper line (has upper line elements)
fn is_upper_line(line: &str) -> bool {
    // Don't classify lines with barlines as upper lines - they should be content lines
    let tokens = tokenize_line(line);

    // Don't classify lines with barlines - they should be content lines
    if tokens.iter().any(|token| is_barline_token(token)) {
        return false;
    }

    // Contains octave markers, slurs, ornaments, mordents, etc.
    line.contains('.') || line.contains(':') || line.contains('*') || line.contains('_') || line.contains('~')
}

/// Check if a line is a lower line (has lower line elements)
fn is_lower_line(line: &str) -> bool {
    // Don't classify lines with barlines as lower lines - they should be content lines
    let tokens = tokenize_line(line);

    // Don't classify lines with barlines - they should be content lines
    if tokens.iter().any(|token| is_barline_token(token)) {
        return false;
    }

    // Contains lower octave markers, beat groups, or syllables
    line.contains('.') || line.contains(':') || line.contains("__") ||
    line.split_whitespace().any(|word| word.chars().all(|c| c.is_alphabetic() || c == '-' || c == '\''))
}


fn detect_notation_system(input: &str) -> NotationSystem {
    if input.chars().any(|c| matches!(c, 'S' | 'R' | 'G' | 'M' | 'P' | 'D' | 'N' | 's' | 'r' | 'g' | 'm' | 'p' | 'd' | 'n')) {
        NotationSystem::Sargam
    } else if input.chars().any(|c| matches!(c, '1'..='7')) {
        NotationSystem::Number
    } else {
        NotationSystem::Western
    }
}

/// Detect notation system from the first content line in the document
fn detect_document_notation_system(input: &str) -> NotationSystem {
    for line in input.lines() {
        if is_content_line(line) {
            return detect_notation_system(line);
        }
    }
    // Default to Western if no content lines found
    NotationSystem::Western
}

/// Parse document body (reuse existing stave parsing logic)
fn parse_document_body(input: &str, start_doc_index: usize) -> Result<Vec<crate::parse::model::DocumentElement>, ParseError> {
    let mut chars = input.chars().peekable();
    let mut line = 1;
    let mut column = 1;
    let mut doc_index: usize = start_doc_index;
    let mut elements = Vec::new();

    // Detect document notation system from first content line
    let document_notation_system = detect_document_notation_system(input);

    // Parse optional leading blank_lines*
    while is_blank_lines_start(&mut chars.clone()) {
        let blank_lines = parse_blank_lines_element(&mut chars, &mut line, &mut column, &mut doc_index)?;
        elements.push(crate::parse::model::DocumentElement::BlankLines(blank_lines));
    }

    // Parse optional (stave (blank_lines stave)*)?
    if chars.peek().is_some() {
        let first_stave = parse_stave_from_chars_with_system(&mut chars, &mut line, &mut column, &mut doc_index, document_notation_system)?;
        elements.push(crate::parse::model::DocumentElement::Stave(first_stave));

        // Parse (blank_lines stave)*
        while chars.peek().is_some() {
            if is_blank_lines_start(&mut chars.clone()) {
                let blank_lines = parse_blank_lines_element(&mut chars, &mut line, &mut column, &mut doc_index)?;
                elements.push(crate::parse::model::DocumentElement::BlankLines(blank_lines));
            }

            if chars.peek().is_some() {
                let stave = parse_stave_from_chars_with_system(&mut chars, &mut line, &mut column, &mut doc_index, document_notation_system)?;
                elements.push(crate::parse::model::DocumentElement::Stave(stave));
            }
        }
    }

    Ok(elements)
}
