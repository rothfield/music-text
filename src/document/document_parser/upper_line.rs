use crate::document::model::{UpperLine, UpperElement, Source, Position};
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
                    value: ch.to_string(),
                    position: Position { line: line_num, column: col },
                },
            },
            ':' => UpperElement::UpperOctaveMarker {
                marker: ch.to_string(),
                source: Source {
                    value: ch.to_string(),
                    position: Position { line: line_num, column: col },
                },
            },
            
            // Slur: consecutive underscores
            '_' => {
                let mut underscores = String::new();
                underscores.push(ch);
                let start_col = col;
                
                // Collect consecutive underscores
                while let Some(&'_') = chars.peek() {
                    underscores.push(chars.next().unwrap());
                    col += 1;
                }
                
                UpperElement::Slur {
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
                
                UpperElement::Space {
                    count,
                    source: Source {
                        value: " ".repeat(count),
                        position: Position { line: line_num, column: start_col },
                    },
                }
            },
            
            // Skip other characters for now (ornaments, chords - 🚧 planned)
            _ => {
                col += 1;
                continue;
            }
        };
        
        elements.push(element);
        col += 1;
    }
    
    Ok(UpperLine {
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
        
        if let UpperElement::Slur { underscores, .. } = &upper_line.elements[0] {
            assert_eq!(underscores, "___");
        } else {
            panic!("Expected Slur");
        }
    }
    
    #[test]
    fn test_parse_mixed_upper_line() {
        let line = ".___  :";
        let upper_line = parse_upper_line(line, 1).unwrap();
        assert_eq!(upper_line.elements.len(), 4); // dot, slur, spaces, colon
        
        // Should be: UpperOctaveMarker("."), Slur("___"), Space(2), UpperOctaveMarker(":")
        match (&upper_line.elements[0], &upper_line.elements[1], &upper_line.elements[2], &upper_line.elements[3]) {
            (UpperElement::UpperOctaveMarker { marker: m1, .. }, 
             UpperElement::Slur { underscores, .. },
             UpperElement::Space { .. },
             UpperElement::UpperOctaveMarker { marker: m2, .. }) => {
                assert_eq!(m1, ".");
                assert_eq!(underscores, "___");
                assert_eq!(m2, ":");
            }
            _ => panic!("Unexpected element sequence"),
        }
    }
}