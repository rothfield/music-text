
// Re-export domain models from the models crate for convenience
pub use crate::models::*;

/// Trait for elements that have position and value information
/// This trait is used by the parsing infrastructure to work with parsed elements uniformly
pub trait HasPosition {
    fn char_index(&self) -> usize;
    fn value(&self) -> Option<&String>;
    fn consumed_elements(&self) -> &[ConsumedElement];
    fn type_name(&self) -> &'static str;
}

// Implementations for enum variants
impl HasPosition for ConsumedElement {
    fn char_index(&self) -> usize {
        match self {
            ConsumedElement::UpperOctaveMarker { char_index, .. } => *char_index,
            ConsumedElement::LowerOctaveMarker { char_index, .. } => *char_index,
            ConsumedElement::SlurIndicator { char_index, .. } => *char_index,
        }
    }

    fn value(&self) -> Option<&String> {
        match self {
            ConsumedElement::UpperOctaveMarker { value, .. } => value.as_ref(),
            ConsumedElement::LowerOctaveMarker { value, .. } => value.as_ref(),
            ConsumedElement::SlurIndicator { value, .. } => value.as_ref(),
        }
    }

    fn consumed_elements(&self) -> &[ConsumedElement] {
        &[] // ConsumedElements don't have their own consumed elements
    }

    fn type_name(&self) -> &'static str {
        match self {
            ConsumedElement::UpperOctaveMarker { .. } => "ConsumedUpperOctaveMarker",
            ConsumedElement::LowerOctaveMarker { .. } => "ConsumedLowerOctaveMarker",
            ConsumedElement::SlurIndicator { .. } => "ConsumedSlurIndicator",
        }
    }
}

impl HasPosition for BeatElement {
    fn char_index(&self) -> usize {
        match self {
            BeatElement::Note(note) => note.char_index,
            BeatElement::Dash(dash) => dash.char_index,
            BeatElement::BreathMark(breath) => breath.char_index,
            BeatElement::Rest(rest) => rest.char_index,
        }
    }

    fn value(&self) -> Option<&String> {
        match self {
            BeatElement::Note(note) => note.value.as_ref(),
            BeatElement::Dash(dash) => dash.value.as_ref(),
            BeatElement::BreathMark(breath) => breath.value.as_ref(),
            BeatElement::Rest(rest) => rest.value.as_ref(),
        }
    }

    fn consumed_elements(&self) -> &[ConsumedElement] {
        match self {
            BeatElement::Note(note) => &note.consumed_elements,
            BeatElement::Dash(dash) => &dash.consumed_elements,
            BeatElement::BreathMark(breath) => &breath.consumed_elements,
            BeatElement::Rest(rest) => &rest.consumed_elements,
        }
    }

    fn type_name(&self) -> &'static str {
        match self {
            BeatElement::Note { .. } => "Note",
            BeatElement::Dash { .. } => "Dash",
            BeatElement::BreathMark { .. } => "BreathMark",
            BeatElement::Rest { .. } => "Rest",
        }
    }
}

impl HasPosition for UpperElement {
    fn char_index(&self) -> usize {
        match self {
            UpperElement::UpperOctaveMarker { char_index, .. } => *char_index,
            UpperElement::SlurIndicator { char_index, .. } => *char_index,
            UpperElement::UpperHashes { char_index, .. } => *char_index,
            UpperElement::Ornament { char_index, .. } => *char_index,
            UpperElement::Chord { char_index, .. } => *char_index,
            UpperElement::Mordent { char_index, .. } => *char_index,
            UpperElement::Space { char_index, .. } => *char_index,
            UpperElement::Unknown { char_index, .. } => *char_index,
            UpperElement::Newline { char_index, .. } => *char_index,
        }
    }

    fn value(&self) -> Option<&String> {
        match self {
            UpperElement::UpperOctaveMarker { value, .. } => value.as_ref(),
            UpperElement::SlurIndicator { value, .. } => value.as_ref(),
            UpperElement::UpperHashes { value, .. } => value.as_ref(),
            UpperElement::Ornament { value, .. } => value.as_ref(),
            UpperElement::Chord { value, .. } => value.as_ref(),
            UpperElement::Mordent { value, .. } => value.as_ref(),
            UpperElement::Space { value, .. } => value.as_ref(),
            UpperElement::Unknown { value, .. } => value.as_ref(),
            UpperElement::Newline { value, .. } => value.as_ref(),
        }
    }

    fn consumed_elements(&self) -> &[ConsumedElement] {
        &[] // UpperElements don't have consumed elements (they get consumed by others)
    }

    fn type_name(&self) -> &'static str {
        match self {
            UpperElement::UpperOctaveMarker { .. } => "UpperOctaveMarker",
            UpperElement::SlurIndicator { .. } => "UpperSlurIndicator",
            UpperElement::UpperHashes { .. } => "UpperHashes",
            UpperElement::Ornament { .. } => "Ornament",
            UpperElement::Chord { .. } => "Chord",
            UpperElement::Mordent { .. } => "Mordent",
            UpperElement::Space { .. } => "UpperSpace",
            UpperElement::Unknown { .. } => "UpperUnknown",
            UpperElement::Newline { .. } => "UpperNewline",
        }
    }
}

impl HasPosition for Barline {
    fn char_index(&self) -> usize {
        match self {
            Barline::Single(b) => b.char_index,
            Barline::Double(b) => b.char_index,
            Barline::Final(b) => b.char_index,
            Barline::RepeatStart(b) => b.char_index,
            Barline::RepeatEnd(b) => b.char_index,
            Barline::RepeatBoth(b) => b.char_index,
        }
    }

    fn value(&self) -> Option<&String> {
        match self {
            Barline::Single(b) => b.value.as_ref(),
            Barline::Double(b) => b.value.as_ref(),
            Barline::Final(b) => b.value.as_ref(),
            Barline::RepeatStart(b) => b.value.as_ref(),
            Barline::RepeatEnd(b) => b.value.as_ref(),
            Barline::RepeatBoth(b) => b.value.as_ref(),
        }
    }

    fn consumed_elements(&self) -> &[ConsumedElement] {
        match self {
            Barline::Single(b) => &b.consumed_elements,
            Barline::Double(b) => &b.consumed_elements,
            Barline::Final(b) => &b.consumed_elements,
            Barline::RepeatStart(b) => &b.consumed_elements,
            Barline::RepeatEnd(b) => &b.consumed_elements,
            Barline::RepeatBoth(b) => &b.consumed_elements,
        }
    }

    fn type_name(&self) -> &'static str {
        match self {
            Barline::Single { .. } => "SingleBarline",
            Barline::Double { .. } => "DoubleBarline",
            Barline::Final { .. } => "FinalBarline",
            Barline::RepeatStart { .. } => "RepeatStartBarline",
            Barline::RepeatEnd { .. } => "RepeatEndBarline",
            Barline::RepeatBoth { .. } => "RepeatBothBarline",
        }
    }
}

impl HasPosition for Beat {
    fn char_index(&self) -> usize {
        self.char_index
    }

    fn value(&self) -> Option<&String> {
        self.value.as_ref()
    }

    fn consumed_elements(&self) -> &[ConsumedElement] {
        &self.consumed_elements
    }

    fn type_name(&self) -> &'static str {
        "Beat"
    }
}

impl HasPosition for LowerElement {
    fn char_index(&self) -> usize {
        match self {
            LowerElement::LowerOctaveMarker { char_index, .. } => *char_index,
            LowerElement::BeatGroupIndicator { char_index, .. } => *char_index,
            LowerElement::Syllable { char_index, .. } => *char_index,
            LowerElement::Space { char_index, .. } => *char_index,
            LowerElement::Unknown { char_index, .. } => *char_index,
            LowerElement::Newline { char_index, .. } => *char_index,
            LowerElement::EndOfInput { char_index, .. } => *char_index,
        }
    }

    fn value(&self) -> Option<&String> {
        match self {
            LowerElement::LowerOctaveMarker { value, .. } => value.as_ref(),
            LowerElement::BeatGroupIndicator { value, .. } => value.as_ref(),
            LowerElement::Syllable { value, .. } => value.as_ref(),
            LowerElement::Space { value, .. } => value.as_ref(),
            LowerElement::Unknown { value, .. } => value.as_ref(),
            LowerElement::Newline { value, .. } => value.as_ref(),
            LowerElement::EndOfInput { value, .. } => value.as_ref(),
        }
    }

    fn consumed_elements(&self) -> &[ConsumedElement] {
        &[] // LowerElements don't have consumed elements
    }

    fn type_name(&self) -> &'static str {
        match self {
            LowerElement::LowerOctaveMarker { .. } => "LowerOctaveMarker",
            LowerElement::BeatGroupIndicator { .. } => "BeatGroupIndicator",
            LowerElement::Syllable { .. } => "Syllable",
            LowerElement::Space { .. } => "LowerSpace",
            LowerElement::Unknown { .. } => "LowerUnknown",
            LowerElement::Newline { .. } => "LowerNewline",
            LowerElement::EndOfInput { .. } => "EndOfInput",
        }
    }
}