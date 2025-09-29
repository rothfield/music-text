//! src/document/edit/structural.rs
// Handles direct, structural manipulation of the document model.

// BIG NOTE: The functions in this module perform edits by modifying the
// string value of a ContentLine and then re-parsing that line. This keeps the
// line's internal structure consistent. However, this does NOT automatically
// trigger a re-analysis of the entire document. Higher-level semantic
// structures that span multiple lines (like rhythm, beaming, spatial
// relationships) might become inconsistent after these operations.
//
// A full implementation would require a "re-analysis" or "re-processing"
// phase to run after each edit to recalculate these document-wide properties.
// That re-analysis phase is NOT implemented here.

use crate::models::{Document, DocumentElement, StaveLine, ContentLine, NotationSystem};
use serde::{Deserialize, Serialize};

/// Represents clipboard content. For now, it's text-based.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clipboard {
    pub content: String,
}

/// Reconstruct the document's value field from its elements
pub fn reconstruct_document_value(doc: &mut Document) {
    let mut text = String::new();
    let mut first_element = true;

    for element in &doc.elements {
        if !first_element {
            // Elements are separated by blank lines (for now, simplify)
            // In the future, track actual whitespace between elements
        }

        match element {
            DocumentElement::BlankLines(blank) => {
                if let Some(value) = &blank.value {
                    text.push_str(value);
                }
            }
            DocumentElement::Stave(stave) => {
                for (line_idx, line) in stave.lines.iter().enumerate() {
                    if line_idx > 0 {
                        text.push('\n');
                    }

                    match line {
                        StaveLine::ContentLine(cl) => {
                            if let Some(value) = &cl.value {
                                text.push_str(value);
                            }
                        }
                        StaveLine::Content(_) => {} // Legacy, skip
                        StaveLine::Lyrics(ll) => {
                            if let Some(value) = &ll.value {
                                text.push_str(value);
                            }
                        }
                        StaveLine::Text(tl) => {
                            if let Some(value) = &tl.value {
                                text.push_str(value);
                            }
                        }
                        StaveLine::Whitespace(wl) => {
                            if let Some(value) = &wl.value {
                                text.push_str(value);
                            }
                        }
                        StaveLine::BlankLines(bl) => {
                            if let Some(value) = &bl.value {
                                text.push_str(value);
                            }
                        }
                    }
                }
            }
        }
        first_element = false;
    }

    doc.value = Some(text);
}

/// Locates the mutable ContentLine and relative position for an absolute character position.
fn find_structural_position<'a>(
    doc: &'a mut Document,
    position: usize,
) -> Option<(usize, usize, usize, usize)> { // (stave_idx, line_idx, relative_pos, current_pos_offset)
    let mut current_pos = 0;
    for (stave_idx, el) in doc.elements.iter_mut().enumerate() {
        if let DocumentElement::Stave(stave) = el {
            for (line_idx, line) in stave.lines.iter_mut().enumerate() {
                if let StaveLine::ContentLine(content_line) = line {
                    let line_len = content_line.value.as_ref().map_or(0, |v| v.len());
                    if position <= current_pos + line_len {
                        return Some((
                            stave_idx,
                            line_idx,
                            position - current_pos,
                            current_pos
                        ));
                    }
                    current_pos += line_len + 1; // +1 for implicit newline
                }
            }
        }
    }
    None
}


pub fn delete_selection(doc: &mut Document, start: usize, end: usize) -> Result<usize, String> {
    // This is a simplified implementation that only handles single-line selections.
    let notation_system = doc.get_detected_notation_systems().first().cloned().unwrap_or(NotationSystem::Number);
    if let Some((stave_idx, line_idx, relative_start, _)) = find_structural_position(doc, start) {
        if let Some(StaveLine::ContentLine(line)) = doc.elements[stave_idx].as_stave_mut().unwrap().lines.get_mut(line_idx) {
            let mut text = line.value.as_ref().cloned().unwrap_or_default();
            let relative_end = end - (start - relative_start);
            if relative_end <= text.len() {
                text.replace_range(relative_start..relative_end, "");
                line.value = Some(text);
                return Ok(start);
            }
        }
    }
    Err("Could not perform selection deletion.".to_string())
}

/// Parse a single character into the appropriate object
/// Returns: parse_pitch | parse_note | parse_dash | parse_barline | parse_whitespace | parse_unknown
pub fn parse_single_char(notation_system: &NotationSystem, ch: char, position: usize) -> Result<crate::models::ContentElement, String> {
    use crate::models::*;

    let obj = match ch {
        // Space - parse whitespace
        ' ' => parse_whitespace(ch, position),

        // Single barline
        '|' => parse_single_barline(ch, position),

        // Numbers 1-7 - parse note (grouped into beat)
        '1'..='7' => parse_note(notation_system, ch, position),

        // Dash - parse dash
        '-' => parse_dash(ch, position),

        // Breath marks - parse breath
        '^' | '~' => parse_breath(ch, position),

        // Everything else - parse unknown
        _ => parse_unknown(ch, position),
    };

    obj
}

fn parse_whitespace(ch: char, position: usize) -> Result<crate::models::ContentElement, String> {
    Ok(crate::models::ContentElement::Whitespace(crate::models::Whitespace {
        id: uuid::Uuid::new_v4(),
        value: Some(ch.to_string()),
    }))
}

fn parse_single_barline(ch: char, position: usize) -> Result<crate::models::ContentElement, String> {
    use crate::models::barlines::*;
    Ok(crate::models::ContentElement::Barline(Barline::Single(SingleBarline {
        id: uuid::Uuid::new_v4(),
        value: Some(ch.to_string()),
    })))
}

fn parse_note(notation_system: &NotationSystem, ch: char, position: usize) -> Result<crate::models::ContentElement, String> {
    // Use pitch_systems to parse the note
    let degree_opt = match notation_system {
        NotationSystem::Number => crate::models::pitch_systems::number::lookup(&ch.to_string()),
        NotationSystem::Western => None, // TODO: implement western note parsing
        NotationSystem::Sargam => None,   // TODO: implement sargam parsing
        NotationSystem::Bhatkhande => None, // TODO: implement bhatkhande parsing
        NotationSystem::Tabla => None,    // TODO: implement tabla parsing
    };

    if let Some(degree) = degree_opt {
        let pitch_code = crate::models::pitch_systems::degree_to_pitch_code(degree);

        // Create a Beat containing a single Note (following grammar: notes are grouped into beats)
        Ok(crate::models::ContentElement::Beat(crate::models::Beat {
            id: uuid::Uuid::new_v4(),
                elements: vec![crate::models::BeatElement::Note(crate::models::Note {
                id: uuid::Uuid::new_v4(),
                        pitch_code,
                octave: 0,
                value: Some(ch.to_string()),
                denominator: None,
                numerator: None,
                notation_system: notation_system.clone(),
            })],
            value: Some(ch.to_string()),
            tied_to_previous: None,
            total_duration: None,
            divisions: None,
            is_tuplet: None,
            tuplet_ratio: None,
        }))
    } else {
        parse_unknown(ch, position)
    }
}

fn parse_dash(ch: char, position: usize) -> Result<crate::models::ContentElement, String> {
    // Create a Beat containing a single Dash (following grammar: dashes are grouped into beats)
    Ok(crate::models::ContentElement::Beat(crate::models::Beat {
        id: uuid::Uuid::new_v4(),
        elements: vec![crate::models::BeatElement::Dash(crate::models::Dash {
            id: uuid::Uuid::new_v4(),
                value: Some(ch.to_string()),
            denominator: None,
            numerator: None,
        })],
        value: Some(ch.to_string()),
        tied_to_previous: None,
        total_duration: None,
        divisions: None,
        is_tuplet: None,
        tuplet_ratio: None,
    }))
}

fn parse_breath(ch: char, position: usize) -> Result<crate::models::ContentElement, String> {
    // TODO: Create proper breath element
    // For now, return unknown
    parse_unknown(ch, position)
}

fn parse_unknown(ch: char, position: usize) -> Result<crate::models::ContentElement, String> {
    Ok(crate::models::ContentElement::UnknownToken(crate::models::UnknownToken {
        id: uuid::Uuid::new_v4(),
        value: Some(ch.to_string()),
        token_value: ch.to_string(),
    }))
}

pub fn insert_char(doc: &mut Document, position: usize, ch: char) -> Result<usize, String> {
    let notation_system = doc.get_detected_notation_systems().first().cloned().unwrap_or(NotationSystem::Number);

    // Parse the character into a ContentElement
    let element = parse_single_char(&notation_system, ch, position)?;
    println!("INSERT_CHAR: char='{}' position={} element={:?}", ch, position, element);

    // Find the target content line and insert the element
    if let Some((stave_idx, line_idx, relative_pos, _)) = find_structural_position(doc, position) {
        if let Some(StaveLine::ContentLine(line)) = doc.elements[stave_idx].as_stave_mut().unwrap().lines.get_mut(line_idx) {
            println!("INSERT_CHAR: before - text={:?} elements.len={}", line.value, line.elements.len());

            // Update the line's text value
            let mut text = line.value.as_ref().cloned().unwrap_or_default();
            text.insert(relative_pos, ch);
            line.value = Some(text);

            // Insert the parsed element at the correct position
            // For now, append to elements
            line.elements.push(element);

            println!("INSERT_CHAR: after - text={:?} elements.len={}", line.value, line.elements.len());
            return Ok(position + 1);
        }
    }
    Err("Could not insert character.".to_string())
}

pub fn delete_char_left(doc: &mut Document, position: usize) -> Result<usize, String> {
    if position == 0 { return Ok(0); }
    let notation_system = doc.get_detected_notation_systems().first().cloned().unwrap_or(NotationSystem::Number);

    if let Some((stave_idx, line_idx, relative_pos, _)) = find_structural_position(doc, position) {
        let stave = doc.elements[stave_idx].as_stave_mut().unwrap();
        if relative_pos == 0 { // Combine lines
            if line_idx > 0 {
                if let Some(StaveLine::ContentLine(prev_line)) = stave.lines.get(line_idx - 1) {
                     let mut prev_text = prev_line.value.as_ref().cloned().unwrap_or_default();
                     let prev_len = prev_text.len();
                     if let Some(StaveLine::ContentLine(current_line)) = stave.lines.get(line_idx) {
                         let current_text = current_line.value.as_ref().cloned().unwrap_or_default();
                         prev_text.push_str(&current_text);
                         
                         let prev_line_mut = stave.lines.get_mut(line_idx - 1).unwrap().as_content_line_mut().unwrap();
                         prev_line_mut.value = Some(prev_text);
                         
                         stave.lines.remove(line_idx);
                         return Ok(position - 1);
                     }
                }
            }
             Ok(position)
        } else { // Standard backspace
            if let Some(StaveLine::ContentLine(line)) = stave.lines.get_mut(line_idx) {
                let mut text = line.value.as_ref().cloned().unwrap_or_default();
                if relative_pos > 0 && relative_pos <= text.len() {
                    text.remove(relative_pos - 1);
                    line.value = Some(text);
                    return Ok(position - 1);
                }
            }
             Ok(position)
        }
    } else {
        Err("Could not find position for backspace.".to_string())
    }
}

pub fn delete_char_right(doc: &mut Document, position: usize) -> Result<usize, String> {
    let notation_system = doc.get_detected_notation_systems().first().cloned().unwrap_or(NotationSystem::Number);
    if let Some((stave_idx, line_idx, relative_pos, _)) = find_structural_position(doc, position) {
        if let Some(StaveLine::ContentLine(line)) = doc.elements[stave_idx].as_stave_mut().unwrap().lines.get_mut(line_idx) {
            let mut text = line.value.as_ref().cloned().unwrap_or_default();
            if relative_pos < text.len() {
                text.remove(relative_pos);
                line.value = Some(text);
            }
        }
    }
    Ok(position)
}

pub fn insert_newline(doc: &mut Document, position: usize) -> Result<usize, String> {
    let notation_system = doc.get_detected_notation_systems().first().cloned().unwrap_or(NotationSystem::Number);
    if let Some((stave_idx, line_idx, relative_pos, _)) = find_structural_position(doc, position) {
        let stave = doc.elements[stave_idx].as_stave_mut().unwrap();
        if let Some(StaveLine::ContentLine(line)) = stave.lines.get_mut(line_idx) {
            let text = line.value.as_ref().cloned().unwrap_or_default();
            let (first_half, second_half) = text.split_at(relative_pos);

            // Update current line
            line.value = Some(first_half.to_string());

            // Create and insert new line
            let mut new_content_line = ContentLine {
                id: uuid::Uuid::new_v4(),
                elements: vec![],
                value: Some(second_half.to_string()),
            };
            
            stave.lines.insert(line_idx + 1, StaveLine::ContentLine(new_content_line));
            return Ok(position + 1);
        }
    }
    Err("Could not insert newline.".to_string())
}

pub fn copy_selection(doc: &Document, start: usize, end: usize) -> Result<Clipboard, String> {
    // Simplified single-line copy
    let mut current_pos = 0;
    for el in &doc.elements {
        if let DocumentElement::Stave(stave) = el {
            for line in &stave.lines {
                if let StaveLine::ContentLine(content_line) = line {
                    let text = content_line.value.as_ref().cloned().unwrap_or_default();
                    let line_len = text.len();
                    if start >= current_pos && end <= current_pos + line_len {
                        let relative_start = start - current_pos;
                        let relative_end = end - current_pos;
                        return Ok(Clipboard {
                            content: text[relative_start..relative_end].to_string(),
                        });
                    }
                    current_pos += line_len + 1;
                }
            }
        }
    }
    Err("Copy selection spans multiple lines, which is not supported.".to_string())
}

pub fn paste(doc: &mut Document, position: usize, clipboard: &Clipboard) -> Result<usize, String> {
    let notation_system = doc.get_detected_notation_systems().first().cloned().unwrap_or(NotationSystem::Number);
    if let Some((stave_idx, line_idx, relative_pos, _)) = find_structural_position(doc, position) {
        if let Some(StaveLine::ContentLine(line)) = doc.elements[stave_idx].as_stave_mut().unwrap().lines.get_mut(line_idx) {
            let mut text = line.value.as_ref().cloned().unwrap_or_default();
            text.insert_str(relative_pos, &clipboard.content);
            line.value = Some(text);
            return Ok(position + clipboard.content.len());
        }
    }
    Err("Could not paste.".to_string())
}

// Helper methods are already defined in models/core.rs, removed duplicates
