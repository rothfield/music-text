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
}