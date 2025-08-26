use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Notation {
    Western,
    Number,
    Sargam,
}

impl fmt::Display for Notation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Notation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Notation::Western => "Western",
            Notation::Number => "Number", 
            Notation::Sargam => "Sargam",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PitchCode {
    // 1 series (Do/Sa/C)
    N1bb, N1b, N1, N1s, N1ss,
    // 2 series (Re/D)
    N2bb, N2b, N2, N2s, N2ss,
    // 3 series (Mi/Ga/E)
    N3bb, N3b, N3, N3s, N3ss,
    // 4 series (Fa/Ma/F)
    N4bb, N4b, N4, N4s, N4ss,
    // 5 series (Sol/Pa/G)
    N5bb, N5b, N5, N5s, N5ss,
    // 6 series (La/Dha/A)
    N6bb, N6b, N6, N6s, N6ss,
    // 7 series (Ti/Ni/B)
    N7bb, N7b, N7, N7s, N7ss,
}

pub fn _parse_octave_from_symbol(symbol: &str) -> i8 {
    // Count octave markers in Sargam notation
    // . = lower octave (-1 per dot)
    // : = upper octave (+1 per colon)
    let mut octave = 0i8;
    
    for ch in symbol.chars() {
        match ch {
            '.' => octave -= 1,   // dot = lower octave
            ':' => octave += 1,   // colon = upper octave
            _ => {}
        }
    }
    
    octave
}

pub fn _strip_octave_markers(symbol: &str) -> &str {
    // Find the base pitch symbol by removing octave markers
    let mut end = 0;
    for (i, ch) in symbol.char_indices() {
        match ch {
            '.' | ':' => break,
            _ => end = i + ch.len_utf8(),
        }
    }
    &symbol[..end]
}

pub fn lookup_pitch(symbol: &str, notation: Notation) -> Option<PitchCode> {
    match notation {
        Notation::Western => match symbol {
            // Natural notes
            "C" => Some(PitchCode::N1),
            "D" => Some(PitchCode::N2),
            "E" => Some(PitchCode::N3),
            "F" => Some(PitchCode::N4),
            "G" => Some(PitchCode::N5),
            "A" => Some(PitchCode::N6),
            "B" => Some(PitchCode::N7),
            // Sharps
            "C#" => Some(PitchCode::N1s),
            "D#" => Some(PitchCode::N2s),
            "E#" => Some(PitchCode::N3s),
            "F#" => Some(PitchCode::N4s),
            "G#" => Some(PitchCode::N5s),
            "A#" => Some(PitchCode::N6s),
            "B#" => Some(PitchCode::N7s),
            // Flats
            "Cb" => Some(PitchCode::N1b),
            "Db" => Some(PitchCode::N2b),
            "Eb" => Some(PitchCode::N3b),
            "Fb" => Some(PitchCode::N4b),
            "Gb" => Some(PitchCode::N5b),
            "Ab" => Some(PitchCode::N6b),
            "Bb" => Some(PitchCode::N7b),
            // Double sharps
            "C##" => Some(PitchCode::N1ss),
            "D##" => Some(PitchCode::N2ss),
            "E##" => Some(PitchCode::N3ss),
            "F##" => Some(PitchCode::N4ss),
            "G##" => Some(PitchCode::N5ss),
            "A##" => Some(PitchCode::N6ss),
            "B##" => Some(PitchCode::N7ss),
            // Double flats
            "Cbb" => Some(PitchCode::N1bb),
            "Dbb" => Some(PitchCode::N2bb),
            "Ebb" => Some(PitchCode::N3bb),
            "Fbb" => Some(PitchCode::N4bb),
            "Gbb" => Some(PitchCode::N5bb),
            "Abb" => Some(PitchCode::N6bb),
            "Bbb" => Some(PitchCode::N7bb),
            _ => None,
        },
        Notation::Number => match symbol {
            // Natural numbers
            "1" => Some(PitchCode::N1),
            "2" => Some(PitchCode::N2),
            "3" => Some(PitchCode::N3),
            "4" => Some(PitchCode::N4),
            "5" => Some(PitchCode::N5),
            "6" => Some(PitchCode::N6),
            "7" => Some(PitchCode::N7),
            // Sharps
            "1#" => Some(PitchCode::N1s),
            "2#" => Some(PitchCode::N2s),
            "3#" => Some(PitchCode::N3s),
            "4#" => Some(PitchCode::N4s),
            "5#" => Some(PitchCode::N5s),
            "6#" => Some(PitchCode::N6s),
            "7#" => Some(PitchCode::N7s),
            // Flats
            "1b" => Some(PitchCode::N1b),
            "2b" => Some(PitchCode::N2b),
            "3b" => Some(PitchCode::N3b),
            "4b" => Some(PitchCode::N4b),
            "5b" => Some(PitchCode::N5b),
            "6b" => Some(PitchCode::N6b),
            "7b" => Some(PitchCode::N7b),
            // Double sharps
            "1##" => Some(PitchCode::N1ss),
            "2##" => Some(PitchCode::N2ss),
            "3##" => Some(PitchCode::N3ss),
            "4##" => Some(PitchCode::N4ss),
            "5##" => Some(PitchCode::N5ss),
            "6##" => Some(PitchCode::N6ss),
            "7##" => Some(PitchCode::N7ss),
            // Double flats
            "1bb" => Some(PitchCode::N1bb),
            "2bb" => Some(PitchCode::N2bb),
            "3bb" => Some(PitchCode::N3bb),
            "4bb" => Some(PitchCode::N4bb),
            "5bb" => Some(PitchCode::N5bb),
            "6bb" => Some(PitchCode::N6bb),
            "7bb" => Some(PitchCode::N7bb),
            _ => None,
        },
        Notation::Sargam => match symbol {
            // Natural sargam
            "S" | "s" => Some(PitchCode::N1),    // Sa (both uppercase and lowercase)
            "R" => Some(PitchCode::N2),    // shuddha Re  
            "G" => Some(PitchCode::N3),    // shuddha Ga
            "m" => Some(PitchCode::N4),    // shuddha Ma
            "P" | "p" => Some(PitchCode::N5),    // Pa (both uppercase and lowercase)
            "D" => Some(PitchCode::N6),    // shuddha Dha
            "N" => Some(PitchCode::N7),    // shuddha Ni
            // Komal (flattened) sargam
            "r" => Some(PitchCode::N2b),   // komal Re
            "g" => Some(PitchCode::N3b),   // komal Ga  
            "d" => Some(PitchCode::N6b),   // komal Dha
            "n" => Some(PitchCode::N7b),   // komal Ni
            // Tivra (sharpened) sargam
            "M" => Some(PitchCode::N4s),   // tivra Ma
            // Extended sargam with explicit accidentals
            "S#" => Some(PitchCode::N1s),
            "S##" => Some(PitchCode::N1ss),
            "Sb" => Some(PitchCode::N1b),
            "S-" => Some(PitchCode::N1b),   // Alternative flat notation
            "Sbb" => Some(PitchCode::N1bb),
            "R#" => Some(PitchCode::N2s),
            "R##" => Some(PitchCode::N2ss),
            "R-" => Some(PitchCode::N2b),   // Alternative flat notation
            "Rbb" => Some(PitchCode::N2bb),
            "G#" => Some(PitchCode::N3s),
            "G##" => Some(PitchCode::N3ss),
            "G-" => Some(PitchCode::N3b),   // Alternative flat notation
            "Gbb" => Some(PitchCode::N3bb),
            "mb" => Some(PitchCode::N4b),
            "mbb" => Some(PitchCode::N4bb),
            "M#" => Some(PitchCode::N4ss), // M# is 4##
            "P#" => Some(PitchCode::N5s),
            "P##" => Some(PitchCode::N5ss),
            "Pb" => Some(PitchCode::N5b),
            "Pbb" => Some(PitchCode::N5bb),
            "D#" => Some(PitchCode::N6s),
            "D##" => Some(PitchCode::N6ss),
            "Dbb" => Some(PitchCode::N6bb),
            "N#" => Some(PitchCode::N7s),
            "N##" => Some(PitchCode::N7ss),
            "Nbb" => Some(PitchCode::N7bb),
            _ => None,
        },
    }
}

pub fn pitchcode_to_string(pitch_code: PitchCode) -> String {
    match pitch_code {
        // 1 series
        PitchCode::N1bb => "1bb".to_string(),
        PitchCode::N1b => "1b".to_string(),
        PitchCode::N1 => "1".to_string(),
        PitchCode::N1s => "1#".to_string(),
        PitchCode::N1ss => "1##".to_string(),
        // 2 series
        PitchCode::N2bb => "2bb".to_string(),
        PitchCode::N2b => "2b".to_string(),
        PitchCode::N2 => "2".to_string(),
        PitchCode::N2s => "2#".to_string(),
        PitchCode::N2ss => "2##".to_string(),
        // 3 series
        PitchCode::N3bb => "3bb".to_string(),
        PitchCode::N3b => "3b".to_string(),
        PitchCode::N3 => "3".to_string(),
        PitchCode::N3s => "3#".to_string(),
        PitchCode::N3ss => "3##".to_string(),
        // 4 series
        PitchCode::N4bb => "4bb".to_string(),
        PitchCode::N4b => "4b".to_string(),
        PitchCode::N4 => "4".to_string(),
        PitchCode::N4s => "4#".to_string(),
        PitchCode::N4ss => "4##".to_string(),
        // 5 series
        PitchCode::N5bb => "5bb".to_string(),
        PitchCode::N5b => "5b".to_string(),
        PitchCode::N5 => "5".to_string(),
        PitchCode::N5s => "5#".to_string(),
        PitchCode::N5ss => "5##".to_string(),
        // 6 series
        PitchCode::N6bb => "6bb".to_string(),
        PitchCode::N6b => "6b".to_string(),
        PitchCode::N6 => "6".to_string(),
        PitchCode::N6s => "6#".to_string(),
        PitchCode::N6ss => "6##".to_string(),
        // 7 series
        PitchCode::N7bb => "7bb".to_string(),
        PitchCode::N7b => "7b".to_string(),
        PitchCode::N7 => "7".to_string(),
        PitchCode::N7s => "7#".to_string(),
        PitchCode::N7ss => "7##".to_string(),
    }
}

/// Convert pitch code to Dutch LilyPond note name
/// Dutch LilyPond uses: "es" for flat, "is" for sharp (e.g., des = D-flat, cis = C-sharp)
pub fn pitchcode_to_dutch_lilypond(pitch_code: PitchCode) -> String {
    match pitch_code {
        // 1 series (C)
        PitchCode::N1bb => "ces".to_string(),  // Double flat: ces
        PitchCode::N1b => "ces".to_string(),   // Single flat: ces (C-flat)
        PitchCode::N1 => "c".to_string(),
        PitchCode::N1s => "cis".to_string(),   // Single sharp: cis (C-sharp)
        PitchCode::N1ss => "cisis".to_string(), // Double sharp: cisis
        
        // 2 series (D)
        PitchCode::N2bb => "deses".to_string(), // Double flat: deses
        PitchCode::N2b => "des".to_string(),    // Single flat: des (D-flat)
        PitchCode::N2 => "d".to_string(),
        PitchCode::N2s => "dis".to_string(),    // Single sharp: dis (D-sharp)
        PitchCode::N2ss => "disis".to_string(), // Double sharp: disis
        
        // 3 series (E)
        PitchCode::N3bb => "eeses".to_string(), // Double flat: eeses
        PitchCode::N3b => "ees".to_string(),    // Single flat: ees (E-flat)
        PitchCode::N3 => "e".to_string(),
        PitchCode::N3s => "eis".to_string(),    // Single sharp: eis (E-sharp)
        PitchCode::N3ss => "eisis".to_string(), // Double sharp: eisis
        
        // 4 series (F)
        PitchCode::N4bb => "feses".to_string(), // Double flat: feses
        PitchCode::N4b => "fes".to_string(),    // Single flat: fes (F-flat)
        PitchCode::N4 => "f".to_string(),
        PitchCode::N4s => "fis".to_string(),    // Single sharp: fis (F-sharp)
        PitchCode::N4ss => "fisis".to_string(), // Double sharp: fisis
        
        // 5 series (G)
        PitchCode::N5bb => "geses".to_string(), // Double flat: geses
        PitchCode::N5b => "ges".to_string(),    // Single flat: ges (G-flat)
        PitchCode::N5 => "g".to_string(),
        PitchCode::N5s => "gis".to_string(),    // Single sharp: gis (G-sharp)
        PitchCode::N5ss => "gisis".to_string(), // Double sharp: gisis
        
        // 6 series (A)
        PitchCode::N6bb => "aeses".to_string(), // Double flat: aeses
        PitchCode::N6b => "aes".to_string(),    // Single flat: aes (A-flat)
        PitchCode::N6 => "a".to_string(),
        PitchCode::N6s => "ais".to_string(),    // Single sharp: ais (A-sharp)
        PitchCode::N6ss => "aisis".to_string(), // Double sharp: aisis
        
        // 7 series (B)
        PitchCode::N7bb => "beses".to_string(), // Double flat: beses
        PitchCode::N7b => "bes".to_string(),    // Single flat: bes (B-flat)
        PitchCode::N7 => "b".to_string(),
        PitchCode::N7s => "bis".to_string(),    // Single sharp: bis (B-sharp)
        PitchCode::N7ss => "bisis".to_string(), // Double sharp: bisis
    }
}

/// Convert pitch code to English LilyPond note name
/// English LilyPond uses: "f" for flat, "s" for sharp (e.g., df = D-flat, cs = C-sharp)
pub fn pitchcode_to_english_lilypond(pitch_code: PitchCode) -> String {
    match pitch_code {
        // 1 series (C)
        PitchCode::N1bb => "cff".to_string(),  // Double flat: cff
        PitchCode::N1b => "cf".to_string(),    // Single flat: cf (C-flat)
        PitchCode::N1 => "c".to_string(),
        PitchCode::N1s => "cs".to_string(),    // Single sharp: cs (C-sharp)
        PitchCode::N1ss => "css".to_string(),  // Double sharp: css
        
        // 2 series (D)
        PitchCode::N2bb => "dff".to_string(),  // Double flat: dff
        PitchCode::N2b => "df".to_string(),    // Single flat: df (D-flat)
        PitchCode::N2 => "d".to_string(),
        PitchCode::N2s => "ds".to_string(),    // Single sharp: ds (D-sharp)
        PitchCode::N2ss => "dss".to_string(),  // Double sharp: dss
        
        // 3 series (E)
        PitchCode::N3bb => "eff".to_string(),  // Double flat: eff
        PitchCode::N3b => "ef".to_string(),    // Single flat: ef (E-flat)
        PitchCode::N3 => "e".to_string(),
        PitchCode::N3s => "es".to_string(),    // Single sharp: es (E-sharp)
        PitchCode::N3ss => "ess".to_string(),  // Double sharp: ess
        
        // 4 series (F)
        PitchCode::N4bb => "fff".to_string(),  // Double flat: fff
        PitchCode::N4b => "ff".to_string(),    // Single flat: ff (F-flat)
        PitchCode::N4 => "f".to_string(),
        PitchCode::N4s => "fs".to_string(),    // Single sharp: fs (F-sharp)
        PitchCode::N4ss => "fss".to_string(),  // Double sharp: fss
        
        // 5 series (G)
        PitchCode::N5bb => "gff".to_string(),  // Double flat: gff
        PitchCode::N5b => "gf".to_string(),    // Single flat: gf (G-flat)
        PitchCode::N5 => "g".to_string(),
        PitchCode::N5s => "gs".to_string(),    // Single sharp: gs (G-sharp)
        PitchCode::N5ss => "gss".to_string(),  // Double sharp: gss
        
        // 6 series (A)
        PitchCode::N6bb => "aff".to_string(),  // Double flat: aff
        PitchCode::N6b => "af".to_string(),    // Single flat: af (A-flat)
        PitchCode::N6 => "a".to_string(),
        PitchCode::N6s => "as".to_string(),    // Single sharp: as (A-sharp) 
        PitchCode::N6ss => "ass".to_string(),  // Double sharp: ass
        
        // 7 series (B)
        PitchCode::N7bb => "bff".to_string(),  // Double flat: bff
        PitchCode::N7b => "bf".to_string(),    // Single flat: bf (B-flat)
        PitchCode::N7 => "b".to_string(),
        PitchCode::N7s => "bs".to_string(),    // Single sharp: bs (B-sharp)
        PitchCode::N7ss => "bss".to_string(),  // Double sharp: bss
    }
}

/// Convert pitch code to Western note name
pub fn pitchcode_to_western(pitch_code: PitchCode) -> String {
    match pitch_code {
        // 1 series (C)
        PitchCode::N1bb => "Cbb".to_string(),
        PitchCode::N1b => "Cb".to_string(),
        PitchCode::N1 => "C".to_string(),
        PitchCode::N1s => "C#".to_string(),
        PitchCode::N1ss => "C##".to_string(),
        
        // 2 series (D)
        PitchCode::N2bb => "Dbb".to_string(),
        PitchCode::N2b => "Db".to_string(),
        PitchCode::N2 => "D".to_string(),
        PitchCode::N2s => "D#".to_string(),
        PitchCode::N2ss => "D##".to_string(),
        
        // 3 series (E)
        PitchCode::N3bb => "Ebb".to_string(),
        PitchCode::N3b => "Eb".to_string(),
        PitchCode::N3 => "E".to_string(),
        PitchCode::N3s => "E#".to_string(),
        PitchCode::N3ss => "E##".to_string(),
        
        // 4 series (F)
        PitchCode::N4bb => "Fbb".to_string(),
        PitchCode::N4b => "Fb".to_string(),
        PitchCode::N4 => "F".to_string(),
        PitchCode::N4s => "F#".to_string(),
        PitchCode::N4ss => "F##".to_string(),
        
        // 5 series (G)
        PitchCode::N5bb => "Gbb".to_string(),
        PitchCode::N5b => "Gb".to_string(),
        PitchCode::N5 => "G".to_string(),
        PitchCode::N5s => "G#".to_string(),
        PitchCode::N5ss => "G##".to_string(),
        
        // 6 series (A)
        PitchCode::N6bb => "Abb".to_string(),
        PitchCode::N6b => "Ab".to_string(),
        PitchCode::N6 => "A".to_string(),
        PitchCode::N6s => "A#".to_string(),
        PitchCode::N6ss => "A##".to_string(),
        
        // 7 series (B)
        PitchCode::N7bb => "Bbb".to_string(),
        PitchCode::N7b => "Bb".to_string(),
        PitchCode::N7 => "B".to_string(),
        PitchCode::N7s => "B#".to_string(),
        PitchCode::N7ss => "B##".to_string(),
    }
}

/// Convert pitch code to Sargam notation
pub fn pitchcode_to_sargam(pitch_code: PitchCode) -> String {
    match pitch_code {
        // 1 series (Sa)
        PitchCode::N1bb => "Sbb".to_string(),
        PitchCode::N1b => "Sb".to_string(),
        PitchCode::N1 => "S".to_string(),
        PitchCode::N1s => "S#".to_string(),
        PitchCode::N1ss => "S##".to_string(),
        
        // 2 series (Re)
        PitchCode::N2bb => "Rbb".to_string(),
        PitchCode::N2b => "r".to_string(),  // komal Re
        PitchCode::N2 => "R".to_string(),
        PitchCode::N2s => "R#".to_string(),
        PitchCode::N2ss => "R##".to_string(),
        
        // 3 series (Ga)
        PitchCode::N3bb => "Gbb".to_string(),
        PitchCode::N3b => "g".to_string(),  // komal Ga
        PitchCode::N3 => "G".to_string(),
        PitchCode::N3s => "G#".to_string(),
        PitchCode::N3ss => "G##".to_string(),
        
        // 4 series (Ma)
        PitchCode::N4bb => "mbb".to_string(),
        PitchCode::N4b => "mb".to_string(),
        PitchCode::N4 => "m".to_string(),   // shuddha Ma
        PitchCode::N4s => "M".to_string(),  // tivra Ma
        PitchCode::N4ss => "M#".to_string(),
        
        // 5 series (Pa)
        PitchCode::N5bb => "Pbb".to_string(),
        PitchCode::N5b => "Pb".to_string(),
        PitchCode::N5 => "P".to_string(),
        PitchCode::N5s => "P#".to_string(),
        PitchCode::N5ss => "P##".to_string(),
        
        // 6 series (Dha)
        PitchCode::N6bb => "Dbb".to_string(),
        PitchCode::N6b => "d".to_string(),  // komal Dha
        PitchCode::N6 => "D".to_string(),
        PitchCode::N6s => "D#".to_string(),
        PitchCode::N6ss => "D##".to_string(),
        
        // 7 series (Ni)
        PitchCode::N7bb => "Nbb".to_string(),
        PitchCode::N7b => "n".to_string(),  // komal Ni
        PitchCode::N7 => "N".to_string(),
        PitchCode::N7s => "N#".to_string(),
        PitchCode::N7ss => "N##".to_string(),
    }
}

pub fn guess_notation(symbols: &[&str]) -> Notation {
    let mut western_score = 0;
    let mut number_score = 0;
    let mut sargam_score = 0;
    
    for symbol in symbols {
        if lookup_pitch(symbol, Notation::Western).is_some() {
            western_score += 1;
        }
        if lookup_pitch(symbol, Notation::Number).is_some() {
            number_score += 1;
        }
        if lookup_pitch(symbol, Notation::Sargam).is_some() {
            sargam_score += 1;
        }
    }
    
    // Return the notation with the highest score
    if western_score >= number_score && western_score >= sargam_score {
        Notation::Western
    } else if number_score >= sargam_score {
        Notation::Number
    } else {
        Notation::Sargam
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pitch_lookup() {
        assert_eq!(lookup_pitch("S", Notation::Sargam), Some(PitchCode::N1));
        assert_eq!(lookup_pitch("r", Notation::Sargam), Some(PitchCode::N2b));
        assert_eq!(lookup_pitch("n", Notation::Sargam), Some(PitchCode::N7b));
        assert_eq!(lookup_pitch("C", Notation::Western), Some(PitchCode::N1));
        assert_eq!(lookup_pitch("Bb", Notation::Western), Some(PitchCode::N7b));
        assert_eq!(lookup_pitch("A#", Notation::Western), Some(PitchCode::N6s));
        assert_eq!(lookup_pitch("1", Notation::Number), Some(PitchCode::N1));
        assert_eq!(lookup_pitch("7b", Notation::Number), Some(PitchCode::N7b));
        assert_eq!(lookup_pitch("X", Notation::Sargam), None);
    }


    #[test]
    fn test_guess_notation() {
        assert_eq!(guess_notation(&["S", "R", "G"]), Notation::Sargam);
        assert_eq!(guess_notation(&["C", "D", "E"]), Notation::Western);
        assert_eq!(guess_notation(&["1", "2", "3"]), Notation::Number);
    }

    #[test]
    fn test_lowercase_sargam_pitches() {
        // Test that lowercase s and p map to the same pitch codes as uppercase S and P
        assert_eq!(lookup_pitch("s", Notation::Sargam), Some(PitchCode::N1)); // Sa
        assert_eq!(lookup_pitch("S", Notation::Sargam), Some(PitchCode::N1)); // Sa
        assert_eq!(lookup_pitch("p", Notation::Sargam), Some(PitchCode::N5)); // Pa
        assert_eq!(lookup_pitch("P", Notation::Sargam), Some(PitchCode::N5)); // Pa
        
        // Verify they are equivalent
        assert_eq!(lookup_pitch("s", Notation::Sargam), lookup_pitch("S", Notation::Sargam));
        assert_eq!(lookup_pitch("p", Notation::Sargam), lookup_pitch("P", Notation::Sargam));
    }
}

/// LilyPond note name systems
#[derive(Debug, Clone, Copy)]
pub enum LilyPondNoteNames {
    Dutch,
    English,
}