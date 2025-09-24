use serde::Serialize;
use std::collections::HashMap;
use crate::parse::model::*;

#[derive(Debug, Serialize, Clone)]
pub struct Span {
    pub r#type: String,
    pub start: usize,
    pub end: usize,
    pub content: String,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub data_attributes: HashMap<String, String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SpanStyle {
    pub pos: usize,
    pub length: usize,
    pub classes: Vec<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub styles: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Generic function to collect any element with position info
fn collect_element<T: HasPosition>(element: &T, is_consumed: bool) -> Option<Span> {
    element.value().map(|v| {
        let type_name = to_kebab_case(element.type_name());
        let mut span = Span {
            r#type: type_name,
            start: element.char_index(),
            end: element.char_index() + v.len(),
            content: v.clone(),
            data_attributes: HashMap::new(),
        };

        if is_consumed {
            span.data_attributes.insert("consumed".to_string(), "true".to_string());
        }

        span
    })
}

/// Main entry point: render document to CodeMirror spans
pub fn render_codemirror_spans(document: &Document, _original_input: &str) -> Vec<Span> {
    let mut spans = Vec::new();

    // Walk all document elements
    for element in &document.elements {
        collect_document_element(element, &mut spans);
    }

    spans
}

fn collect_document_element(element: &DocumentElement, spans: &mut Vec<Span>) {
    match element {
        DocumentElement::Stave(stave) => {
            for line in &stave.lines {
                collect_stave_line(line, spans);
            }
        },
        DocumentElement::BlankLines(_) => {
            // No position info to collect
        },
    }
}

fn collect_stave_line(line: &StaveLine, spans: &mut Vec<Span>) {
    match line {
        StaveLine::Upper(upper) => {
            for element in &upper.elements {
                if let Some(span) = collect_element(element, false) {
                    spans.push(span);
                }
            }
        },
        StaveLine::Lower(lower) => {
            for element in &lower.elements {
                if let Some(span) = collect_element(element, false) {
                    spans.push(span);
                }
            }
        },
        StaveLine::ContentLine(content) => {
            // Collect consumed elements
            for consumed in &content.consumed_elements {
                if let Some(span) = collect_element(consumed, true) {
                    spans.push(span);
                }
            }
            // Collect content elements
            for element in &content.elements {
                collect_content_element(element, spans);
            }
        },
        _ => {}, // Other line types - add if needed
    }
}

fn collect_content_element(element: &ContentElement, spans: &mut Vec<Span>) {
    match element {
        ContentElement::Beat(beat) => {
            // Collect beat's consumed elements
            for consumed in beat.consumed_elements() {
                if let Some(span) = collect_element(consumed, true) {
                    spans.push(span);
                }
            }

            // Collect beat elements with beat loop calculation
            for (idx, beat_el) in beat.elements.iter().enumerate() {
                // First collect consumed elements for this beat element using trait
                for consumed in beat_el.consumed_elements() {
                    if let Some(span) = collect_element(consumed, true) {
                        spans.push(span);
                    }
                }

                // Then collect the beat element itself
                if let Some(mut span) = collect_element(beat_el, false) {
                    // Add beat loop data to first element if beat has multiple divisions
                    if idx == 0 && beat.divisions.unwrap_or(1) > 1 && beat.elements.len() > 1 {
                        if let (Some(first), Some(last)) = (beat.elements.first(), beat.elements.last()) {
                            let beat_char_length = last.char_index() - first.char_index() +
                                last.value().map_or(1, |v| v.len());
                            span.data_attributes.insert("beat-loop-length".to_string(), beat_char_length.to_string());
                        }
                    }

                    // Add octave data for notes
                    if let crate::models::elements::BeatElement::Note(note) = beat_el {
                        if note.octave > 0 {
                            span.data_attributes.insert("octave".to_string(), note.octave.to_string());
                        } else if note.octave < 0 {
                            span.data_attributes.insert("octave-negative".to_string(), (-note.octave).to_string());
                        }
                    }

                    spans.push(span);
                }
            }
        },
        ContentElement::Barline(barline) => {
            // Collect barline's consumed elements
            for consumed in barline.consumed_elements() {
                if let Some(span) = collect_element(consumed, true) {
                    spans.push(span);
                }
            }

            // Collect the barline itself using the generic function
            if let Some(span) = collect_element(barline, false) {
                spans.push(span);
            }
        },
        ContentElement::Whitespace(ws) => {
            // Collect whitespace's consumed elements
            for consumed in ws.consumed_elements() {
                if let Some(span) = collect_element(consumed, true) {
                    spans.push(span);
                }
            }

            // Collect the whitespace itself
            if let Some(span) = collect_element(ws, false) {
                spans.push(span);
            }
        },
    }
}


// Implement HasPosition for structs that need it but aren't covered by enum implementations
impl HasPosition for Whitespace {
    fn char_index(&self) -> usize { self.char_index }
    fn value(&self) -> Option<&String> { self.value.as_ref() }
    fn consumed_elements(&self) -> &[ConsumedElement] { &self.consumed_elements }
    fn type_name(&self) -> &'static str { "Whitespace" }
}

/// Convert PascalCase to kebab-case for CSS-friendly names
fn to_kebab_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && c.is_uppercase() {
            result.push('-');
        }
        result.push(c.to_lowercase().next().unwrap());
    }
    result
}

/// Convert byte position to character position
fn byte_pos_to_char_pos(text: &str, byte_pos: usize) -> usize {
    text.char_indices().take_while(|(i, _)| *i < byte_pos).count()
}

/// Convert spans to span styles for CodeMirror
pub fn render_codemirror_styles(spans: &[Span], original_input: &str) -> Vec<SpanStyle> {
    spans.iter().map(|span| {
        let mut classes = vec![format!("cm-{}", span.r#type)];
        let mut styles = HashMap::new();

        // Add consumed class
        if span.data_attributes.get("consumed") == Some(&"true".to_string()) {
            classes.push("consumed".to_string());
        }

        // Add beat loop styling
        if let Some(beat_loop_len) = span.data_attributes.get("beat-loop-length") {
            styles.insert("--beat-loop-length".to_string(), beat_loop_len.clone());
            classes.push("beat-start".to_string());
        }

        // Add octave styling
        if let Some(octave) = span.data_attributes.get("octave") {
            styles.insert("--octave".to_string(), octave.clone());
            classes.push(format!("octave-{}", octave));
        }
        if let Some(octave_negative) = span.data_attributes.get("octave-negative") {
            styles.insert("--octave-negative".to_string(), octave_negative.clone());
            classes.push(format!("octave-neg-{}", octave_negative));
        }

        SpanStyle {
            pos: byte_pos_to_char_pos(original_input, span.start),
            length: span.content.chars().count(),
            classes,
            styles,
            title: None,
        }
    }).collect()
}

/// Main entry point
pub fn render_codemirror(document: &Document, original_input: &str) -> (Vec<Span>, Vec<SpanStyle>) {
    let spans = render_codemirror_spans(document, original_input);
    let styles = render_codemirror_styles(&spans, original_input);
    (spans, styles)
}