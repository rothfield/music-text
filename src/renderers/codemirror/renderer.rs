use serde::Serialize;
use std::collections::HashMap;
use crate::parse::model::{Document, DocumentElement, StaveLine, ContentLine, ContentElement, BeatElement};

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
                    // First, create spans for content line consumed elements
                    spans.extend(render_content_line_consumed_elements(content_line, original_input));

                    // Then, create spans for the actual content elements
                    for content_element in &content_line.elements {
                        spans.extend(render_content_element(content_element, content_line, original_input));
                    }
                }
            }
        }
    }

    spans
}

/// Create spans for elements consumed by the content line itself
fn render_content_line_consumed_elements(content_line: &ContentLine, original_input: &str) -> Vec<Span> {
    let mut spans = Vec::new();

    for consumed_element in &content_line.consumed_elements {
        match consumed_element {
            crate::parse::model::ConsumedElement::SlurIndicator { value, char_index } => {
                if let Some(ref slur_value) = value {
                    let mut span = Span::simple(
                        "consumed-slur",
                        *char_index,
                        *char_index + slur_value.len(),
                        slur_value.clone()
                    );
                    // Add data attributes for consumed slur styling
                    span.data_attributes.insert("data-consumed".to_string(), "true".to_string());
                    span.data_attributes.insert("data-consumed-type".to_string(), "slur".to_string());
                    span.data_attributes.insert("data-original-value".to_string(), slur_value.clone());
                    spans.push(span);
                }
            },
            crate::parse::model::ConsumedElement::UpperOctaveMarker { value, char_index } => {
                if let Some(ref marker_value) = value {
                    let mut span = Span::simple(
                        "consumed-upper-octave",
                        *char_index,
                        *char_index + marker_value.len(),
                        marker_value.clone()
                    );
                    // Add data attributes for consumed octave marker styling
                    span.data_attributes.insert("data-consumed".to_string(), "true".to_string());
                    span.data_attributes.insert("data-consumed-type".to_string(), "upper-octave".to_string());
                    span.data_attributes.insert("data-original-value".to_string(), marker_value.clone());
                    spans.push(span);
                }
            },
            crate::parse::model::ConsumedElement::LowerOctaveMarker { value, char_index } => {
                if let Some(ref marker_value) = value {
                    let mut span = Span::simple(
                        "consumed-lower-octave",
                        *char_index,
                        *char_index + marker_value.len(),
                        marker_value.clone()
                    );
                    // Add data attributes for consumed octave marker styling
                    span.data_attributes.insert("data-consumed".to_string(), "true".to_string());
                    span.data_attributes.insert("data-consumed-type".to_string(), "lower-octave".to_string());
                    span.data_attributes.insert("data-original-value".to_string(), marker_value.clone());
                    spans.push(span);
                }
            },
        }
    }

    spans
}

fn render_content_element(content_element: &ContentElement, content_line: &ContentLine, original_input: &str) -> Vec<Span> {
    match content_element {
        ContentElement::Beat(beat) => {
            let mut spans: Vec<Span> = beat.elements.iter()
                .enumerate()
                .flat_map(|(index, beat_element)| render_beat_element(beat_element, beat, index == 0, original_input))
                .collect();

            // Add content line consumed elements to all spans from this beat
            for span in &mut spans {
                add_content_line_consumed_elements(span, content_line);
            }

            spans
        },
        ContentElement::Barline(barline) => {
            let (value, char_index) = match barline {
                crate::parse::model::Barline::Single(b) => (&b.value, b.char_index),
                crate::parse::model::Barline::Double(b) => (&b.value, b.char_index),
                crate::parse::model::Barline::Final(b) => (&b.value, b.char_index),
                crate::parse::model::Barline::RepeatStart(b) => (&b.value, b.char_index),
                crate::parse::model::Barline::RepeatEnd(b) => (&b.value, b.char_index),
                crate::parse::model::Barline::RepeatBoth(b) => (&b.value, b.char_index),
            };
            // Create temporary Attributes for compatibility
            let temp_attrs = crate::parse::model::Attributes {
                slur_position: crate::parse::model::SlurPosition::None,
                value: value.clone(),
                position: crate::parse::model::Position {
                    line: 0,
                    column: 0,
                    index_in_line: 0,
                    index_in_doc: char_index,
                },
            };
            let mut span = render_simple_element("barline", &temp_attrs, "|", original_input);
            add_content_line_consumed_elements(&mut span, content_line);
            vec![span]
        },
        ContentElement::Whitespace(_) => vec![],
    }
}

fn render_beat_element(beat_element: &BeatElement, beat: &crate::parse::model::Beat, is_first_in_beat: bool, original_input: &str) -> Vec<Span> {
    match beat_element {
        BeatElement::Note(note) => {
            // Create temporary Attributes for compatibility
            let temp_attrs = crate::parse::model::Attributes {
                slur_position: crate::parse::model::SlurPosition::None,
                value: note.value.clone(),
                position: crate::parse::model::Position {
                    line: 0,
                    column: 0,
                    index_in_line: 0,
                    index_in_doc: note.char_index,
                },
            };
            let mut span = render_simple_element("note", &temp_attrs,
                &note.value.as_ref().unwrap_or(&"?".to_string()), original_input);
            let mut additional_spans = add_note_specific_attributes(&mut span, note);
            add_beat_context_attributes(&mut span, beat, is_first_in_beat);

            let mut result = vec![span];
            result.append(&mut additional_spans);
            result
        },
        BeatElement::Dash(dash) => {
            // Create temporary Attributes for compatibility
            let temp_attrs = crate::parse::model::Attributes {
                slur_position: crate::parse::model::SlurPosition::None,
                value: dash.value.clone(),
                position: crate::parse::model::Position {
                    line: 0,
                    column: 0,
                    index_in_line: 0,
                    index_in_doc: dash.char_index,
                },
            };
            let mut span = render_simple_element("dash", &temp_attrs, "-", original_input);
            add_dash_specific_attributes(&mut span, dash);
            add_beat_context_attributes(&mut span, beat, is_first_in_beat);
            vec![span]
        },
        BeatElement::BreathMark(breath) => {
            // Create temporary Attributes for compatibility
            let temp_attrs = crate::parse::model::Attributes {
                slur_position: crate::parse::model::SlurPosition::None,
                value: breath.value.clone(),
                position: crate::parse::model::Position {
                    line: 0,
                    column: 0,
                    index_in_line: 0,
                    index_in_doc: breath.char_index,
                },
            };
            let mut span = render_simple_element("breath", &temp_attrs, "'", original_input);
            add_breath_specific_attributes(&mut span, breath);
            vec![span]
        },
    }
}

fn render_simple_element(span_type: &str, source: &crate::parse::model::Attributes, content: &str, _original_input: &str) -> Span {
    let start_pos = source.position.index_in_doc;
    let content_len = source.value.as_ref().map(|v| v.len()).unwrap_or(content.len());
    let span = Span::simple(span_type, start_pos, start_pos + content_len, content.to_string());
    span
}


fn add_note_specific_attributes(span: &mut Span, note: &crate::parse::model::Note) -> Vec<Span> {
    let mut additional_spans = Vec::new();

    // Note-specific semantic data
    span.data_attributes.insert("data-type".to_string(), "note".to_string());
    span.data_attributes.insert("data-pitch-code".to_string(), format!("{:?}", note.pitch_code));

    // Octave information
    span.data_attributes.insert("data-octave".to_string(), note.octave.to_string());


    // Consumed elements information - process octave markers using new pattern
    for consumed_element in &note.consumed_elements {
        match consumed_element {
            crate::parse::model::ConsumedElement::UpperOctaveMarker { value, char_index } => {
                if let Some(ref marker) = value {
                    span.data_attributes.insert("data-consumed-octave-marker".to_string(), marker.clone());
                }
                span.data_attributes.insert("data-consumed-octave-direction".to_string(), "upper".to_string());
                span.data_attributes.insert("data-consumed-source-position".to_string(), char_index.to_string());
            },
            crate::parse::model::ConsumedElement::LowerOctaveMarker { value, char_index } => {
                if let Some(ref marker) = value {
                    span.data_attributes.insert("data-consumed-octave-marker".to_string(), marker.clone());
                }
                span.data_attributes.insert("data-consumed-octave-direction".to_string(), "lower".to_string());
                span.data_attributes.insert("data-consumed-source-position".to_string(), char_index.to_string());
            },
            crate::parse::model::ConsumedElement::SlurIndicator { value, char_index } => {
                if let Some(ref slur) = value {
                    span.data_attributes.insert("data-consumed-slur-indicator".to_string(), slur.clone());
                    // Add slur character loop length for the first element (to draw arc)
                    let slur_length = slur.chars().filter(|c| *c == '_').count();
                    if slur_length > 0 {
                        span.data_attributes.insert("data-slur-char-loop-length".to_string(), slur_length.to_string());
                    }
                }
                span.data_attributes.insert("data-consumed-slur-position".to_string(), char_index.to_string());
                span.data_attributes.insert("data-consumed-element-type".to_string(), "slur".to_string());
            },
        }
    }


    // Original pitch information for HTML tooltips
    if let Some(ref pitch_string) = note.value {
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

    additional_spans
}

fn add_dash_specific_attributes(span: &mut Span, dash: &crate::parse::model::Dash) -> Vec<Span> {
    let mut additional_spans = Vec::new();

    // Dash-specific semantic data
    span.data_attributes.insert("data-type".to_string(), "dash".to_string());

    // Slur position information (handled via consumed elements)

    // Consumed elements information
    for consumed_element in &dash.consumed_elements {
        match consumed_element {
            crate::parse::model::ConsumedElement::SlurIndicator { value, char_index } => {
                if let Some(ref slur) = value {
                    span.data_attributes.insert("data-consumed-slur-indicator".to_string(), slur.clone());
                    // Add slur character loop length for the first element (to draw arc)
                    let slur_length = slur.chars().filter(|c| *c == '_').count();
                    if slur_length > 0 {
                        span.data_attributes.insert("data-slur-char-loop-length".to_string(), slur_length.to_string());
                    }
                }
                span.data_attributes.insert("data-consumed-slur-position".to_string(), char_index.to_string());
                span.data_attributes.insert("data-consumed-element-type".to_string(), "slur".to_string());
            },
            _ => {
                // Other consumed element types can be added here
            }
        }
    }

    additional_spans
}

fn add_breath_specific_attributes(span: &mut Span, breath: &crate::parse::model::BreathMark) -> Vec<Span> {
    let mut additional_spans = Vec::new();

    // Breath mark-specific semantic data
    span.data_attributes.insert("data-type".to_string(), "breath".to_string());

    // Slur position information (handled via consumed elements)

    // Consumed elements information
    for consumed_element in &breath.consumed_elements {
        match consumed_element {
            crate::parse::model::ConsumedElement::SlurIndicator { value, char_index } => {
                if let Some(ref slur) = value {
                    span.data_attributes.insert("data-consumed-slur-indicator".to_string(), slur.clone());
                    // Add slur character loop length for the first element (to draw arc)
                    let slur_length = slur.chars().filter(|c| *c == '_').count();
                    if slur_length > 0 {
                        span.data_attributes.insert("data-slur-char-loop-length".to_string(), slur_length.to_string());
                    }
                }
                span.data_attributes.insert("data-consumed-slur-position".to_string(), char_index.to_string());
                span.data_attributes.insert("data-consumed-element-type".to_string(), "slur".to_string());
            },
            _ => {
                // Other consumed element types can be added here
            }
        }
    }

    additional_spans
}

fn add_content_line_consumed_elements(span: &mut Span, content_line: &ContentLine) {
    // Add information about elements consumed by the content line itself
    for consumed_element in &content_line.consumed_elements {
        match consumed_element {
            crate::parse::model::ConsumedElement::SlurIndicator { value, char_index } => {
                if let Some(ref slur) = value {
                    span.data_attributes.insert("data-line-consumed-slur-indicator".to_string(), slur.clone());
                    // Add slur character loop length (number of underscores = number of notes in slur)
                    let slur_length = slur.chars().filter(|c| *c == '_').count();
                    if slur_length > 0 {
                        span.data_attributes.insert("data-slur-char-loop-length".to_string(), slur_length.to_string());
                    }
                }
                span.data_attributes.insert("data-line-consumed-slur-position".to_string(), char_index.to_string());
                span.data_attributes.insert("data-line-consumed-element-type".to_string(), "slur".to_string());
            },
            crate::parse::model::ConsumedElement::UpperOctaveMarker { value, char_index } => {
                if let Some(ref marker) = value {
                    span.data_attributes.insert("data-line-consumed-octave-marker".to_string(), marker.clone());
                }
                span.data_attributes.insert("data-line-consumed-octave-position".to_string(), char_index.to_string());
                span.data_attributes.insert("data-line-consumed-element-type".to_string(), "upper-octave".to_string());
            },
            crate::parse::model::ConsumedElement::LowerOctaveMarker { value, char_index } => {
                if let Some(ref marker) = value {
                    span.data_attributes.insert("data-line-consumed-octave-marker".to_string(), marker.clone());
                }
                span.data_attributes.insert("data-line-consumed-octave-position".to_string(), char_index.to_string());
                span.data_attributes.insert("data-line-consumed-element-type".to_string(), "lower-octave".to_string());
            },
        }
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
                        n.value.as_ref().map(|v| v.chars().count()).unwrap_or(1)
                    },
                    crate::parse::model::BeatElement::Dash(d) => {
                        d.value.as_ref().map(|v| v.chars().count()).unwrap_or(1)
                    },
                    crate::parse::model::BeatElement::BreathMark(b) => {
                        b.value.as_ref().map(|v| v.chars().count()).unwrap_or(1)
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
                "data-slur-position" => {
                    styles.insert("--slur-position".to_string(), format!("\"{}\"", value));
                    classes.push(format!("slur-{}", value.to_lowercase()));
                }
                "data-consumed-slur-indicator" => {
                    styles.insert("--consumed-slur-indicator".to_string(), format!("\"{}\"", value));
                    classes.push("consumed-slur".to_string());
                }
                "data-consumed-slur-position" => {
                    styles.insert("--consumed-slur-position".to_string(), value.clone());
                }
                "data-line-consumed-slur-indicator" => {
                    styles.insert("--line-consumed-slur-indicator".to_string(), format!("\"{}\"", value));
                    classes.push("line-consumed-slur".to_string());
                }
                "data-line-consumed-slur-position" => {
                    styles.insert("--line-consumed-slur-position".to_string(), value.clone());
                }
                "data-slur-char-loop-length" => {
                    styles.insert("--slur-char-loop-length".to_string(), value.clone());
                }
                "data-line-consumed-octave-marker" => {
                    styles.insert("--line-consumed-octave-marker".to_string(), format!("\"{}\"", value));
                    classes.push("line-consumed-octave".to_string());
                }
                "data-line-consumed-octave-position" => {
                    styles.insert("--line-consumed-octave-position".to_string(), value.clone());
                }
                "data-line-consumed-element-type" => {
                    styles.insert("--line-consumed-element-type".to_string(), format!("\"{}\"", value));
                    classes.push(format!("line-consumed-{}", value));
                }
                "data-consumed" => {
                    if value == "true" {
                        classes.push("consumed".to_string());
                        classes.push("greyed-out".to_string());
                    }
                }
                "data-consumed-type" => {
                    styles.insert("--consumed-type".to_string(), format!("\"{}\"", value));
                    classes.push(format!("consumed-{}", value));
                }
                "data-original-value" => {
                    styles.insert("--original-value".to_string(), format!("\"{}\"", value));
                }
                _ => {
                    // Convert other data attributes to CSS custom properties
                    let css_prop = key.replace("data-", "--");
                    styles.insert(css_prop, value.clone());
                }
            }
        }

        // Deduplicate classes
        classes.sort();
        classes.dedup();

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