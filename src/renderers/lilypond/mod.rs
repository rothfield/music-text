// Temporarily disable complex renderer modules
// pub mod renderer;
// pub mod templates;
pub mod generator;

// pub use renderer::*;
// pub use templates::*;
pub use generator::*;

// Function to render from our Document type using rhythm analysis
pub fn render_lilypond_from_document(document: &crate::parse::model::Document) -> String {
    let mut output = String::from("\\version \"2.24.0\"\n\\language \"english\"\n\n");

    // Add title/author if present
    for directive in &document.directives {
        match directive.key.as_str() {
            "title" => output.push_str(&format!("\\header {{\n  title = \"{}\"\n}}\n\n", directive.value)),
            "author" => {
                if !output.contains("\\header") {
                    output.push_str("\\header {\n");
                }
                output = output.replace("}\n\n", &format!("  composer = \"{}\"\n}}\n\n", directive.value));
            },
            _ => {}
        }
    }

    output.push_str("\\score {\n  \\new Staff {\n    \\fixed c' {\n      \\key c \\major\n      \\time 4/4\n      ");

    // Convert each stave using rhythm analysis results
    for element in &document.elements {
        if let crate::parse::model::DocumentElement::Stave(stave) = element {
            if let Some(rhythm_items) = &stave.rhythm_items {
                output.push_str(&process_rhythm_items_to_lilypond(rhythm_items));
            }
        }
    }

    output.push_str("\n    }\n  }\n}");
    output
}

/// Process rhythm items to LilyPond notation
fn process_rhythm_items_to_lilypond(rhythm_items: &[crate::rhythm::analyzer::Item]) -> String {
    let mut output = String::new();

    for item in rhythm_items {
        match item {
            crate::rhythm::analyzer::Item::Beat(beat) => {
                if beat.is_tuplet {
                    // Handle tuplet: \tuplet 3/2 { c'8 c'8 c'8 }
                    let tuplet_ratio = beat.tuplet_ratio.unwrap_or((beat.divisions, 2));
                    output.push_str(&format!("\\tuplet {}/{} {{ ", tuplet_ratio.0, tuplet_ratio.1));

                    for element in &beat.elements {
                        output.push_str(&convert_beat_element_to_lilypond(element));
                        output.push(' ');
                    }

                    output.push_str("} ");
                } else {
                    // Regular beat - just output elements
                    for element in &beat.elements {
                        output.push_str(&convert_beat_element_to_lilypond(element));
                        output.push(' ');
                    }
                }
            },
            crate::rhythm::analyzer::Item::Barline(_, _) => {
                output.push_str("| ");
            },
            crate::rhythm::analyzer::Item::Breathmark => {
                output.push_str("\\breathe ");
            },
            crate::rhythm::analyzer::Item::Tonic(_) => {
                // Tonic doesn't generate visual elements
            }
        }
    }

    output
}

/// Convert a BeatElement to LilyPond notation with duration
fn convert_beat_element_to_lilypond(element: &crate::rhythm::analyzer::BeatElement) -> String {
    match &element.event {
        crate::rhythm::analyzer::Event::Note { degree, .. } => {
            let note_name = degree_to_lilypond(*degree);
            let duration_suffix = fraction_to_lilypond_duration(element.tuplet_duration);
            format!("{}{}", note_name, duration_suffix)
        },
        crate::rhythm::analyzer::Event::Rest => {
            let duration_suffix = fraction_to_lilypond_duration(element.tuplet_duration);
            format!("r{}", duration_suffix)
        },
        crate::rhythm::analyzer::Event::Unknown { .. } => {
            String::new() // Skip unknown elements
        }
    }
}

/// Convert degree to LilyPond note name
fn degree_to_lilypond(degree: crate::models::pitch::Degree) -> &'static str {
    use crate::models::pitch::Degree::*;
    match degree {
        N1 | N1s | N1ss | N1b | N1bb => "c'",
        N2 | N2s | N2ss | N2b | N2bb => "d'",
        N3 | N3s | N3ss | N3b | N3bb => "e'",
        N4 | N4s | N4ss | N4b | N4bb => "f'",
        N5 | N5s | N5ss | N5b | N5bb => "g'",
        N6 | N6s | N6ss | N6b | N6bb => "a'",
        N7 | N7s | N7ss | N7b | N7bb => "b'",
    }
}

/// Convert fraction to LilyPond duration suffix
fn fraction_to_lilypond_duration(duration: fraction::Fraction) -> String {
    let num = *duration.numer().unwrap() as usize;
    let den = *duration.denom().unwrap() as usize;
    match (num, den) {
        (1, 1) => "1".to_string(),      // whole note
        (1, 2) => "2".to_string(),      // half note
        (1, 4) => "4".to_string(),      // quarter note
        (1, 8) => "8".to_string(),      // eighth note
        (1, 16) => "16".to_string(),    // sixteenth note
        (1, 32) => "32".to_string(),    // thirty-second note
        // Single-dotted durations (3/2 of basic duration)
        (3, 2) => "1.".to_string(),     // dotted whole
        (3, 4) => "2.".to_string(),     // dotted half
        (3, 8) => "4.".to_string(),     // dotted quarter
        (3, 16) => "8.".to_string(),    // dotted eighth
        (3, 32) => "16.".to_string(),   // dotted sixteenth
        // Double-dotted durations (7/4 of basic duration)
        (7, 4) => "1..".to_string(),    // double-dotted whole
        (7, 8) => "2..".to_string(),    // double-dotted half
        (7, 16) => "4..".to_string(),   // double-dotted quarter
        (7, 32) => "8..".to_string(),   // double-dotted eighth
        (7, 64) => "16..".to_string(),  // double-dotted sixteenth
        _ => "4".to_string(),           // default to quarter note
    }
}