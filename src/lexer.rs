// src/lexer/mod.rs
// Extracted lexer functionality from main.rs - no logic changes

use regex::Regex;
use std::collections::{HashSet, HashMap};
use crate::models::{ChunkInfo, LineInfo, Token, Title, Directive, Metadata}; // TokenType DELETED - unused

fn split_barlines_from_chunk(chunk: &str, start_col: usize) -> Vec<ChunkInfo> {
    let mut result = Vec::new();
    let mut current_pos = 0;
    
    // Regex to find barline patterns
    let barline_regex = Regex::new(r"\|{1,2}[:.]*|:[|:.]*").unwrap();
    
    for mat in barline_regex.find_iter(chunk) {
        // Add any content before the barline
        if mat.start() > current_pos {
            let before_barline = &chunk[current_pos..mat.start()];
            result.push(ChunkInfo {
                value: before_barline.to_string(),
                col: start_col + current_pos,
            });
        }
        
        // Add the barline itself
        result.push(ChunkInfo {
            value: mat.as_str().to_string(),
            col: start_col + mat.start(),
        });
        
        current_pos = mat.end();
    }
    
    // Add any remaining content after the last barline
    if current_pos < chunk.len() {
        let remaining = &chunk[current_pos..];
        result.push(ChunkInfo {
            value: remaining.to_string(),
            col: start_col + current_pos,
        });
    }
    
    // If no barlines were found, return the original chunk
    if result.is_empty() {
        result.push(ChunkInfo {
            value: chunk.to_string(),
            col: start_col,
        });
    }
    
    result
}

pub fn lex_text(raw_text: &str) -> Vec<LineInfo> {
    let mut lines_data = Vec::new();
    let chunk_regex = Regex::new(r"\S+").unwrap();

    for (line_num, line_text) in raw_text.lines().enumerate() {
        let mut chunks = Vec::new();
        for mat in chunk_regex.find_iter(line_text) {
            // Split chunks that contain barlines adjacent to other characters
            let split_chunks = split_barlines_from_chunk(mat.as_str(), mat.start());
            chunks.extend(split_chunks);
        }
        lines_data.push(LineInfo {
            line_number: line_num + 1,
            line_text: line_text.to_string(),
            chunks,
        });
    }
    lines_data
}

// DELETED - unused V1 function
/*
pub fn tokenize_chunk(chunk: &str, line_num: usize, col_num: usize) -> Vec<Token> {
    let pitch_regex = Regex::new(r"^([SrRgGmMPdDnN](##|#|bb|b)?|[1-7](##|#|bb|b)?|[A-G](##|#|bb|b)?)+$").unwrap();
    if pitch_regex.is_match(chunk) {
        let mut tokens = Vec::new();
        let pitch_finder_regex = Regex::new(r"[SrRgGmMPdDnN](##|#|bb|b)?|[1-7](##|#|bb|b)?|[A-G](##|#|bb|b)?").unwrap();
        for mat in pitch_finder_regex.find_iter(chunk) {
            tokens.push(Token {
                token_type: TokenType::Pitch.as_str().to_string(),
                value: mat.as_str().to_string(),
                line: line_num,
                col: col_num + mat.start(),
            });
        }
        return tokens;
    }
    if ["|", "||", "|.", ":|", "|:", ":|:"].contains(&chunk) {
        return vec![Token {
            token_type: TokenType::Barline.as_str().to_string(),
            value: chunk.to_string(),
            line: line_num,
            col: col_num,
        }];
    }
    
    // Check for mixed symbol+pitch combinations like ".S", "'S", ":S" 
    let mixed_symbol_pitch_regex = Regex::new(r"^([.':]+)([SrRgGmMPdDnN](##|#|bb|b)?|[1-7](##|#|bb|b)?|[A-G](##|#|bb|b)?)$").unwrap();
    if let Some(captures) = mixed_symbol_pitch_regex.captures(chunk) {
        let mut tokens = Vec::new();
        let symbols = captures.get(1).unwrap().as_str();
        let pitch = captures.get(2).unwrap().as_str();
        
        // Add symbol token
        tokens.push(Token {
            token_type: TokenType::Symbols.as_str().to_string(),
            value: symbols.to_string(),
            line: line_num,
            col: col_num,
        });
        
        // Add pitch token
        tokens.push(Token {
            token_type: TokenType::Pitch.as_str().to_string(),
            value: pitch.to_string(),
            line: line_num,
            col: col_num + symbols.len(),
        });
        
        return tokens;
    }
    
    // Check for pitch+breath marker combinations like "S'", "R'", etc.
    let pitch_breath_regex = Regex::new(r"^([SrRgGmMPdDnN](##|#|bb|b)?|[1-7](##|#|bb|b)?|[A-G](##|#|bb|b)?)(['])$").unwrap();
    if let Some(captures) = pitch_breath_regex.captures(chunk) {
        let mut tokens = Vec::new();
        let pitch = captures.get(1).unwrap().as_str();
        let breath_marker = captures.get(2).unwrap().as_str();
        
        // Add pitch token
        tokens.push(Token {
            token_type: TokenType::Pitch.as_str().to_string(),
            value: pitch.to_string(),
            line: line_num,
            col: col_num,
        });
        
        // Add breath marker as symbols token
        tokens.push(Token {
            token_type: TokenType::Symbols.as_str().to_string(),
            value: breath_marker.to_string(),
            line: line_num,
            col: col_num + pitch.len(),
        });
        
        return tokens;
    }
    
    if chunk.chars().any(|c| c.is_alphabetic())
        || (chunk.chars().any(|c| c.is_numeric())
            && chunk.chars().any(|c| !c.is_alphanumeric()))
    {
        return vec![Token {
            token_type: TokenType::Word.as_str().to_string(),
            value: chunk.to_string(),
            line: line_num,
            col: col_num,
        }];
    }
    if chunk.chars().all(|c| !c.is_alphanumeric()) {
        return vec![Token {
            token_type: TokenType::Symbols.as_str().to_string(),
            value: chunk.to_string(),
            line: line_num,
            col: col_num,
        }];
    }
    vec![Token {
        token_type: TokenType::Unknown.as_str().to_string(),
        value: chunk.to_string(),
        line: line_num,
        col: col_num,
    }]
}
*/

fn group_words_into_segments<'a>(words: &'a [&'a Token]) -> Vec<Vec<&'a Token>> {
    if words.is_empty() {
        return Vec::new();
    }
    
    // Build the line text from word positions
    let _line_num = words[0].line;
    let max_col = words.iter().map(|w| w.col + w.value.len()).max().unwrap_or(0);
    let mut line_chars = vec![' '; max_col];
    
    // Place words in their positions
    for word in words {
        for (i, ch) in word.value.chars().enumerate() {
            if word.col + i < line_chars.len() {
                line_chars[word.col + i] = ch;
            }
        }
    }
    
    let line_text: String = line_chars.iter().collect();
    
    // Use regex to find word segments separated by 3+ spaces
    let _segment_regex = Regex::new(r"\S+(?:\s+\S+)*").unwrap();
    let large_gap_regex = Regex::new(r"\s{3,}").unwrap();
    
    // Split on large gaps to get segments
    let segment_texts: Vec<&str> = large_gap_regex.split(&line_text)
        .filter(|s| !s.trim().is_empty())
        .collect();
    
    let mut segments = Vec::new();
    
    for segment_text in segment_texts {
        let mut segment_words = Vec::new();
        
        // Find words in this segment
        for word in words {
            let word_start = word.col;
            let word_end = word.col + word.value.len();
            
            // Check if this word falls within the current segment boundaries
            if let Some(segment_start) = line_text.find(segment_text.trim()) {
                let segment_end = segment_start + segment_text.trim().len();
                
                if word_start >= segment_start && word_end <= segment_end {
                    segment_words.push(*word);
                }
            }
        }
        
        if !segment_words.is_empty() {
            segment_words.sort_by_key(|w| w.col);
            segments.push(segment_words);
        }
    }
    
    segments
}

pub fn parse_metadata(tokens: &[Token]) -> (Metadata, Vec<Token>) {
    let mut metadata = Metadata {
        title: None,
        directives: Vec::new(),
        detected_system: None,
        attributes: HashMap::new(),
    };
    let mut consumed_lines = HashSet::new();
    let mut consumed_tokens = HashSet::new();

    // First, identify all musical lines (lines with pitches or barlines)
    // But exclude lines that start with a WORD followed by a colon (directives)
    let mut musical_lines: HashSet<usize> = HashSet::new();
    let mut directive_lines: HashSet<usize> = HashSet::new();
    
    // Identify directive lines first
    for (i, token) in tokens.iter().enumerate() {
        if token.token_type == "WORD" && token.col == 0 {
            // Check if next non-whitespace token is a colon
            if let Some(colon_token) = tokens.iter().skip(i + 1).find(|t| t.line == token.line && t.token_type != "WHITESPACE") {
                if colon_token.token_type == "SYMBOLS" && colon_token.value == ":" {
                    directive_lines.insert(token.line);
                }
            }
        }
    }
    
    // Now identify musical lines, excluding directive lines
    for token in tokens {
        if (token.token_type == "PITCH" || token.token_type == "BARLINE") && !directive_lines.contains(&token.line) {
            musical_lines.insert(token.line);
        }
    }

    // Find the first musical line
    let first_musical_line = musical_lines.iter().min().copied().unwrap_or(usize::MAX);
    
    // Only process metadata from lines that come before the first musical line
    let max_line_num = tokens.iter().map(|t| t.line).max().unwrap_or(0);
    let search_limit = std::cmp::min(first_musical_line, max_line_num + 1);

    // 1. First, extract directives before looking for titles (to avoid "Key: Value" being treated as title/author)
    for (i, token) in tokens.iter().enumerate() {
        if token.line >= first_musical_line { continue; }

        // Look for WORD token at start of line followed by colon
        if token.token_type == "WORD" && token.col == 0 {
            // Check if next non-whitespace token is a colon
            if let Some(colon_token) = tokens.iter().skip(i + 1).find(|t| t.line == token.line && t.token_type != "WHITESPACE") {
                if colon_token.token_type == "SYMBOLS" && colon_token.value == ":" {
                    let key = token.value.clone();
                    
                    // Collect all remaining tokens on the line after the colon (including pitches!)
                    let value_tokens: Vec<_> = tokens
                        .iter()
                        .skip(i + 1)
                        .filter(|t| t.line == token.line && t.token_type != "NEWLINE" && t.token_type != "WHITESPACE" && !(t.token_type == "SYMBOLS" && t.value == ":"))
                        .collect();

                    if !value_tokens.is_empty() {
                        let value = value_tokens.iter().map(|t| t.value.as_str()).collect::<Vec<_>>().join(" ");
                        metadata.directives.push(Directive {
                            key: key.clone(),
                            value: value.clone(),
                            row: token.line,
                            col: token.col,
                        });
                        // Also store in attributes HashMap for easy access
                        metadata.attributes.insert(key, value);
                        consumed_lines.insert(token.line);
                    }
                }
            }
        }
        
        // Also handle "key-value" patterns as before
        if token.token_type == "WORD" && token.value.contains('-') {
            let parts: Vec<&str> = token.value.splitn(2, '-').collect();
            if parts.len() == 2 {
                let key = parts[0].to_string();
                let value = parts[1].to_string();
                metadata.directives.push(Directive {
                    key: key.clone(),
                    value: value.clone(),
                    row: token.line,
                    col: token.col,
                });
                metadata.attributes.insert(key, value);
                consumed_lines.insert(token.line);
            }
        }
    }

    // 2. Look for lines with exactly 2 word segments (title and author) - but skip consumed lines
    let mut found_title_author = false;
    for line_num in 1..search_limit {
        if consumed_lines.contains(&line_num) {
            continue; // Skip lines already consumed as directives
        }
        let line_words: Vec<_> = tokens
            .iter()
            .filter(|t| t.line == line_num && t.token_type == "WORD")
            .collect();
        
        if line_words.is_empty() {
            continue;
        }
        
        // Group words into segments based on gaps
        let segments = group_words_into_segments(&line_words);
        
        if segments.len() == 2 {
            // Check if this looks like a directive (WORD: VALUE pattern) - if so, skip it
            let first_segment_text = segments[0].iter().map(|t| t.value.as_str()).collect::<Vec<_>>().join(" ");
            let has_colon_after_first_segment = tokens
                .iter()
                .any(|t| t.line == line_num && t.token_type == "SYMBOLS" && t.value == ":" && 
                     t.col > segments[0].last().unwrap().col && t.col < segments[1][0].col);
            
            if has_colon_after_first_segment {
                // This looks like a directive (e.g., "Key: D"), skip it
                continue;
            }
            
            // Found a line with exactly 2 segments - first is title, second is author  
            let title_text = first_segment_text;
            let author_text = segments[1].iter().map(|t| t.value.as_str()).collect::<Vec<_>>().join(" ");
            
            metadata.title = Some(Title {
                text: title_text.clone(),
                row: line_num,
                col: segments[0][0].col,
            });
            
            // Store author information in a directive but don't display it in colorized output
            metadata.directives.push(Directive {
                key: "Author".to_string(),
                value: author_text,
                row: line_num,
                col: segments[1][0].col,
            });
            
            // Mark title tokens as consumed, but leave author tokens for display
            for token in &segments[0] {
                consumed_tokens.insert((token.line, token.col));
            }
            
            found_title_author = true;
            break;
        }
    }
    
    // Fallback: Extract Title from first line of WORDs if no 2-segment line found
    // BUT only if the line contains ONLY words and NO pitches (to avoid consuming musical content)
    if !found_title_author {
        if let Some(first_word) = tokens
            .iter()
            .find(|t| t.token_type == "WORD" && t.line < first_musical_line)
        {
            let title_line = first_word.line;
            
            // Extract title from WORD tokens but preserve all other tokens on the line
            let title_words: Vec<_> = tokens
                .iter()
                .filter(|t| t.line == title_line && t.token_type == "WORD")
                .collect();
            
            metadata.title = Some(Title {
                text: title_words
                    .iter()
                    .map(|t| t.value.as_str())
                    .collect::<Vec<_>>()
                    .join(" "),
                row: title_line,
                col: first_word.col,
            });
            
            // Mark only the WORD tokens as consumed, not the entire line
            // This preserves PITCH tokens and other content on the same line
            for word_token in title_words {
                consumed_tokens.insert((word_token.line, word_token.col));
            }
        }
    }


    // 3. Filter out consumed tokens
    let remaining_tokens: Vec<Token> = tokens
        .iter()
        .filter(|t| {
            let line_consumed = consumed_lines.contains(&t.line);
            let token_consumed = consumed_tokens.contains(&(t.line, t.col));
            
            // Always preserve WHITESPACE and NEWLINE tokens for spatial layout
            if t.token_type == "WHITESPACE" || t.token_type == "NEWLINE" {
                return !token_consumed; // Only exclude if specifically consumed
            }
            
            !line_consumed && !token_consumed
        })
        .cloned()
        .collect();

    (metadata, remaining_tokens)
}