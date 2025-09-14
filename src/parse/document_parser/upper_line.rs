use crate::parse::model::{UpperLine, UpperElement, Source, Position};
use super::error::ParseError;

/// Parse an upper line into spatial annotation elements
/// 
/// UpperLine contains: UpperOctaveMarker (• typed as .), Slur (___), Ornaments, etc.
/// Per MUSIC_TEXT_SPECIFICATION.md
pub fn parse_upper_line(line: &str, line_num: usize) -> Result<UpperLine, ParseError> {
    let mut elements = Vec::new();
    let mut col = 1;
    let mut chars = line.chars().peekable();
    
    while let Some(ch) = chars.next() {
        let element = match ch {
            // UpperOctaveMarker: dots (.), bullets (•), and colons (:)
            '.' | '•' => UpperElement::UpperOctaveMarker {
                marker: ch.to_string(),
                source: Source {
                    value: Some(ch.to_string()),
                    position: Position { line: line_num, column: col },
                },
            },
            ':' => UpperElement::UpperOctaveMarker {
                marker: ch.to_string(),
                source: Source {
                    value: Some(ch.to_string()),
                    position: Position { line: line_num, column: col },
                },
            },
            
            // UpperUnderscores: consecutive underscores for slurs (requires >= 2)
            '_' => {
                let mut chars_collected = String::new();
                chars_collected.push(ch);
                let start_col = col;
                
                // Collect consecutive underscores
                while let Some(&'_') = chars.peek() {
                    chars_collected.push(chars.next().unwrap());
                    col += 1;
                }
                
                // Only create UpperUnderscores token if >= 2 characters (slurs require multiple elements)
                if chars_collected.len() >= 2 {
                    UpperElement::UpperUnderscores {
                        value: chars_collected.clone(),
                        source: Source {
                            value: Some(chars_collected),
                            position: Position { line: line_num, column: start_col },
                        },
                    }
                } else {
                    // Single underscore becomes Unknown token
                    UpperElement::Unknown {
                        value: chars_collected.clone(),
                        source: Source {
                            value: Some(chars_collected),
                            position: Position { line: line_num, column: start_col },
                        },
                    }
                }
            },
            
            // UpperHashes: consecutive hashes for multi-stave markers (requires >= 2)
            '#' => {
                let mut chars_collected = String::new();
                chars_collected.push(ch);
                let start_col = col;
                
                // Collect consecutive hashes
                while let Some(&'#') = chars.peek() {
                    chars_collected.push(chars.next().unwrap());
                    col += 1;
                }
                
                // Only create UpperHashes token if >= 2 characters (multi-stave markers require multiple elements)
                if chars_collected.len() >= 2 {
                    UpperElement::UpperHashes {
                        value: chars_collected.clone(),
                        source: Source {
                            value: Some(chars_collected),
                            position: Position { line: line_num, column: start_col },
                        },
                    }
                } else {
                    // Single hash becomes Unknown token
                    UpperElement::Unknown {
                        value: chars_collected.clone(),
                        source: Source {
                            value: Some(chars_collected),
                            position: Position { line: line_num, column: start_col },
                        },
                    }
                }
            },
            
            // Space: count consecutive spaces
            ' ' => {
                let mut count = 1;
                let start_col = col;
                
                while let Some(&' ') = chars.peek() {
                    chars.next();
                    count += 1;
                    col += 1;
                }
                
                UpperElement::Space {
                    count,
                    source: Source {
                        value: Some(" ".repeat(count)),
                        position: Position { line: line_num, column: start_col },
                    },
                }
            },
            
            // Unknown characters should be collected consecutively
            _ => {
                let mut chars_collected = String::new();
                chars_collected.push(ch);
                let start_col = col;
                
                // Collect consecutive unknown characters (not spaces, dots, underscores, etc.)
                while let Some(&next_ch) = chars.peek() {
                    match next_ch {
                        '.' | '•' | ':' | '_' | '#' | ' ' => break, // Stop at known tokens
                        _ => {
                            chars_collected.push(chars.next().unwrap());
                            col += 1;
                        }
                    }
                }
                
                UpperElement::Unknown {
                    value: chars_collected.clone(),
                    source: Source {
                        value: Some(chars_collected),
                        position: Position { line: line_num, column: start_col },
                    },
                }
            }
        };
        
        elements.push(element);
        col += 1;
    }
    
    Ok(UpperLine {
        elements,
        source: Source {
            value: Some(line.to_string()),
            position: Position { line: line_num, column: 1 },
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_upper_octave_markers() {
        let line = ".  :";
        let upper_line = parse_upper_line(line, 1).unwrap();
        assert_eq!(upper_line.elements.len(), 3); // dot, 2 spaces, colon
        
        // Check first element is UpperOctaveMarker with dot
        if let UpperElement::UpperOctaveMarker { marker, .. } = &upper_line.elements[0] {
            assert_eq!(marker, ".");
        } else {
            panic!("Expected UpperOctaveMarker");
        }
    }
    
    #[test]
    fn test_parse_slur_underscores() {
        let line = "___";
        let upper_line = parse_upper_line(line, 1).unwrap();
        assert_eq!(upper_line.elements.len(), 1);
        
        if let UpperElement::UpperUnderscores { value, .. } = &upper_line.elements[0] {
            assert_eq!(value, "___");
        } else {
            panic!("Expected UpperUnderscores");
        }
    }
    
    #[test]
    fn test_parse_mixed_upper_line() {
        let line = ".___  :";
        let upper_line = parse_upper_line(line, 1).unwrap();
        assert_eq!(upper_line.elements.len(), 4); // dot, slur, spaces, colon
        
        // Should be: UpperOctaveMarker("."), UpperUnderscores("___"), Space(2), UpperOctaveMarker(":")
        match (&upper_line.elements[0], &upper_line.elements[1], &upper_line.elements[2], &upper_line.elements[3]) {
            (UpperElement::UpperOctaveMarker { marker: m1, .. }, 
             UpperElement::UpperUnderscores { value, .. },
             UpperElement::Space { .. },
             UpperElement::UpperOctaveMarker { marker: m2, .. }) => {
                assert_eq!(m1, ".");
                assert_eq!(value, "___");
                assert_eq!(m2, ":");
            }
            _ => panic!("Unexpected element sequence"),
        }
    }
    
    #[test]
    fn test_parse_unknown_characters() {
        let line = "x@%";
        let upper_line = parse_upper_line(line, 1).unwrap();
        assert_eq!(upper_line.elements.len(), 1); // 1 collected unknown token
        
        // Should be a single Unknown element with all consecutive chars collected
        if let UpperElement::Unknown { value, .. } = &upper_line.elements[0] {
            assert_eq!(value, "x@%");
        } else {
            panic!("Expected Unknown element");
        }
    }
    
    #[test]
    fn test_parse_mixed_unknown_and_known() {
        let line = "abc.def";
        let upper_line = parse_upper_line(line, 1).unwrap();
        assert_eq!(upper_line.elements.len(), 3); // "abc", ".", "def"
        
        // Should be: Unknown("abc"), UpperOctaveMarker("."), Unknown("def")
        match (&upper_line.elements[0], &upper_line.elements[1], &upper_line.elements[2]) {
            (UpperElement::Unknown { value: v1, .. },
             UpperElement::UpperOctaveMarker { marker, .. },
             UpperElement::Unknown { value: v2, .. }) => {
                assert_eq!(v1, "abc");
                assert_eq!(marker, ".");
                assert_eq!(v2, "def");
            }
            _ => panic!("Unexpected element sequence"),
        }
    }
}