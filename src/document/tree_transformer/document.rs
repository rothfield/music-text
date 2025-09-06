use crate::document::pest_interface::{Pairs, Rule};
use crate::document::model::{Document, Source, Position};
use super::stave::transform_stave;

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
                    Rule::stave_list => {
                        for stave_pair in inner_pair.into_inner() {
                            if stave_pair.as_rule() == Rule::stave {
                                staves.push(transform_stave(stave_pair)?);
                            }
                        }
                    }
                    Rule::EOI => {} // End of input, ignore
                    _ => {}
                }
            }
        }
    }
    
    Ok(Document { staves, source: document_source })
}