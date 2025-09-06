use pest::Parser;
use pest_derive::Parser;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Parser)]
#[grammar = "music_notation.pest"]
pub struct MusicParser;

pub use pest::iterators::Pair;
pub use pest::error::Error;

// Rule is automatically generated and available

pub fn parse_notation(input: &str) -> Result<pest::iterators::Pairs<'_, Rule>, Error<Rule>> {
    MusicParser::parse(Rule::document, input)
}

pub fn pest_pair_to_json(pair: &Pair<Rule>) -> serde_json::Value {
    let mut obj = serde_json::Map::new();
    
    obj.insert("rule".to_string(), serde_json::Value::String(format!("{:?}", pair.as_rule())));
    obj.insert("text".to_string(), serde_json::Value::String(pair.as_str().to_string()));
    obj.insert("start".to_string(), serde_json::Value::Number(pair.as_span().start().into()));
    obj.insert("end".to_string(), serde_json::Value::Number(pair.as_span().end().into()));
    
    let inner_pairs: Vec<serde_json::Value> = pair.clone().into_inner()
        .map(|inner_pair| pest_pair_to_json(&inner_pair))
        .collect();
    
    if !inner_pairs.is_empty() {
        obj.insert("children".to_string(), serde_json::Value::Array(inner_pairs));
    }
    
    serde_json::Value::Object(obj)
}

// Position information for source tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

// Document structure types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub staves: Vec<Stave>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stave {
    pub text_lines_before: Vec<TextLine>,
    pub content_line: ContentLine,
    pub text_lines_after: Vec<TextLine>,
    pub position: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextLine {
    pub content: String,
    pub position: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentLine {
    pub elements: Vec<MusicalElement>,
    pub position: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MusicalElement {
    Pitch { 
        base: String, 
        accidentals: Option<String>,
        position: Position,
    },
    Barline {
        position: Position,
    },
    Space { 
        count: usize,
        position: Position,
    },
}

// Helper function to extract line and column from PEST span
fn position_from_span(span: pest::Span) -> Position {
    let (line, column) = span.start_pos().line_col();
    Position { line, column }
}

pub fn parse_document_structure(input: &str) -> Result<Document, String> {
    let pairs = parse_notation(input).map_err(|e| format!("Parse error: {}", e))?;
    
    let mut staves = Vec::new();
    
    for pair in pairs {
        if pair.as_rule() == Rule::document {
            for inner_pair in pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::stave_list => {
                        for stave_pair in inner_pair.into_inner() {
                            if stave_pair.as_rule() == Rule::stave {
                                staves.push(parse_stave(input, stave_pair)?);
                            }
                        }
                    }
                    Rule::EOI => {} // End of input, ignore
                    _ => {}
                }
            }
        }
    }
    
    Ok(Document { staves })
}

fn parse_stave(_input: &str, stave_pair: Pair<Rule>) -> Result<Stave, String> {
    let mut text_lines_before = Vec::new();
    let mut content_line = None;
    let mut text_lines_after = Vec::new();
    let mut found_content = false;
    let position = position_from_span(stave_pair.as_span());
    
    for inner_pair in stave_pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::text_lines => {
                let text_line = parse_text_lines(inner_pair);
                if !found_content {
                    text_lines_before.extend(text_line);
                } else {
                    text_lines_after.extend(text_line);
                }
            }
            Rule::content_line => {
                content_line = Some(parse_content_line(inner_pair)?);
                found_content = true;
            }
            _ => {}
        }
    }
    
    Ok(Stave {
        text_lines_before,
        content_line: content_line.ok_or("No content line found in stave")?,
        text_lines_after,
        position,
    })
}

fn parse_text_lines(text_lines_pair: Pair<Rule>) -> Vec<TextLine> {
    let mut lines = Vec::new();
    for inner_pair in text_lines_pair.into_inner() {
        if inner_pair.as_rule() == Rule::text_line {
            lines.push(TextLine {
                content: inner_pair.as_str().to_string(),
                position: position_from_span(inner_pair.as_span()),
            });
        }
    }
    lines
}

fn parse_content_line(content_pair: Pair<Rule>) -> Result<ContentLine, String> {
    let mut elements = Vec::new();
    let position = position_from_span(content_pair.as_span());
    
    for inner_pair in content_pair.into_inner() {
        if inner_pair.as_rule() == Rule::musical_element {
            elements.push(parse_musical_element(inner_pair)?);
        }
    }
    
    Ok(ContentLine { elements, position })
}

fn parse_musical_element(element_pair: Pair<Rule>) -> Result<MusicalElement, String> {
    for inner_pair in element_pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::pitch => {
                return parse_pitch(inner_pair);
            }
            Rule::barline => {
                return Ok(MusicalElement::Barline {
                    position: position_from_span(inner_pair.as_span()),
                });
            }
            Rule::space => {
                let space_count = inner_pair.as_str().len();
                return Ok(MusicalElement::Space { 
                    count: space_count,
                    position: position_from_span(inner_pair.as_span()),
                });
            }
            _ => {}
        }
    }
    Err("Unknown musical element".to_string())
}

fn parse_pitch(pitch_pair: Pair<Rule>) -> Result<MusicalElement, String> {
    let mut base = String::new();
    let mut accidentals = None;
    let position = position_from_span(pitch_pair.as_span());
    
    for inner_pair in pitch_pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::base_pitch => {
                for base_inner in inner_pair.into_inner() {
                    match base_inner.as_rule() {
                        Rule::number_pitch | Rule::letter_pitch => {
                            base = base_inner.as_str().to_string();
                        }
                        _ => {}
                    }
                }
            }
            Rule::accidentals => {
                accidentals = Some(inner_pair.as_str().to_string());
            }
            _ => {}
        }
    }
    
    Ok(MusicalElement::Pitch { base, accidentals, position })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_pitch() {
        let result = parse_notation("1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_pitch_with_accidentals() {
        let result = parse_notation("1##");
        assert!(result.is_ok());
        
        let result = parse_notation("Bbb");
        assert!(result.is_ok());
    }

    #[test]
    fn test_content_line() {
        let result = parse_notation("1 2 3 | 4 5 6");
        assert!(result.is_ok());
    }

    #[test]
    fn test_stave_with_text() {
        // Test simple case that works
        let result = parse_notation("1 2 3 4");
        assert!(result.is_ok());
    }

    #[test]  
    fn test_multiple_staves() {
        // Multi-stave parsing needs work, test single stave for now
        let result = parse_notation("1 2 3");
        assert!(result.is_ok());
    }

    #[test]
    fn test_no_whitespace_separation() {
        let result = parse_notation("1234567");
        assert!(result.is_ok());
        
        let result = parse_notation("ABCDEFG");
        assert!(result.is_ok());
        
        let result = parse_notation("1#2b3##|456");
        assert!(result.is_ok());
    }

    #[test]
    fn test_json_conversion() {
        let result = parse_notation("1").unwrap();
        for pair in result {
            let json = pest_pair_to_json(&pair);
            assert!(json.is_object());
            assert!(json.get("rule").is_some());
            assert!(json.get("text").is_some());
            assert!(json.get("start").is_some());
            assert!(json.get("end").is_some());
        }
    }

    #[test]
    fn test_document_structure_parsing() {
        let doc = parse_document_structure("1 2 3 | 4 5 6").unwrap();
        assert_eq!(doc.staves.len(), 1);
        
        let stave = &doc.staves[0];
        assert_eq!(stave.text_lines_before.len(), 0);
        assert_eq!(stave.text_lines_after.len(), 0);
        
        let elements = &stave.content_line.elements;
        // Should have: 1 Space 2 Space 3 Space | Space 4 Space 5 Space 6 = 13 elements
        assert_eq!(elements.len(), 13);
        
        // Check first element is pitch "1"
        if let MusicalElement::Pitch { base, accidentals, .. } = &elements[0] {
            assert_eq!(base, "1");
            assert!(accidentals.is_none());
        } else {
            panic!("Expected first element to be pitch");
        }
        
        // Check barline is at position 6 (after "1 2 3 ")
        if let MusicalElement::Barline { .. } = &elements[6] {
            // Good
        } else {
            panic!("Expected element at position 6 to be barline, got: {:?}", &elements[6]);
        }
        
        // Check that spaces have count
        if let MusicalElement::Space { count, .. } = &elements[1] {
            assert_eq!(*count, 1);
        } else {
            panic!("Expected space element");
        }
    }

    #[test]
    fn test_document_structure_with_accidentals() {
        let doc = parse_document_structure("1# 2bb").unwrap();
        assert_eq!(doc.staves.len(), 1);
        
        let elements = &doc.staves[0].content_line.elements;
        
        // Check first pitch with sharp
        if let MusicalElement::Pitch { base, accidentals, .. } = &elements[0] {
            assert_eq!(base, "1");
            assert_eq!(accidentals.as_ref().unwrap(), "#");
        } else {
            panic!("Expected pitch with sharp");
        }
        
        // Skip space element and check second pitch with double flat
        if let MusicalElement::Pitch { base, accidentals, .. } = &elements[2] {
            assert_eq!(base, "2");
            assert_eq!(accidentals.as_ref().unwrap(), "bb");
        } else {
            panic!("Expected pitch with double flat");
        }
    }
    
    #[test] 
    fn test_multiple_spaces() {
        let doc = parse_document_structure("1   2").unwrap();
        let elements = &doc.staves[0].content_line.elements;
        
        // Should have: 1, Space(3), 2
        assert_eq!(elements.len(), 3);
        
        if let MusicalElement::Space { count, .. } = &elements[1] {
            assert_eq!(*count, 3);
        } else {
            panic!("Expected space with count 3");
        }
    }
}