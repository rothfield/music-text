use crate::models::{Document, DocumentElement, Stave, StaveLine, ContentElement, BeatElement};

pub fn render_simple_svg(document: &Document) -> String {
    eprintln!("DEBUG: render_simple_svg called with document: {:#?}", document);

    let mut y_pos = 80.0;
    let mut svg_elements = Vec::new();

    // Traverse document structure and create semantic SVG elements
    for doc_element in &document.elements {
        match doc_element {
            DocumentElement::Stave(stave) => {
                svg_elements.push(render_stave(stave, &mut y_pos));
            },
            DocumentElement::BlankLines(_) => {
                y_pos += 20.0; // Add vertical space for blank lines
            }
        }
    }

    let actual_height = y_pos + 40.0;
    format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="1600" height="{}" viewBox="0 0 1600 {}"
     style="background-color: #f9f9f9;">
  <style>
    .note {{ font-family: Liberation Sans, Arial, sans-serif; font-size: 24px; fill: #333; }}
    .barline {{ font-family: Liberation Sans, Arial, sans-serif; font-size: 24px; fill: #666; }}
    .beat {{ }}
    .rest {{ font-family: Liberation Sans, Arial, sans-serif; font-size: 24px; fill: #999; }}
    .dash {{ font-family: Liberation Sans, Arial, sans-serif; font-size: 24px; fill: #999; }}
    .whitespace {{ }}
    .stave {{ }}
    .content-line {{ }}
  </style>
  <g class="document">
    {}
  </g>
</svg>"#, actual_height, actual_height, svg_elements.join("\n    "))
}

fn render_stave(stave: &Stave, y_pos: &mut f64) -> String {
    let mut stave_elements = Vec::new();
    let stave_y = *y_pos;

    for line in &stave.lines {
        match line {
            StaveLine::ContentLine(content_line) => {
                stave_elements.push(render_content_line(content_line, *y_pos));
                *y_pos += 30.0;
            },
            StaveLine::Upper(upper_line) => {
                stave_elements.push(format!(r#"<g class="upper-line" transform="translate(0, {})">{}</g>"#,
                    *y_pos - 20.0, render_upper_line(upper_line)));
            },
            StaveLine::Lower(lower_line) => {
                stave_elements.push(format!(r#"<g class="lower-line" transform="translate(0, {})">{}</g>"#,
                    *y_pos + 20.0, render_lower_line(lower_line)));
            },
            StaveLine::Whitespace(_) => {
                *y_pos += 10.0; // Small vertical space
            },
            _ => {} // Handle other line types as needed
        }
    }

    format!(r#"<g class="stave" data-notation-system="{:?}" transform="translate(0, {})">
      {}
    </g>"#, stave.notation_system, stave_y, stave_elements.join("\n      "))
}

fn render_content_line(content_line: &crate::models::ContentLine, y_pos: f64) -> String {
    let mut x_pos = 50.0;
    let mut elements = Vec::new();

    for element in &content_line.elements {
        match element {
            ContentElement::Beat(beat) => {
                let beat_svg = render_beat(beat, x_pos, y_pos);
                elements.push(beat_svg);
                x_pos += 30.0 * beat.elements.len() as f64; // Rough spacing
            },
            ContentElement::Barline(barline) => {
                elements.push(format!(r#"<text x="{}" y="{}" class="barline">|</text>"#, x_pos, y_pos));
                x_pos += 20.0;
            },
            ContentElement::Whitespace(_) => {
                x_pos += 10.0; // Add horizontal space
            }
        }
    }

    format!(r#"<g class="content-line">{}</g>"#, elements.join(""))
}

fn render_beat(beat: &crate::models::Beat, x_pos: f64, y_pos: f64) -> String {
    let mut beat_x = x_pos;
    let mut beat_elements = Vec::new();

    for element in &beat.elements {
        match element {
            BeatElement::Note(note) => {
                let note_text = note.value.as_ref().map(|s| s.as_str()).unwrap_or("?");
                beat_elements.push(format!(r#"<text x="{}" y="{}" class="note" data-pitch="{:?}" data-octave="{}">{}</text>"#,
                    beat_x, y_pos, note.pitch_code, note.octave, note_text));
                beat_x += 25.0;
            },
            BeatElement::Rest(rest) => {
                let rest_text = rest.value.as_ref().map(|s| s.as_str()).unwrap_or("r");
                beat_elements.push(format!(r#"<text x="{}" y="{}" class="rest">{}</text>"#, beat_x, y_pos, rest_text));
                beat_x += 25.0;
            },
            BeatElement::Dash(dash) => {
                let dash_text = dash.value.as_ref().map(|s| s.as_str()).unwrap_or("-");
                beat_elements.push(format!(r#"<text x="{}" y="{}" class="dash">{}</text>"#, beat_x, y_pos, dash_text));
                beat_x += 25.0;
            },
            BeatElement::BreathMark(breath) => {
                let breath_text = breath.value.as_ref().map(|s| s.as_str()).unwrap_or(",");
                beat_elements.push(format!(r#"<text x="{}" y="{}" class="breath-mark">{}</text>"#, beat_x, y_pos, breath_text));
                beat_x += 15.0;
            }
        }
    }

    let css_classes = if beat.css_classes.is_empty() {
        "beat".to_string()
    } else {
        format!("beat {}", beat.css_classes.join(" "))
    };

    format!(r#"<g class="{}" data-divisions="{:?}" data-tuplet="{:?}">{}</g>"#,
        css_classes, beat.divisions, beat.is_tuplet, beat_elements.join(""))
}

fn render_upper_line(upper_line: &crate::models::UpperLine) -> String {
    // Simplified rendering for upper line elements
    format!(r#"<text x="50" y="0" class="upper-line" opacity="0.7">{}</text>"#,
        upper_line.value.as_ref().unwrap_or(&"".to_string()))
}

fn render_lower_line(lower_line: &crate::models::LowerLine) -> String {
    // Simplified rendering for lower line elements
    format!(r#"<text x="50" y="0" class="lower-line" opacity="0.7">{}</text>"#,
        lower_line.value.as_ref().unwrap_or(&"".to_string()))
}
