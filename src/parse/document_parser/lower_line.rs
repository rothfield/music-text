use crate::parse::model::{LowerLine, LowerElement, Source, Position};
use super::error::ParseError;

/// Parse a lower line into spatial annotation elements
/// 
/// LowerLine contains: LowerOctaveMarker (• typed as .), BeatGroup (___), FlatMarker, etc.
/// Per MUSIC_TEXT_SPECIFICATION.md
pub fn parse_lower_line(line: &str, line_num: usize) -> Result<LowerLine, ParseError> {
    let mut elements = Vec::new();
    let mut col = 1;
    let mut chars = line.chars().peekable();
    
    while let Some(ch) = chars.next() {
        let element = match ch {
            // LowerOctaveMarker: dots (.), bullets (•), and colons (:)
            '.' | '•' => {
                let marker_element = LowerElement::LowerOctaveMarker {
                    marker: ch.to_string(),
                    source: Source {
                        value: ch.to_string(),
                        position: Position { line: line_num, column: col },
                    },
                };
                col += 1;
                marker_element
            },
            ':' => {
                let marker_element = LowerElement::LowerOctaveMarker {
                    marker: ch.to_string(),
                    source: Source {
                        value: ch.to_string(),
                        position: Position { line: line_num, column: col },
                    },
                };
                col += 1;
                marker_element
            },
            
            // LowerUnderscores: consecutive underscores for beat grouping (requires >= 2)
            '_' => {
                let mut chars_collected = String::new();
                chars_collected.push(ch);
                let start_col = col;
                
                // Collect consecutive underscores
                while let Some(&'_') = chars.peek() {
                    chars_collected.push(chars.next().unwrap());
                    col += 1;
                }
                
                // Increment col for the first underscore that was consumed
                col += 1;
                
                // Only create LowerUnderscores token if >= 2 characters (groups require multiple elements)
                if chars_collected.len() >= 2 {
                    LowerElement::LowerUnderscores {
                        value: chars_collected.clone(),
                        source: Source {
                            value: chars_collected,
                            position: Position { line: line_num, column: start_col },
                        },
                    }
                } else {
                    // Single underscore becomes Unknown token
                    LowerElement::Unknown {
                        value: chars_collected.clone(),
                        source: Source {
                            value: chars_collected,
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
                
                // Increment col for the first space that was consumed
                col += 1;
                
                LowerElement::Space {
                    count,
                    source: Source {
                        value: " ".repeat(count),
                        position: Position { line: line_num, column: start_col },
                    },
                }
            },
            
            // Unknown characters should be collected consecutively
            _ => {
                let mut chars_collected = String::new();
                chars_collected.push(ch);
                let start_col = col;
                
                // Collect consecutive unknown characters (not spaces, dots, underscores)
                while let Some(&next_ch) = chars.peek() {
                    match next_ch {
                        '.' | '•' | ':' | '_' | ' ' => break, // Stop at known tokens
                        _ => {
                            chars_collected.push(chars.next().unwrap());
                            col += 1;
                        }
                    }
                }
                
                // Increment col for the first unknown character that was consumed
                col += 1;
                
                LowerElement::Unknown {
                    value: chars_collected.clone(),
                    source: Source {
                        value: chars_collected,
                        position: Position { line: line_num, column: start_col },
                    },
                }
            }
        };
        
        elements.push(element);
        // Don't increment col here - it's already handled in the match arms
    }
    
    Ok(LowerLine {
        elements,
        source: Source {
            value: line.to_string(),
            position: Position { line: line_num, column: 1 },
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lower_octave_markers() {
        let line = ".  :";
        let lower_line = parse_lower_line(line, 1).unwrap();
        assert_eq!(lower_line.elements.len(), 3); // dot, 2 spaces, colon
        
        // Check first element is LowerOctaveMarker with dot
        if let LowerElement::LowerOctaveMarker { marker, .. } = &lower_line.elements[0] {
            assert_eq!(marker, ".");
        } else {
            panic!("Expected LowerOctaveMarker");
        }
    }
    
    #[test]
    fn test_parse_beat_group_underscores() {
        let line = "___";
        let lower_line = parse_lower_line(line, 1).unwrap();
        assert_eq!(lower_line.elements.len(), 1);
        
        if let LowerElement::LowerUnderscores { value, .. } = &lower_line.elements[0] {
            assert_eq!(value, "___");
        } else {
            panic!("Expected LowerUnderscores");
        }
    }
    
    #[test]
    fn test_parse_mixed_lower_line() {
        let line = ".___  :";
        let lower_line = parse_lower_line(line, 1).unwrap();
        assert_eq!(lower_line.elements.len(), 4); // dot, beat group, spaces, colon
        
        // Should be: LowerOctaveMarker("."), LowerUnderscores("___"), Space(2), LowerOctaveMarker(":")
        match (&lower_line.elements[0], &lower_line.elements[1], &lower_line.elements[2], &lower_line.elements[3]) {
            (LowerElement::LowerOctaveMarker { marker: m1, .. }, 
             LowerElement::LowerUnderscores { value, .. },
             LowerElement::Space { .. },
             LowerElement::LowerOctaveMarker { marker: m2, .. }) => {
                assert_eq!(m1, ".");
                assert_eq!(value, "___");
                assert_eq!(m2, ":");
            }
            _ => panic!("Unexpected element sequence"),
        }
    }
    
    #[test]
    fn test_parse_unknown_characters() {
        let line = "xyz";
        let lower_line = parse_lower_line(line, 1).unwrap();
        assert_eq!(lower_line.elements.len(), 1); // 1 collected unknown token
        
        // Should be a single Unknown element with all consecutive chars collected
        if let LowerElement::Unknown { value, .. } = &lower_line.elements[0] {
            assert_eq!(value, "xyz");
        } else {
            panic!("Expected Unknown element");
        }
    }
    
    #[test]
    fn test_parse_mixed_unknown_and_known_lower() {
        let line = "abc.def";
        let lower_line = parse_lower_line(line, 1).unwrap();
        assert_eq!(lower_line.elements.len(), 3); // "abc", ".", "def"
        
        // Should be: Unknown("abc"), LowerOctaveMarker("."), Unknown("def")
        match (&lower_line.elements[0], &lower_line.elements[1], &lower_line.elements[2]) {
            (LowerElement::Unknown { value: v1, .. },
             LowerElement::LowerOctaveMarker { marker, .. },
             LowerElement::Unknown { value: v2, .. }) => {
                assert_eq!(v1, "abc");
                assert_eq!(marker, ".");
                assert_eq!(v2, "def");
            }
            _ => panic!("Unexpected element sequence"),
        }
    }
}