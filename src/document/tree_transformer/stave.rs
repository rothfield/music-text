use crate::document::pest_interface::{Pair, Rule};
use crate::document::model::{Stave, TextLine, ContentLine, MusicalElement};
use super::helpers::source_from_span;
use super::content_line::transform_content_line;
use super::pitch::transform_pitch;

pub(super) fn transform_stave(stave_pair: Pair<Rule>) -> Result<Stave, String> {
    let mut text_lines_before = Vec::new();
    let mut content_lines = Vec::new();
    let mut text_lines_after = Vec::new();
    let mut found_content = false;
    let source = source_from_span(stave_pair.as_span());

    for inner_pair in stave_pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::text_lines => {
                let text_lines = transform_text_lines(inner_pair);
                if !found_content {
                    text_lines_before.extend(text_lines);
                } else {
                    text_lines_after.extend(text_lines);
                }
            }
            Rule::text_line => {
                // This handles the optional trailing text_line without a newline
                if found_content {
                    text_lines_after.push(TextLine {
                        content: inner_pair.as_str().to_string(),
                        source: source_from_span(inner_pair.as_span()),
                    });
                } else {
                    // Should not happen before content based on grammar, but handle defensively
                    text_lines_before.push(TextLine {
                        content: inner_pair.as_str().to_string(),
                        source: source_from_span(inner_pair.as_span()),
                    });
                }
            }
            Rule::content_line | Rule::simple_content_line => {
                let (content_line, _notation_system) = transform_content_line(inner_pair, &text_lines_before, &text_lines_after)?;
                content_lines.push(content_line);
                found_content = true;
            }
            _ => {}
        }
    }

    if content_lines.is_empty() {
        return Err("No content line found in stave".to_string());
    }

    // For now, we'll just merge the elements of all content lines into the first one.
    // A more sophisticated approach might be needed later.
    let mut final_content_line = content_lines.remove(0);
    for other_content_line in content_lines {
        final_content_line.elements.extend(other_content_line.elements);
    }

    let notation_system = crate::document::model::NotationSystem::Number; // Default for now

    let begin_multi_stave = text_lines_before.first()
        .map(|line| is_underscore_line(&line.content))
        .unwrap_or(false);

    let end_multi_stave = text_lines_after.last()
        .map(|line| is_underscore_line(&line.content))
        .unwrap_or(false);

    Ok(Stave {
        text_lines_before,
        content_line: final_content_line,
        text_lines_after,
        notation_system,
        source,
        begin_multi_stave,
        end_multi_stave,
    })
}

fn transform_text_lines(text_lines_pair: Pair<Rule>) -> Vec<TextLine> {
    let mut lines = Vec::new();
    for inner_pair in text_lines_pair.into_inner() {
        if inner_pair.as_rule() == Rule::text_line {
            lines.push(TextLine {
                content: inner_pair.as_str().to_string(),
                source: source_from_span(inner_pair.as_span()),
            });
        }
    }
    lines
}

/// Transform a simple_content_line (no barlines) into a Stave
pub(super) fn transform_simple_content_to_stave(content_pair: Pair<Rule>) -> Result<Stave, String> {
    use crate::document::model::NotationSystem;
    
    let source = source_from_span(content_pair.as_span());
    let mut elements = Vec::new();
    
    // Extract musical elements from simple_content_line
    for element in content_pair.clone().into_inner() {
        if element.as_rule() == Rule::musical_element_no_barline {
            for inner in element.into_inner() {
                match inner.as_rule() {
                    Rule::pitch => {
                        // Default values for simple content (no slurs, no beat groups)
                        let musical_elem = transform_pitch(inner, false, false, NotationSystem::Number)?;
                        elements.push(musical_elem);
                    }
                    Rule::space => {
                        elements.push(MusicalElement::Space {
                            count: 1,
                            in_slur: false,
                            in_beat_group: false,
                            source: source_from_span(inner.as_span()),
                        });
                    }
                    _ => {}
                }
            }
        }
    }
    
    let content_line = ContentLine {
        elements,
        source: source.clone(),
    };
    
    Ok(Stave {
        text_lines_before: Vec::new(),
        content_line,
        text_lines_after: Vec::new(),
        notation_system: NotationSystem::Number, // Default to Number system
        source,
        begin_multi_stave: false,
        end_multi_stave: false,
    })
}

fn is_underscore_line(content: &str) -> bool {
    let trimmed = content.trim();
    trimmed.len() >= 3 && trimmed.chars().all(|c| c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_underscore_line() {
        assert!(is_underscore_line("___"));
        assert!(is_underscore_line("____"));
        assert!(is_underscore_line("  ___  "));
        assert!(is_underscore_line(" ____________ "));
        
        assert!(!is_underscore_line("__"));
        assert!(!is_underscore_line("abc"));
        assert!(!is_underscore_line("___abc"));
        assert!(!is_underscore_line("  "));
        assert!(!is_underscore_line(""));
    }
}
