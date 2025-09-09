use crate::document::model::{LowerLine, LowerElement, Source, Position};
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
            '.' | '•' => LowerElement::LowerOctaveMarker {
                marker: ch.to_string(),
                source: Source {
                    value: ch.to_string(),
                    position: Position { line: line_num, column: col },
                },
            },
            ':' => LowerElement::LowerOctaveMarker {
                marker: ch.to_string(),
                source: Source {
                    value: ch.to_string(),
                    position: Position { line: line_num, column: col },
                },
            },
            
            // BeatGroup: consecutive underscores (same symbol as slurs, different spatial context)
            '_' => {
                let mut underscores = String::new();
                underscores.push(ch);
                let start_col = col;
                
                // Collect consecutive underscores
                while let Some(&'_') = chars.peek() {
                    underscores.push(chars.next().unwrap());
                    col += 1;
                }
                
                LowerElement::BeatGroup {
                    underscores: underscores.clone(),
                    source: Source {
                        value: underscores,
                        position: Position { line: line_num, column: start_col },
                    },
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
                
                LowerElement::Space {
                    count,
                    source: Source {
                        value: " ".repeat(count),
                        position: Position { line: line_num, column: start_col },
                    },
                }
            },
            
            // Skip other characters for now (flat markers - 🚧 planned for Bhatkande notation)
            _ => {
                col += 1;
                continue;
            }
        };
        
        elements.push(element);
        col += 1;
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
        
        if let LowerElement::BeatGroup { underscores, .. } = &lower_line.elements[0] {
            assert_eq!(underscores, "___");
        } else {
            panic!("Expected BeatGroup");
        }
    }
    
    #[test]
    fn test_parse_mixed_lower_line() {
        let line = ".___  :";
        let lower_line = parse_lower_line(line, 1).unwrap();
        assert_eq!(lower_line.elements.len(), 4); // dot, beat group, spaces, colon
        
        // Should be: LowerOctaveMarker("."), BeatGroup("___"), Space(2), LowerOctaveMarker(":")
        match (&lower_line.elements[0], &lower_line.elements[1], &lower_line.elements[2], &lower_line.elements[3]) {
            (LowerElement::LowerOctaveMarker { marker: m1, .. }, 
             LowerElement::BeatGroup { underscores, .. },
             LowerElement::Space { .. },
             LowerElement::LowerOctaveMarker { marker: m2, .. }) => {
                assert_eq!(m1, ".");
                assert_eq!(underscores, "___");
                assert_eq!(m2, ":");
            }
            _ => panic!("Unexpected element sequence"),
        }
    }
}