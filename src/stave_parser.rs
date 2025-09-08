use crate::document::{Document, Stave};
use crate::rhythm_fsm::process_rhythm;

/// Pipeline step: Parse document staves into processed staves
/// 
/// This function takes the Document from document_parser and processes each stave
/// through the rhythm FSM to group elements into beats with subdivision information.
pub fn parse_document_staves(document: Document) -> Result<Vec<ProcessedStave>, String> {
    let mut processed_staves = Vec::new();
    
    for stave in document.staves {
        // Process content line elements through rhythm FSM
        let rhythm_items = process_rhythm(&stave.content_line.elements);
        
        let processed_stave = ProcessedStave {
            text_lines_before: stave.text_lines_before,
            rhythm_items,
            text_lines_after: stave.text_lines_after,
            notation_system: stave.notation_system,
            source: stave.source,
        };
        
        processed_staves.push(processed_stave);
    }
    
    Ok(processed_staves)
}

/// A stave that has been processed through the rhythm FSM
/// Contains beat-grouped elements instead of flat element lists
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessedStave {
    pub text_lines_before: Vec<crate::document::TextLine>,
    pub rhythm_items: Vec<crate::rhythm_fsm::Item>,
    pub text_lines_after: Vec<crate::document::TextLine>,
    pub notation_system: crate::document::model::NotationSystem,
    pub source: crate::document::model::Source,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::parse_document;

    #[test]
    fn test_parse_document_staves() {
        let input = "|1 2 3";
        let document = parse_document(input).unwrap();
        let staves = parse_document_staves(document).unwrap();
        
        // Should return the same staves as input
        assert_eq!(staves.len(), 1);
        assert_eq!(staves[0].content_line.elements.len(), 6); // |, 1, space, 2, space, 3 (barline included as element)
    }

    #[test]
    fn test_parse_multi_stave() {
        let input = "|1 2\n\n|3 4";
        let document = parse_document(input).unwrap();
        let staves = parse_document_staves(document).unwrap();
        
        // Should return both staves
        assert_eq!(staves.len(), 2);
    }
}