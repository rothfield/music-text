use crate::models::pitch::{Degree, Notation};

// Import all pitch system modules
pub mod western;
pub mod number;
pub mod sargam;
pub mod bhatkhande;
pub mod tabla;

/// Main dispatcher for pitch lookups across all notation systems
/// 
/// This function routes symbol lookups to the appropriate notation system
/// and maintains backward compatibility with the original lookup_pitch API.
pub fn lookup_pitch(symbol: &str, notation: Notation) -> Option<Degree> {
    match notation {
        Notation::Western => western::lookup(symbol),
        Notation::Number => number::lookup(symbol),
        Notation::Sargam => sargam::lookup(symbol),
        Notation::Bhatkhande => bhatkhande::lookup(symbol),
        Notation::Tabla => tabla::lookup(symbol),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatcher_western() {
        assert_eq!(lookup_pitch("C", Notation::Western), Some(Degree::N1));
        assert_eq!(lookup_pitch("Bb", Notation::Western), Some(Degree::N7b));
        assert_eq!(lookup_pitch("A#", Notation::Western), Some(Degree::N6s));
    }

    #[test]
    fn test_dispatcher_number() {
        assert_eq!(lookup_pitch("1", Notation::Number), Some(Degree::N1));
        assert_eq!(lookup_pitch("7b", Notation::Number), Some(Degree::N7b));
        assert_eq!(lookup_pitch("4#", Notation::Number), Some(Degree::N4s));
    }

    #[test]
    fn test_dispatcher_sargam() {
        assert_eq!(lookup_pitch("S", Notation::Sargam), Some(Degree::N1));
        assert_eq!(lookup_pitch("r", Notation::Sargam), Some(Degree::N2b));
        assert_eq!(lookup_pitch("n", Notation::Sargam), Some(Degree::N7b));
    }

    #[test]
    fn test_dispatcher_bhatkhande() {
        assert_eq!(lookup_pitch("स", Notation::Bhatkhande), Some(Degree::N1));
        assert_eq!(lookup_pitch("S", Notation::Bhatkhande), Some(Degree::N1));
        assert_eq!(lookup_pitch("M#", Notation::Bhatkhande), Some(Degree::N4s));
    }

    #[test]
    fn test_dispatcher_tabla() {
        assert_eq!(lookup_pitch("dha", Notation::Tabla), Some(Degree::N1));
        assert_eq!(lookup_pitch("ge", Notation::Tabla), Some(Degree::N1));
        assert_eq!(lookup_pitch("unknown", Notation::Tabla), None);
    }

    #[test]
    fn test_dispatcher_invalid() {
        assert_eq!(lookup_pitch("X", Notation::Sargam), None);
        assert_eq!(lookup_pitch("", Notation::Western), None);
    }
}