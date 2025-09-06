use crate::document::{Document, Stave};

/// Pipeline step: Parse document staves into processed staves
/// 
/// This function takes the Document from document_parser and processes each stave.
/// For now, it just returns the staves as-is, but this is where we'll add
/// musical processing logic in the future.
pub fn parse_document_staves(document: Document) -> Result<Vec<Stave>, String> {
    Ok(document.staves)
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