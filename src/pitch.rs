#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Notation {
    Western,
    Number,
    Sargam,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PitchCode {
    C,
    Db,
    D,
    Eb,
    E,
    F,
    Fs,
    G,
    Ab,
    A,
    Bb,
    B,
}

pub fn parse_octave_from_symbol(symbol: &str) -> i8 {
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

pub fn strip_octave_markers(symbol: &str) -> &str {
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
            "C" => Some(PitchCode::C),
            "D" => Some(PitchCode::D),
            "E" => Some(PitchCode::E),
            "F" => Some(PitchCode::F),
            "G" => Some(PitchCode::G),
            "A" => Some(PitchCode::A),
            "B" => Some(PitchCode::B),
            // Sharps
            "C#" => Some(PitchCode::Db),
            "D#" => Some(PitchCode::Eb),
            "F#" => Some(PitchCode::Fs),
            "G#" => Some(PitchCode::Ab),
            "A#" => Some(PitchCode::Bb),
            // Flats  
            "Db" => Some(PitchCode::Db),
            "Eb" => Some(PitchCode::Eb),
            "Gb" => Some(PitchCode::Fs),
            "Ab" => Some(PitchCode::Ab),
            "Bb" => Some(PitchCode::Bb),
            _ => None,
        },
        Notation::Number => match symbol {
            "1" => Some(PitchCode::C),      // Do
            "1#" => Some(PitchCode::Db),    // Do#
            "2b" => Some(PitchCode::Db),    // Re♭
            "2" => Some(PitchCode::D),      // Re
            "2#" => Some(PitchCode::Eb),    // Re#
            "3b" => Some(PitchCode::Eb),    // Mi♭
            "3" => Some(PitchCode::E),      // Mi
            "4" => Some(PitchCode::F),      // Fa
            "4#" => Some(PitchCode::Fs),    // Fa#
            "5b" => Some(PitchCode::Fs),    // Sol♭
            "5" => Some(PitchCode::G),      // Sol
            "5#" => Some(PitchCode::Ab),    // Sol#
            "6b" => Some(PitchCode::Ab),    // La♭
            "6" => Some(PitchCode::A),      // La
            "6#" => Some(PitchCode::Bb),    // La#
            "7b" => Some(PitchCode::Bb),    // Ti♭
            "7" => Some(PitchCode::B),      // Ti
            _ => None,
        },
        Notation::Sargam => match symbol {
            "S" => Some(PitchCode::C),   // Sa = C
            "s" => Some(PitchCode::C),   // lower Sa = C (same as S)
            "r" => Some(PitchCode::Db),  // komal Re = Db
            "R" => Some(PitchCode::D),   // shuddha Re = D
            "g" => Some(PitchCode::Eb),  // komal Ga = Eb
            "G" => Some(PitchCode::E),   // shuddha Ga = E
            "m" => Some(PitchCode::F),   // shuddha Ma = F
            "M" => Some(PitchCode::Fs),  // tivra Ma = F#
            "P" => Some(PitchCode::G),   // Pa = G
            "p" => Some(PitchCode::G),   // Pa = G (same as P)
            "P#" => Some(PitchCode::Ab), // P# = G# (using Ab for enharmonic)
            "d" => Some(PitchCode::Ab),  // komal Dha = Ab
            "D" => Some(PitchCode::A),   // shuddha Dha = A
            "n" => Some(PitchCode::Bb),  // komal Ni = Bb
            "N" => Some(PitchCode::B),   // shuddha Ni = B
            _ => None,
        },
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
        assert_eq!(lookup_pitch("S", Notation::Sargam), Some(PitchCode::C));
        assert_eq!(lookup_pitch("r", Notation::Sargam), Some(PitchCode::Db));
        assert_eq!(lookup_pitch("C", Notation::Western), Some(PitchCode::C));
        assert_eq!(lookup_pitch("1", Notation::Number), Some(PitchCode::C));
        assert_eq!(lookup_pitch("X", Notation::Sargam), None);
    }


    #[test]
    fn test_guess_notation() {
        assert_eq!(guess_notation(&["S", "R", "G"]), Notation::Sargam);
        assert_eq!(guess_notation(&["C", "D", "E"]), Notation::Western);
        assert_eq!(guess_notation(&["1", "2", "3"]), Notation::Number);
    }
}