use fontdue::{Font, FontSettings};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

/// Default monospace font to use for measurements
static DEFAULT_FONT_DATA: &[u8] = include_bytes!("../webapp/public/fonts/Inter.ttc");

/// Global font instance for measurements
static FONT: Lazy<Font> = Lazy::new(|| {
    // Load the font with default settings
    // Inter.ttc contains multiple fonts, fontdue will pick the first one
    Font::from_bytes(DEFAULT_FONT_DATA, FontSettings::default())
        .expect("Failed to load Inter font")
});

/// Cache for character widths at different font sizes
static WIDTH_CACHE: Lazy<RwLock<HashMap<(char, u32), f32>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Get the width of a character at a given font size
pub fn get_char_width(ch: char, font_size: f32) -> f32 {
    // Round font size to nearest integer for cache key
    let size_key = font_size.round() as u32;

    // Check cache first
    {
        let cache = WIDTH_CACHE.read().unwrap();
        if let Some(&width) = cache.get(&(ch, size_key)) {
            return width;
        }
    }

    // Calculate width using fontdue
    let (metrics, _) = FONT.rasterize(ch, font_size);

    // The advance_width is in pixels and represents how far to advance
    // the cursor after rendering this character
    let width = metrics.advance_width;

    // Store in cache
    {
        let mut cache = WIDTH_CACHE.write().unwrap();
        cache.insert((ch, size_key), width);
    }

    width
}

/// Get the width of a string at a given font size
pub fn get_string_width(text: &str, font_size: f32) -> f32 {
    text.chars()
        .map(|ch| get_char_width(ch, font_size))
        .sum()
}

/// Get the height metrics for a given font size
pub fn get_font_height(font_size: f32) -> (f32, f32) {
    // Use a sample character to get typical metrics
    let (metrics, _) = FONT.rasterize('M', font_size);

    // Return (ascent, descent) approximation
    // Note: fontdue doesn't provide exact ascent/descent, so we approximate
    let height = metrics.height as f32;
    let baseline = metrics.ymin.abs() as f32;

    (baseline, height - baseline)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_width() {
        // Test that we get reasonable widths
        let width_space = get_char_width(' ', 20.0);
        let width_m = get_char_width('M', 20.0);
        let width_i = get_char_width('i', 20.0);

        // M should be wider than i in most fonts
        assert!(width_m > width_i);

        // All widths should be positive
        assert!(width_space > 0.0);
        assert!(width_m > 0.0);
        assert!(width_i > 0.0);
    }

    #[test]
    fn test_string_width() {
        let text = "Hello";
        let width = get_string_width(text, 20.0);

        // Should be sum of individual character widths
        let expected: f32 = text.chars()
            .map(|ch| get_char_width(ch, 20.0))
            .sum();

        assert_eq!(width, expected);
    }

    #[test]
    fn test_cache() {
        // First call should calculate
        let width1 = get_char_width('A', 20.0);

        // Second call should use cache (should be fast)
        let width2 = get_char_width('A', 20.0);

        assert_eq!(width1, width2);
    }
}