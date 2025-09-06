use crate::document_parser::{parse_document_structure, Document};
use crate::stave_parser::{parse_document_staves};
use crate::document_parser::Stave;
use serde::{Deserialize, Serialize};

/// The complete processing pipeline output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    pub original_input: String,
    pub parsed_document: Document,
    pub processed_staves: Vec<Stave>,
}

/// Orchestrates the complete parsing pipeline
/// 
/// Input String → document_parser → stave_parser → ProcessingResult
pub fn process_notation(input: &str) -> Result<ProcessingResult, String> {
    // Stage 1: Parse text into Document structure
    let parsed_document = parse_document_structure(input)?;
    
    // Stage 2: Process document into staves
    let processed_staves = parse_document_staves(parsed_document.clone())?;
    
    Ok(ProcessingResult {
        original_input: input.to_string(),
        parsed_document,
        processed_staves,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_notation_single_stave() {
        let input = "|1 2 3";
        let result = process_notation(input).unwrap();
        
        assert_eq!(result.original_input, input);
        assert_eq!(result.parsed_document.staves.len(), 1);
        assert_eq!(result.processed_staves.len(), 1);
    }

    #[test]
    fn test_process_notation_multi_stave() {
        let input = "|1 2\n\n|3 4";
        let result = process_notation(input).unwrap();
        
        assert_eq!(result.parsed_document.staves.len(), 2);
        assert_eq!(result.processed_staves.len(), 2);
    }
}