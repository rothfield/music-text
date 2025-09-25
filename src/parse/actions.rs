//! actions.rs - User-initiated mutations on the document model/text.

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
