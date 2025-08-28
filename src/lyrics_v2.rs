// Lyrics processing module V2 - Works with ParsedElement
use crate::models::Token;
use crate::models_v2::{ParsedElement, ParsedChild};

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

/// Distribute syllables to note elements (respecting slurs for melismas)
pub fn distribute_syllables_to_elements(
    elements: &mut Vec<ParsedElement>,
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
    
    attach_syllables_respecting_slurs(elements, &all_syllables, &mut syl_idx, &mut in_slur, &mut slur_has_syllable);
}

/// Parse lyrics from raw input text (same as original)
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

/// Attach syllables to note elements, respecting slur boundaries for melismas
fn attach_syllables_respecting_slurs(
    elements: &mut [ParsedElement], 
    syllables: &[String], 
    syl_idx: &mut usize,
    in_slur: &mut bool,
    slur_has_syllable: &mut bool,
) {
    for element in elements {
        match element {
            ParsedElement::SlurStart { .. } => {
                *in_slur = true;
                *slur_has_syllable = false;
            }
            ParsedElement::SlurEnd { .. } => {
                *in_slur = false;
                *slur_has_syllable = false;
            }
            ParsedElement::Note { children, .. } => {
                if *in_slur {
                    // Inside a slur - melismatic phrase
                    if !*slur_has_syllable && *syl_idx < syllables.len() {
                        // First note in slur gets the syllable
                        children.push(ParsedChild::Syllable {
                            text: syllables[*syl_idx].clone(),
                            distance: 1, // Below the note
                        });
                        *syl_idx += 1;
                        *slur_has_syllable = true;
                    } else {
                        // Subsequent notes in slur get underscore (lyric extender)
                        children.push(ParsedChild::Syllable {
                            text: "_".to_string(),
                            distance: 1,
                        });
                    }
                } else {
                    // Not in a slur - regular syllable assignment
                    if *syl_idx < syllables.len() {
                        children.push(ParsedChild::Syllable {
                            text: syllables[*syl_idx].clone(),
                            distance: 1,
                        });
                        *syl_idx += 1;
                    }
                }
            }
            _ => {}
        }
    }
}


