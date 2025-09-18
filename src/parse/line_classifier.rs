/// Line classifier that adds hash prefixes to lines based on document context
/// Solves ambiguous cases like single "1" by analyzing the whole document first

use crate::parse::model::NotationSystem;

#[derive(Debug, Clone)]
pub enum LineType {
    Title,
    Directive,
    Text,
    Content(NotationSystem),
    Upper,
    Lower,
    Lyrics,
}

impl LineType {
    pub fn to_prefix(&self) -> String {
        match self {
            LineType::Title => "#title#".to_string(),
            LineType::Directive => "#directive#".to_string(),
            LineType::Text => "#text#".to_string(),
            LineType::Content(system) => {
                let system_name = match system {
                    NotationSystem::Number => "number",
                    NotationSystem::Sargam => "sargam",
                    NotationSystem::Western => "western",
                    NotationSystem::Bhatkhande => "bhatkhande",
                    NotationSystem::Tabla => "tabla",
                };
                format!("#content {}#", system_name)
            }
            LineType::Upper => "#upper#".to_string(),
            LineType::Lower => "#lower#".to_string(),
            LineType::Lyrics => "#lyrics#".to_string(),
        }
    }
}

pub fn classify_lines(input: &str) -> Vec<String> {
    let lines: Vec<&str> = input.lines().collect();

    // First pass: analyze document context
    let context = analyze_document_context(&lines);

    // Second pass: classify each line with context
    let mut classified_lines = Vec::new();
    let mut in_header = true;

    for (idx, line) in lines.iter().enumerate() {
        let line_type = classify_line(line, idx, &lines, &context, &mut in_header);
        let prefix = line_type.to_prefix();
        classified_lines.push(format!("{} {}", prefix, line));
    }

    classified_lines
}

#[derive(Debug)]
struct DocumentContext {
    has_musical_content: bool,
    detected_notation_system: Option<NotationSystem>,
    musical_line_indices: Vec<usize>,
    has_barlines: bool,
}

fn analyze_document_context(lines: &[&str]) -> DocumentContext {
    let mut has_musical_content = false;
    let mut musical_line_indices = Vec::new();
    let mut has_barlines = false;

    // Look for clear musical indicators
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Check for barlines (strongest indicator)
        if trimmed.contains('|') {
            has_barlines = true;
            has_musical_content = true;
            musical_line_indices.push(idx);
            continue;
        }

        // Check for musical sequences (multiple notes with spaces)
        if is_musical_sequence(trimmed) {
            has_musical_content = true;
            musical_line_indices.push(idx);
        }
    }

    // Detect notation system from musical content
    let detected_notation_system = if has_musical_content {
        Some(detect_notation_system_from_lines(&musical_line_indices, lines))
    } else {
        None
    };

    DocumentContext {
        has_musical_content,
        detected_notation_system,
        musical_line_indices,
        has_barlines,
    }
}

fn is_musical_sequence(line: &str) -> bool {
    // Check if line looks like a sequence of musical notes
    if !line.contains(' ') {
        return false; // Single characters are ambiguous
    }

    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.len() < 2 {
        return false; // Need multiple tokens
    }

    // Check if most tokens are musical notes
    let musical_tokens = tokens.iter()
        .filter(|&token| is_musical_note(token))
        .count();

    musical_tokens as f32 / tokens.len() as f32 > 0.7 // 70% threshold
}

fn is_musical_note(token: &str) -> bool {
    // Check for musical note patterns
    if token.is_empty() {
        return false;
    }

    let first_char = token.chars().next().unwrap();

    // Number system: 1-7 (but only if it's just a number or has musical modifiers)
    if matches!(first_char, '1'..='7') {
        return token.len() <= 3 && token.chars().all(|c| matches!(c, '1'..='7' | '#' | 'b' | '-'));
    }

    // Sargam system: Single letter notes only
    if matches!(first_char, 'S' | 'R' | 'G' | 'M' | 'P' | 'D' | 'N' |
                              's' | 'r' | 'g' | 'm' | 'p' | 'd' | 'n') {
        return token.len() <= 3 && token.chars().all(|c| matches!(c, 'S' | 'R' | 'G' | 'M' | 'P' | 'D' | 'N' |
                                                                     's' | 'r' | 'g' | 'm' | 'p' | 'd' | 'n' |
                                                                     '#' | 'b' | '-'));
    }

    // Western system: Single letter notes only
    if matches!(first_char, 'A'..='G' | 'a'..='g') {
        return token.len() <= 3 && token.chars().all(|c| matches!(c, 'A'..='G' | 'a'..='g' | '#' | 'b' | '-'));
    }

    // Extensions: dashes, rests
    if token == "-" || token == "," {
        return true;
    }

    false
}

fn detect_notation_system_from_lines(musical_indices: &[usize], lines: &[&str]) -> NotationSystem {
    let mut musical_text = String::new();

    for &idx in musical_indices {
        musical_text.push_str(lines[idx]);
        musical_text.push(' ');
    }

    // Detect based on character presence
    if musical_text.chars().any(|c| matches!(c, 'S' | 'R' | 'G' | 'M' | 'P' | 'D' | 'N' |
                                                's' | 'r' | 'g' | 'm' | 'p' | 'd' | 'n')) {
        NotationSystem::Sargam
    } else if musical_text.chars().any(|c| matches!(c, '1'..='7')) {
        NotationSystem::Number
    } else if musical_text.contains("dha") || musical_text.contains("ta") || musical_text.contains("ka") {
        NotationSystem::Tabla
    } else {
        NotationSystem::Western
    }
}

fn classify_line(
    line: &str,
    idx: usize,
    all_lines: &[&str],
    context: &DocumentContext,
    in_header: &mut bool
) -> LineType {
    let trimmed = line.trim();

    // Empty lines don't change header state but end header if there's content after
    if trimmed.is_empty() {
        // Check if this blank line separates header from content
        if *in_header && has_content_after_blank_line(idx, all_lines, context) {
            *in_header = false;
        }
        return LineType::Text;
    }

    // Check if we're transitioning out of header due to musical content
    if *in_header && context.musical_line_indices.contains(&idx) {
        *in_header = false;
    }

    // If we're in header, classify header lines
    if *in_header {
        return classify_header_line(line, context);
    }

    // Musical content classification
    if context.musical_line_indices.contains(&idx) {
        if let Some(system) = context.detected_notation_system {
            return LineType::Content(system);
        }
    }

    // Annotation line classification (only after content)
    if is_upper_annotation(trimmed) {
        return LineType::Upper;
    }

    if is_lower_annotation(trimmed) {
        return LineType::Lower;
    }

    if is_lyrics_line(trimmed) {
        return LineType::Lyrics;
    }

    // Default to text
    LineType::Text
}

fn has_content_after_blank_line(blank_idx: usize, all_lines: &[&str], context: &DocumentContext) -> bool {
    // Check if there are musical content lines after this blank line
    context.musical_line_indices.iter().any(|&musical_idx| musical_idx > blank_idx)
}

fn classify_header_line(line: &str, _context: &DocumentContext) -> LineType {
    let trimmed = line.trim();

    // Check for directive pattern (key: value) - must have colon
    if let Some(colon_pos) = trimmed.find(':') {
        let before_colon = trimmed[..colon_pos].trim();
        // Make sure it's a simple identifier before the colon (not a complex title)
        if before_colon.chars().all(|c| c.is_alphanumeric() || c == '_') && before_colon.len() <= 20 {
            return LineType::Directive;
        }
    }

    // Check for title pattern (indented with large spacing)
    if line.len() > 6 {
        let leading_spaces = line.len() - line.trim_start().len();
        if leading_spaces >= 3 {
            let content = line.trim_start();
            if has_large_internal_spacing(content, 3) {
                return LineType::Title;
            }
        }
    }

    // Default to text for header
    LineType::Text
}

fn has_large_internal_spacing(s: &str, min_spaces: usize) -> bool {
    let chars: Vec<char> = s.chars().collect();
    let mut space_count = 0;

    for &ch in &chars {
        if ch == ' ' {
            space_count += 1;
        } else {
            if space_count >= min_spaces {
                return true;
            }
            space_count = 0;
        }
    }

    space_count >= min_spaces
}

fn is_upper_annotation(line: &str) -> bool {
    // Upper annotations: octave markers, ornaments, slurs
    line.contains('.') || line.contains('*') || line.contains(':') ||
    line.contains('~') || line.contains("__")
}

fn is_lower_annotation(line: &str) -> bool {
    // Lower annotations: octave markers, beat groups
    line.contains('.') || line.contains(':') || line.contains("__")
}

fn is_lyrics_line(line: &str) -> bool {
    // Lyrics: mostly alphabetic with hyphens/apostrophes
    line.split_whitespace()
        .any(|word| word.chars().all(|c| c.is_alphabetic() || c == '-' || c == '\''))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_ambiguous_single_note() {
        // Single "1" without context should be text
        let input = "1";
        let result = classify_lines(input);
        assert_eq!(result[0], "#text# 1");

        // "1" with musical context should be content
        let input = "|1 2 3 4|\n1";
        let result = classify_lines(input);
        assert!(result[0].starts_with("#content number#"));
        assert!(result[1].starts_with("#content number#"));
    }

    #[test]
    fn test_classify_title_and_directive() {
        let input = "        Amazing Grace        Bach\nAuthor: John Newton\n\n|1 2 3 4|";
        let result = classify_lines(input);

        assert_eq!(result[0], "#title#         Amazing Grace        Bach");
        assert_eq!(result[1], "#directive# Author: John Newton");
        assert_eq!(result[2], "#text# ");
        assert!(result[3].starts_with("#content number#"));
    }

    #[test]
    fn test_detect_notation_systems() {
        let sargam_input = "|S R G M|";
        let result = classify_lines(sargam_input);
        assert!(result[0].contains("#content sargam#"));

        let number_input = "|1 2 3 4|";
        let result = classify_lines(number_input);
        assert!(result[0].contains("#content number#"));
    }
}