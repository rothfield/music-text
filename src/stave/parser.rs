use crate::parse::Document;
use crate::rhythm::process_rhythm_batch;

/// Pipeline step: Parse document staves into processed staves with batch rhythm processing
/// 
/// This function takes the Document from document_parser and processes ALL staves
/// through batch rhythm FSM processing after spatial analysis is complete.
pub fn parse_document_staves(document: Document) -> Result<Vec<ProcessedStave>, String> {
    // Step 1: All staves are already spatially complete from parse_document
    // Step 2: Batch rhythm processing - collect all content lines  
    let all_content_lines: Vec<&Vec<crate::rhythm::types::ParsedElement>> = document.staves
        .iter()
        .map(|stave| &stave.content_line)
        .collect();
    
    // Step 3: Process rhythm for all staves in batch
    let all_rhythm_items = process_rhythm_batch(&all_content_lines);
    
    // Step 4: Build ProcessedStave objects with rhythm results
    let mut processed_staves = Vec::new();
    
    for (i, stave) in document.staves.into_iter().enumerate() {
        let rhythm_items = all_rhythm_items.get(i)
            .cloned()
            .unwrap_or_else(Vec::new);
            
        let processed_stave = ProcessedStave {
            text_lines_before: stave.text_lines_before,
            rhythm_items,
            text_lines_after: stave.text_lines_after,
            lyrics_lines: stave.lyrics_lines,
            notation_system: stave.notation_system,
            source: stave.source,
            begin_multi_stave: stave.begin_multi_stave,
            end_multi_stave: stave.end_multi_stave,
        };
        
        processed_staves.push(processed_stave);
    }
    
    Ok(processed_staves)
}

/// A stave that has been processed through the rhythm FSM
/// Contains beat-grouped elements instead of flat element lists
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessedStave {
    pub text_lines_before: Vec<crate::parse::TextLine>,
    pub rhythm_items: Vec<crate::rhythm::Item>,
    pub text_lines_after: Vec<crate::parse::TextLine>,
    pub lyrics_lines: Vec<crate::parse::model::LyricsLine>,
    pub notation_system: crate::parse::model::NotationSystem,
    pub source: crate::parse::model::Source,
    pub begin_multi_stave: bool,  // True if this stave begins a multi-stave group
    pub end_multi_stave: bool,    // True if this stave ends a multi-stave group
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_document;

    #[test]
    fn test_parse_document_staves() {
        let input = "|1 2 3";
        let document = parse_document(input).unwrap();
        let staves = parse_document_staves(document).unwrap();
        
        // Should return the same staves as input
        assert_eq!(staves.len(), 1);
        assert!(!staves[0].rhythm_items.is_empty()); // Should have rhythm items
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