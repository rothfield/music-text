
// Re-export domain models from the models crate for convenience
pub use crate::models::*;

/// Trait for elements that have position and value information
/// This trait is used by the parsing infrastructure to work with parsed elements uniformly
pub trait HasPosition {
    fn value(&self) -> Option<&String>;
    fn type_name(&self) -> &'static str;
}

// Implementations for enum variants

impl HasPosition for BeatElement {
fn value(&self) -> Option<&String> {
        match self {
            BeatElement::Note(note) => note.value.as_ref(),
            BeatElement::Dash(dash) => dash.value.as_ref(),
            BeatElement::BreathMark(breath) => breath.value.as_ref(),
            BeatElement::Rest(rest) => rest.value.as_ref(),
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


impl HasPosition for Barline {
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
fn value(&self) -> Option<&String> {
        self.value.as_ref()
    }


    fn type_name(&self) -> &'static str {
        "Beat"
    }
}

