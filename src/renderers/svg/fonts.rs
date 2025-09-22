use std::collections::HashMap;

/// SMuFL symbol types
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum SymbolType {
    BarlineSingle,
    BarlineDouble,
    RepeatStart,
    RepeatEnd,
    Sharp,
    Flat,
    Mordent,
    Trill,
}

/// SMuFL font mapper with fallback support
pub struct SMuFLMapper {
    symbols: HashMap<SymbolType, char>,
    fallbacks: HashMap<SymbolType, &'static str>,
}

impl SMuFLMapper {
    pub fn new() -> Self {
        let mut symbols = HashMap::new();
        let mut fallbacks = HashMap::new();

        // Barlines (SMuFL E030-E04F)
        symbols.insert(SymbolType::BarlineSingle, '\u{E030}');
        fallbacks.insert(SymbolType::BarlineSingle, "|");

        symbols.insert(SymbolType::BarlineDouble, '\u{E031}');
        fallbacks.insert(SymbolType::BarlineDouble, "‖");

        symbols.insert(SymbolType::RepeatStart, '\u{E040}');
        fallbacks.insert(SymbolType::RepeatStart, "|:");

        symbols.insert(SymbolType::RepeatEnd, '\u{E041}');
        fallbacks.insert(SymbolType::RepeatEnd, ":|");

        // Accidentals (SMuFL E260-E2AF)
        symbols.insert(SymbolType::Sharp, '\u{E262}');
        fallbacks.insert(SymbolType::Sharp, "♯");

        symbols.insert(SymbolType::Flat, '\u{E260}');
        fallbacks.insert(SymbolType::Flat, "♭");

        // Ornaments (SMuFL E560-E5FF)
        symbols.insert(SymbolType::Mordent, '\u{E56C}');
        fallbacks.insert(SymbolType::Mordent, "∿");

        symbols.insert(SymbolType::Trill, '\u{E566}');
        fallbacks.insert(SymbolType::Trill, "tr");

        Self { symbols, fallbacks }
    }

    pub fn get_symbol(&self, symbol_type: SymbolType, use_smufl: bool) -> String {
        if use_smufl {
            if let Some(symbol) = self.symbols.get(&symbol_type) {
                return symbol.to_string();
            }
        }
        self.fallbacks
            .get(&symbol_type)
            .unwrap_or(&"?")
            .to_string()
    }

    pub fn get_symbol_class(&self, use_smufl: bool) -> &'static str {
        if use_smufl {
            "smufl-symbol"
        } else {
            "unicode-symbol"
        }
    }
}

impl Default for SMuFLMapper {
    fn default() -> Self {
        Self::new()
    }
}

/// Font strategy configuration
#[derive(Debug, Clone)]
pub struct FontStrategy {
    pub primary_music_font: &'static str,
    pub fallback_music_font: &'static str,
    pub text_font: &'static str,
    pub use_smufl: bool,
}

impl Default for FontStrategy {
    fn default() -> Self {
        Self {
            primary_music_font: "Bravura",      // Professional, open source
            fallback_music_font: "Leland",       // MuseScore font, more casual
            text_font: "Georgia",                 // For lyrics, labels
            use_smufl: true,                     // Enable SMuFL by default
        }
    }
}

impl FontStrategy {
    /// Generate font-face CSS declarations for SMuFL fonts
    pub fn generate_font_face_css(&self) -> String {
        format!(
            r#"
@font-face {{
  font-family: '{}';
  src: url('{}.woff2') format('woff2'),
       url('{}.woff') format('woff');
  font-display: swap;
  unicode-range: U+E000-E8FF; /* SMuFL Private Use Area */
}}

@font-face {{
  font-family: '{}';
  src: url('{}.woff2') format('woff2');
  font-display: swap;
  unicode-range: U+E000-E8FF;
}}
"#,
            self.primary_music_font,
            self.primary_music_font.to_lowercase(),
            self.primary_music_font.to_lowercase(),
            self.fallback_music_font,
            self.fallback_music_font.to_lowercase()
        )
    }

    /// Get the font family string for music symbols
    pub fn get_music_font_family(&self) -> String {
        if self.use_smufl {
            format!(
                "'{}', '{}', serif",
                self.primary_music_font, self.fallback_music_font
            )
        } else {
            "serif".to_string()
        }
    }
}