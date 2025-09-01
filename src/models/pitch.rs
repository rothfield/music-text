use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Notation {
    Western,
    Number,
    Sargam,
    Tabla,
    Bhatkhande,
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
            Notation::Tabla => "Tabla",
            Notation::Bhatkhande => "Bhatkhande",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Degree {
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

pub fn lookup_pitch(symbol: &str, notation: Notation) -> Option<Degree> {
    match notation {
        Notation::Western => match symbol {
            // Natural notes
            "C" => Some(Degree::N1),
            "D" => Some(Degree::N2),
            "E" => Some(Degree::N3),
            "F" => Some(Degree::N4),
            "G" => Some(Degree::N5),
            "A" => Some(Degree::N6),
            "B" => Some(Degree::N7),
            // Sharps
            "C#" => Some(Degree::N1s),
            "D#" => Some(Degree::N2s),
            "E#" => Some(Degree::N3s),
            "F#" => Some(Degree::N4s),
            "G#" => Some(Degree::N5s),
            "A#" => Some(Degree::N6s),
            "B#" => Some(Degree::N7s),
            // Flats
            "Cb" => Some(Degree::N1b),
            "Db" => Some(Degree::N2b),
            "Eb" => Some(Degree::N3b),
            "Fb" => Some(Degree::N4b),
            "Gb" => Some(Degree::N5b),
            "Ab" => Some(Degree::N6b),
            "Bb" => Some(Degree::N7b),
            // Double sharps
            "C##" => Some(Degree::N1ss),
            "D##" => Some(Degree::N2ss),
            "E##" => Some(Degree::N3ss),
            "F##" => Some(Degree::N4ss),
            "G##" => Some(Degree::N5ss),
            "A##" => Some(Degree::N6ss),
            "B##" => Some(Degree::N7ss),
            // Double flats
            "Cbb" => Some(Degree::N1bb),
            "Dbb" => Some(Degree::N2bb),
            "Ebb" => Some(Degree::N3bb),
            "Fbb" => Some(Degree::N4bb),
            "Gbb" => Some(Degree::N5bb),
            "Abb" => Some(Degree::N6bb),
            "Bbb" => Some(Degree::N7bb),
            _ => None,
        },
        Notation::Number => match symbol {
            // Natural numbers
            "1" => Some(Degree::N1),
            "2" => Some(Degree::N2),
            "3" => Some(Degree::N3),
            "4" => Some(Degree::N4),
            "5" => Some(Degree::N5),
            "6" => Some(Degree::N6),
            "7" => Some(Degree::N7),
            // Sharps
            "1#" => Some(Degree::N1s),
            "2#" => Some(Degree::N2s),
            "3#" => Some(Degree::N3s),
            "4#" => Some(Degree::N4s),
            "5#" => Some(Degree::N5s),
            "6#" => Some(Degree::N6s),
            "7#" => Some(Degree::N7s),
            // Flats
            "1b" => Some(Degree::N1b),
            "2b" => Some(Degree::N2b),
            "3b" => Some(Degree::N3b),
            "4b" => Some(Degree::N4b),
            "5b" => Some(Degree::N5b),
            "6b" => Some(Degree::N6b),
            "7b" => Some(Degree::N7b),
            // Double sharps
            "1##" => Some(Degree::N1ss),
            "2##" => Some(Degree::N2ss),
            "3##" => Some(Degree::N3ss),
            "4##" => Some(Degree::N4ss),
            "5##" => Some(Degree::N5ss),
            "6##" => Some(Degree::N6ss),
            "7##" => Some(Degree::N7ss),
            // Double flats
            "1bb" => Some(Degree::N1bb),
            "2bb" => Some(Degree::N2bb),
            "3bb" => Some(Degree::N3bb),
            "4bb" => Some(Degree::N4bb),
            "5bb" => Some(Degree::N5bb),
            "6bb" => Some(Degree::N6bb),
            "7bb" => Some(Degree::N7bb),
            _ => None,
        },
        Notation::Sargam => match symbol {
            // Natural sargam
            "S" | "s" => Some(Degree::N1),    // Sa (both uppercase and lowercase)
            "R" => Some(Degree::N2),    // shuddha Re  
            "G" => Some(Degree::N3),    // shuddha Ga
            "m" => Some(Degree::N4),    // shuddha Ma
            "P" | "p" => Some(Degree::N5),    // Pa (both uppercase and lowercase)
            "D" => Some(Degree::N6),    // shuddha Dha
            "N" => Some(Degree::N7),    // shuddha Ni
            // Komal (flattened) sargam
            "r" => Some(Degree::N2b),   // komal Re
            "g" => Some(Degree::N3b),   // komal Ga  
            "d" => Some(Degree::N6b),   // komal Dha
            "n" => Some(Degree::N7b),   // komal Ni
            // Tivra (sharpened) sargam
            "M" => Some(Degree::N4s),   // tivra Ma
            // Extended sargam with explicit accidentals
            "S#" => Some(Degree::N1s),
            "S##" => Some(Degree::N1ss),
            "Sb" => Some(Degree::N1b),
            "S-" => Some(Degree::N1b),   // Alternative flat notation
            "Sbb" => Some(Degree::N1bb),
            "R#" => Some(Degree::N2s),
            "R##" => Some(Degree::N2ss),
            "R-" => Some(Degree::N2b),   // Alternative flat notation
            "Rbb" => Some(Degree::N2bb),
            "G#" => Some(Degree::N3s),
            "G##" => Some(Degree::N3ss),
            "G-" => Some(Degree::N3b),   // Alternative flat notation
            "Gbb" => Some(Degree::N3bb),
            "mb" => Some(Degree::N4b),
            "mbb" => Some(Degree::N4bb),
            "M#" => Some(Degree::N4ss), // M# is 4##
            "P#" => Some(Degree::N5s),
            "P##" => Some(Degree::N5ss),
            "Pb" => Some(Degree::N5b),
            "Pbb" => Some(Degree::N5bb),
            "D#" => Some(Degree::N6s),
            "D##" => Some(Degree::N6ss),
            "Dbb" => Some(Degree::N6bb),
            "N#" => Some(Degree::N7s),
            "N##" => Some(Degree::N7ss),
            "Nbb" => Some(Degree::N7bb),
            _ => None,
        },
        Notation::Tabla => match symbol {
            // Tabla bols - all map to degree 1 since tabla is percussion (pitch doesn't matter)
            "dha" => Some(Degree::N1),
            "ge" => Some(Degree::N1),
            "na" => Some(Degree::N1),
            "ka" => Some(Degree::N1),
            "ta" => Some(Degree::N1),
            "trka" => Some(Degree::N1),
            "terekita" => Some(Degree::N1),
            "dhin" => Some(Degree::N1),
            _ => None,
        },
        Notation::Bhatkhande => match symbol {
            // Basic Bhatkhande sargam notes
            "स" | "S" => Some(Degree::N1),    // Sa
            "रे" | "R" => Some(Degree::N2),   // Re  
            "ग" | "G" => Some(Degree::N3),    // Ga
            "म" | "M" => Some(Degree::N4),    // Ma
            "प" | "P" => Some(Degree::N5),    // Pa
            "ध" | "D" => Some(Degree::N6),    // Dha
            "नि" | "N" => Some(Degree::N7),   // Ni
            // Sharp accidentals 
            "स#" | "S#" => Some(Degree::N1s),  // Sa sharp
            "रे#" | "R#" => Some(Degree::N2s), // Re sharp  
            "ग#" | "G#" => Some(Degree::N3s),  // Ga sharp
            "म#" | "M#" => Some(Degree::N4s),  // Ma sharp (corresponds to F#)
            "प#" | "P#" => Some(Degree::N5s),  // Pa sharp
            "ध#" | "D#" => Some(Degree::N6s),  // Dha sharp
            "नि#" | "N#" => Some(Degree::N7s), // Ni sharp
            // Flat accidentals
            "सb" | "Sb" => Some(Degree::N1b),  // Sa flat
            "रेb" | "Rb" => Some(Degree::N2b), // Re flat
            "गb" | "Gb" => Some(Degree::N3b),  // Ga flat
            "मb" | "Mb" => Some(Degree::N4b),  // Ma flat
            "पb" | "Pb" => Some(Degree::N5b),  // Pa flat
            "धb" | "Db" => Some(Degree::N6b),  // Dha flat
            "निb" | "Nb" => Some(Degree::N7b), // Ni flat
            _ => None,
        },
    }
}

pub fn pitchcode_to_string(degree: Degree) -> String {
    match degree {
        // 1 series
        Degree::N1bb => "1bb".to_string(),
        Degree::N1b => "1b".to_string(),
        Degree::N1 => "1".to_string(),
        Degree::N1s => "1#".to_string(),
        Degree::N1ss => "1##".to_string(),
        // 2 series
        Degree::N2bb => "2bb".to_string(),
        Degree::N2b => "2b".to_string(),
        Degree::N2 => "2".to_string(),
        Degree::N2s => "2#".to_string(),
        Degree::N2ss => "2##".to_string(),
        // 3 series
        Degree::N3bb => "3bb".to_string(),
        Degree::N3b => "3b".to_string(),
        Degree::N3 => "3".to_string(),
        Degree::N3s => "3#".to_string(),
        Degree::N3ss => "3##".to_string(),
        // 4 series
        Degree::N4bb => "4bb".to_string(),
        Degree::N4b => "4b".to_string(),
        Degree::N4 => "4".to_string(),
        Degree::N4s => "4#".to_string(),
        Degree::N4ss => "4##".to_string(),
        // 5 series
        Degree::N5bb => "5bb".to_string(),
        Degree::N5b => "5b".to_string(),
        Degree::N5 => "5".to_string(),
        Degree::N5s => "5#".to_string(),
        Degree::N5ss => "5##".to_string(),
        // 6 series
        Degree::N6bb => "6bb".to_string(),
        Degree::N6b => "6b".to_string(),
        Degree::N6 => "6".to_string(),
        Degree::N6s => "6#".to_string(),
        Degree::N6ss => "6##".to_string(),
        // 7 series
        Degree::N7bb => "7bb".to_string(),
        Degree::N7b => "7b".to_string(),
        Degree::N7 => "7".to_string(),
        Degree::N7s => "7#".to_string(),
        Degree::N7ss => "7##".to_string(),
    }
}

/// Convert pitch code to Dutch LilyPond note name
/// Dutch LilyPond uses: "es" for flat, "is" for sharp (e.g., des = D-flat, cis = C-sharp)
// DELETED - unused V1 function
/*
pub fn pitchcode_to_dutch_lilypond(degree: Degree) -> String {
    match degree {
        // 1 series (C)
        Degree::N1bb => "ces".to_string(),  // Double flat: ces
        Degree::N1b => "ces".to_string(),   // Single flat: ces (C-flat)
        Degree::N1 => "c".to_string(),
        Degree::N1s => "cis".to_string(),   // Single sharp: cis (C-sharp)
        Degree::N1ss => "cisis".to_string(), // Double sharp: cisis
        
        // 2 series (D)
        Degree::N2bb => "deses".to_string(), // Double flat: deses
        Degree::N2b => "des".to_string(),    // Single flat: des (D-flat)
        Degree::N2 => "d".to_string(),
        Degree::N2s => "dis".to_string(),    // Single sharp: dis (D-sharp)
        Degree::N2ss => "disis".to_string(), // Double sharp: disis
        
        // 3 series (E)
        Degree::N3bb => "eeses".to_string(), // Double flat: eeses
        Degree::N3b => "ees".to_string(),    // Single flat: ees (E-flat)
        Degree::N3 => "e".to_string(),
        Degree::N3s => "eis".to_string(),    // Single sharp: eis (E-sharp)
        Degree::N3ss => "eisis".to_string(), // Double sharp: eisis
        
        // 4 series (F)
        Degree::N4bb => "feses".to_string(), // Double flat: feses
        Degree::N4b => "fes".to_string(),    // Single flat: fes (F-flat)
        Degree::N4 => "f".to_string(),
        Degree::N4s => "fis".to_string(),    // Single sharp: fis (F-sharp)
        Degree::N4ss => "fisis".to_string(), // Double sharp: fisis
        
        // 5 series (G)
        Degree::N5bb => "geses".to_string(), // Double flat: geses
        Degree::N5b => "ges".to_string(),    // Single flat: ges (G-flat)
        Degree::N5 => "g".to_string(),
        Degree::N5s => "gis".to_string(),    // Single sharp: gis (G-sharp)
        Degree::N5ss => "gisis".to_string(), // Double sharp: gisis
        
        // 6 series (A)
        Degree::N6bb => "aeses".to_string(), // Double flat: aeses
        Degree::N6b => "aes".to_string(),    // Single flat: aes (A-flat)
        Degree::N6 => "a".to_string(),
        Degree::N6s => "ais".to_string(),    // Single sharp: ais (A-sharp)
        Degree::N6ss => "aisis".to_string(), // Double sharp: aisis
        
        // 7 series (B)
        Degree::N7bb => "beses".to_string(), // Double flat: beses
        Degree::N7b => "bes".to_string(),    // Single flat: bes (B-flat)
        Degree::N7 => "b".to_string(),
        Degree::N7s => "bis".to_string(),    // Single sharp: bis (B-sharp)
        Degree::N7ss => "bisis".to_string(), // Double sharp: bisis
    }
}

/// Convert pitch code to English LilyPond note name
/// English LilyPond uses: "f" for flat, "s" for sharp (e.g., df = D-flat, cs = C-sharp)
*/

// DELETED - unused V1 function
/*
pub fn pitchcode_to_english_lilypond(degree: Degree) -> String {
    match degree {
        // 1 series (C)
        Degree::N1bb => "cff".to_string(),  // Double flat: cff
        Degree::N1b => "cf".to_string(),    // Single flat: cf (C-flat)
        Degree::N1 => "c".to_string(),
        Degree::N1s => "cs".to_string(),    // Single sharp: cs (C-sharp)
        Degree::N1ss => "css".to_string(),  // Double sharp: css
        
        // 2 series (D)
        Degree::N2bb => "dff".to_string(),  // Double flat: dff
        Degree::N2b => "df".to_string(),    // Single flat: df (D-flat)
        Degree::N2 => "d".to_string(),
        Degree::N2s => "ds".to_string(),    // Single sharp: ds (D-sharp)
        Degree::N2ss => "dss".to_string(),  // Double sharp: dss
        
        // 3 series (E)
        Degree::N3bb => "eff".to_string(),  // Double flat: eff
        Degree::N3b => "ef".to_string(),    // Single flat: ef (E-flat)
        Degree::N3 => "e".to_string(),
        Degree::N3s => "es".to_string(),    // Single sharp: es (E-sharp)
        Degree::N3ss => "ess".to_string(),  // Double sharp: ess
        
        // 4 series (F)
        Degree::N4bb => "fff".to_string(),  // Double flat: fff
        Degree::N4b => "ff".to_string(),    // Single flat: ff (F-flat)
        Degree::N4 => "f".to_string(),
        Degree::N4s => "fs".to_string(),    // Single sharp: fs (F-sharp)
        Degree::N4ss => "fss".to_string(),  // Double sharp: fss
        
        // 5 series (G)
        Degree::N5bb => "gff".to_string(),  // Double flat: gff
        Degree::N5b => "gf".to_string(),    // Single flat: gf (G-flat)
        Degree::N5 => "g".to_string(),
        Degree::N5s => "gs".to_string(),    // Single sharp: gs (G-sharp)
        Degree::N5ss => "gss".to_string(),  // Double sharp: gss
        
        // 6 series (A)
        Degree::N6bb => "aff".to_string(),  // Double flat: aff
        Degree::N6b => "af".to_string(),    // Single flat: af (A-flat)
        Degree::N6 => "a".to_string(),
        Degree::N6s => "as".to_string(),    // Single sharp: as (A-sharp) 
        Degree::N6ss => "ass".to_string(),  // Double sharp: ass
        
        // 7 series (B)
        Degree::N7bb => "bff".to_string(),  // Double flat: bff
        Degree::N7b => "bf".to_string(),    // Single flat: bf (B-flat)
        Degree::N7 => "b".to_string(),
        Degree::N7s => "bs".to_string(),    // Single sharp: bs (B-sharp)
        Degree::N7ss => "bss".to_string(),  // Double sharp: bss
    }
}


*/

// DELETED - unused V1 function
/*
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
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pitch_lookup() {
        assert_eq!(lookup_pitch("S", Notation::Sargam), Some(Degree::N1));
        assert_eq!(lookup_pitch("r", Notation::Sargam), Some(Degree::N2b));
        assert_eq!(lookup_pitch("n", Notation::Sargam), Some(Degree::N7b));
        assert_eq!(lookup_pitch("C", Notation::Western), Some(Degree::N1));
        assert_eq!(lookup_pitch("Bb", Notation::Western), Some(Degree::N7b));
        assert_eq!(lookup_pitch("A#", Notation::Western), Some(Degree::N6s));
        assert_eq!(lookup_pitch("1", Notation::Number), Some(Degree::N1));
        assert_eq!(lookup_pitch("7b", Notation::Number), Some(Degree::N7b));
        assert_eq!(lookup_pitch("X", Notation::Sargam), None);
    }


    // #[test]
    // fn test_guess_notation() - DELETED with guess_notation function

    #[test]
    fn test_lowercase_sargam_pitches() {
        // Test that lowercase s and p map to the same pitch codes as uppercase S and P
        assert_eq!(lookup_pitch("s", Notation::Sargam), Some(Degree::N1)); // Sa
        assert_eq!(lookup_pitch("S", Notation::Sargam), Some(Degree::N1)); // Sa
        assert_eq!(lookup_pitch("p", Notation::Sargam), Some(Degree::N5)); // Pa
        assert_eq!(lookup_pitch("P", Notation::Sargam), Some(Degree::N5)); // Pa
        
        // Verify they are equivalent
        assert_eq!(lookup_pitch("s", Notation::Sargam), lookup_pitch("S", Notation::Sargam));
        assert_eq!(lookup_pitch("p", Notation::Sargam), lookup_pitch("P", Notation::Sargam));
    }

    #[test]
    fn test_tabla_pitch_lookup() {
        // Test that all tabla bols map to degree N1 (since tabla is percussion)
        assert_eq!(lookup_pitch("dha", Notation::Tabla), Some(Degree::N1));
        assert_eq!(lookup_pitch("ge", Notation::Tabla), Some(Degree::N1));
        assert_eq!(lookup_pitch("na", Notation::Tabla), Some(Degree::N1));
        assert_eq!(lookup_pitch("ka", Notation::Tabla), Some(Degree::N1));
        assert_eq!(lookup_pitch("ta", Notation::Tabla), Some(Degree::N1));
        assert_eq!(lookup_pitch("trka", Notation::Tabla), Some(Degree::N1));
        assert_eq!(lookup_pitch("terekita", Notation::Tabla), Some(Degree::N1));
        assert_eq!(lookup_pitch("dhin", Notation::Tabla), Some(Degree::N1));
        
        // Test unknown tabla bol
        assert_eq!(lookup_pitch("unknown", Notation::Tabla), None);
    }

    #[test]
    fn test_bhatkhande_pitch_lookup() {
        // Test basic Bhatkhande sargam notes (Devanagari)
        assert_eq!(lookup_pitch("स", Notation::Bhatkhande), Some(Degree::N1)); // Sa
        assert_eq!(lookup_pitch("रे", Notation::Bhatkhande), Some(Degree::N2)); // Re
        assert_eq!(lookup_pitch("ग", Notation::Bhatkhande), Some(Degree::N3)); // Ga
        assert_eq!(lookup_pitch("म", Notation::Bhatkhande), Some(Degree::N4)); // Ma
        assert_eq!(lookup_pitch("प", Notation::Bhatkhande), Some(Degree::N5)); // Pa
        assert_eq!(lookup_pitch("ध", Notation::Bhatkhande), Some(Degree::N6)); // Dha
        assert_eq!(lookup_pitch("नि", Notation::Bhatkhande), Some(Degree::N7)); // Ni
        
        // Test basic Bhatkhande sargam notes (Roman)
        assert_eq!(lookup_pitch("S", Notation::Bhatkhande), Some(Degree::N1)); // Sa
        assert_eq!(lookup_pitch("R", Notation::Bhatkhande), Some(Degree::N2)); // Re
        assert_eq!(lookup_pitch("G", Notation::Bhatkhande), Some(Degree::N3)); // Ga
        assert_eq!(lookup_pitch("M", Notation::Bhatkhande), Some(Degree::N4)); // Ma
        assert_eq!(lookup_pitch("P", Notation::Bhatkhande), Some(Degree::N5)); // Pa
        assert_eq!(lookup_pitch("D", Notation::Bhatkhande), Some(Degree::N6)); // Dha
        assert_eq!(lookup_pitch("N", Notation::Bhatkhande), Some(Degree::N7)); // Ni

        // Test accidentals
        assert_eq!(lookup_pitch("S#", Notation::Bhatkhande), Some(Degree::N1s));
        assert_eq!(lookup_pitch("स#", Notation::Bhatkhande), Some(Degree::N1s));
        assert_eq!(lookup_pitch("M#", Notation::Bhatkhande), Some(Degree::N4s)); // Ma sharp = F#
        assert_eq!(lookup_pitch("म#", Notation::Bhatkhande), Some(Degree::N4s)); // Ma sharp = F#
        assert_eq!(lookup_pitch("Db", Notation::Bhatkhande), Some(Degree::N6b));
        assert_eq!(lookup_pitch("धb", Notation::Bhatkhande), Some(Degree::N6b));
        
        // Test unknown symbol
        assert_eq!(lookup_pitch("X", Notation::Bhatkhande), None);
    }
}

/// LilyPond note name systems
#[derive(Debug, Clone, Copy)]
pub enum LilyPondNoteNames {
    Dutch,
    English,
}