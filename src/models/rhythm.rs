use fraction::Fraction;

#[derive(Debug, Clone)]
pub struct RhythmConverter;

impl RhythmConverter {

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

    // #[test] - DELETED broken test for removed function
    // fn test_is_common_dotted_duration() {

    // #[test] - DELETED broken test for removed function
    // fn test_is_standard_duration() {

    #[test]
    fn test_empty_fraction_fallback() {
        let zero = Fraction::new(0u64, 1u64);
        let result = RhythmConverter::decompose_fraction_to_standard_durations(zero);
        assert_eq!(result, vec![Fraction::new(1u64, 32u64)]);
    }

    // #[test] - DELETED broken test for removed function
    // fn test_fraction_to_lilypond_double_dotted() {

    #[test]
    fn test_fraction_to_vexflow_double_dotted() {
        // Double dotted eighth (S------R case)
        let double_dotted_eighth = Fraction::new(7u64, 32u64);
        let result = RhythmConverter::fraction_to_vexflow(double_dotted_eighth);
        assert_eq!(result, vec![("8".to_string(), 2)]);
    }

    // #[test] - DELETED broken test for removed function
    // fn test_fraction_to_lilypond_basic() {

    #[test]
    fn test_fraction_to_vexflow_basic() {
        let quarter = Fraction::new(1u64, 4u64);
        let result = RhythmConverter::fraction_to_vexflow(quarter);
        assert_eq!(result, vec![("q".to_string(), 0)]);
    }
}