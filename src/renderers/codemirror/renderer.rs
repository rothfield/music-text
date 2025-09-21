use serde::Serialize;
use std::collections::HashMap;
use crate::parse::model::{Document, DocumentElement, StaveLine, ContentElement, BeatElement};

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

impl Span {
    fn simple(span_type: &str, start: usize, end: usize, content: String) -> Self {
        Span {
            r#type: span_type.to_string(),
            start,
            end,
            content,
            data_attributes: HashMap::new(),
        }
    }
}

/// Render document to CodeMirror spans with rich semantic data
pub fn render_codemirror_spans(document: &Document, original_input: &str) -> Vec<Span> {
    let mut spans = Vec::new();

    for element in &document.elements {
        if let DocumentElement::Stave(stave) = element {
            for line in &stave.lines {
                if let StaveLine::ContentLine(content_line) = line {
                    for content_element in &content_line.elements {
                        spans.extend(render_content_element(content_element, original_input));
                    }
                }
            }
        }
    }

    spans
}

fn render_content_element(content_element: &ContentElement, original_input: &str) -> Vec<Span> {
    match content_element {
        ContentElement::Beat(beat) => beat.elements.iter()
            .enumerate()
            .map(|(index, beat_element)| render_beat_element(beat_element, beat, index == 0, original_input))
            .collect(),
        ContentElement::Barline(barline) => vec![
            render_simple_element("barline", &barline.source, "|", original_input)
        ],
        ContentElement::Whitespace(_) => vec![],
    }
}

fn render_beat_element(beat_element: &BeatElement, beat: &crate::parse::model::Beat, is_first_in_beat: bool, original_input: &str) -> Span {
    match beat_element {
        BeatElement::Note(note) => {
            let mut span = render_simple_element("note", &note.source,
                &note.source.value.as_ref().unwrap_or(&"?".to_string()), original_input);
            add_note_specific_attributes(&mut span, note);
            add_beat_context_attributes(&mut span, beat, is_first_in_beat);
            span
        },
        BeatElement::Dash(dash) => {
            let mut span = render_simple_element("dash", &dash.source, "-", original_input);
            add_beat_context_attributes(&mut span, beat, is_first_in_beat);
            span
        },
        BeatElement::BreathMark(breath) => render_simple_element("breath", &breath.source, "'", original_input),
    }
}

fn render_simple_element(span_type: &str, source: &crate::parse::model::Source, content: &str, _original_input: &str) -> Span {
    let start_pos = source.position.index_in_doc;
    let content_len = source.value.as_ref().map(|v| v.len()).unwrap_or(content.len());
    Span::simple(span_type, start_pos, start_pos + content_len, content.to_string())
}


fn add_note_specific_attributes(span: &mut Span, note: &crate::parse::model::Note) {
    // Note-specific semantic data
    span.data_attributes.insert("data-type".to_string(), "note".to_string());
    span.data_attributes.insert("data-pitch-code".to_string(), format!("{:?}", note.pitch_code));

    // Octave information
    span.data_attributes.insert("data-octave".to_string(), note.octave.to_string());

    // Consumed elements information - process octave markers using new pattern
    for consumed_element in &note.consumed_elements {
        match consumed_element {
            crate::parse::model::ConsumedElement::UpperOctaveMarker { source } => {
                if let Some(ref marker) = source.value {
                    span.data_attributes.insert("data-consumed-octave-marker".to_string(), marker.clone());
                }
                span.data_attributes.insert("data-consumed-octave-direction".to_string(), "upper".to_string());
                span.data_attributes.insert("data-consumed-source-position".to_string(), source.position.index_in_doc.to_string());
            },
            crate::parse::model::ConsumedElement::LowerOctaveMarker { source } => {
                if let Some(ref marker) = source.value {
                    span.data_attributes.insert("data-consumed-octave-marker".to_string(), marker.clone());
                }
                span.data_attributes.insert("data-consumed-octave-direction".to_string(), "lower".to_string());
                span.data_attributes.insert("data-consumed-source-position".to_string(), source.position.index_in_doc.to_string());
            },
        }
    }

    // Original pitch information for HTML tooltips
    if let Some(ref pitch_string) = note.source.value {
        span.data_attributes.insert("data-original-pitch".to_string(), pitch_string.clone());

        // Create HTML data-title attribute for browser tooltips
        let notation_system = format!("{:?}", note.notation_system);
        let tooltip = format!("{} ({})", pitch_string, notation_system);
        span.data_attributes.insert("data-title".to_string(), tooltip);
    }
    span.data_attributes.insert("data-notation-system".to_string(), format!("{:?}", note.notation_system));

    // Duration information from rhythm analysis
    if let (Some(numer), Some(denom)) = (note.numerator, note.denominator) {
        span.data_attributes.insert("data-duration".to_string(), format!("{}/{}", numer, denom));
    }
}

fn add_beat_context_attributes(span: &mut Span, beat: &crate::parse::model::Beat, is_first_in_beat: bool) {
    // Beat context - shared by all elements in the beat
    if let Some(divisions) = beat.divisions {
        span.data_attributes.insert("data-beat-divisions".to_string(), divisions.to_string());
    }
    if let Some(total_duration) = &beat.total_duration {
        let num = *total_duration.numer().unwrap_or(&1u64);
        let den = *total_duration.denom().unwrap_or(&4u64);
        span.data_attributes.insert("data-beat-duration".to_string(), format!("{}/{}", num, den));
    }

    // Beat loop information for first element in beat
    if is_first_in_beat {
        span.data_attributes.insert("data-beat-start".to_string(), "true".to_string());

        // Count pitch and dash elements (exclude breath marks)
        let pitch_dash_count = beat.elements.iter().filter(|element| {
            matches!(element,
                crate::parse::model::BeatElement::Note(_) |
                crate::parse::model::BeatElement::Dash(_)
            )
        }).count();

        // Only add loop data if there are 2 or more pitch/dash elements
        if pitch_dash_count >= 2 {
            // Calculate total character length of all elements in the beat
            let beat_char_length: usize = beat.elements.iter().map(|element| {
                match element {
                    crate::parse::model::BeatElement::Note(n) => {
                        n.source.value.as_ref().map(|v| v.chars().count()).unwrap_or(1)
                    },
                    crate::parse::model::BeatElement::Dash(d) => {
                        d.source.value.as_ref().map(|v| v.chars().count()).unwrap_or(1)
                    },
                    crate::parse::model::BeatElement::BreathMark(b) => {
                        b.source.value.as_ref().map(|v| v.chars().count()).unwrap_or(1)
                    },
                }
            }).sum();

            span.data_attributes.insert("data-beat-char-loop-length".to_string(), beat_char_length.to_string());
        }
    }
}

/// Convert byte position to character position in a string
fn byte_pos_to_char_pos(text: &str, byte_pos: usize) -> usize {
    text.char_indices().take_while(|(i, _)| *i < byte_pos).count()
}

/// Convert spans to span styles for CodeMirror
pub fn render_codemirror_styles(spans: &[Span], original_input: &str) -> Vec<SpanStyle> {
    spans.iter().map(|span| {
        let mut styles = HashMap::new();
        let mut classes = vec![format!("cm-music-{}", span.r#type)];

        // Extract title for HTML tooltip (don't convert to CSS)
        let title = span.data_attributes.get("data-title").cloned();

        // Convert data attributes to CSS custom properties and classes
        for (key, value) in &span.data_attributes {
            match key.as_str() {
                "data-title" => {
                    // Convert to both --title CSS variable AND separate title field
                    styles.insert("--title".to_string(), value.clone());
                }
                "data-duration" => {
                    styles.insert("--duration".to_string(), value.clone());
                    classes.push(format!("duration-{}", value.replace("/", "-")));
                }
                "data-beat-divisions" => {
                    styles.insert("--beat-divisions".to_string(), value.clone());
                    classes.push(format!("beat-{}", value));
                }
                "data-pitch-code" => {
                    classes.push(format!("pitch-{}", value.to_lowercase()));
                }
                "data-beat-start" => {
                    if value == "true" {
                        classes.push("beat-start".to_string());
                    }
                }
                "data-beat-char-loop-length" => {
                    styles.insert("--beat-char-loop-length".to_string(), value.clone());
                }
                "data-original-pitch" => {
                    styles.insert("--original-pitch".to_string(), format!("\"{}\"", value));
                }
                "data-notation-system" => {
                    styles.insert("--notation-system".to_string(), format!("\"{}\"", value));
                }
                "data-octave" => {
                    styles.insert("--octave".to_string(), value.clone());
                    // Generate octave-specific CSS classes
                    if let Ok(octave_val) = value.parse::<i8>() {
                        if octave_val != 0 {
                            classes.push(format!("octave-{}", octave_val));
                        }
                    }
                }
                "data-octave-marker" => {
                    styles.insert("--octave-marker".to_string(), format!("\"{}\"", value));
                    classes.push(format!("octave-marker-{}",
                        match value.as_str() {
                            "." => "dot",
                            ":" => "colon",
                            _ => "unknown"
                        }));
                }
                "data-octave-direction" => {
                    styles.insert("--octave-direction".to_string(), format!("\"{}\"", value));
                    classes.push(format!("octave-{}", value));
                }
                "data-consumed-octave-marker" => {
                    styles.insert("--consumed-octave-marker".to_string(), format!("\"{}\"", value));
                    classes.push("consumed-octave-marker".to_string());
                    classes.push(format!("consumed-marker-{}",
                        match value.as_str() {
                            "." => "dot",
                            ":" => "colon",
                            _ => "unknown"
                        }));
                }
                "data-consumed-octave-direction" => {
                    styles.insert("--consumed-octave-direction".to_string(), format!("\"{}\"", value));
                    classes.push(format!("consumed-octave-{}", value));
                }
                "data-consumed-source-position" => {
                    styles.insert("--consumed-source-position".to_string(), value.clone());
                }
                _ => {
                    // Convert other data attributes to CSS custom properties
                    let css_prop = key.replace("data-", "--");
                    styles.insert(css_prop, value.clone());
                }
            }
        }

        SpanStyle {
            pos: byte_pos_to_char_pos(original_input, span.start),
            length: span.content.chars().count(),
            classes,
            styles,
            title,
        }
    }).collect()
}

/// Main entry point: render document to CodeMirror spans and styles
pub fn render_codemirror(document: &Document, original_input: &str) -> (Vec<Span>, Vec<SpanStyle>) {
    let spans = render_codemirror_spans(document, original_input);
    let styles = render_codemirror_styles(&spans, original_input);
    (spans, styles)
}