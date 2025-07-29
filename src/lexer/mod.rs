// src/lexer/mod.rs
// Extracted lexer functionality from main.rs - no logic changes

use regex::Regex;
use std::collections::HashSet;
use crate::models::{ChunkInfo, LineInfo, Token, Title, Directive, Metadata, TokenType};

pub fn lex_text(raw_text: &str) -> Vec<LineInfo> {
    let mut lines_data = Vec::new();
    let chunk_regex = Regex::new(r"\S+").unwrap();

    for (line_num, line_text) in raw_text.lines().enumerate() {
        let mut chunks = Vec::new();
        for mat in chunk_regex.find_iter(line_text) {
            chunks.push(ChunkInfo {
                value: mat.as_str().to_string(),
                col: mat.start(), // 0-based for internal use
            });
        }
        lines_data.push(LineInfo {
            line_number: line_num + 1,
            line_text: line_text.to_string(),
            chunks,
        });
    }
    lines_data
}

pub fn tokenize_chunk(chunk: &str, line_num: usize, col_num: usize) -> Vec<Token> {
    let pitch_regex = Regex::new(r"^([SrRgGmMPdDnN]#?|[1-7][b#]?|[A-G][b#]?|-)+$").unwrap();
    if pitch_regex.is_match(chunk) {
        let mut tokens = Vec::new();
        let pitch_finder_regex = Regex::new(r"[SrRgGmMPdDnN]#?|[1-7][b#]?|[A-G][b#]?|-").unwrap();
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
    if chunk.chars().all(|c| c == '-') {
        return vec![Token {
            token_type: TokenType::Pitch.as_str().to_string(),
            value: chunk.to_string(),
            line: line_num,
            col: col_num,
        }];
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

pub fn parse_metadata(tokens: &[Token]) -> (Metadata, Vec<Token>) {
    let mut metadata = Metadata {
        title: None,
        directives: Vec::new(),
    };
    let mut consumed_lines = HashSet::new();

    // Find the first line containing a barline
    let first_main_line = tokens
        .iter()
        .find(|t| t.token_type == "BARLINE")
        .map(|t| t.line)
        .unwrap_or(usize::MAX);

    // 1. Extract Title: First line of WORDs before any main content
    if let Some(first_word) = tokens
        .iter()
        .find(|t| t.token_type == "WORD" && t.line < first_main_line)
    {
        let title_line = first_word.line;
        metadata.title = Some(Title {
            text: tokens
                .iter()
                .filter(|t| t.line == title_line && t.token_type == "WORD")
                .map(|t| t.value.as_str())
                .collect::<Vec<_>>()
                .join(" "),
            row: title_line,
            col: first_word.col,
        });
        consumed_lines.insert(title_line);
    }

    // 2. Extract Directives: Look for "key: value" or "key-value" patterns
    for (i, token) in tokens.iter().enumerate() {
        if token.line >= first_main_line { continue; }

        if token.token_type == "WORD" {
            if token.value.ends_with(':') {
                let key = token.value.trim_end_matches(':').to_string();
                let value_tokens: Vec<_> = tokens
                    .iter()
                    .skip(i + 1)
                    .take_while(|t| t.line == token.line && t.token_type != "NEWLINE")
                    .filter(|t| t.token_type != "WHITESPACE")
                    .collect();

                if !value_tokens.is_empty() {
                    metadata.directives.push(Directive {
                        key,
                        value: value_tokens.iter().map(|t| t.value.as_str()).collect::<Vec<_>>().join(" "),
                        row: token.line,
                        col: token.col,
                    });
                    consumed_lines.insert(token.line);
                }
            } else if token.value.contains('-') {
                let parts: Vec<&str> = token.value.splitn(2, '-').collect();
                if parts.len() == 2 {
                    metadata.directives.push(Directive {
                        key: parts[0].to_string(),
                        value: parts[1].to_string(),
                        row: token.line,
                        col: token.col,
                    });
                    consumed_lines.insert(token.line);
                }
            }
        }
    }

    // 3. Filter out consumed tokens
    let remaining_tokens: Vec<Token> = tokens
        .iter()
        .filter(|t| !consumed_lines.contains(&t.line))
        .cloned()
        .collect();

    (metadata, remaining_tokens)
}