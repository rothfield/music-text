/// PitchCode to LilyPond note name conversion
use crate::parse::model::PitchCode;
use crate::renderers::transposition::transpose_pitchcode_with_octave;

/// Convert PitchCode and octave to LilyPond note name
/// Integrates with tonic-based transposition system
pub fn pitchcode_to_lilypond(pitchcode: PitchCode, octave: i8, tonic: Option<PitchCode>) -> Result<String, String> {
    // Apply transposition if tonic is specified
    let (transposed_pitchcode, adjusted_octave) = if let Some(tonic) = tonic {
        transpose_pitchcode_with_octave(pitchcode, octave, tonic)
    } else {
        (pitchcode, octave)
    };
    
    // Convert transposed PitchCode to LilyPond note name - handle all 35 variants
    let base_note = match transposed_pitchcode {
        // 1 series (Do/Sa/C)
        PitchCode::N1bb => "cff",   PitchCode::N1b => "cf",     PitchCode::N1 => "c",
        PitchCode::N1s => "cs",     PitchCode::N1ss => "css",
        // 2 series (Re/D)  
        PitchCode::N2bb => "dff",   PitchCode::N2b => "df",     PitchCode::N2 => "d",
        PitchCode::N2s => "ds",     PitchCode::N2ss => "dss",
        // 3 series (Mi/Ga/E)
        PitchCode::N3bb => "eff",   PitchCode::N3b => "ef",     PitchCode::N3 => "e",
        PitchCode::N3s => "es",     PitchCode::N3ss => "ess",
        // 4 series (Fa/Ma/F)  
        PitchCode::N4bb => "fff",   PitchCode::N4b => "ff",     PitchCode::N4 => "f",
        PitchCode::N4s => "fs",     PitchCode::N4ss => "fss",
        // 5 series (Sol/Pa/G)
        PitchCode::N5bb => "gff",   PitchCode::N5b => "gf",     PitchCode::N5 => "g",
        PitchCode::N5s => "gs",     PitchCode::N5ss => "gss",
        // 6 series (La/Dha/A)
        PitchCode::N6bb => "aff",   PitchCode::N6b => "af",     PitchCode::N6 => "a",
        PitchCode::N6s => "as",     PitchCode::N6ss => "ass",
        // 7 series (Ti/Ni/B)
        PitchCode::N7bb => "bff",   PitchCode::N7b => "bf",     PitchCode::N7 => "b",
        PitchCode::N7s => "bs",     PitchCode::N7ss => "bss",
    };
    
    // Handle octave modifications (use adjusted_octave from transposition)
    let octave_marks = match adjusted_octave {
        -3 => ",,,",
        -2 => ",,",
        -1 => ",",
        0 => "",        // Middle octave (octave 4 in absolute pitch)
        1 => "'",
        2 => "''",
        3 => "'''",
        _ => "",        // Default to middle for extreme octaves
    };
    
    Ok(format!("{}{}", base_note, octave_marks))
}

/// Convert PitchCode without octave information to LilyPond note name (for simple cases)
pub fn pitchcode_to_lilypond_simple(pitchcode: PitchCode) -> &'static str {
    match pitchcode {
        PitchCode::N1bb => "cff", PitchCode::N1b => "cf", PitchCode::N1 => "c",
        PitchCode::N1s => "cs", PitchCode::N1ss => "css",
        PitchCode::N2bb => "dff", PitchCode::N2b => "df", PitchCode::N2 => "d", 
        PitchCode::N2s => "ds", PitchCode::N2ss => "dss",
        PitchCode::N3bb => "eff", PitchCode::N3b => "ef", PitchCode::N3 => "e",
        PitchCode::N3s => "es", PitchCode::N3ss => "ess",
        PitchCode::N4bb => "fff", PitchCode::N4b => "ff", PitchCode::N4 => "f",
        PitchCode::N4s => "fs", PitchCode::N4ss => "fss",
        PitchCode::N5bb => "gff", PitchCode::N5b => "gf", PitchCode::N5 => "g",
        PitchCode::N5s => "gs", PitchCode::N5ss => "gss",
        PitchCode::N6bb => "aff", PitchCode::N6b => "af", PitchCode::N6 => "a",
        PitchCode::N6s => "as", PitchCode::N6ss => "ass",
        PitchCode::N7bb => "bff", PitchCode::N7b => "bf", PitchCode::N7 => "b",
        PitchCode::N7s => "bs", PitchCode::N7ss => "bss",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pitchcode_to_lilypond_basic() {
        // Test basic conversion without transposition
        assert_eq!(pitchcode_to_lilypond(PitchCode::N1, 0, None).unwrap(), "c");
        assert_eq!(pitchcode_to_lilypond(PitchCode::N1s, 0, None).unwrap(), "cs");
        assert_eq!(pitchcode_to_lilypond(PitchCode::N1ss, 1, None).unwrap(), "css'");
        assert_eq!(pitchcode_to_lilypond(PitchCode::N7b, -1, None).unwrap(), "bf,");
    }
    
    #[test]
    fn test_pitchcode_to_lilypond_with_transposition() {
        // Test transposition: scale degree 1 in D major should be D
        let result = pitchcode_to_lilypond(PitchCode::N1, 0, Some(PitchCode::N2)).unwrap();
        assert_eq!(result, "d");
        
        // Test transposition: scale degree 7 in D major should be C# in upper octave
        let result = pitchcode_to_lilypond(PitchCode::N7, 0, Some(PitchCode::N2)).unwrap();
        assert_eq!(result, "cs'");
    }
    
    #[test]
    fn test_double_modifiers() {
        // Test double sharps and flats
        assert_eq!(pitchcode_to_lilypond_simple(PitchCode::N1bb), "cff");
        assert_eq!(pitchcode_to_lilypond_simple(PitchCode::N1ss), "css");
        assert_eq!(pitchcode_to_lilypond_simple(PitchCode::N4ss), "fss");
    }
}