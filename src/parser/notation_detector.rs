#[derive(Debug, Clone, PartialEq)]
pub enum NotationType {
    Western,
    Sargam,
    Number,
}

impl NotationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotationType::Western => "Western",
            NotationType::Sargam => "Sargam",
            NotationType::Number => "Number",
        }
    }
}

pub fn detect_notation_type(text: &str) -> NotationType {
    let mut western_count = 0;
    let mut sargam_count = 0;
    let mut number_count = 0;
    
    for ch in text.chars() {
        match ch {
            // Western-only pitches
            'A' | 'B' | 'C' | 'E' | 'F' => western_count += 1,
            // Sargam-only pitches  
            'S' | 'r' | 'R' | 'g' | 'm' | 'M' | 'P' | 'n' | 'N' => sargam_count += 1,
            // Shared pitches (G and D appear in both Western and Sargam)
            'G' => {
                western_count += 1;
                sargam_count += 1;
            }
            'd' | 'D' => {
                western_count += 1;
                sargam_count += 1;
            }
            // Number pitches
            '1'..='7' => number_count += 1,
            _ => {}
        }
    }
    
    [
        (NotationType::Western, western_count),
        (NotationType::Sargam, sargam_count),
        (NotationType::Number, number_count),
    ]
    .into_iter()
    .max_by_key(|&(_, count)| count)
    .unwrap()
    .0
}