use crate::document::pest_interface::{Pairs, Pair, Rule};
use crate::document::model::{Document, Source, Position};
use super::stave::{transform_stave, transform_simple_content_to_stave};

pub(super) fn transform_document(pairs: Pairs<Rule>) -> Result<Document, String> {
    let mut staves = Vec::new();
    let mut document_source = Source {
        value: String::new(),
        position: Position { line: 1, column: 1 },
    };

    for pair in pairs {
        if pair.as_rule() == Rule::document {
            document_source = Source {
                value: pair.as_str().to_string(),
                position: {
                    let (line, column) = pair.as_span().start_pos().line_col();
                    Position { line, column }
                },
            };

            for inner_pair in pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::stave => {
                        staves.push(transform_stave(inner_pair)?);
                    }
                    Rule::simple_content_line => {
                        if is_musical_line(&inner_pair) {
                            staves.push(transform_simple_content_to_stave(inner_pair)?);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(Document { staves, source: document_source })
}

fn is_musical_line(pair: &Pair<Rule>) -> bool {
    let mut pitch_count = 0;
    for element in pair.clone().into_inner() {
        if element.as_rule() == Rule::musical_element_no_barline {
            for inner in element.into_inner() {
                if inner.as_rule() == Rule::pitch {
                    pitch_count += 1;
                }
            }
        }
    }
    pitch_count >= 3
}
