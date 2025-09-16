use serde::Serialize;
use crate::parse::{Document, UpperElement, LowerElement};
use crate::parse::model::{DocumentElement, StaveLine};
use crate::rhythm::types::{ParsedElement, ParsedChild};

/// Convert row/col position to absolute character position in original input
fn position_to_absolute_offset(position: &crate::rhythm::types::Position, original_input: &str) -> Option<usize> {
    let lines: Vec<&str> = original_input.split('\n').collect();

    // Convert 1-based row to 0-based for array indexing
    let zero_based_row = if position.row > 0 { position.row - 1 } else { 0 };

    if zero_based_row >= lines.len() {
        return None;
    }

    let mut offset = 0;
    // Add lengths of all previous lines (including newlines)
    for i in 0..zero_based_row {
        offset += lines[i].len() + 1; // +1 for newline
    }

    // Add column offset within the current line (col is already 0-based)
    if position.col <= lines[zero_based_row].len() {
        offset += position.col;
    }

    Some(offset)
}

fn source_position_to_absolute_offset(line: usize, column: usize, original_input: &str) -> Option<usize> {
    let lines: Vec<&str> = original_input.split('\n').collect();

    if line == 0 || line > lines.len() {
        return None;
    }

    let mut offset = 0;
    // Add lengths of all previous lines (including newlines)
    for i in 0..(line - 1) {
        offset += lines[i].len() + 1; // +1 for newline
    }

    // Add column offset within the current line
    if column <= lines[line - 1].len() {
        offset += column; // Already 0-based
    }

    Some(offset)
}

#[derive(Debug, Serialize, Clone)]
pub struct SyntaxToken {
    pub token_type: String,
    pub start: usize,
    pub end: usize,
    pub content: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct CharacterStyle {
    pub pos: usize,
    pub classes: Vec<String>,
}

/// Escape XML special characters
pub fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Convert syntax tokens to character styles for client-side application
pub fn generate_character_styles(tokens: &[SyntaxToken]) -> Vec<CharacterStyle> {
    // Filter out whitespace and newline tokens, then map to character styles
    let mut styles: Vec<CharacterStyle> = tokens
        .iter()
        .filter(|token| token.token_type != "whitespace" && token.token_type != "newline")
        .flat_map(|token| {
            let css_class = format!("cm-music-{}", token.token_type);
            (token.start..token.end).map(move |pos| CharacterStyle {
                pos,
                classes: vec![css_class.clone()],
            })
        })
        .collect();

    // Sort by position
    styles.sort_by_key(|style| style.pos);

    // Fill gaps to make CodeMirror happy
    if let Some(&max_pos) = tokens.iter().map(|t| t.end).max_by_key(|&x| x).as_ref() {
        (0..max_pos)
            .map(|pos| {
                styles
                    .iter()
                    .find(|s| s.pos == pos)
                    .cloned()
                    .unwrap_or(CharacterStyle {
                        pos,
                        classes: vec![],
                    })
            })
            .collect()
    } else {
        styles
    }
}

/// Enhanced character styles generation with beat group and slur information
/// Uses rhythm-analyzed document to identify both explicit beat groups (marked with ___)
/// and implicit beat groups (consecutive musical elements with same beat)
/// Also processes slurs (marked with ___ in upper lines)
pub fn generate_character_styles_with_beat_groups(tokens: &[SyntaxToken], document: &crate::parse::Document) -> Vec<CharacterStyle> {
    use crate::parse::model::{DocumentElement, StaveLine};
    use crate::rhythm::types::{ParsedElement, BeatGroupRole};

    let mut styles = generate_character_styles(tokens);

    // Add beat group and slur classes to elements
    for element in &document.elements {
        if let DocumentElement::Stave(stave) = element {
            for line in &stave.lines {
                if let StaveLine::Content(content_elements) = line {
                    // Process explicit beat groups (marked with ___)
                    process_explicit_beat_groups(&mut styles, content_elements, document);

                    // Process implicit beat groups (consecutive elements with same beat timing)
                    process_implicit_beat_groups(&mut styles, content_elements, document);

                    // Process slurs (marked with ___ in upper lines)
                    process_slurs(&mut styles, content_elements, document);
                }
            }
        }
    }

    styles
}

/// Process explicit beat groups by spatially detecting elements under beat group indicators
fn process_explicit_beat_groups(
    styles: &mut Vec<CharacterStyle>,
    content_elements: &[ParsedElement],
    document: &crate::parse::Document
) {
    use crate::parse::model::{DocumentElement, StaveLine, LowerElement};

    // Find beat group indicators in lower lines and collect their spans
    let mut beat_group_spans = Vec::new();

    for element in &document.elements {
        if let DocumentElement::Stave(stave) = element {
            for line in &stave.lines {
                if let StaveLine::Lower(lower_line) = line {
                    for lower_element in &lower_line.elements {
                        if let LowerElement::BeatGroupIndicator { value, source } = lower_element {
                            let start_pos = source.position.column; // Convert to 0-based
                            let end_pos = start_pos + value.len() - 1;
                            beat_group_spans.push((start_pos, end_pos));
                        }
                    }
                }
            }
        }
    }

    // For each beat group span, find ALL elements within that span
    for (start_pos, end_pos) in beat_group_spans {
        let mut elements_in_beat_group = Vec::new();

        // Collect all elements within this beat group span
        for element in content_elements {
            let element_pos = match element {
                ParsedElement::Note { position, .. } => Some(position.col),
                ParsedElement::Rest { position, .. } => Some(position.col),
                ParsedElement::Dash { position, .. } => Some(position.col),
                ParsedElement::Barline { position, .. } => Some(position.col),
                ParsedElement::Whitespace { position, .. } => Some(position.col),
                _ => None,
            };

            if let Some(pos) = element_pos {
                if pos >= start_pos && pos <= end_pos {
                    if let Some(absolute_pos) = position_to_absolute_offset(&crate::rhythm::types::Position { row: 1, col: pos }, &document.source.value.clone().unwrap_or_default()) {
                        elements_in_beat_group.push(absolute_pos);
                    }
                }
            }
        }

        // Apply beat group classes to all elements in this span
        if elements_in_beat_group.len() >= 2 {
            elements_in_beat_group.sort();
            add_beat_group_classes(styles, &elements_in_beat_group, elements_in_beat_group.len());
        }
    }
}

/// Process implicit beat groups (consecutive musical elements with same beat timing)
fn process_implicit_beat_groups(
    styles: &mut Vec<CharacterStyle>,
    content_elements: &[ParsedElement],
    document: &crate::parse::Document
) {
    use crate::rhythm::types::ParsedElement;

    let mut musical_elements = Vec::new();

    // Collect all musical elements (notes, rests, dashes) with their positions and durations
    for element in content_elements {
        match element {
            ParsedElement::Note { position, duration, in_beat_group, .. } => {
                // Skip notes already in explicit beat groups
                if !*in_beat_group {
                    if let Some(absolute_pos) = position_to_absolute_offset(&position, &document.source.value.clone().unwrap_or_default()) {
                        musical_elements.push((absolute_pos, duration.clone()));
                    }
                }
            }
            ParsedElement::Rest { position, duration, .. } => {
                if let Some(absolute_pos) = position_to_absolute_offset(&position, &document.source.value.clone().unwrap_or_default()) {
                    musical_elements.push((absolute_pos, duration.clone()));
                }
            }
            ParsedElement::Dash { position, duration, .. } => {
                if let Some(absolute_pos) = position_to_absolute_offset(&position, &document.source.value.clone().unwrap_or_default()) {
                    musical_elements.push((absolute_pos, duration.clone()));
                }
            }
            _ => {}
        }
    }

    // Group consecutive elements with same duration (implicit beat grouping)
    let mut current_implicit_group = Vec::new();
    let mut last_duration: Option<(usize, usize)> = None;

    for (pos, duration) in musical_elements {
        let should_group = match (&last_duration, &duration) {
            (Some(last), Some(current)) => last == current && last.1 >= 4, // Same duration and at least quarter notes
            _ => false,
        };

        if should_group {
            current_implicit_group.push(pos);
        } else {
            // Process previous implicit group if it has 2+ elements
            if current_implicit_group.len() >= 2 {
                add_beat_group_classes(styles, &current_implicit_group, current_implicit_group.len());
            }
            // Start new group
            current_implicit_group = vec![pos];
        }

        last_duration = duration;
    }

    // Process final implicit group
    if current_implicit_group.len() >= 2 {
        add_beat_group_classes(styles, &current_implicit_group, current_implicit_group.len());
    }
}

/// Process slurs by spatially detecting elements under slur indicators
fn process_slurs(
    styles: &mut Vec<CharacterStyle>,
    content_elements: &[ParsedElement],
    document: &crate::parse::Document
) {
    use crate::parse::model::{DocumentElement, StaveLine, UpperElement};

    // Find slur indicators in upper lines and collect their spans
    let mut slur_spans = Vec::new();

    for element in &document.elements {
        if let DocumentElement::Stave(stave) = element {
            for line in &stave.lines {
                if let StaveLine::Upper(upper_line) = line {
                    for upper_element in &upper_line.elements {
                        if let UpperElement::SlurIndicator { value, source } = upper_element {
                            let start_pos = source.position.column; // Convert to 0-based
                            let end_pos = start_pos + value.len() - 1;
                            slur_spans.push((start_pos, end_pos));
                        }
                    }
                }
            }
        }
    }

    // For each slur span, find ALL elements within that span
    for (start_pos, end_pos) in slur_spans {
        let mut elements_in_slur = Vec::new();

        // Collect all elements within this slur span
        for element in content_elements {
            let element_pos = match element {
                ParsedElement::Note { position, .. } => Some(position.col),
                ParsedElement::Rest { position, .. } => Some(position.col),
                ParsedElement::Dash { position, .. } => Some(position.col),
                ParsedElement::Barline { position, .. } => Some(position.col),
                ParsedElement::Whitespace { position, .. } => Some(position.col),
                _ => None,
            };

            if let Some(pos) = element_pos {
                if pos >= start_pos && pos <= end_pos {
                    if let Some(absolute_pos) = position_to_absolute_offset(&crate::rhythm::types::Position { row: 1, col: pos }, &document.source.value.clone().unwrap_or_default()) {
                        elements_in_slur.push(absolute_pos);
                    }
                }
            }
        }

        // Apply slur classes to all elements in this span
        if elements_in_slur.len() >= 2 {
            elements_in_slur.sort();
            add_slur_classes(styles, &elements_in_slur, elements_in_slur.len());
        }
    }
}

fn add_slur_classes(styles: &mut Vec<CharacterStyle>, positions: &[usize], count: usize) {
    for (i, &pos) in positions.iter().enumerate() {
        // Find the style at this position and add slur classes
        if let Some(style) = styles.iter_mut().find(|s| s.pos == pos) {
            // Add base in-slur class
            style.classes.push("in-slur".to_string());

            // Add role-specific class based on position in group
            if i == 0 {
                style.classes.push("slur-start".to_string());
                // Add count class for arc sizing
                style.classes.push(format!("slur-{}", count));
            } else if i == positions.len() - 1 {
                style.classes.push("slur-end".to_string());
            } else {
                style.classes.push("slur-middle".to_string());
            }
        }
    }
}

fn add_beat_group_classes(styles: &mut Vec<CharacterStyle>, positions: &[usize], count: usize) {
    for (i, &pos) in positions.iter().enumerate() {
        // Find the style at this position and add beat group classes
        if let Some(style) = styles.iter_mut().find(|s| s.pos == pos) {
            // Add base in-beat-group class
            style.classes.push("in-beat-group".to_string());

            // Add role-specific class based on position in group
            if i == 0 {
                style.classes.push("beat-group-start".to_string());
                // Add count class for arc sizing
                style.classes.push(format!("beat-group-{}", count));
            } else if i == positions.len() - 1 {
                style.classes.push("beat-group-end".to_string());
            } else {
                style.classes.push("beat-group-middle".to_string());
            }
        }
    }
}

/// Generate XML representation from parsed document
pub fn generate_xml_representation(document: &serde_json::Value) -> String {
    let mut xml = String::new();

    // Process elements array (actual document format)
    if let Some(elements) = document.get("elements").and_then(|e| e.as_array()) {
        for element in elements {
            // Handle BlankLines elements
            if let Some(blank_lines) = element.get("BlankLines") {
                if let Some(content) = blank_lines.get("content").and_then(|c| c.as_str()) {
                    xml.push_str(&format!("<blank_lines>{}</blank_lines>", escape_xml(content)));
                }
            }
            // Handle Stave elements
            else if let Some(stave) = element.get("Stave") {
                if let Some(lines) = stave.get("lines").and_then(|l| l.as_array()) {
                    for line in lines {
                        // Process BlankLines within stave
                        if let Some(blank_lines) = line.get("BlankLines") {
                            if let Some(content) = blank_lines.get("content").and_then(|c| c.as_str()) {
                                xml.push_str(&format!("<stave_blank_lines>{}</stave_blank_lines>", escape_xml(content)));
                            }
                        }
                        // Process Content lines
                        else if let Some(content) = line.get("Content").and_then(|c| c.as_array()) {
                            xml.push_str("<pre class=\"CodeMirror-line\" role=\"presentation\">");
                            for el in content {
                                if let Some(note) = el.get("Note") {
                                    if let Some(value) = note.get("value").and_then(|v| v.as_str()) {
                                        xml.push_str(&format!("<note>{}</note>", escape_xml(value)));
                                    }
                                } else if let Some(barline) = el.get("Barline") {
                                    if let Some(style) = barline.get("style").and_then(|s| s.as_str()) {
                                        xml.push_str(&format!("<barline>{}</barline>", escape_xml(style)));
                                    }
                                } else if let Some(whitespace) = el.get("Whitespace") {
                                    if let Some(value) = whitespace.get("value").and_then(|v| v.as_str()) {
                                        xml.push_str(&format!("<whitespace>{}</whitespace>", escape_xml(value)));
                                    }
                                } else if let Some(unknown) = el.get("Unknown") {
                                    if let Some(value) = unknown.get("value").and_then(|v| v.as_str()) {
                                        xml.push_str(&format!("<unknown>{}</unknown>", escape_xml(value)));
                                    }
                                } else if let Some(_dash) = el.get("Dash") {
                                    xml.push_str("<dash>-</dash>");
                                } else if let Some(_breath) = el.get("Breath") {
                                    xml.push_str("<breath>'</breath>");
                                } else if let Some(rest) = el.get("Rest") {
                                    if let Some(value) = rest.get("value").and_then(|v| v.as_str()) {
                                        xml.push_str(&format!("<rest>{}</rest>", escape_xml(value)));
                                    }
                                } else if let Some(newline) = el.get("Newline") {
                                    if let Some(value) = newline.get("value").and_then(|v| v.as_str()) {
                                        xml.push_str(&format!("<newline>{}</newline>", escape_xml(value)));
                                    }
                                }
                            }
                        }

                        xml.push_str("</pre>");
                    }
                }
            }
        }
    }

    xml
}

/// Generate syntax tokens from parsed document for CodeMirror highlighting (JSON-based for backward compatibility)
pub fn generate_syntax_tokens_from_json(document: &serde_json::Value) -> Vec<SyntaxToken> {
    let mut tokens = Vec::new();
    let mut position = 0usize;

    // Process elements array (actual document format)
    if let Some(elements) = document.get("elements").and_then(|e| e.as_array()) {
        for element in elements {
            // Handle BlankLines elements
            if let Some(blank_lines) = element.get("BlankLines") {
                if let Some(content) = blank_lines.get("content").and_then(|c| c.as_str()) {
                    tokens.push(SyntaxToken {
                        token_type: "blank_lines".to_string(),
                        start: position,
                        end: position + content.len(),
                        content: content.to_string(),
                    });
                    position += content.len();
                }
            }
            // Handle Stave elements
            else if let Some(stave) = element.get("Stave") {
                if let Some(lines) = stave.get("lines").and_then(|l| l.as_array()) {
                    for line in lines {
                        // Process BlankLines within stave
                        if let Some(blank_lines) = line.get("BlankLines") {
                            if let Some(content) = blank_lines.get("content").and_then(|c| c.as_str()) {
                                tokens.push(SyntaxToken {
                                    token_type: "stave_blank_lines".to_string(),
                                    start: position,
                                    end: position + content.len(),
                                    content: content.to_string(),
                                });
                                position += content.len();
                            }
                        }
                        // Process Content lines
                        else if let Some(content) = line.get("Content").and_then(|c| c.as_array()) {
                            for el in content {
                                if let Some(note) = el.get("Note") {
                                    if let Some(value) = note.get("value").and_then(|v| v.as_str()) {
                                        tokens.push(SyntaxToken {
                                            token_type: "note".to_string(),
                                            start: position,
                                            end: position + value.len(),
                                            content: value.to_string(),
                                        });
                                        position += value.len();
                                    }
                                } else if let Some(whitespace) = el.get("Whitespace") {
                                    if let Some(value) = whitespace.get("value").and_then(|v| v.as_str()) {
                                        // Just update position, don't create a token for whitespace
                                        position += value.len();
                                    }
                                } else if let Some(barline) = el.get("Barline") {
                                    if let Some(style) = barline.get("style").and_then(|s| s.as_str()) {
                                        tokens.push(SyntaxToken {
                                            token_type: "barline".to_string(),
                                            start: position,
                                            end: position + style.len(),
                                            content: style.to_string(),
                                        });
                                        position += style.len();
                                    }
                                } else if let Some(_dash) = el.get("Dash") {
                                    tokens.push(SyntaxToken {
                                        token_type: "dash".to_string(),
                                        start: position,
                                        end: position + 1,
                                        content: "-".to_string(),
                                    });
                                    position += 1;
                                } else if let Some(rest) = el.get("Rest") {
                                    if let Some(value) = rest.get("value").and_then(|v| v.as_str()) {
                                        tokens.push(SyntaxToken {
                                            token_type: "rest".to_string(),
                                            start: position,
                                            end: position + value.len(),
                                            content: value.to_string(),
                                        });
                                        position += value.len();
                                    }
                                } else if let Some(_breath) = el.get("Breath") {
                                    tokens.push(SyntaxToken {
                                        token_type: "breath".to_string(),
                                        start: position,
                                        end: position + 1,
                                        content: "'".to_string(),
                                    });
                                    position += 1;
                                } else if let Some(newline) = el.get("Newline") {
                                    if let Some(value) = newline.get("value").and_then(|v| v.as_str()) {
                                        // Just update position, don't create a token for newlines
                                        position += value.len();
                                    }
                                } else if let Some(unknown) = el.get("Unknown") {
                                    if let Some(value) = unknown.get("value").and_then(|v| v.as_str()) {
                                        tokens.push(SyntaxToken {
                                            token_type: "unknown".to_string(),
                                            start: position,
                                            end: position + value.len(),
                                            content: value.to_string(),
                                        });
                                        position += value.len();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    tokens
}

/// Generate syntax tokens from parsed document for CodeMirror highlighting
/// This version works directly with the Document struct for better performance and type safety
pub fn generate_syntax_tokens(document: &Document, original_input: &str) -> Vec<SyntaxToken> {
    let mut tokens = Vec::new();
    let mut position = 0usize;

    // Process directives first (if any)
    for directive in &document.directives {
        // Directive key
        let key_len = directive.key.len();
        if key_len > 0 {
            tokens.push(SyntaxToken {
                token_type: "directive_key".to_string(),
                start: position,
                end: position + key_len,
                content: directive.key.clone(),
            });
            position += key_len;
        }

        // Colon separator
        tokens.push(SyntaxToken {
            token_type: "directive_sep".to_string(),
            start: position,
            end: position + 1,
            content: ":".to_string(),
        });
        position += 1;

        // Skip space after colon - just update position
        position += 1;

        // Directive value
        let value_len = directive.value.len();
        if value_len > 0 {
            tokens.push(SyntaxToken {
                token_type: "directive_value".to_string(),
                start: position,
                end: position + value_len,
                content: directive.value.clone(),
            });
            position += value_len;
        }

        // Skip newline after directive - just update position
        position += 1;
    }

    // Process document elements
    for element in &document.elements {
        match element {
            DocumentElement::BlankLines(blank_lines) => {
                // Skip blank lines - just update position
                position += blank_lines.content.len();
            }
            DocumentElement::Stave(stave) => {
                // Process all lines in the stave
                for line in &stave.lines {
                    match line {
                        StaveLine::Text(text_line) => {
                            // Skip text lines - just update position
                            position += text_line.content.len();
                            if !text_line.content.ends_with('\n') {
                                position += 1; // newline
                            }
                        }
                        StaveLine::Upper(upper_line) => {
                            // Process upper line elements for slurs and ornaments
                            // Even though they're consumed, we need tokens for editor highlighting
                            for element in &upper_line.elements {
                                match element {
                                    UpperElement::SlurIndicator { value, source } => {
                                        // Generate token for slur indicator
                                        if let Some(start_pos) = source_position_to_absolute_offset(
                                            source.position.line,
                                            source.position.column,
                                            original_input
                                        ) {
                                            tokens.push(SyntaxToken {
                                                token_type: "slur".to_string(),
                                                start: start_pos,
                                                end: start_pos + value.len(),
                                                content: value.clone(),
                                            });
                                            position = start_pos + value.len();
                                        }
                                    }
                                    UpperElement::UpperOctaveMarker { marker, source } => {
                                        // Generate token for upper octave marker
                                        if let Some(start_pos) = source_position_to_absolute_offset(
                                            source.position.line,
                                            source.position.column,
                                            original_input
                                        ) {
                                            tokens.push(SyntaxToken {
                                                token_type: "upper-octave-marker".to_string(),
                                                start: start_pos,
                                                end: start_pos + marker.len(),
                                                content: marker.clone(),
                                            });
                                            position = start_pos + marker.len();
                                        }
                                    }
                                    _ => {
                                        // Other upper elements might need tokens too
                                    }
                                }
                            }

                            // Update position for the entire upper line if needed
                            if let Some(ref source_value) = upper_line.source.value {
                                let line_end = source_position_to_absolute_offset(
                                    upper_line.source.position.line,
                                    upper_line.source.position.column + source_value.len(),
                                    original_input
                                ).unwrap_or(position);
                                position = line_end;
                            }
                        }
                        StaveLine::Content(parsed_elements) => {
                            // Process content line elements
                            for element in parsed_elements {
                                process_parsed_element(element, &mut tokens, &mut position, original_input);
                            }
                        }
                        StaveLine::Lower(lower_line) => {
                            // Process lower line elements for beat group indicators
                            // Even though they're consumed, we need tokens for editor highlighting
                            for element in &lower_line.elements {
                                match element {
                                    LowerElement::BeatGroupIndicator { value, source } => {
                                        // Generate token for beat group indicator
                                        // The position tracker maintains the current position in the input
                                        // Since lower lines come after content, use current position
                                        tokens.push(SyntaxToken {
                                            token_type: "beat_group".to_string(),
                                            start: position,
                                            end: position + value.len(),
                                            content: value.clone(),
                                        });
                                        position += value.len();
                                    }
                                    LowerElement::LowerOctaveMarker { marker, source } => {
                                        // Generate token for octave marker (even if consumed)
                                        tokens.push(SyntaxToken {
                                            token_type: "lower-octave-marker".to_string(),
                                            start: position,
                                            end: position + marker.len(),
                                            content: marker.clone(),
                                        });
                                        position += marker.len();
                                    }
                                    _ => {
                                        // Other lower elements don't generate tokens
                                    }
                                }
                            }

                            // Update position for the entire lower line if needed
                            if let Some(ref source_value) = lower_line.source.value {
                                let line_end = source_position_to_absolute_offset(
                                    lower_line.source.position.line,
                                    lower_line.source.position.column + source_value.len(),
                                    original_input
                                ).unwrap_or(position);
                                position = line_end;
                            }
                        }
                        StaveLine::Lyrics(lyrics_line) => {
                            // Process lyrics syllables
                            for (i, syllable) in lyrics_line.syllables.iter().enumerate() {
                                if i > 0 {
                                    // Skip space between syllables - just update position
                                    position += 1;
                                }

                                let syllable_len = syllable.content.len();
                                if syllable_len > 0 {
                                    tokens.push(SyntaxToken {
                                        token_type: "syllable".to_string(),
                                        start: position,
                                        end: position + syllable_len,
                                        content: syllable.content.clone(),
                                    });
                                    position += syllable_len;
                                }
                            }
                            // Skip newline after lyrics - just update position
                            position += 1;
                        }
                        StaveLine::Whitespace(_whitespace_line) => {
                            // Skip whitespace line elements - don't track position since filtered out
                        }
                        StaveLine::BlankLines(blank_lines) => {
                            // Skip blank lines - just update position
                            position += blank_lines.content.len();
                        }
                    }
                }
            }
        }
    }

    // Don't fill gaps - just return the tokens we have
    // Whitespace, newlines and consumed elements should not generate tokens
    tokens
        .into_iter()
        .filter(|t| t.token_type != "whitespace" && t.token_type != "newline")
        .collect()
}


fn fill_token_gaps(mut tokens: Vec<SyntaxToken>, input_length: usize, original_input: &str) -> Vec<SyntaxToken> {
    // Step 1: Filter out whitespace and newline tokens (already done by caller)
    // tokens should only contain real content tokens at this point

    // Step 2: Create a coverage map
    let mut covered = vec![false; input_length];
    for token in &tokens {
        for pos in token.start..token.end {
            if pos < input_length {
                covered[pos] = true;
            }
        }
    }

    // Step 3: Fill gaps - step through each character
    let mut result = tokens; // Start with existing tokens
    let input_chars: Vec<char> = original_input.chars().collect();
    let mut gap_start = None;

    for (pos, &is_covered) in covered.iter().enumerate() {
        if !is_covered {
            // Start of a gap
            if gap_start.is_none() {
                gap_start = Some(pos);
            }
        } else {
            // End of a gap
            if let Some(start) = gap_start {
                let gap_content: String = input_chars[start..pos].iter().collect();
                result.push(SyntaxToken {
                    token_type: "dummy".to_string(),
                    start: start,
                    end: pos,
                    content: gap_content,
                });
                gap_start = None;
            }
        }
    }

    // Handle final gap if it extends to end of input
    if let Some(start) = gap_start {
        let gap_content: String = input_chars[start..].iter().collect();
        result.push(SyntaxToken {
            token_type: "dummy".to_string(),
            start: start,
            end: input_length,
            content: gap_content,
        });
    }

    // Step 4: Sort the final token list by position
    result.sort_by_key(|t| t.start);
    result
}

// Helper function to process upper line elements
fn process_upper_element(element: &crate::parse::UpperElement, tokens: &mut Vec<SyntaxToken>, position: &mut usize, original_input: &str) {
    use crate::parse::UpperElement;

    match element {
        UpperElement::UpperOctaveMarker { marker, source } => {
            let marker_len = marker.len();
            let token_type = match marker.as_str() {
                "." => "upper-octave-marker",
                ":" => "upper-octave-marker-2",
                _ => "upper-octave-marker"
            };
            // Use the actual source column position (already 0-based)
            let start_pos = source.position.column;
            tokens.push(SyntaxToken {
                token_type: token_type.to_string(),
                start: start_pos,
                end: start_pos + marker_len,
                content: marker.clone(),
            });
            *position += marker_len;
        }
        UpperElement::SlurIndicator { value, source } => {
            let value_len = value.len();
            // Use the actual source column position (already 0-based)
            let start_pos = source.position.column;
            tokens.push(SyntaxToken {
                token_type: "slur-indicator".to_string(),
                start: start_pos,
                end: start_pos + value_len,
                content: value.clone(),
            });
            *position += value_len;
        }
        UpperElement::UpperHashes { value, source } => {
            let value_len = value.len();
            // Use the actual source column position (already 0-based)
            let start_pos = source.position.column;
            tokens.push(SyntaxToken {
                token_type: "multi_stave_marker".to_string(),
                start: start_pos,
                end: start_pos + value_len,
                content: value.clone(),
            });
            *position += value_len;
        }
        UpperElement::Ornament { pitches, source } => {
            // Join pitches into ornament string
            let ornament_str = pitches.join("");
            let ornament_len = ornament_str.len();
            // Use the actual source column position (already 0-based)
            let start_pos = source.position.column;
            tokens.push(SyntaxToken {
                token_type: "ornament".to_string(),
                start: start_pos,
                end: start_pos + ornament_len,
                content: ornament_str,
            });
            *position += ornament_len;
        }
        UpperElement::Chord { chord, source } => {
            // Include brackets in chord token
            let chord_str = format!("[{}]", chord);
            let chord_len = chord_str.len();
            // Use the actual source column position (already 0-based)
            let start_pos = source.position.column;
            tokens.push(SyntaxToken {
                token_type: "chord".to_string(),
                start: start_pos,
                end: start_pos + chord_len,
                content: chord_str,
            });
            *position += chord_len;
        }
        UpperElement::Mordent { source } => {
            // Use the actual source column position (already 0-based)
            let start_pos = source.position.column;
            tokens.push(SyntaxToken {
                token_type: "mordent".to_string(),
                start: start_pos,
                end: start_pos + 1,
                content: "~".to_string(),
            });
            *position += 1;
        }
        UpperElement::Space { count, source } => {
            let spaces = " ".repeat(*count);
            // Use the actual source column position (already 0-based)
            let start_pos = source.position.column;
            tokens.push(SyntaxToken {
                token_type: "whitespace".to_string(),
                start: start_pos,
                end: start_pos + count,
                content: spaces,
            });
            *position += count;
        }
        UpperElement::Unknown { value, source } => {
            let value_len = value.len();
            // Use the actual source column position (already 0-based)
            let start_pos = source.position.column;
            tokens.push(SyntaxToken {
                token_type: "unknown".to_string(),
                start: start_pos,
                end: start_pos + value_len,
                content: value.clone(),
            });
            *position += value_len;
        }
        UpperElement::Newline { value, source } => {
            let value_len = value.len();
            // Use the actual source column position (already 0-based)
            let start_pos = source.position.column;
            tokens.push(SyntaxToken {
                token_type: "newline".to_string(),
                start: start_pos,
                end: start_pos + value_len,
                content: value.clone(),
            });
            *position += value_len;
        }
    }
}

// Helper function to process lower line elements
fn process_lower_element(element: &crate::parse::LowerElement, tokens: &mut Vec<SyntaxToken>, position: &mut usize, original_input: &str) {
    use crate::parse::LowerElement;

    match element {
        LowerElement::LowerOctaveMarker { marker, source } => {
            let marker_len = marker.len();
            let token_type = match marker.as_str() {
                "." => "lower-octave-marker",
                ":" => "lower-octave-marker-2",
                _ => "lower-octave-marker"
            };
            // Use the actual source column position (already 0-based)
            let start_pos = source.position.column;
            tokens.push(SyntaxToken {
                token_type: token_type.to_string(),
                start: start_pos,
                end: start_pos + marker_len,
                content: marker.clone(),
            });
            *position += marker_len;
        }
        LowerElement::BeatGroupIndicator { value, source } => {
            let value_len = value.len();
            // Convert source position to absolute offset
            let start_pos = source_position_to_absolute_offset(source.position.line, source.position.column, original_input)
                .unwrap_or_else(|| {
                    eprintln!("Warning: Failed to convert beat group position line={}, column={}", source.position.line, source.position.column);
                    0
                });
            tokens.push(SyntaxToken {
                token_type: "beat_group".to_string(),
                start: start_pos,
                end: start_pos + value_len,
                content: value.clone(),
            });
            *position += value_len;
        }
        LowerElement::Syllable { content, source } => {
            let content_len = content.len();
            // Use the actual source column position (already 0-based)
            let start_pos = source.position.column;
            tokens.push(SyntaxToken {
                token_type: "syllable".to_string(),
                start: start_pos,
                end: start_pos + content_len,
                content: content.clone(),
            });
            *position += content_len;
        }
        LowerElement::Space { count, source } => {
            let spaces = " ".repeat(*count);
            // Use the actual source column position (already 0-based)
            let start_pos = source.position.column;
            tokens.push(SyntaxToken {
                token_type: "whitespace".to_string(),
                start: start_pos,
                end: start_pos + count,
                content: spaces,
            });
            *position += count;
        }
        LowerElement::Unknown { value, source } => {
            let value_len = value.len();
            // Use the actual source column position (already 0-based)
            let start_pos = source.position.column;
            tokens.push(SyntaxToken {
                token_type: "unknown".to_string(),
                start: start_pos,
                end: start_pos + value_len,
                content: value.clone(),
            });
            *position += value_len;
        }
        LowerElement::Newline { value, source } => {
            let value_len = value.len();
            // Use the actual source column position (already 0-based)
            let start_pos = source.position.column;
            tokens.push(SyntaxToken {
                token_type: "newline".to_string(),
                start: start_pos,
                end: start_pos + value_len,
                content: value.clone(),
            });
            *position += value_len;
        }
        LowerElement::EndOfInput { .. } => {
            // End of input doesn't generate a token
        }
    }
}

// Helper function to process parsed elements from content lines
fn process_parsed_element(element: &ParsedElement, tokens: &mut Vec<SyntaxToken>, position: &mut usize, original_input: &str) {
    match element {
        ParsedElement::Note { value, position: pos, in_slur, in_beat_group, beat_group, .. } => {
            // Calculate absolute position from row/col
            if let Some(start_pos) = position_to_absolute_offset(pos, original_input) {
                let value_len = value.len();
                let mut token_type = "note".to_string();

                // Keep token type as just "note" - beat group styling handled via character styles

                tokens.push(SyntaxToken {
                    token_type,
                    start: start_pos,
                    end: start_pos + value_len,
                    content: value.clone(),
                });
                *position = start_pos + value_len;
            }
        }
        ParsedElement::Rest { value, position: pos, .. } => {
            // Calculate absolute position from row/col
            if let Some(start_pos) = position_to_absolute_offset(pos, original_input) {
                let value_len = value.len();
                tokens.push(SyntaxToken {
                    token_type: "rest".to_string(),
                    start: start_pos,
                    end: start_pos + value_len,
                    content: value.clone(),
                });
                *position = start_pos + value_len;
            }
        }
        ParsedElement::Dash { position: pos, .. } => {
            // Calculate absolute position from row/col
            if let Some(start_pos) = position_to_absolute_offset(pos, original_input) {
                tokens.push(SyntaxToken {
                    token_type: "dash".to_string(),
                    start: start_pos,
                    end: start_pos + 1,
                    content: "-".to_string(),
                });
                *position = start_pos + 1;
            }
        }
        ParsedElement::Barline { style, position: pos, .. } => {
            // Calculate absolute position from row/col
            if let Some(start_pos) = position_to_absolute_offset(pos, original_input) {
                let style_len = style.len();
                tokens.push(SyntaxToken {
                    token_type: "barline".to_string(),
                    start: start_pos,
                    end: start_pos + style_len,
                    content: style.clone(),
                });
                *position = start_pos + style_len;
            }
        }
        ParsedElement::Whitespace { value, position: pos, .. } => {
            // Calculate absolute position from row/col
            if let Some(start_pos) = position_to_absolute_offset(pos, original_input) {
                let value_len = value.len();
                tokens.push(SyntaxToken {
                    token_type: "whitespace".to_string(),
                    start: start_pos,
                    end: start_pos + value_len,
                    content: value.clone(),
                });
                *position = start_pos + value_len;
            }
        }
        ParsedElement::Symbol { value, position: pos, .. } => {
            // Calculate absolute position from row/col
            if let Some(start_pos) = position_to_absolute_offset(pos, original_input) {
                let value_len = value.len();
                tokens.push(SyntaxToken {
                    token_type: "symbol".to_string(),
                    start: start_pos,
                    end: start_pos + value_len,
                    content: value.clone(),
                });
                *position = start_pos + value_len;
            }
        }
        ParsedElement::Unknown { value, position: pos } => {
            // Calculate absolute position from row/col
            if let Some(start_pos) = position_to_absolute_offset(pos, original_input) {
                let value_len = value.len();
                tokens.push(SyntaxToken {
                    token_type: "unknown".to_string(),
                    start: start_pos,
                    end: start_pos + value_len,
                    content: value.clone(),
                });
                *position = start_pos + value_len;
            }
        }
        ParsedElement::Newline { value, position: pos, .. } => {
            // Calculate absolute position from row/col
            if let Some(start_pos) = position_to_absolute_offset(pos, original_input) {
                let value_len = value.len();
                tokens.push(SyntaxToken {
                    token_type: "newline".to_string(),
                    start: start_pos,
                    end: start_pos + value_len,
                    content: value.clone(),
                });
                *position = start_pos + value_len;
            }
        }
        ParsedElement::EndOfInput { .. } => {
            // End of input doesn't generate a token
        }
    }
}