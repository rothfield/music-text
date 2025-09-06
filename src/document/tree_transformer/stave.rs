use crate::document::pest_interface::{Pair, Rule};
use crate::document::model::{Stave, TextLine};
use super::helpers::source_from_span;
use super::content_line::transform_content_line;

pub(super) fn transform_stave(stave_pair: Pair<Rule>) -> Result<Stave, String> {
    let mut text_lines_before = Vec::new();
    let mut content_line_data = None;
    let mut text_lines_after = Vec::new();
    let mut found_content = false;
    let source = source_from_span(stave_pair.as_span());
    
    for inner_pair in stave_pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::text_lines => {
                let text_line = transform_text_lines(inner_pair);
                if !found_content {
                    text_lines_before.extend(text_line);
                } else {
                    text_lines_after.extend(text_line);
                }
            }
            Rule::content_line => {
                content_line_data = Some(transform_content_line(inner_pair, &text_lines_before, &text_lines_after)?);
                found_content = true;
            }
            _ => {}
        }
    }
    
    let (content_line, notation_system) = content_line_data.ok_or("No content line found in stave")?;
    
    Ok(Stave {
        text_lines_before,
        content_line,
        text_lines_after,
        notation_system,
        source,
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