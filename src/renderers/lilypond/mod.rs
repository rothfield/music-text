// Temporarily disable complex renderer modules
pub mod renderer;
pub mod templates;
pub mod generator;

pub use renderer::*;
pub use templates::*;
pub use generator::*;
use fraction::Fraction;

/// Convert a fraction to LilyPond duration string
fn fraction_to_lilypond_duration(duration: Fraction) -> String {
    // Handle common durations
    if duration == Fraction::new(1u64, 1u64) {
        "1".to_string() // whole note
    } else if duration == Fraction::new(1u64, 2u64) {
        "2".to_string() // half note
    } else if duration == Fraction::new(1u64, 4u64) {
        "4".to_string() // quarter note
    } else if duration == Fraction::new(1u64, 8u64) {
        "8".to_string() // eighth note
    } else if duration == Fraction::new(1u64, 16u64) {
        "16".to_string() // sixteenth note
    } else if duration == Fraction::new(1u64, 32u64) {
        "32".to_string() // thirty-second note
    } else if duration == Fraction::new(1u64, 64u64) {
        "64".to_string() // sixty-fourth note
    } else if duration == Fraction::new(3u64, 8u64) {
        "4.".to_string() // dotted quarter
    } else if duration == Fraction::new(3u64, 16u64) {
        "8.".to_string() // dotted eighth
    } else if duration == Fraction::new(1u64, 12u64) {
        "8".to_string() // eighth note triplet (handled by tuplet markup)
    } else if duration == Fraction::new(1u64, 24u64) {
        "16".to_string() // sixteenth note triplet
    } else if duration == Fraction::new(1u64, 48u64) {
        "32".to_string() // thirty-second note triplet
    } else {
        // For unusual durations, try to find the closest standard duration
        let denom = *duration.denom().unwrap_or(&4u64);
        if denom >= 64 {
            "64".to_string()
        } else if denom >= 32 {
            "32".to_string()
        } else if denom >= 16 {
            "16".to_string()
        } else if denom >= 8 {
            "8".to_string()
        } else if denom >= 4 {
            "4".to_string()
        } else if denom >= 2 {
            "2".to_string()
        } else {
            "1".to_string()
        }
    }
}

// Function to render from our Document type using rhythm analysis
pub fn render_lilypond_from_document(document: &crate::parse::model::Document) -> String {
    let mut output = String::from("\\version \"2.24.0\"\n\\language \"english\"\n\n");

    // Add title/author if present
    let title = document.title.as_ref().or_else(|| document.directives.get("title"));
    let author = document.author.as_ref().or_else(|| document.directives.get("author"));

    if title.is_some() || author.is_some() {
        output.push_str("\\header {\n");
        if let Some(title_str) = title {
            output.push_str(&format!("  title = \"{}\"\n", title_str));
        }
        if let Some(author_str) = author {
            output.push_str(&format!("  composer = \"{}\"\n", author_str));
        }
        output.push_str("}\n\n");
    }

    output.push_str("\\score {\n  \\new Staff {\n    \\fixed c' {\n      \\key c \\major\n      \\time 4/4\n      ");

    // Convert each stave using ContentLine beats directly
    for element in &document.elements {
        if let crate::parse::model::DocumentElement::Stave(stave) = element {
            for line in &stave.lines {
                if let crate::parse::model::StaveLine::ContentLine(content_line) = line {
                    for content_element in &content_line.elements {
                        match content_element {
                            crate::parse::model::ContentElement::Beat(beat) => {
                                // Convert each beat element directly
                                for beat_element in &beat.elements {
                                    match beat_element {
                                        crate::parse::model::BeatElement::Note(note) => {
                                            // Use proper pitch conversion with accidentals
                                            let pitch_name = crate::renderers::converters_lilypond::pitch::pitchcode_to_lilypond(
                                                note.pitch_code,
                                                note.octave,
                                                None // No transposition for now
                                            ).unwrap_or_else(|_| "c'".to_string());

                                            // Get the duration (use rhythm analysis if available, fallback to quarter note)
                                            let duration_str = if let (Some(numer), Some(denom)) = (note.numerator, note.denominator) {
                                                let duration = fraction::Fraction::new(numer, denom);
                                                fraction_to_lilypond_duration(duration)
                                            } else {
                                                "4".to_string() // Fallback to quarter note
                                            };

                                            let note_name = format!("{}{}", pitch_name, duration_str);
                                            output.push_str(&format!("{} ", note_name));
                                        }
                                        crate::parse::model::BeatElement::Dash(_) => {
                                            output.push_str("r4 ");
                                        }
                                        crate::parse::model::BeatElement::BreathMark(_) => {
                                            output.push_str("\\breathe ");
                                        }
                                    }
                                }
                            }
                            crate::parse::model::ContentElement::Barline(_) => {
                                output.push_str("| ");
                            }
                            crate::parse::model::ContentElement::Whitespace(_) => {
                                // Skip whitespace
                            }
                        }
                    }
                }
            }
        }
    }

    output.push_str("\n    }\n  }\n}");
    output
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


