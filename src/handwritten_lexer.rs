use crate::models::{Token, TokenType};
use crate::notation_detector::NotationType;

pub struct HandwrittenLexer<'a> {
    _input: &'a str,
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    notation_type: NotationType,
}

impl<'a> HandwrittenLexer<'a> {
    pub fn new(input: &'a str, notation_type: NotationType) -> Self {
        Self {
            _input: input,
            chars: input.chars().collect(),
            pos: 0,
            line: 0,
            col: 0,
            notation_type,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        
        while !self.is_at_end() {
            let start_line = self.line;
            let start_col = self.col;
            
            if let Some(token) = self.next_token() {
                // Update token position to start position
                let mut token = token;
                token.line = start_line;
                token.col = start_col;
                tokens.push(token);
            }
        }
        
        tokens
    }

    fn next_token(&mut self) -> Option<Token> {
        let start_pos = self.pos;
        let ch = self.advance()?;

        match ch {
            // Whitespace
            ' ' | '\t' => {
                // Consume consecutive whitespace
                while self.peek() == Some(' ') || self.peek() == Some('\t') {
                    self.advance();
                }
                Some(Token {
                    token_type: TokenType::Whitespace.as_str().to_string(),
                    value: self.chars[start_pos..self.pos].iter().collect(),
                    line: 0, // Will be set by caller
                    col: 0,  // Will be set by caller
                })
            }

            // Newlines
            '\n' => {
                self.line += 1;
                self.col = 0;
                Some(Token {
                    token_type: "NEWLINE".to_string(),
                    value: "\n".to_string(),
                    line: 0, // Will be set by caller
                    col: 0,  // Will be set by caller
                })
            }

            '\r' => {
                // Handle \r\n
                if self.peek() == Some('\n') {
                    self.advance();
                }
                self.line += 1;
                self.col = 0;
                Some(Token {
                    token_type: "NEWLINE".to_string(),
                    value: self.chars[start_pos..self.pos].iter().collect(),
                    line: 0, // Will be set by caller
                    col: 0,  // Will be set by caller
                })
            }

            // Barlines
            '|' => {
                let mut value = String::from("|");
                
                // Check for compound barlines
                match self.peek() {
                    Some('|') => {
                        self.advance();
                        value.push('|');
                    }
                    Some(']') => {
                        self.advance();
                        value.push(']');
                    }
                    Some(':') => {
                        self.advance();
                        value.push(':');
                    }
                    _ => {}
                }
                
                Some(Token {
                    token_type: TokenType::Barline.as_str().to_string(),
                    value,
                    line: 0, // Will be set by caller
                    col: 0,  // Will be set by caller
                })
            }


            // Right repeat barline  
            ':' if self.peek() == Some('|') => {
                self.advance(); // consume '|'
                Some(Token {
                    token_type: TokenType::Barline.as_str().to_string(),
                    value: ":|".to_string(),
                    line: 0, // Will be set by caller
                    col: 0,  // Will be set by caller
                })
            }

            // Check if character is a pitch in the detected notation system
            _ if self.is_pitch_char(ch) => {
                self.parse_pitch(ch)
            }

            // Dashes (rhythmic placeholders) - each dash is a separate token
            '-' => {
                Some(Token {
                    token_type: TokenType::Dash.as_str().to_string(),
                    value: "-".to_string(),
                    line: 0, // Will be set by caller
                    col: 0,  // Will be set by caller
                })
            }

            // Underscores - slur markers (consume consecutive underscores)
            '_' => {
                let slur_start_pos = start_pos;
                while let Some('_') = self.peek() {
                    self.advance();
                }
                let slur_value: String = self.chars[slur_start_pos..self.pos].iter().collect();
                Some(Token {
                    token_type: "SLUR".to_string(),
                    value: slur_value,
                    line: 0, // Will be set by caller
                    col: 0,  // Will be set by caller
                })
            }

            // Slur start
            '(' => {
                Some(Token {
                    token_type: "SLUR_START".to_string(),
                    value: "(".to_string(),
                    line: 0, // Will be set by caller
                    col: 0,  // Will be set by caller
                })
            }

            // Slur end
            ')' => {
                Some(Token {
                    token_type: "SLUR_END".to_string(),
                    value: ")".to_string(),
                    line: 0, // Will be set by caller
                    col: 0,  // Will be set by caller
                })
            }

            // Symbols (breath markers, octave markers, etc.)
            '\'' | '.' | ':' | '~' | '#' | 'b' => {
                Some(Token {
                    token_type: TokenType::Symbols.as_str().to_string(),
                    value: ch.to_string(),
                    line: 0, // Will be set by caller
                    col: 0,  // Will be set by caller
                })
            }

            // Words/metadata (anything else alphabetic)
            c if c.is_alphabetic() => {
                self.parse_word(c)
            }

            // Unknown
            _ => {
                Some(Token {
                    token_type: TokenType::Unknown.as_str().to_string(),
                    value: ch.to_string(),
                    line: 0, // Will be set by caller
                    col: 0,  // Will be set by caller
                })
            }
        }
    }

    fn parse_pitch(&mut self, _first_char: char) -> Option<Token> {
        let start_pos = self.pos - 1;
        
        // Check for accidentals (sharps/flats) after the pitch
        while let Some(ch) = self.peek() {
            match ch {
                '#' | 'b' => {
                    self.advance();
                }
                _ => break,
            }
        }
        
        Some(Token {
            token_type: TokenType::Pitch.as_str().to_string(),
            value: self.chars[start_pos..self.pos].iter().collect(),
            line: 0, // Will be set by caller
            col: 0,  // Will be set by caller
        })
    }

    fn parse_word(&mut self, _first_char: char) -> Option<Token> {
        // Note: first_char has already been consumed, and pos points to the next character
        // So we need to include the character at pos-1 (which is first_char)
        let start_pos = self.pos - 1;
        
        // Consume alphanumeric characters and some punctuation
        while let Some(ch) = self.peek() {
            match ch {
                c if c.is_alphanumeric() => {
                    self.advance();
                }
                '-' => {
                    self.advance();
                }
                _ => break,
            }
        }
        
        Some(Token {
            token_type: TokenType::Word.as_str().to_string(),
            value: self.chars[start_pos..self.pos].iter().collect(),
            line: 0, // Will be set by caller
            col: 0,  // Will be set by caller
        })
    }

    fn advance(&mut self) -> Option<char> {
        if self.is_at_end() {
            return None;
        }
        
        let ch = self.chars[self.pos];
        self.pos += 1;
        self.col += 1;
        Some(ch)
    }

    fn peek(&self) -> Option<char> {
        if self.is_at_end() {
            None
        } else {
            Some(self.chars[self.pos])
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.chars.len()
    }
    
    fn is_pitch_char(&self, ch: char) -> bool {
        match self.notation_type {
            NotationType::Western => matches!(ch, 'A'..='G'),
            NotationType::Sargam => matches!(ch, 'S' | 's' | 'r' | 'R' | 'g' | 'G' | 'm' | 'M' | 'P' | 'p' | 'd' | 'D' | 'n' | 'N'),
            NotationType::Number => matches!(ch, '1'..='7'),
        }
    }
}

// Simple function to replace the regex-based tokenizer
pub fn tokenize_with_handwritten_lexer(input: &str) -> Vec<Token> {
    // First detect the notation type
    let notation_type = crate::notation_detector::detect_notation_type(input);
    let mut lexer = HandwrittenLexer::new(input, notation_type);
    lexer.tokenize()
}