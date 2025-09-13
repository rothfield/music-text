// Configuration Module
// Centralized configuration constants and settings for the Music Text webapp

export const STORAGE_KEYS = {
    INPUT_TEXT: 'music-text-parser-input',
    ACTIVE_TAB: 'music-text-parser-active-tab'
};

export const UNICODE_REPLACEMENTS = {
    '-': '▬',  // Black rectangle for dashes
    '.': '•',  // Bullet for dots
    '|': '┃',  // Heavy vertical line for barlines
    '~': '≋',  // Triple tilde (U+224B) - represents ornament/mordent
    '#': '♯',  // Musical sharp sign (U+266F)
    'b': '♭'   // Musical flat sign (U+266D) - replaces b only in musical contexts
};

export const UNICODE_CAPABLE_FONTS = [
    'JuliaMono',
    'JuliaMono Latin', 
    'DejaVu Sans Mono'
    // Note: Kurinto Mono removed - breaks monospace alignment for | character
    // Note: JuliaMono Latin likely doesn't have Unicode musical symbols - will fallback to ASCII
];

export const ON_DEMAND_FONTS = {
    'Kurinto Mono': 'https://github.com/welai/kurinto/raw/master/fonts/ttf/KurintoMono-Regular.ttf'
};

export const DEFAULT_FONT_SETTINGS = {
    spacing: 0.1,
    height: 1.6,
    size: 10,
    font: "'JuliaMono', monospace"
};

export const DEBOUNCE_DELAYS = {
    PARSE: 1000,      // 1 second for general parsing
    SVG: 3000         // 3 seconds for SVG generation
};

export const API_ENDPOINTS = {
    PARSE: '/api/parse',
    VALID_PITCHES: '/api/valid-pitches',
    LILYPOND_SVG: '/api/lilypond-svg'
};