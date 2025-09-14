use crate::parse::model::{Stave, TextLine, Source, NotationSystem, Position as ModelPosition};
use crate::parse::recursive_descent::ParseError;

/// Parse a single stave according to formal grammar:
/// stave = upper_line* content_line (lower_line | lyrics_line)*
pub fn parse_stave(stave_text: &str) -> Result<Stave, ParseError> {
    let lines: Vec<&str> = stave_text.lines().collect();

    // Find the content line (line with barline or musical content)
    let content_line_idx = lines.iter().position(|line| is_content_line(line));

    let (content_line_elements, content_idx) = if let Some(idx) = content_line_idx {
        (crate::parse::content_line_parser::parse_content_line(lines[idx])?, idx)
    } else {
        // If no explicit content line, treat the whole thing as content
        (crate::parse::content_line_parser::parse_content_line(stave_text)?, 0)
    };

    // Parse annotation lines before and after content
    let mut upper_lines = Vec::new();
    let mut lower_lines = Vec::new();
    let mut text_lines_before = Vec::new();
    let mut text_lines_after = Vec::new();

    // Lines before content
    for i in 0..content_idx {
        let line = lines[i];
        if is_upper_line(line) {
            // TODO: Parse as upper line
        } else {
            text_lines_before.push(TextLine {
                content: line.to_string(),
                source: Source {
                    value: Some(line.to_string()),
                    position: ModelPosition { line: i + 1, column: 1 },
                },
            });
        }
    }

    // Lines after content
    for i in (content_idx + 1)..lines.len() {
        let line = lines[i];
        if is_lower_line(line) {
            // Parse as lower line using our lower_line parser
            if let Ok(parsed_lower) = crate::parse::lower_line_parser::parse_lower_line(line, i + 1) {
                lower_lines.push(parsed_lower);
            }
        } else {
            text_lines_after.push(TextLine {
                content: line.to_string(),
                source: Source {
                    value: Some(line.to_string()),
                    position: ModelPosition { line: i + 1, column: 1 },
                },
            });
        }
    }

    // Build the lines vector in order
    let mut lines = Vec::new();

    // Add text lines before content
    for text_line in text_lines_before {
        lines.push(crate::parse::model::StaveLine::Text(text_line));
    }

    // Add upper lines
    for upper_line in upper_lines {
        lines.push(crate::parse::model::StaveLine::Upper(upper_line));
    }

    // Add the content line
    lines.push(crate::parse::model::StaveLine::Content(content_line_elements));

    // Add lower lines
    for lower_line in lower_lines {
        lines.push(crate::parse::model::StaveLine::Lower(lower_line));
    }

    // Add text lines after content
    for text_line in text_lines_after {
        lines.push(crate::parse::model::StaveLine::Text(text_line));
    }

    Ok(Stave {
        lines,
        rhythm_items: None,
        notation_system: detect_notation_system(stave_text),
        source: Source {
            value: Some(stave_text.to_string()),
            position: ModelPosition { line: 1, column: 1 },
        },
    })
}

/// Check if a line is a content line (has barline or musical elements)
fn is_content_line(line: &str) -> bool {
    line.contains('|') || line.chars().any(|c| matches!(c, '1'..='7' | 'S' | 'R' | 'G' | 'M' | 'P' | 'D' | 'N' | 's' | 'r' | 'g' | 'm' | 'p' | 'd' | 'n'))
}

/// Check if a line is an upper line (has upper line elements)
fn is_upper_line(line: &str) -> bool {
    // Contains octave markers, slurs, ornaments, mordents, etc.
    line.contains('.') || line.contains(':') || line.contains('*') || line.contains('_') || line.contains('~')
}

/// Check if a line is a lower line (has lower line elements)
fn is_lower_line(line: &str) -> bool {
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