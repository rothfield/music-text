use serde::{Deserialize, Serialize};

// Barline models

// Individual barline types matching grammar productions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleBarline {
    pub value: Option<String>,
    pub char_index: usize,
    pub consumed_elements: Vec<super::position::ConsumedElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoubleBarline {
    pub value: Option<String>,
    pub char_index: usize,
    pub consumed_elements: Vec<super::position::ConsumedElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalBarline {
    pub value: Option<String>,
    pub char_index: usize,
    pub consumed_elements: Vec<super::position::ConsumedElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeatStartBarline {
    pub value: Option<String>,
    pub char_index: usize,
    pub consumed_elements: Vec<super::position::ConsumedElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeatEndBarline {
    pub value: Option<String>,
    pub char_index: usize,
    pub consumed_elements: Vec<super::position::ConsumedElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeatBothBarline {
    pub value: Option<String>,
    pub char_index: usize,
    pub consumed_elements: Vec<super::position::ConsumedElement>,
}

// Unified barline enum for ContentElement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Barline {
    Single(SingleBarline),
    Double(DoubleBarline),
    Final(FinalBarline),
    RepeatStart(RepeatStartBarline),
    RepeatEnd(RepeatEndBarline),
    RepeatBoth(RepeatBothBarline),
}