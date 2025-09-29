use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Barline models

// Individual barline types matching grammar productions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleBarline {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoubleBarline {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalBarline {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeatStartBarline {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeatEndBarline {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeatBothBarline {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub value: Option<String>,
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