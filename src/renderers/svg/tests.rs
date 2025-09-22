use crate::renderers::svg::{Document, Element, Ornament, OrnamentNote, SvgRenderer, SvgRendererConfig};

/// Test cases from the POC specification
pub fn run_all_tests() -> Result<(), String> {
    println!("Running SVG renderer test cases...\n");

    test_basic_elements()?;
    test_octave_markers()?;
    test_accidentals()?;
    test_smufl_font()?;
    test_complete_example()?;
    test_sargam_ornaments()?;

    println!("✅ All SVG renderer test cases passed!");
    Ok(())
}

/// Test Case 7.1: Basic Elements
fn test_basic_elements() -> Result<(), String> {
    println!("🧪 Test 7.1: Basic Elements");

    let doc = Document {
        title: None,
        composer: None,
        notation_type: "number".to_string(),
        font_size: 14.0,
        supports_utf8: true,
        elements: vec![
            Element::Pitch {
                value: "1".to_string(),
                octave: 0,
                accidental: None,
                ornaments: vec![],
                lyrics: vec![],
            },
            Element::Dash { is_rest: false },
            Element::Pitch {
                value: "2".to_string(),
                octave: 0,
                accidental: None,
                ornaments: vec![],
                lyrics: vec![],
            },
            Element::Barline {
                style: "single".to_string(),
            },
        ],
    };

    let mut renderer = SvgRenderer::new(SvgRendererConfig::default());
    let svg = renderer.render(&doc)?;

    // Verify it contains expected elements
    assert!(svg.contains("1"), "Should contain note '1'");
    assert!(svg.contains("–"), "Should contain dash");
    assert!(svg.contains("2"), "Should contain note '2'");
    assert!(svg.contains("barline"), "Should contain barline");

    println!("   ✓ Expected: '1–2|' with proper spacing");
    Ok(())
}

/// Test Case 7.2: Octave Markers
fn test_octave_markers() -> Result<(), String> {
    println!("🧪 Test 7.2: Octave Markers");

    let doc = Document {
        title: None,
        composer: None,
        notation_type: "number".to_string(),
        font_size: 14.0,
        supports_utf8: true,
        elements: vec![
            Element::Pitch {
                value: "1".to_string(),
                octave: 1,
                accidental: None,
                ornaments: vec![],
                lyrics: vec![],
            },
            Element::Pitch {
                value: "2".to_string(),
                octave: -1,
                accidental: None,
                ornaments: vec![],
                lyrics: vec![],
            },
        ],
    };

    let mut renderer = SvgRenderer::new(SvgRendererConfig::default());
    let svg = renderer.render(&doc)?;

    // Verify octave markers
    assert!(svg.contains("upper-octave"), "Should contain upper octave marker");
    assert!(svg.contains("lower-octave"), "Should contain lower octave marker");
    assert!(svg.contains("•"), "Should contain octave dots");

    println!("   ✓ Expected: '1' with dot above, '2' with dot below");
    Ok(())
}

/// Test Case 7.3: Accidentals
fn test_accidentals() -> Result<(), String> {
    println!("🧪 Test 7.3: Accidentals");

    let doc = Document {
        title: None,
        composer: None,
        notation_type: "number".to_string(),
        font_size: 14.0,
        supports_utf8: true,
        elements: vec![
            Element::Pitch {
                value: "1".to_string(),
                octave: 0,
                accidental: Some("sharp".to_string()),
                ornaments: vec![],
                lyrics: vec![],
            },
            Element::Pitch {
                value: "2".to_string(),
                octave: 0,
                accidental: Some("flat".to_string()),
                ornaments: vec![],
                lyrics: vec![],
            },
        ],
    };

    let mut renderer = SvgRenderer::new(SvgRendererConfig::default());
    let svg = renderer.render(&doc)?;

    // Verify accidentals are present
    assert!(svg.contains("accidental"), "Should contain accidental class");

    println!("   ✓ Expected: '1♯ 2♭' with proper positioning");
    Ok(())
}

/// Test Case 7.4: SMuFL Font Test
fn test_smufl_font() -> Result<(), String> {
    println!("🧪 Test 7.4: SMuFL Font Test");

    let doc = Document {
        title: Some("SMuFL Test".to_string()),
        composer: None,
        notation_type: "number".to_string(),
        font_size: 14.0,
        supports_utf8: true,
        elements: vec![
            Element::Barline {
                style: "repeat_start".to_string(),
            },
            Element::Pitch {
                value: "1".to_string(),
                octave: 0,
                accidental: Some("sharp".to_string()),
                ornaments: vec![],
                lyrics: vec![],
            },
            Element::Pitch {
                value: "2".to_string(),
                octave: 0,
                accidental: Some("flat".to_string()),
                ornaments: vec![
                    Ornament::SymbolicOrnament {
                        symbol: "mordent".to_string(),
                    },
                ],
                lyrics: vec![],
            },
            Element::Barline {
                style: "repeat_end".to_string(),
            },
        ],
    };

    let mut renderer = SvgRenderer::new(SvgRendererConfig::default());
    let svg = renderer.render(&doc)?;

    // Verify SMuFL-related classes and fonts
    assert!(svg.contains("smufl-symbol"), "Should contain SMuFL symbol classes");
    assert!(svg.contains("Bravura"), "Should reference Bravura font");
    assert!(svg.contains("ornament"), "Should contain ornament");

    println!("   ✓ Expected: SMuFL repeat barlines with accidentals and mordent");
    Ok(())
}

/// Test Case 7.5: Complete Example with All Features
fn test_complete_example() -> Result<(), String> {
    println!("🧪 Test 7.5: Complete Example");

    let doc = Document {
        title: Some("Complete Test Piece".to_string()),
        composer: Some("Test Composer".to_string()),
        notation_type: "number".to_string(),
        font_size: 14.0,
        supports_utf8: true,
        elements: vec![
            Element::Pitch {
                value: "1".to_string(),
                octave: 1,
                accidental: Some("sharp".to_string()),
                ornaments: vec![
                    Ornament::OnTopGraceNotes {
                        notes: vec![
                            OrnamentNote {
                                value: "N".to_string(),
                                octave: 0,
                                accidental: None,
                            },
                            OrnamentNote {
                                value: "R".to_string(),
                                octave: 0,
                                accidental: None,
                            },
                            OrnamentNote {
                                value: "S".to_string(),
                                octave: 0,
                                accidental: None,
                            },
                        ],
                    },
                ],
                lyrics: vec!["La".to_string()],
            },
            Element::Dash { is_rest: false },
            Element::Pitch {
                value: "2".to_string(),
                octave: 0,
                accidental: None,
                ornaments: vec![],
                lyrics: vec!["la".to_string()],
            },
            Element::Barline {
                style: "single".to_string(),
            },
        ],
    };

    let mut renderer = SvgRenderer::new(SvgRendererConfig::default());
    let svg = renderer.render(&doc)?;

    // Verify all components
    assert!(svg.contains("Complete Test Piece"), "Should contain title");
    assert!(svg.contains("Test Composer"), "Should contain composer");
    assert!(svg.contains("grace-note"), "Should contain grace notes");
    assert!(svg.contains("lyric"), "Should contain lyrics");
    assert!(svg.contains("upper-octave"), "Should contain octave marker");
    assert!(svg.contains("accidental"), "Should contain accidental");

    println!("   ✓ Expected: Complex rendering with all elements");
    Ok(())
}

/// Test Case 7.6: Comprehensive Sargam Ornament Test
fn test_sargam_ornaments() -> Result<(), String> {
    println!("🧪 Test 7.6: Comprehensive Sargam Ornament Test");

    let doc = Document {
        title: Some("Sargam Ornament Test".to_string()),
        composer: None,
        notation_type: "sargam".to_string(),
        font_size: 14.0,
        supports_utf8: true,
        elements: vec![
            Element::Pitch {
                value: "S".to_string(),
                octave: 0,
                accidental: None,
                ornaments: vec![
                    Ornament::BeforeGraceNotes {
                        notes: vec![
                            OrnamentNote {
                                value: "G".to_string(),
                                octave: 0,
                                accidental: None,
                            },
                            OrnamentNote {
                                value: "M".to_string(),
                                octave: 0,
                                accidental: None,
                            },
                        ],
                    },
                ],
                lyrics: vec![],
            },
            Element::Pitch {
                value: "R".to_string(),
                octave: 0,
                accidental: None,
                ornaments: vec![
                    Ornament::OnTopGraceNotes {
                        notes: vec![
                            OrnamentNote {
                                value: "N".to_string(),
                                octave: 0,
                                accidental: None,
                            },
                            OrnamentNote {
                                value: "R".to_string(),
                                octave: 0,
                                accidental: None,
                            },
                            OrnamentNote {
                                value: "S".to_string(),
                                octave: 0,
                                accidental: None,
                            },
                            OrnamentNote {
                                value: "N".to_string(),
                                octave: 0,
                                accidental: None,
                            },
                            OrnamentNote {
                                value: "S".to_string(),
                                octave: 0,
                                accidental: None,
                            },
                        ],
                    },
                ],
                lyrics: vec![],
            },
            Element::Pitch {
                value: "G".to_string(),
                octave: 0,
                accidental: None,
                ornaments: vec![
                    Ornament::AfterGraceNotes {
                        notes: vec![
                            OrnamentNote {
                                value: "P".to_string(),
                                octave: 0,
                                accidental: None,
                            },
                            OrnamentNote {
                                value: "D".to_string(),
                                octave: 0,
                                accidental: None,
                            },
                        ],
                    },
                ],
                lyrics: vec![],
            },
            Element::Pitch {
                value: "M".to_string(),
                octave: 0,
                accidental: None,
                ornaments: vec![
                    Ornament::SymbolicOrnament {
                        symbol: "mordent".to_string(),
                    },
                ],
                lyrics: vec![],
            },
            Element::Barline {
                style: "single".to_string(),
            },
        ],
    };

    let mut renderer = SvgRenderer::new(SvgRendererConfig::default());
    let svg = renderer.render(&doc)?;

    // Verify all ornament types
    assert!(svg.contains("grace-note"), "Should contain grace notes");
    assert!(svg.contains("ornament"), "Should contain ornaments");

    println!("   ✓ Expected: All 4 ornament types rendered correctly with sargam notation");
    Ok(())
}

fn assert(condition: bool, message: &str) {
    if !condition {
        panic!("Assertion failed: {}", message);
    }
}