use serde::Serialize;
use std::collections::HashMap;
use crate::parse::{Document, UpperElement, LowerElement};
use crate::parse::model::{DocumentElement, StaveLine};
use crate::rhythm::types::{ParsedElement, ParsedChild, Position};

/// Convert row/col position to absolute character position in original input
fn position_to_absolute_offset(position: &crate::rhythm::types::Position, original_input: &str) -> Option<usize> {
    let lines: Vec<&str> = original_input.split('\n').collect();

    // Handle the case where position.row is 0-based vs 1-based
    // The parser seems to use 1-based row numbering for content, but we need 0-based for array indexing
    // For content lines, row 1 = lines[0], row 0 = lines[0] (both map to first line)
    let target_row = if position.row >= 1 { position.row - 1 } else { position.row };

    if target_row >= lines.len() {
        return None;
    }

    let mut offset = 0;
    // Add lengths of all previous lines (including newlines)
    for i in 0..target_row {
        offset += lines[i].len();
        // Add newline character unless it's the last line
        if i < lines.len() - 1 {
            offset += 1;
        }
    }

    // Add column offset within the current line (col is already 0-based)
    if position.col <= lines[target_row].len() {
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
pub struct Span {
    pub r#type: String,
    pub start: usize,
    pub end: usize,
    pub content: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct CharacterStyle {
    pub pos: usize,
    pub length: usize,  // Length of the token
    pub classes: Vec<String>,
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub styles: std::collections::HashMap<String, String>, // CSS custom properties
}

#[derive(Debug, Serialize, Clone)]
pub struct DocumentNode {
    pub tag: String,        // "note", "dash", "barline", etc.
    pub pos: usize,         // Character position (from row/col conversion)
    pub char_index: usize,  // Zero-based index into whole document
    pub length: usize,      // Character length
    pub content: String,    // Original text content
    pub classes: Vec<String>, // Semantic classes like "beat-loop-4"
}

#[derive(Debug, Serialize, Clone)]
pub struct TokenInfo {
    pub pos: usize,
    pub length: usize,
    pub r#type: String,
    pub content: String,
}


/// Generate normalized elements from rhythm-analyzed document
/// Single tree walk to create semantic annotations
pub fn generate_normalized_elements(rhythm_doc: &crate::parse::Document, original_input: &str) -> Vec<DocumentNode> {
    use crate::parse::model::DocumentElement;

    let mut elements = Vec::new();

    for doc_element in &rhythm_doc.elements {
        if let DocumentElement::Stave(_stave) = doc_element {
            // TODO: Process ContentLine beats instead of rhythm_items
            // Disabled for now until ContentLine beats are properly integrated
        }
    }

    // Sort by position
    elements.sort_by_key(|e: &DocumentNode| e.pos);
    elements
}

/// Convert document nodes to spans for editor syntax highlighting
pub fn nodes_to_spans(nodes: &[DocumentNode]) -> Vec<Span> {
    nodes.iter()
        .filter(|node| node.tag != "whitespace")  // Skip whitespace in spans
        .map(|node| Span {
            r#type: node.tag.clone(),
            start: node.pos,
            end: node.pos + node.length,
            content: node.content.clone(),
        })
        .collect()
}

/// Generate spans and styles in a single pass with consistent positioning
pub fn nodes_to_spans_and_styles(nodes: &[DocumentNode]) -> (Vec<Span>, Vec<CharacterStyle>) {
    let mut spans = Vec::new();
    let mut styles = Vec::new();

    // First pass: create base spans and styles with consistent positioning
    for node in nodes.iter() {
        // Create span (skip whitespace for spans)
        if node.tag != "whitespace" {
            spans.push(Span {
                r#type: node.tag.clone(),
                start: node.pos,
                end: node.pos + node.length,
                content: node.content.clone(),
            });
        }

        // Create base style for all nodes
        let mut css_styles = HashMap::new();
        let mut css_classes = vec![format!("cm-music-{}", node.tag)];

        // Add classes from the node
        css_classes.extend(node.classes.iter().cloned());

        styles.push(CharacterStyle {
            pos: node.pos,
            length: node.length,
            classes: css_classes,
            styles: css_styles,
        });
    }

    // Second pass: add beat loop styling
    for (i, node) in nodes.iter().enumerate() {
        if node.tag == "whitespace" {
            continue;
        }

        // Check if this is the first element of a beat group
        let is_beat_loop_first = node.classes.iter().any(|c| c.starts_with("beat-loop-"));

        if is_beat_loop_first {
            // Extract loop size and add CSS styles
            if let Some(beat_loop_class) = node.classes.iter().find(|c| c.starts_with("beat-loop-")) {
                if let Some(captures) = regex::Regex::new(r"beat-loop-(\d+)").unwrap().captures(beat_loop_class) {
                    let loop_size = &captures[1];
                    if let Some(style) = styles.iter_mut().find(|s| s.pos == node.pos) {
                        style.styles.insert("--lower-loop-char-count".to_string(), loop_size.to_string());
                        style.styles.insert("--show-divisions".to_string(), loop_size.to_string());
                    }
                }
            }
        }

        // Add tuplet labels to middle elements
        let beat_loop_size: Option<usize> = if node.classes.iter().any(|c| c.starts_with("beat-loop-")) {
            // Current element starts the group
            node.classes.iter()
                .find(|c| c.starts_with("beat-loop-"))
                .and_then(|c| regex::Regex::new(r"beat-loop-(\d+)").unwrap().captures(c))
                .and_then(|captures| captures[1].parse().ok())
        } else {
            // Find previous element with beat-loop class
            let mut prev_loop_size = None;
            for j in (0..i).rev() {
                if nodes[j].tag == "whitespace" {
                    continue;
                }
                if let Some(beat_loop_class) = nodes[j].classes.iter().find(|c| c.starts_with("beat-loop-")) {
                    if let Some(captures) = regex::Regex::new(r"beat-loop-(\d+)").unwrap().captures(beat_loop_class) {
                        prev_loop_size = captures[1].parse().ok();
                        break;
                    }
                }
                break;
            }
            prev_loop_size
        };

        if let Some(loop_size) = beat_loop_size {
            // Find beat group start
            let beat_group_start = if node.classes.iter().any(|c| c.starts_with("beat-loop-")) {
                i
            } else {
                let mut start = i;
                for j in (0..i).rev() {
                    if nodes[j].tag == "whitespace" {
                        continue;
                    }
                    if nodes[j].classes.iter().any(|c| c.starts_with("beat-loop-")) {
                        start = j;
                        break;
                    }
                    break;
                }
                start
            };

            // Count position within beat group
            let mut musical_count = 0;
            for j in beat_group_start..=i {
                if j < nodes.len() && nodes[j].tag != "whitespace" {
                    musical_count += 1;
                }
            }

            // Add tuplet to middle element
            let should_show_tuplet = (loop_size % 2 == 1 && loop_size >= 3) || (loop_size > 9);
            let is_middle = if loop_size % 2 == 1 {
                musical_count == (loop_size + 1) / 2
            } else {
                musical_count == loop_size / 2
            };

            if is_middle && should_show_tuplet {
                if let Some(style) = styles.iter_mut().find(|s| s.pos == node.pos) {
                    style.styles.insert("--tuplet".to_string(), format!("'{}'", loop_size));
                }
            }
        }
    }

    (spans, styles)
}

/// Convert document nodes to character styles for web display (DEPRECATED)
pub fn nodes_to_styles(nodes: &[DocumentNode]) -> Vec<CharacterStyle> {
    let mut styles: Vec<CharacterStyle> = Vec::new();

    for (i, node) in nodes.iter().enumerate() {
        if node.tag == "whitespace" {
            continue; // Skip whitespace in styles
        }

        let mut classes = vec![format!("cm-music-{}", node.tag)];
        classes.extend(node.classes.clone());

        let mut css_styles = std::collections::HashMap::new();

        // Check if this is a beat-loop first element
        let is_beat_loop_first = node.classes.iter().any(|c| c.starts_with("beat-loop-"));

        if is_beat_loop_first {
            // Extract loop size from beat-loop-N class
            if let Some(beat_loop_class) = node.classes.iter().find(|c| c.starts_with("beat-loop-")) {
                if let Some(captures) = regex::Regex::new(r"beat-loop-(\d+)").unwrap().captures(beat_loop_class) {
                    let loop_size = &captures[1];
                    css_styles.insert("--lower-loop-char-count".to_string(), loop_size.to_string());
                    css_styles.insert("--show-divisions".to_string(), loop_size.to_string());

                    // Find middle element for --show-divisions
                    let mut beat_elements = vec![i];

                    // Find all subsequent elements that are part of this beat group
                    for j in (i + 1)..nodes.len() {
                        if nodes[j].tag == "whitespace" {
                            continue;
                        }
                        // Check if this element is part of the same beat group by checking position proximity
                        // Since we don't have explicit beat grouping info, we'll count consecutive musical elements
                        if j - i <= loop_size.parse::<usize>().unwrap_or(1) {
                            beat_elements.push(j);
                        } else {
                            break;
                        }
                    }

                    // Add --show-divisions to middle element(s)
                    if beat_elements.len() > 2 {
                        let middle_idx = beat_elements[beat_elements.len() / 2];
                        if middle_idx != i {
                            // We'll mark this for later processing
                            // For now, just note which elements need --show-divisions
                        }
                    }
                }
            }
        }

        styles.push(CharacterStyle {
            pos: node.pos,
            length: node.length,
            classes,
            styles: css_styles,
        });
    }

    // Second pass: add --show-divisions to middle elements
    for (i, node) in nodes.iter().enumerate() {
        if node.tag == "whitespace" {
            continue;
        }

        // Check if current or previous element has beat-loop class
        let mut beat_loop_size: Option<usize> = None;

        // First check if current element has beat-loop class
        if let Some(beat_loop_class) = node.classes.iter().find(|c| c.starts_with("beat-loop-")) {
            if let Some(captures) = regex::Regex::new(r"beat-loop-(\d+)").unwrap().captures(beat_loop_class) {
                beat_loop_size = captures[1].parse().ok();
            }
        } else {
            // If not, check previous element
            for j in (0..i).rev() {
                if nodes[j].tag == "whitespace" {
                    continue;
                }
                if let Some(beat_loop_class) = nodes[j].classes.iter().find(|c| c.starts_with("beat-loop-")) {
                    if let Some(captures) = regex::Regex::new(r"beat-loop-(\d+)").unwrap().captures(beat_loop_class) {
                        beat_loop_size = captures[1].parse().ok();
                        break;
                    }
                }
                break; // Only check immediate previous musical element
            }
        }

        // If we have a beat loop, determine position within the group
        if let Some(loop_size) = beat_loop_size {
            // Find the start of the current beat group
            let mut beat_group_start = i;
            if node.classes.iter().any(|c| c.starts_with("beat-loop-")) {
                // Current element starts the group
                beat_group_start = i;
            } else {
                // Find the previous element with beat-loop class
                for j in (0..i).rev() {
                    if nodes[j].tag == "whitespace" {
                        continue;
                    }
                    if nodes[j].classes.iter().any(|c| c.starts_with("beat-loop-")) {
                        beat_group_start = j;
                        break;
                    }
                    break;
                }
            }

            // Count position within the beat group
            let mut musical_count = 0;
            for j in beat_group_start..=i {
                if j < nodes.len() && nodes[j].tag != "whitespace" {
                    musical_count += 1;
                }
            }

            // Only add --show-divisions for:
            // - Odd numbers: 3, 5, 7, 9
            // - Numbers above 9: 10, 11, 12, 13+
            let should_show_divisions = (loop_size % 2 == 1 && loop_size >= 3) || (loop_size > 9);

            // Add tuplet to middle element (for odd-sized groups)
            let is_middle = if loop_size % 2 == 1 {
                musical_count == (loop_size + 1) / 2
            } else {
                musical_count == loop_size / 2
            };

            if is_middle && should_show_divisions {
                // This is the middle element, add --tuplet with the division number
                if let Some(style) = styles.iter_mut().find(|s| s.pos == node.pos) {
                    style.styles.insert("--tuplet".to_string(), format!("'{}'", loop_size));
                }
            }
        }
    }

    styles
}

/// DEPRECATED: Use nodes_to_spans_and_styles() instead
/// Generate spans and styles from document nodes (legacy function)
pub fn generate_spans_and_styles(nodes: &[DocumentNode]) -> (Vec<Span>, Vec<CharacterStyle>) {
    nodes_to_spans_and_styles(nodes)
}

/// Convert syntax spans to token-based styles for client-side application
pub fn generate_character_styles(spans: &[Span]) -> Vec<CharacterStyle> {
    // Map each token to a single style entry (not per-character)
    let mut styles: Vec<CharacterStyle> = spans
        .iter()
        .filter(|token| token.r#type != "whitespace" && token.r#type != "newline")
        .map(|token| {
            let css_class = format!("cm-music-{}", token.r#type);
            CharacterStyle {
                pos: token.start,  // Token start position
                length: token.end - token.start,  // Token length
                classes: vec![css_class],
                styles: std::collections::HashMap::new(),
            }
        })
        .collect();

    // Sort by position
    styles.sort_by_key(|style| style.pos);

    // Return token-based styles (no gap filling needed)
    styles
}

/// Enhanced character styles generation with beat group and slur information
/// Uses rhythm-analyzed document to identify both explicit beat groups (marked with ___)
/// and implicit beat groups (consecutive musical elements with same beat)
/// Also processes slurs (marked with ___ in upper lines)
pub fn generate_character_styles_with_beat_groups(spans: &[Span], document: &crate::parse::Document) -> Vec<CharacterStyle> {
    use crate::parse::model::{DocumentElement, StaveLine};
    use crate::rhythm::types::{ParsedElement, BeatGroupRole};

    let mut styles = generate_character_styles(spans);

    // Add beat group and slur classes to elements
    for element in &document.elements {
        if let DocumentElement::Stave(stave) = element {
            for line in &stave.lines {
                if let StaveLine::Content(content_elements) = line {
                    // SKIP: Process explicit beat groups (marked with ___)
                    // process_explicit_beat_groups(&mut styles, content_elements, document);

                    // Process implicit beat groups (spatially-delimited beats with multiple elements)
                    process_rhythm_based_implicit_beats(&mut styles, content_elements, stave);

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
                    // Use the actual row from the element's position, not hardcoded row: 1
                    let element_position = match element {
                        ParsedElement::Note { position, .. } => position,
                        ParsedElement::Rest { position, .. } => position,
                        ParsedElement::Dash { position, .. } => position,
                        ParsedElement::Barline { position, .. } => position,
                        ParsedElement::Whitespace { position, .. } => position,
                        _ => continue,
                    };
                    if let Some(absolute_pos) = position_to_absolute_offset(element_position, &document.source.value.clone().unwrap_or_default()) {
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

/// Process rhythm-based implicit beats (spatially-delimited beats from rhythm analysis)
fn process_rhythm_based_implicit_beats(
    styles: &mut Vec<CharacterStyle>,
    content_elements: &[crate::rhythm::types::ParsedElement],
    stave: &crate::parse::model::Stave
) {
    use crate::rhythm::Item;

    // Get rhythm items from the stave
    if let Some(_rhythm_items) = None::<&Vec<()>> { // TODO: adapt to new structure
        // TODO: Process ContentLine beats instead
        if false { // Disabled: old rhythm processing
            if let Item::Beat(beat) = &Item::Beat(crate::rhythm::analyzer::Beat { divisions: 1, elements: vec![], tied_to_previous: false, is_tuplet: false, tuplet_ratio: None }) {
                // Only process beats with multiple elements (spatially-delimited beats)
                if beat.elements.len() >= 2 {
                    let mut beat_positions = Vec::new();

                    // Collect positions of all elements in this beat
                    for beat_element in &beat.elements {
                        if let Some(absolute_pos) = position_to_absolute_offset(&beat_element.position, &stave.source.value.as_ref().unwrap()) {
                            beat_positions.push(absolute_pos);
                        }
                    }

                    // Apply implicit beat classes
                    if beat_positions.len() >= 2 {
                        beat_positions.sort();
                        add_implicit_beat_classes(styles, &beat_positions, beat_positions.len());
                    }
                }
            }
        }
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
                    // Use the actual row from the element's position, not hardcoded row: 1
                    let element_position = match element {
                        ParsedElement::Note { position, .. } => position,
                        ParsedElement::Rest { position, .. } => position,
                        ParsedElement::Dash { position, .. } => position,
                        ParsedElement::Barline { position, .. } => position,
                        ParsedElement::Whitespace { position, .. } => position,
                        _ => continue,
                    };
                    if let Some(absolute_pos) = position_to_absolute_offset(element_position, &document.source.value.clone().unwrap_or_default()) {
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
                // Note: Width will be set dynamically via CSS custom properties
            } else if i == positions.len() - 1 {
                style.classes.push("beat-group-end".to_string());
            } else {
                style.classes.push("beat-group-middle".to_string());
            }
        }
    }
}

fn add_implicit_beat_classes(styles: &mut Vec<CharacterStyle>, positions: &[usize], count: usize) {
    for (i, &pos) in positions.iter().enumerate() {
        // Find the style at this position and add implicit beat classes
        if let Some(style) = styles.iter_mut().find(|s| s.pos == pos) {
            // Add base in-implicit-beat class
            style.classes.push("in-implicit-beat".to_string());

            // Add role-specific class based on position in group
            if i == 0 {
                style.classes.push("implicit-beat-start".to_string());
                // Note: Width will be set dynamically via CSS custom properties
            } else if i == positions.len() - 1 {
                style.classes.push("implicit-beat-end".to_string());
            } else {
                style.classes.push("implicit-beat-middle".to_string());
            }
        }
    }
}


/// Generate syntax spans from parsed document for CodeMirror highlighting (JSON-based for backward compatibility)
pub fn generate_syntax_spans_from_json(document: &serde_json::Value) -> Vec<Span> {
    let mut spans = Vec::new();
    let mut position = 0usize;

    // Process elements array (actual document format)
    if let Some(elements) = document.get("elements").and_then(|e| e.as_array()) {
        for element in elements {
            // Handle BlankLines elements
            if let Some(blank_lines) = element.get("BlankLines") {
                if let Some(content) = blank_lines.get("content").and_then(|c| c.as_str()) {
                    spans.push(Span {
                        r#type: "blank_lines".to_string(),
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
                                spans.push(Span {
                                    r#type: "stave_blank_lines".to_string(),
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
                                        spans.push(Span {
                                            r#type: "note".to_string(),
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
                                        spans.push(Span {
                                            r#type: "barline".to_string(),
                                            start: position,
                                            end: position + style.len(),
                                            content: style.to_string(),
                                        });
                                        position += style.len();
                                    }
                                } else if let Some(_dash) = el.get("Dash") {
                                    spans.push(Span {
                                        r#type: "dash".to_string(),
                                        start: position,
                                        end: position + 1,
                                        content: "-".to_string(),
                                    });
                                    position += 1;
                                } else if let Some(rest) = el.get("Rest") {
                                    if let Some(value) = rest.get("value").and_then(|v| v.as_str()) {
                                        spans.push(Span {
                                            r#type: "rest".to_string(),
                                            start: position,
                                            end: position + value.len(),
                                            content: value.to_string(),
                                        });
                                        position += value.len();
                                    }
                                } else if let Some(_breath) = el.get("Breath") {
                                    spans.push(Span {
                                        r#type: "breath".to_string(),
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
                                        spans.push(Span {
                                            r#type: "unknown".to_string(),
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

    spans
}

/// Generate syntax spans from parsed document for CodeMirror highlighting
/// This version works directly with the Document struct for better performance and type safety
pub fn generate_syntax_spans(document: &Document, original_input: &str) -> Vec<Span> {
    let mut spans = Vec::new();
    let mut position = 0usize;

    // Process directives first (if any)
    for (key, value) in &document.directives {
        // Directive key
        let key_len = key.len();
        if key_len > 0 {
            spans.push(Span {
                r#type: "directive_key".to_string(),
                start: position,
                end: position + key_len,
                content: key.clone(),
            });
            position += key_len;
        }

        // Colon separator
        spans.push(Span {
            r#type: "directive_sep".to_string(),
            start: position,
            end: position + 1,
            content: ":".to_string(),
        });
        position += 1;

        // Skip space after colon - just update position
        position += 1;

        // Directive value
        let value_len = value.len();
        if value_len > 0 {
            spans.push(Span {
                r#type: "directive_value".to_string(),
                start: position,
                end: position + value_len,
                content: value.clone(),
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
                            // Even though they're consumed, we need spans for editor highlighting
                            for element in &upper_line.elements {
                                match element {
                                    UpperElement::SlurIndicator { value, source } => {
                                        // Generate token for slur indicator
                                        if let Some(start_pos) = source_position_to_absolute_offset(
                                            source.position.line,
                                            source.position.column,
                                            original_input
                                        ) {
                                            spans.push(Span {
                                                r#type: "note-slur".to_string(),
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
                                            spans.push(Span {
                                                r#type: "upper-octave-marker".to_string(),
                                                start: start_pos,
                                                end: start_pos + marker.len(),
                                                content: marker.clone(),
                                            });
                                            position = start_pos + marker.len();
                                        }
                                    }
                                    _ => {
                                        // Other upper elements might need spans too
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
                            // Process rhythm items first if available, otherwise fall back to individual elements
                            if let Some(_rhythm_items) = None::<&Vec<()>> { // TODO: adapt to new structure
                                // TODO: Process ContentLine beats for spans
                            } else {
                                // Fallback: process content line elements individually
                                for element in parsed_elements {
                                    process_parsed_element(element, &mut spans, &mut position, original_input, &None);
                                }
                            }
                        }
                        StaveLine::Lower(lower_line) => {
                            // Process lower line elements for beat group indicators
                            // Even though they're consumed, we need spans for editor highlighting
                            for element in &lower_line.elements {
                                match element {
                                    LowerElement::BeatGroupIndicator { value, source } => {
                                        // Generate token for beat group indicator
                                        // The position tracker maintains the current position in the input
                                        // Since lower lines come after content, use current position
                                        spans.push(Span {
                                            r#type: "beat_group".to_string(),
                                            start: position,
                                            end: position + value.len(),
                                            content: value.clone(),
                                        });
                                        position += value.len();
                                    }
                                    LowerElement::LowerOctaveMarker { marker, source } => {
                                        // Generate token for octave marker (even if consumed)
                                        spans.push(Span {
                                            r#type: "lower-octave-marker".to_string(),
                                            start: position,
                                            end: position + marker.len(),
                                            content: marker.clone(),
                                        });
                                        position += marker.len();
                                    }
                                    _ => {
                                        // Other lower elements don't generate spans
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
                                    spans.push(Span {
                                        r#type: "syllable".to_string(),
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
                        StaveLine::ContentLine(_content_line) => {
                            // TODO: Process ContentLine beats for spans
                            // For now, just skip
                        }
                    }
                }
            }
        }
    }

    // Don't fill gaps - just return the spans we have
    // Whitespace, newlines and consumed elements should not generate spans
    spans
        .into_iter()
        .filter(|t| t.r#type != "whitespace" && t.r#type != "newline")
        .collect()
}


/// Process rhythm items to generate beat-aware spans
fn process_rhythm_items_for_spans(
    rhythm_items: &[crate::rhythm::Item],
    spans: &mut Vec<Span>,
    position: &mut usize,
    original_input: &str
) {
    for item in rhythm_items {
        match item {
            crate::rhythm::Item::Beat(beat) => {
                // Process each element in the beat individually, but use beat context for token type
                let beat_size = beat.divisions;

                for (index, element) in beat.elements.iter().enumerate() {
                    if let Some(start_pos) = position_to_absolute_offset(&element.position, original_input) {
                        // Just use plain "note" for all notes
                        let r#type = "note".to_string();

                        spans.push(Span {
                            r#type,
                            start: start_pos,
                            end: start_pos + element.value.len(),
                            content: element.value.clone(),
                        });
                        *position = start_pos + element.value.len();
                    }
                }
            }
            crate::rhythm::Item::Barline(barline_type, tala) => {
                // Handle barlines
                // For now, skip barlines in tokenization
            }
            crate::rhythm::Item::Breathmark => {
                // Handle breathmarks
                // For now, skip breathmarks in tokenization
            }
            crate::rhythm::Item::Tonic(_) => {
                // Handle tonic declarations
                // For now, skip tonic in tokenization
            }
        }
    }
}

fn fill_token_gaps(mut spans: Vec<Span>, input_length: usize, original_input: &str) -> Vec<Span> {
    // Step 1: Filter out whitespace and newline spans (already done by caller)
    // spans should only contain real content spans at this point

    // Step 2: Create a coverage map
    let mut covered = vec![false; input_length];
    for token in &spans {
        for pos in token.start..token.end {
            if pos < input_length {
                covered[pos] = true;
            }
        }
    }

    // Step 3: Fill gaps - step through each character
    let mut result = spans; // Start with existing spans
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
                result.push(Span {
                    r#type: "dummy".to_string(),
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
        result.push(Span {
            r#type: "dummy".to_string(),
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
fn process_upper_element(element: &crate::parse::UpperElement, spans: &mut Vec<Span>, position: &mut usize, original_input: &str) {
    use crate::parse::UpperElement;

    match element {
        UpperElement::UpperOctaveMarker { marker, source } => {
            let marker_len = marker.len();
            let r#type = match marker.as_str() {
                "." => "upper-octave-marker",
                ":" => "upper-octave-marker-2",
                _ => "upper-octave-marker"
            };
            // Use the actual source column position (already 0-based)
            let start_pos = source.position.column;
            spans.push(Span {
                r#type: r#type.to_string(),
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
            spans.push(Span {
                r#type: "slur-indicator".to_string(),
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
            spans.push(Span {
                r#type: "multi_stave_marker".to_string(),
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
            spans.push(Span {
                r#type: "ornament".to_string(),
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
            spans.push(Span {
                r#type: "chord".to_string(),
                start: start_pos,
                end: start_pos + chord_len,
                content: chord_str,
            });
            *position += chord_len;
        }
        UpperElement::Mordent { source } => {
            // Use the actual source column position (already 0-based)
            let start_pos = source.position.column;
            spans.push(Span {
                r#type: "mordent".to_string(),
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
            spans.push(Span {
                r#type: "whitespace".to_string(),
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
            spans.push(Span {
                r#type: "unknown".to_string(),
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
            spans.push(Span {
                r#type: "newline".to_string(),
                start: start_pos,
                end: start_pos + value_len,
                content: value.clone(),
            });
            *position += value_len;
        }
    }
}

// Helper function to process lower line elements
fn process_lower_element(element: &crate::parse::LowerElement, spans: &mut Vec<Span>, position: &mut usize, original_input: &str) {
    use crate::parse::LowerElement;

    match element {
        LowerElement::LowerOctaveMarker { marker, source } => {
            let marker_len = marker.len();
            let r#type = match marker.as_str() {
                "." => "lower-octave-marker",
                ":" => "lower-octave-marker-2",
                _ => "lower-octave-marker"
            };
            // Use the actual source column position (already 0-based)
            let start_pos = source.position.column;
            spans.push(Span {
                r#type: r#type.to_string(),
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
            spans.push(Span {
                r#type: "beat_group".to_string(),
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
            spans.push(Span {
                r#type: "syllable".to_string(),
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
            spans.push(Span {
                r#type: "whitespace".to_string(),
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
            spans.push(Span {
                r#type: "unknown".to_string(),
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
            spans.push(Span {
                r#type: "newline".to_string(),
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

// Helper function to find the number of elements in the beat containing this position
fn find_beat_element_count(rhythm_items: &[crate::rhythm::Item], target_position: &Position) -> usize {
    use crate::rhythm::Item;

    for item in rhythm_items {
        if let Item::Beat(beat) = item {
            // Check if any element in this beat matches the target position
            for beat_element in &beat.elements {
                if beat_element.position.row == target_position.row &&
                   beat_element.position.col == target_position.col {
                    // Found the beat containing this position
                    // Calculate the character span from first to last element
                    if beat.elements.is_empty() {
                        return 1;
                    }

                    let first_element = &beat.elements[0];
                    let last_element = &beat.elements[beat.elements.len() - 1];

                    // Calculate span: from start of first element to end of last element
                    let start_col = first_element.position.col;
                    let end_col = last_element.position.col + last_element.value.len();

                    return end_col - start_col;
                }
            }
        }
    }

    1 // fallback if position not found in any beat
}

// Helper function to process parsed elements from content lines
fn process_parsed_element(
    element: &ParsedElement,
    spans: &mut Vec<Span>,
    position: &mut usize,
    original_input: &str,
    rhythm_items: &Option<Vec<crate::rhythm::Item>>
) {
    match element {
        ParsedElement::Note { value, position: pos, in_beat_group, beat_group, .. } => {
            // Calculate absolute position from row/col
            if let Some(start_pos) = position_to_absolute_offset(pos, original_input) {
                let value_len = value.len();
                let r#type = if *in_beat_group && matches!(beat_group, Some(crate::rhythm::types::BeatGroupRole::Start)) {
                    // Find the beat containing this note position and count its elements
                    let span = if let Some(rhythm_items) = rhythm_items {
                        find_beat_element_count(rhythm_items, pos)
                    } else {
                        1 // fallback if no rhythm data
                    };
                    "note".to_string()
                } else {
                    "note".to_string()
                };

                spans.push(Span {
                    r#type,
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
                spans.push(Span {
                    r#type: "rest".to_string(),
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
                // Check if this dash is the first element of a beat (starts a beat group)
                let r#type = if let Some(rhythm_items) = rhythm_items {
                    // Check if this dash position matches the first element of any beat
                    let mut is_first = false;
                    let mut span = 1;

                    // TODO: Process ContentLine beats instead
                    if false { // Disabled
                        if let crate::rhythm::Item::Beat(beat) = &crate::rhythm::Item::Beat(crate::rhythm::analyzer::Beat { divisions: 1, elements: vec![], tied_to_previous: false, is_tuplet: false, tuplet_ratio: None }) {
                            if let Some(first) = beat.elements.first() {
                                if first.position.row == pos.row && first.position.col == pos.col {
                                    is_first = true;
                                    span = beat.elements.len();
                                    // break; // Removed break from non-loop
                                }
                            }
                        }
                    }

                    "dash".to_string()
                } else {
                    "dash".to_string()
                };

                spans.push(Span {
                    r#type,
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
                spans.push(Span {
                    r#type: "barline".to_string(),
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
                spans.push(Span {
                    r#type: "whitespace".to_string(),
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
                spans.push(Span {
                    r#type: "symbol".to_string(),
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
                spans.push(Span {
                    r#type: "unknown".to_string(),
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
                spans.push(Span {
                    r#type: "newline".to_string(),
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