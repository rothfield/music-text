// src/document/edit/text.rs

use crate::parse::Document;

/// Inserts text into the document's raw value and returns the new cursor position.
pub fn insert_text(
    document: &mut Document,
    position: usize,
    text_to_insert: &str,
) -> Result<usize, String> {
    let mut content = document.value.as_ref().cloned().unwrap_or_default();
    
    if position > content.len() {
        return Err(format!(
            "Insert position {} is out of bounds for content length {}",
            position,
            content.len()
        ));
    }

    content.insert_str(position, text_to_insert);
    document.value = Some(content);

    Ok(position + text_to_insert.len())
}

/// Deletes text from the document's raw value and returns the new cursor position.
pub fn delete_text(
    document: &mut Document,
    position: usize,
    direction: &str,
    selection_start: Option<usize>,
    selection_end: Option<usize>,
) -> Result<usize, String> {
    let mut content = document.value.as_ref().cloned().unwrap_or_default();
    let mut new_cursor_position = position;

    if let (Some(start), Some(end)) = (selection_start, selection_end) {
        if start < end && end <= content.len() {
            // Selection deletion
            content.replace_range(start..end, "");
            new_cursor_position = start;
        }
    } else if direction == "backward" {
        // Backspace
        if position > 0 && position <= content.len() {
            let char_boundary = content.char_indices().rev().find(|(i, _)| *i < position);
            if let Some((start, _)) = char_boundary {
                 content.replace_range(start..position, "");
                 new_cursor_position = start;
            } else if position > 0 { // beginning of string
                 content.replace_range(0..position, "");
                 new_cursor_position = 0;
            }
        }
    } else if direction == "forward" {
        // Delete
        if position < content.len() {
            let char_boundary = content.char_indices().find(|(i, _)| *i >= position);
            if let Some((start, ch)) = char_boundary {
                content.replace_range(start..start+ch.len_utf8(), "");
            }
            new_cursor_position = position;
        }
    }

    document.value = Some(content);
    Ok(new_cursor_position)
}
