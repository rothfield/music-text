//! actions.rs - User-initiated mutations on the document model/text.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct TransformRequest {
    pub text: String,
    pub selection_start: usize,
    pub selection_end: usize,
    pub cursor_position: usize,
    pub action: String,
    pub octave_type: Option<String>,
    pub selected_uuids: Option<Vec<String>>, // New UUID-based selection
}

// New semantic command structure for document-first operations
#[derive(Debug, Deserialize)]
pub struct SemanticCommand {
    pub command_type: String, // "apply_slur", "set_octave", etc.
    pub target_uuids: Vec<String>, // Elements to operate on
    pub parameters: serde_json::Value, // Command-specific parameters
}

#[derive(Debug, Serialize)]
pub struct SemanticResponse {
    pub success: bool,
    pub message: String,
    pub updated_elements: Vec<String>, // UUIDs of elements that changed
}

#[derive(Debug, Serialize)]
pub struct TransformResponse {
    pub text: String,
    pub selection_start: usize,
    pub selection_end: usize,
    pub cursor_position: usize,
}

/// Apply a slur transformation to selected text
pub fn apply_slur_transform(
    text: &str,
    selection_start: usize,
    selection_end: usize,
) -> TransformResponse {
    let lines: Vec<&str> = text.lines().collect();
    let mut result_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();

    // Find the line(s) containing the selection
    let mut char_count = 0;
    let mut selection_line_idx = None;
    let mut selection_start_col = 0;
    let mut selection_end_col = 0;

    for (i, line) in lines.iter().enumerate() {
        let line_start = char_count;
        let line_end = char_count + line.len();

        // Check if selection starts in this line
        if selection_start >= line_start && selection_start <= line_end {
            selection_line_idx = Some(i);
            selection_start_col = selection_start - line_start;
        }

        // Check if selection ends in this line
        if selection_end >= line_start && selection_end <= line_end {
            selection_end_col = selection_end - line_start;
            break;
        }

        char_count = line_end + 1; // +1 for newline
    }

    if let Some(line_idx) = selection_line_idx {
        // Create underscores for the slur above the selected region
        let content_line = &lines[line_idx];
        let slur_length = if selection_end_col > selection_start_col {
            selection_end_col - selection_start_col
        } else {
            1 // Minimum slur length
        };

        // Create the slur line with underscores
        let mut slur_line = " ".repeat(content_line.len());
        let slur_start = selection_start_col;
        let slur_str = "_".repeat(slur_length);

        // Replace spaces with underscores for the slur
        if slur_start + slur_str.len() <= slur_line.len() {
            slur_line.replace_range(slur_start..slur_start + slur_str.len(), &slur_str);
        }

        // Insert the slur line above the content line
        result_lines.insert(line_idx, slur_line.trim_end().to_string());
    }

    let result_text = result_lines.join("\n");

    // Calculate new selection positions - they shift down by the length of the inserted slur line + 1 (for newline)
    let slur_line_length = if let Some(line_idx) = selection_line_idx {
        if line_idx < result_lines.len() - 1 {
            result_lines[line_idx].len() + 1 // +1 for the newline character
        } else {
            0
        }
    } else {
        0
    };

    let new_selection_start = selection_start + slur_line_length;
    let new_selection_end = selection_end + slur_line_length;

    TransformResponse {
        text: result_text,
        selection_start: new_selection_start,
        selection_end: new_selection_end,
        cursor_position: new_selection_end,
    }
}

/// Apply an octave transformation to selected text
pub fn apply_octave_transform(
    text: &str,
    selection_start: usize,
    selection_end: usize,
    octave_type: &str,
) -> TransformResponse {
    // For now, return a simple implementation
    // The full implementation would parse the document and apply the octave markers

    // Find the lines containing the selection
    let mut lines: Vec<&str> = text.lines().collect();
    let mut char_count = 0;
    let mut start_line_idx = 0;
    let mut end_line_idx = 0;

    for (i, line) in lines.iter().enumerate() {
        let line_start = char_count;
        let line_end = char_count + line.len();

        if selection_start >= line_start && selection_start <= line_end {
            start_line_idx = i;
        }
        if selection_end >= line_start && selection_end <= line_end {
            end_line_idx = i;
            break;
        }

        char_count = line_end + 1; // +1 for newline
    }

    // Apply octave markers based on type
    let marker = match octave_type {
        "lowest" => "'''",
        "lowish" => "''",
        "lower" => "'",
        "higher" => "^",
        "highish" => "^^",
        "highest" => "^^^",
        _ => return TransformResponse {
            text: text.to_string(),
            selection_start,
            selection_end,
            cursor_position: selection_end,
        },
    };

    // For simplicity, just wrap the selected text with markers
    let selected_text = &text[selection_start..selection_end];
    let modified = format!("{}{}{}", marker, selected_text, marker);

    let mut result = String::new();
    result.push_str(&text[..selection_start]);
    result.push_str(&modified);
    result.push_str(&text[selection_end..]);

    let new_selection_end = selection_end + marker.len() * 2;

    TransformResponse {
        text: result,
        selection_start,
        selection_end: new_selection_end,
        cursor_position: new_selection_end,
    }
}

/// Splits a line at the given cursor position, handling multi-line staves correctly.
///
/// This is a complex operation that requires understanding the document structure.
/// For now, we will implement a simpler text-based split. A more sophisticated version
/// would parse the document, identify the stave, and split all related lines
/// (content, upper, lower, lyrics) at the same column.
///
/// # Arguments
/// * `text` - The full text content of the document.
/// * `cursor_position` - The byte offset of the cursor within the text.
///
/// # Returns
/// A tuple containing:
/// * `String` - The new text content with the line split.
/// * `usize` - The new cursor position (should be at the start of the new line).
pub fn split_line_at_cursor(text: &str, cursor_position: usize) -> (String, usize) {
    if cursor_position > text.len() {
        // Invalid cursor position, return original text
        return (text.to_string(), cursor_position);
    }

    // Find the line and column from the cursor position
    let mut line_start = 0;
    let mut current_line = "";
    for line in text.lines() {
        let line_end = line_start + line.len();
        if cursor_position >= line_start && cursor_position <= line_end + 1 { // +1 for newline
            current_line = line;
            break;
        }
        line_start = line_end + 1; // Move to the start of the next line
    }

    let column = cursor_position - line_start;

    // Split the current line at the column
    let (first_half, second_half) = current_line.split_at(column);

    // Reconstruct the text
    let mut new_text = String::with_capacity(text.len() + 1);
    new_text.push_str(&text[..line_start]); // Text before the current line
    new_text.push_str(first_half);
    new_text.push('\n');
    new_text.push_str(second_half);
    
    // Append the rest of the document
    let rest_of_text_start = line_start + current_line.len();
    if rest_of_text_start < text.len() {
        // +1 to skip the original newline if it exists
        let start_index = if text.chars().nth(rest_of_text_start) == Some('\n') {
            rest_of_text_start + 1
        } else {
            rest_of_text_start
        };
        new_text.push_str(&text[start_index..]);
    }

    // Calculate the new cursor position: start of the second half of the split line
    let new_cursor_position = line_start + first_half.len() + 1; // +1 for the new newline

    (new_text, new_cursor_position)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_line_simple() {
        let text = "12345";
        let cursor = 3;
        let (new_text, new_cursor) = split_line_at_cursor(text, cursor);
        assert_eq!(new_text, "123\n45");
        assert_eq!(new_cursor, 4);
    }

    #[test]
    fn test_split_line_middle_of_multiline() {
        let text = "line one\nline two\nline three";
        let cursor = 14; // In the middle of "two"
        let (new_text, new_cursor) = split_line_at_cursor(text, cursor);
        assert_eq!(new_text, "line one\nline t\nwo\nline three");
        assert_eq!(new_cursor, 15);
    }

    #[test]
    fn test_split_at_end_of_line() {
        let text = "first line\nsecond line";
        let cursor = 10; // After "first line"
        let (new_text, new_cursor) = split_line_at_cursor(text, cursor);
        assert_eq!(new_text, "first line\n\nsecond line");
        assert_eq!(new_cursor, 11);
    }

    #[test]
    fn test_split_at_start_of_line() {
        let text = "first\nsecond";
        let cursor = 6; // Before "second"
        let (new_text, new_cursor) = split_line_at_cursor(text, cursor);
        assert_eq!(new_text, "first\n\nsecond");
        assert_eq!(new_cursor, 7);
    }
}
