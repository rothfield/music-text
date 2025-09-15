use serde::Serialize;
use crate::parse::{Document, UpperElement, LowerElement};
use crate::parse::model::{DocumentElement, StaveLine};
use crate::rhythm::types::{ParsedElement, ParsedChild};

#[derive(Debug, Serialize)]
pub struct SyntaxToken {
    pub token_type: String,
    pub start: usize,
    pub end: usize,
    pub content: String,
}

#[derive(Debug, Serialize)]
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
    let mut styles = Vec::new();

    for token in tokens {
        // Generate a CSS class name from token type
        let css_class = format!("cm-music-{}", token.token_type);

        // Create a style entry for each character position in this token
        for pos in token.start..token.end {
            styles.push(CharacterStyle {
                pos,
                classes: vec![css_class.clone()],
            });
        }
    }

    // Sort by position to ensure correct ordering
    styles.sort_by_key(|style| style.pos);
    styles
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
                                        tokens.push(SyntaxToken {
                                            token_type: "whitespace".to_string(),
                                            start: position,
                                            end: position + value.len(),
                                            content: value.to_string(),
                                        });
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
                                        tokens.push(SyntaxToken {
                                            token_type: "newline".to_string(),
                                            start: position,
                                            end: position + value.len(),
                                            content: value.to_string(),
                                        });
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
pub fn generate_syntax_tokens(document: &Document) -> Vec<SyntaxToken> {
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

        // Space after colon
        tokens.push(SyntaxToken {
            token_type: "whitespace".to_string(),
            start: position,
            end: position + 1,
            content: " ".to_string(),
        });
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

        // Newline after directive
        tokens.push(SyntaxToken {
            token_type: "newline".to_string(),
            start: position,
            end: position + 1,
            content: "\n".to_string(),
        });
        position += 1;
    }

    // Process document elements
    for element in &document.elements {
        match element {
            DocumentElement::BlankLines(blank_lines) => {
                let content_len = blank_lines.content.len();
                if content_len > 0 {
                    tokens.push(SyntaxToken {
                        token_type: "blank_lines".to_string(),
                        start: position,
                        end: position + content_len,
                        content: blank_lines.content.clone(),
                    });
                    position += content_len;
                }
            }
            DocumentElement::Stave(stave) => {
                // Process all lines in the stave
                for line in &stave.lines {
                    match line {
                        StaveLine::Text(text_line) => {
                            let content_len = text_line.content.len();
                            if content_len > 0 {
                                tokens.push(SyntaxToken {
                                    token_type: "text".to_string(),
                                    start: position,
                                    end: position + content_len,
                                    content: text_line.content.clone(),
                                });
                                position += content_len;
                            }
                            // Add newline if not already in content
                            if !text_line.content.ends_with('\n') {
                                tokens.push(SyntaxToken {
                                    token_type: "newline".to_string(),
                                    start: position,
                                    end: position + 1,
                                    content: "\n".to_string(),
                                });
                                position += 1;
                            }
                        }
                        StaveLine::Upper(upper_line) => {
                            // Process upper line elements
                            for element in &upper_line.elements {
                                process_upper_element(element, &mut tokens, &mut position);
                            }
                        }
                        StaveLine::Content(parsed_elements) => {
                            // Process content line elements
                            for element in parsed_elements {
                                process_parsed_element(element, &mut tokens, &mut position);
                            }
                        }
                        StaveLine::Lower(lower_line) => {
                            // Process lower line elements
                            for element in &lower_line.elements {
                                process_lower_element(element, &mut tokens, &mut position);
                            }
                        }
                        StaveLine::Lyrics(lyrics_line) => {
                            // Process lyrics syllables
                            for (i, syllable) in lyrics_line.syllables.iter().enumerate() {
                                if i > 0 {
                                    // Add space between syllables
                                    tokens.push(SyntaxToken {
                                        token_type: "whitespace".to_string(),
                                        start: position,
                                        end: position + 1,
                                        content: " ".to_string(),
                                    });
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
                            // Add newline after lyrics
                            tokens.push(SyntaxToken {
                                token_type: "newline".to_string(),
                                start: position,
                                end: position + 1,
                                content: "\n".to_string(),
                            });
                            position += 1;
                        }
                        StaveLine::Whitespace(whitespace_line) => {
                            // Process whitespace line elements (like Content lines)
                            for element in &whitespace_line.elements {
                                process_parsed_element(element, &mut tokens, &mut position);
                            }
                        }
                        StaveLine::BlankLines(blank_lines) => {
                            let content_len = blank_lines.content.len();
                            if content_len > 0 {
                                tokens.push(SyntaxToken {
                                    token_type: "stave_blank_lines".to_string(),
                                    start: position,
                                    end: position + content_len,
                                    content: blank_lines.content.clone(),
                                });
                                position += content_len;
                            }
                        }
                    }
                }
            }
        }
    }

    tokens
}

// Helper function to process upper line elements
fn process_upper_element(element: &crate::parse::UpperElement, tokens: &mut Vec<SyntaxToken>, position: &mut usize) {
    use crate::parse::UpperElement;

    match element {
        UpperElement::UpperOctaveMarker { marker, .. } => {
            let marker_len = marker.len();
            let token_type = match marker.as_str() {
                "." => "upper_octave_marker",
                ":" => "upper_octave_marker_2",
                _ => "upper_octave_marker"
            };
            tokens.push(SyntaxToken {
                token_type: token_type.to_string(),
                start: *position,
                end: *position + marker_len,
                content: marker.clone(),
            });
            *position += marker_len;
        }
        UpperElement::UpperUnderscores { value, .. } => {
            let value_len = value.len();
            tokens.push(SyntaxToken {
                token_type: "slur".to_string(),
                start: *position,
                end: *position + value_len,
                content: value.clone(),
            });
            *position += value_len;
        }
        UpperElement::UpperHashes { value, .. } => {
            let value_len = value.len();
            tokens.push(SyntaxToken {
                token_type: "multi_stave_marker".to_string(),
                start: *position,
                end: *position + value_len,
                content: value.clone(),
            });
            *position += value_len;
        }
        UpperElement::Ornament { pitches, .. } => {
            // Join pitches into ornament string
            let ornament_str = pitches.join("");
            let ornament_len = ornament_str.len();
            tokens.push(SyntaxToken {
                token_type: "ornament".to_string(),
                start: *position,
                end: *position + ornament_len,
                content: ornament_str,
            });
            *position += ornament_len;
        }
        UpperElement::Chord { chord, .. } => {
            // Include brackets in chord token
            let chord_str = format!("[{}]", chord);
            let chord_len = chord_str.len();
            tokens.push(SyntaxToken {
                token_type: "chord".to_string(),
                start: *position,
                end: *position + chord_len,
                content: chord_str,
            });
            *position += chord_len;
        }
        UpperElement::Mordent { .. } => {
            tokens.push(SyntaxToken {
                token_type: "mordent".to_string(),
                start: *position,
                end: *position + 1,
                content: "~".to_string(),
            });
            *position += 1;
        }
        UpperElement::Space { count, .. } => {
            let spaces = " ".repeat(*count);
            tokens.push(SyntaxToken {
                token_type: "whitespace".to_string(),
                start: *position,
                end: *position + count,
                content: spaces,
            });
            *position += count;
        }
        UpperElement::Unknown { value, .. } => {
            let value_len = value.len();
            tokens.push(SyntaxToken {
                token_type: "unknown".to_string(),
                start: *position,
                end: *position + value_len,
                content: value.clone(),
            });
            *position += value_len;
        }
        UpperElement::Newline { value, .. } => {
            let value_len = value.len();
            tokens.push(SyntaxToken {
                token_type: "newline".to_string(),
                start: *position,
                end: *position + value_len,
                content: value.clone(),
            });
            *position += value_len;
        }
    }
}

// Helper function to process lower line elements
fn process_lower_element(element: &crate::parse::LowerElement, tokens: &mut Vec<SyntaxToken>, position: &mut usize) {
    use crate::parse::LowerElement;

    match element {
        LowerElement::LowerOctaveMarker { marker, .. } => {
            let marker_len = marker.len();
            let token_type = match marker.as_str() {
                "." => "lower_octave_marker",
                ":" => "lower_octave_marker_2",
                _ => "lower_octave_marker"
            };
            tokens.push(SyntaxToken {
                token_type: token_type.to_string(),
                start: *position,
                end: *position + marker_len,
                content: marker.clone(),
            });
            *position += marker_len;
        }
        LowerElement::LowerUnderscores { value, .. } => {
            let value_len = value.len();
            tokens.push(SyntaxToken {
                token_type: "beat_group".to_string(),
                start: *position,
                end: *position + value_len,
                content: value.clone(),
            });
            *position += value_len;
        }
        LowerElement::Syllable { content, .. } => {
            let content_len = content.len();
            tokens.push(SyntaxToken {
                token_type: "syllable".to_string(),
                start: *position,
                end: *position + content_len,
                content: content.clone(),
            });
            *position += content_len;
        }
        LowerElement::Space { count, .. } => {
            let spaces = " ".repeat(*count);
            tokens.push(SyntaxToken {
                token_type: "whitespace".to_string(),
                start: *position,
                end: *position + count,
                content: spaces,
            });
            *position += count;
        }
        LowerElement::Unknown { value, .. } => {
            let value_len = value.len();
            tokens.push(SyntaxToken {
                token_type: "unknown".to_string(),
                start: *position,
                end: *position + value_len,
                content: value.clone(),
            });
            *position += value_len;
        }
        LowerElement::Newline { value, .. } => {
            let value_len = value.len();
            tokens.push(SyntaxToken {
                token_type: "newline".to_string(),
                start: *position,
                end: *position + value_len,
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
fn process_parsed_element(element: &ParsedElement, tokens: &mut Vec<SyntaxToken>, position: &mut usize) {
    match element {
        ParsedElement::Note { value, in_slur, in_beat_group, .. } => {
            let value_len = value.len();
            let mut token_type = "note".to_string();

            // Add modifiers based on context
            if *in_slur {
                token_type.push_str("_slur");
            }
            if *in_beat_group {
                token_type.push_str("_beat");
            }

            tokens.push(SyntaxToken {
                token_type,
                start: *position,
                end: *position + value_len,
                content: value.clone(),
            });
            *position += value_len;
        }
        ParsedElement::Rest { value, .. } => {
            let value_len = value.len();
            tokens.push(SyntaxToken {
                token_type: "rest".to_string(),
                start: *position,
                end: *position + value_len,
                content: value.clone(),
            });
            *position += value_len;
        }
        ParsedElement::Dash { .. } => {
            tokens.push(SyntaxToken {
                token_type: "dash".to_string(),
                start: *position,
                end: *position + 1,
                content: "-".to_string(),
            });
            *position += 1;
        }
        ParsedElement::Barline { style, .. } => {
            let style_len = style.len();
            tokens.push(SyntaxToken {
                token_type: "barline".to_string(),
                start: *position,
                end: *position + style_len,
                content: style.clone(),
            });
            *position += style_len;
        }
        ParsedElement::Whitespace { value, .. } => {
            let value_len = value.len();
            tokens.push(SyntaxToken {
                token_type: "whitespace".to_string(),
                start: *position,
                end: *position + value_len,
                content: value.clone(),
            });
            *position += value_len;
        }
        ParsedElement::Symbol { value, .. } => {
            let value_len = value.len();
            tokens.push(SyntaxToken {
                token_type: "symbol".to_string(),
                start: *position,
                end: *position + value_len,
                content: value.clone(),
            });
            *position += value_len;
        }
        ParsedElement::Unknown { value, .. } => {
            let value_len = value.len();
            tokens.push(SyntaxToken {
                token_type: "unknown".to_string(),
                start: *position,
                end: *position + value_len,
                content: value.clone(),
            });
            *position += value_len;
        }
        ParsedElement::Newline { value, .. } => {
            let value_len = value.len();
            tokens.push(SyntaxToken {
                token_type: "newline".to_string(),
                start: *position,
                end: *position + value_len,
                content: value.clone(),
            });
            *position += value_len;
        }
        ParsedElement::EndOfInput { .. } => {
            // End of input doesn't generate a token
        }
    }
}