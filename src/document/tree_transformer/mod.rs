// Tree transformer module - modular structure for scalability
// Each sub-module handles transformation of specific grammar rules

mod helpers;
mod document;
mod stave;
mod content_line;
mod pitch;

use crate::document::pest_interface::{parse, Pair, Rule};
use crate::document::model::Document;
use serde_json;

// Re-export the main transformation function
use self::document::transform_document;

// Main entry point - transforms parsed input into Document structure  
pub fn build_document(input: &str) -> Result<Document, String> {
    let pairs = parse(input).map_err(|e| {
        let error_msg = format!("{}", e);
        
        // Check for missing barline error pattern
        if error_msg.contains("expected musical_element_no_barline or barline") {
            // Check if input has musical content but no barlines
            let has_musical_content = input.chars().any(|c| {
                match c {
                    '1'..='7' | 'A'..='G' | 'S' | 'R' | 'M' | 'P' | 'N' |
                    's' | 'r' | 'g' | 'm' | 'p' | 'd' | 'n' => true,
                    _ => false
                }
            }) || input.contains("स") || input.contains("रे") || input.contains("र") || 
                 input.contains("ग") || input.contains("म") || input.contains("प") || 
                 input.contains("ध") || input.contains("द") || input.contains("नि") || input.contains("न");
            let has_barline = input.contains('|');
            
            if has_musical_content && !has_barline {
                return "Content line requires at least one barline (|)".to_string();
            }
        }
        
        format!("Parse error: {}", error_msg)
    })?;
    transform_document(pairs)
}

// JSON conversion utility for debugging
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        // Just test that build_document doesn't crash
        let result = build_document("1 2 3 | 4 5 6");
        assert!(result.is_ok());
    }
}