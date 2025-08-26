// Lyrics processing module
// Handles parsing lyrics and attaching syllables to note groups

use crate::models::{Node, Token};

/// Dedicated lyrics parser - tokenizes a text line as words, not musical notation
pub fn parse_text_as_word_tokens(raw_line: &str, line_number: usize) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current_col = 0;
    
    // Split into words and track their positions
    for word in raw_line.split_whitespace() {
        // Skip leading whitespace to find word start position
        while current_col < raw_line.len() {
            if let Some(ch) = raw_line.chars().nth(current_col) {
                if !ch.is_whitespace() {
                    break;
                }
                current_col += 1;
            } else {
                break;
            }
        }
        
        // Create WORD token at the current position
        tokens.push(Token {
            token_type: "WORD".to_string(),
            value: word.to_string(),
            line: line_number,
            col: current_col,
        });
        
        // Move past this word
        current_col += word.len();
        
        // Skip any trailing whitespace for next word
        while current_col < raw_line.len() {
            if let Some(ch) = raw_line.chars().nth(current_col) {
                if !ch.is_whitespace() {
                    break;
                }
                current_col += 1;
            } else {
                break;
            }
        }
    }
    
    tokens
}

/// Parse lyrics from raw input text, re-parsing lyrics lines as words rather than musical notation
pub fn parse_lyrics_lines(tokens: &[Token], raw_text: &str) -> Vec<Vec<String>> {
    // First identify lyrics lines (lines that contain WORD tokens)
    let mut lyrics_line_numbers = std::collections::HashSet::new();
    for token in tokens {
        if token.token_type == "WORD" {
            lyrics_line_numbers.insert(token.line);
        }
    }
    
    if lyrics_line_numbers.is_empty() {
        return Vec::new();
    }
    
    // Split raw text into lines and process lyrics lines
    let lines: Vec<&str> = raw_text.lines().collect();
    let mut lyrics_by_line = std::collections::HashMap::new();
    
    for &line_num in &lyrics_line_numbers {
        if let Some(line_text) = lines.get(line_num) {
            // Re-parse this line as plain text words, ignoring musical notation
            let words: Vec<String> = line_text
                .split_whitespace()
                .filter(|word| {
                    // Filter out obvious musical symbols that might have been mixed in
                    let w = word.trim();
                    !w.is_empty() && 
                    !w.starts_with('|') && 
                    !w.ends_with('|') && 
                    w != "(" && w != ")" &&
                    w != "[" && w != "]" &&
                    !w.chars().all(|c| c == '.' || c == ':' || c == ',' || c == '\'')
                })
                .map(|w| w.to_string())
                .collect();
            
            if !words.is_empty() {
                lyrics_by_line.insert(line_num, words);
            }
        }
    }
    
    // Convert to sorted vec of lyrics lines
    let mut lines: Vec<(usize, Vec<String>)> = lyrics_by_line.into_iter().collect();
    lines.sort_by_key(|&(line_num, _)| line_num);
    
    lines.into_iter().map(|(_, words)| words).collect()
}


/// Distribute syllables to note groups (respecting slurs for melismas)
pub fn distribute_syllables_to_notes(
    nodes: &mut Vec<Node>,
    lyrics_lines: Vec<Vec<String>>,
) {
    // Flatten all lyrics into a single list of syllables
    let mut all_syllables: Vec<String> = Vec::new();
    for line in lyrics_lines {
        for word in line {
            // Split on hyphens to get individual syllables, preserving the hyphen with the syllable
            if word.contains('-') {
                let parts: Vec<&str> = word.split('-').collect();
                for (i, syl) in parts.iter().enumerate() {
                    if !syl.is_empty() {
                        // Add hyphen to all syllables except the last one
                        if i < parts.len() - 1 {
                            all_syllables.push(format!("{}-", syl));
                        } else {
                            all_syllables.push(syl.to_string());
                        }
                    }
                }
            } else {
                all_syllables.push(word);
            }
        }
    }
    
    // Process syllables with slur-aware distribution
    let mut syl_idx = 0;
    let mut in_slur = false;
    let mut slur_has_syllable = false;
    
    attach_syllables_respecting_slurs(nodes, &all_syllables, &mut syl_idx, &mut in_slur, &mut slur_has_syllable);
}

/// Attach syllables to pitch nodes, respecting slur boundaries for melismas
fn attach_syllables_respecting_slurs(
    nodes: &mut [Node], 
    syllables: &[String], 
    syl_idx: &mut usize,
    in_slur: &mut bool,
    slur_has_syllable: &mut bool,
) {
    for node in nodes {
        match node.node_type.as_str() {
            "SLUR_START" => {
                *in_slur = true;
                *slur_has_syllable = false;
            }
            "SLUR_END" => {
                *in_slur = false;
                *slur_has_syllable = false;
            }
            "PITCH" => {
                if *in_slur {
                    // Inside a slur - melismatic phrase
                    if !*slur_has_syllable && *syl_idx < syllables.len() {
                        // First note in slur gets the syllable
                        node.syl = Some(syllables[*syl_idx].clone());
                        *syl_idx += 1;
                        *slur_has_syllable = true;
                    } else {
                        // Subsequent notes in slur get underscore (lyric extender)
                        node.syl = Some("_".to_string());
                    }
                } else {
                    // Not in a slur - regular syllable assignment
                    if *syl_idx < syllables.len() {
                        node.syl = Some(syllables[*syl_idx].clone());
                        *syl_idx += 1;
                    }
                }
            }
            _ => {}
        }
        
        // Recursively process children
        attach_syllables_respecting_slurs(&mut node.nodes, syllables, syl_idx, in_slur, slur_has_syllable);
    }
}


/// Check if there are lyrics tokens below music lines
pub fn has_lyrics(tokens: &[Token], music_lines: &[usize]) -> bool {
    if music_lines.is_empty() {
        return false;
    }
    
    let max_music_line = *music_lines.iter().max().unwrap();
    
    // Check for WORD tokens below the last music line
    tokens.iter().any(|t| {
        t.token_type == "WORD" && t.line > max_music_line
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lyrics_lines() {
        let tokens = vec![
            Token {
                token_type: "WORD".to_string(),
                value: "ta".to_string(),
                line: 2,
                col: 0,
            },
            Token {
                token_type: "WORD".to_string(),
                value: "re".to_string(),
                line: 2,
                col: 3,
            },
            Token {
                token_type: "WORD".to_string(),
                value: "ga-ma".to_string(),
                line: 3,
                col: 0,
            },
        ];
        
        // Mock raw text with musical notation on lines 0-1 and lyrics on lines 2-3
        let raw_text = "| S R G M |\n| P D N S' |\nta re mi fa\nga-ma pa dha";
        
        let lyrics = parse_lyrics_lines(&tokens, raw_text);
        assert_eq!(lyrics.len(), 2);
        assert_eq!(lyrics[0], vec!["ta", "re", "mi", "fa"]);
        assert_eq!(lyrics[1], vec!["ga-ma", "pa", "dha"]);
    }

    #[test]
    fn test_syllable_distribution_with_melisma() {
        // Create test nodes: S (r g) m
        // This represents: one note, then a slurred group (melisma), then another note
        // With lyrics "ta re ga", we expect:
        // - "ta" on S
        // - "re" on r (first note of slur), "_" on g (continuation)
        // - "ga" on m
        let mut nodes = vec![
            Node::new("PITCH".to_string(), "S".to_string(), 0, 0),
            Node::new("SLUR_START".to_string(), "(".to_string(), 0, 2),
            Node::new("PITCH".to_string(), "r".to_string(), 0, 3),
            Node::new("PITCH".to_string(), "g".to_string(), 0, 5),
            Node::new("SLUR_END".to_string(), ")".to_string(), 0, 6),
            Node::new("PITCH".to_string(), "m".to_string(), 0, 8),
        ];
        
        let lyrics = vec![vec!["ta".to_string(), "re".to_string(), "ga".to_string()]];
        
        distribute_syllables_to_notes(&mut nodes, lyrics);
        
        // Check syllable assignment
        assert_eq!(nodes[0].syl, Some("ta".to_string()), "First note should have 'ta'");
        assert_eq!(nodes[2].syl, Some("re".to_string()), "First note in slur should have 're'");
        assert_eq!(nodes[3].syl, Some("_".to_string()), "Second note in slur should have underscore");
        assert_eq!(nodes[5].syl, Some("ga".to_string()), "Last note should have 'ga'");
    }
    
    #[test]
    fn test_hyphenated_syllables() {
        // Test that hyphenated words preserve hyphens correctly
        // "geor-gia" should become "geor-" and "gia"
        let mut nodes = vec![
            Node::new("PITCH".to_string(), "S".to_string(), 0, 0),
            Node::new("PITCH".to_string(), "R".to_string(), 0, 2),
            Node::new("PITCH".to_string(), "G".to_string(), 0, 4),
            Node::new("PITCH".to_string(), "M".to_string(), 0, 6),
        ];
        
        let lyrics = vec![vec!["geor-gia".to_string(), "on".to_string(), "my".to_string()]];
        
        distribute_syllables_to_notes(&mut nodes, lyrics);
        
        // Check syllable assignment with preserved hyphens
        assert_eq!(nodes[0].syl, Some("geor-".to_string()), "First syllable should have trailing hyphen");
        assert_eq!(nodes[1].syl, Some("gia".to_string()), "Second syllable should complete the word");
        assert_eq!(nodes[2].syl, Some("on".to_string()));
        assert_eq!(nodes[3].syl, Some("my".to_string()));
    }
    
    #[test]
    fn test_syllable_distribution_no_slurs() {
        // Test without slurs - each note gets its own syllable
        let mut nodes = vec![
            Node::new("PITCH".to_string(), "S".to_string(), 0, 0),
            Node::new("PITCH".to_string(), "R".to_string(), 0, 2),
            Node::new("PITCH".to_string(), "G".to_string(), 0, 4),
            Node::new("PITCH".to_string(), "M".to_string(), 0, 6),
        ];
        
        let lyrics = vec![vec!["do".to_string(), "re".to_string(), "mi".to_string(), "fa".to_string()]];
        
        distribute_syllables_to_notes(&mut nodes, lyrics);
        
        assert_eq!(nodes[0].syl, Some("do".to_string()));
        assert_eq!(nodes[1].syl, Some("re".to_string()));
        assert_eq!(nodes[2].syl, Some("mi".to_string()));
        assert_eq!(nodes[3].syl, Some("fa".to_string()));
    }
}