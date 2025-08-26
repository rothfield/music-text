use fraction::Fraction;

#[derive(Debug, Clone)]
pub struct RhythmConverter;

impl RhythmConverter {
    pub fn new() -> Self {
        Self
    }

    pub fn decompose_fraction_to_standard_durations(frac: Fraction) -> Vec<Fraction> {
        let mut remaining = frac;
        let mut result = Vec::new();
        
        let standard_durations = [
            Fraction::new(1u64, 1u64),   // whole note
            Fraction::new(1u64, 2u64),   // half note
            Fraction::new(1u64, 4u64),   // quarter note
            Fraction::new(1u64, 8u64),   // eighth note
            Fraction::new(1u64, 16u64),  // sixteenth note
            Fraction::new(1u64, 32u64),  // thirty-second note
        ];
        
        for dur_frac in &standard_durations {
            while remaining >= *dur_frac {
                result.push(*dur_frac);
                remaining = remaining - *dur_frac;
            }
        }
        
        if result.is_empty() {
            vec![Fraction::new(1u64, 32u64)] // Fallback to thirty-second note
        } else {
            result
        }
    }

    pub fn is_common_dotted_duration(frac: Fraction) -> bool {
        let dotted_durations = [
            Fraction::new(3u64, 8u64),   // dotted quarter (4.)
            Fraction::new(3u64, 16u64),  // dotted eighth (8.)
            Fraction::new(3u64, 32u64),  // dotted sixteenth (16.)
        ];
        
        dotted_durations.contains(&frac)
    }

    pub fn is_standard_duration(frac: Fraction) -> bool {
        let standard_durations = [
            Fraction::new(1u64, 1u64),
            Fraction::new(1u64, 2u64),
            Fraction::new(1u64, 4u64),
            Fraction::new(1u64, 8u64),
            Fraction::new(1u64, 16u64),
            Fraction::new(1u64, 32u64),
        ];
        
        standard_durations.contains(&frac)
    }

    /// Convert fraction to LilyPond duration notation
    pub fn fraction_to_lilypond(frac: Fraction) -> Vec<String> {
        // Lookup table for common fractions to LilyPond durations
        let lookup = [
            (Fraction::new(1u64, 1u64), vec!["1".to_string()]),
            (Fraction::new(1u64, 2u64), vec!["2".to_string()]),
            (Fraction::new(1u64, 4u64), vec!["4".to_string()]),
            (Fraction::new(1u64, 8u64), vec!["8".to_string()]),
            (Fraction::new(1u64, 16u64), vec!["16".to_string()]),
            (Fraction::new(1u64, 32u64), vec!["32".to_string()]),
            (Fraction::new(3u64, 8u64), vec!["4.".to_string()]),   // dotted quarter
            (Fraction::new(3u64, 16u64), vec!["8.".to_string()]),  // dotted eighth
            (Fraction::new(3u64, 32u64), vec!["16.".to_string()]), // dotted sixteenth
            (Fraction::new(7u64, 8u64), vec!["2..".to_string()]),  // double dotted half
            (Fraction::new(7u64, 16u64), vec!["4..".to_string()]), // double dotted quarter
            (Fraction::new(7u64, 32u64), vec!["8..".to_string()]), // double dotted eighth
        ];
        
        // Check for direct match first
        for (lookup_frac, durations) in &lookup {
            if frac == *lookup_frac {
                return durations.clone();
            }
        }
        
        // If no direct match, decompose into tied notes
        let fraction_parts = Self::decompose_fraction_to_standard_durations(frac);
        let mut result = Vec::new();
        
        for (i, part_frac) in fraction_parts.iter().enumerate() {
            // Recursively convert each part
            let part_durations = Self::fraction_to_lilypond(*part_frac);
            result.extend(part_durations);
            
            // Add ties between parts (not after the last one)
            if i < fraction_parts.len() - 1 {
                result.push("~".to_string());
            }
        }
        
        if result.is_empty() {
            vec!["32".to_string()] // Fallback
        } else {
            result
        }
    }

    /// Convert fraction to VexFlow duration notation (duration, dots)
    pub fn fraction_to_vexflow(frac: Fraction) -> Vec<(String, u8)> {
        // Lookup table for common fractions to VexFlow durations
        let lookup = [
            (Fraction::new(1u64, 1u64), vec![("w".to_string(), 0)]),    // whole note
            (Fraction::new(1u64, 2u64), vec![("h".to_string(), 0)]),    // half note
            (Fraction::new(1u64, 4u64), vec![("q".to_string(), 0)]),    // quarter note
            (Fraction::new(1u64, 8u64), vec![("8".to_string(), 0)]),    // eighth note
            (Fraction::new(1u64, 16u64), vec![("16".to_string(), 0)]),  // sixteenth note
            (Fraction::new(1u64, 32u64), vec![("32".to_string(), 0)]),  // thirty-second note
            (Fraction::new(3u64, 8u64), vec![("q".to_string(), 1)]),    // dotted quarter
            (Fraction::new(3u64, 16u64), vec![("8".to_string(), 1)]),   // dotted eighth
            (Fraction::new(3u64, 32u64), vec![("16".to_string(), 1)]),  // dotted sixteenth
            (Fraction::new(7u64, 8u64), vec![("h".to_string(), 2)]),    // double dotted half
            (Fraction::new(7u64, 16u64), vec![("q".to_string(), 2)]),   // double dotted quarter
            (Fraction::new(7u64, 32u64), vec![("8".to_string(), 2)]),   // double dotted eighth
        ];
        
        // Check for direct match first
        for (lookup_frac, durations) in &lookup {
            if frac == *lookup_frac {
                return durations.clone();
            }
        }
        
        // If no direct match, decompose into tied notes
        let fraction_parts = Self::decompose_fraction_to_standard_durations(frac);
        fraction_parts.iter().flat_map(|f| {
            // Recursively convert each part
            Self::fraction_to_vexflow(*f)
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompose_standard_fraction() {
        let quarter_note = Fraction::new(1u64, 4u64);
        let result = RhythmConverter::decompose_fraction_to_standard_durations(quarter_note);
        assert_eq!(result, vec![Fraction::new(1u64, 4u64)]);
    }

    #[test]
    fn test_decompose_complex_fraction() {
        let complex = Fraction::new(5u64, 8u64);
        let result = RhythmConverter::decompose_fraction_to_standard_durations(complex);
        assert_eq!(result, vec![
            Fraction::new(1u64, 2u64),
            Fraction::new(1u64, 8u64)
        ]);
    }

    #[test]
    fn test_is_common_dotted_duration() {
        assert!(RhythmConverter::is_common_dotted_duration(Fraction::new(3u64, 8u64)));
        assert!(RhythmConverter::is_common_dotted_duration(Fraction::new(3u64, 16u64)));
        assert!(!RhythmConverter::is_common_dotted_duration(Fraction::new(1u64, 4u64)));
    }

    #[test]
    fn test_is_standard_duration() {
        assert!(RhythmConverter::is_standard_duration(Fraction::new(1u64, 4u64)));
        assert!(RhythmConverter::is_standard_duration(Fraction::new(1u64, 8u64)));
        assert!(!RhythmConverter::is_standard_duration(Fraction::new(3u64, 8u64)));
    }

    #[test]
    fn test_empty_fraction_fallback() {
        let zero = Fraction::new(0u64, 1u64);
        let result = RhythmConverter::decompose_fraction_to_standard_durations(zero);
        assert_eq!(result, vec![Fraction::new(1u64, 32u64)]);
    }

    #[test]
    fn test_fraction_to_lilypond_double_dotted() {
        // Double dotted eighth (S------R case)
        let double_dotted_eighth = Fraction::new(7u64, 32u64);
        let result = RhythmConverter::fraction_to_lilypond(double_dotted_eighth);
        assert_eq!(result, vec!["8..".to_string()]);
    }

    #[test]
    fn test_fraction_to_vexflow_double_dotted() {
        // Double dotted eighth (S------R case)
        let double_dotted_eighth = Fraction::new(7u64, 32u64);
        let result = RhythmConverter::fraction_to_vexflow(double_dotted_eighth);
        assert_eq!(result, vec![("8".to_string(), 2)]);
    }

    #[test]
    fn test_fraction_to_lilypond_basic() {
        let quarter = Fraction::new(1u64, 4u64);
        let result = RhythmConverter::fraction_to_lilypond(quarter);
        assert_eq!(result, vec!["4".to_string()]);
    }

    #[test]
    fn test_fraction_to_vexflow_basic() {
        let quarter = Fraction::new(1u64, 4u64);
        let result = RhythmConverter::fraction_to_vexflow(quarter);
        assert_eq!(result, vec![("q".to_string(), 0)]);
    }
}