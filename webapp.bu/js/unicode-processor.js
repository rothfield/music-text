// Unicode Processing Module
// Handle all Unicode-related functionality and font management

import { UNICODE_REPLACEMENTS, UNICODE_CAPABLE_FONTS, ON_DEMAND_FONTS } from './config.js';

// Track loaded fonts to avoid duplicate loading
const loadedFonts = new Set();

// Check if current font supports Unicode replacements
export function isUnicodeCapableFont(fontFamily) {
    if (!fontFamily) return false;
    
    return UNICODE_CAPABLE_FONTS.some(font => 
        fontFamily.toLowerCase().includes(font.toLowerCase())
    );
}

// Load font on demand
export async function loadFontOnDemand(fontName) {
    if (loadedFonts.has(fontName)) {
        return; // Already loaded
    }
    
    const fontUrl = ON_DEMAND_FONTS[fontName];
    if (!fontUrl) {
        return; // Not an on-demand font
    }
    
    try {
        console.log(`Loading font on demand: ${fontName}`);
        const fontFace = new FontFace(fontName, `url(${fontUrl})`);
        const loadedFont = await fontFace.load();
        document.fonts.add(loadedFont);
        loadedFonts.add(fontName);
        console.log(`Successfully loaded: ${fontName}`);
    } catch (error) {
        console.warn(`Failed to load font ${fontName}:`, error);
    }
}

// Apply Unicode replacements to input (for display) - only for Unicode-capable fonts
export function applyUnicodeReplacements(text, fontFamily) {
    // Only apply Unicode replacements if font supports them
    if (!fontFamily || !isUnicodeCapableFont(fontFamily)) {
        return text; // Return original ASCII characters
    }
    
    let result = text;
    
    // Apply most replacements globally
    for (const [original, replacement] of Object.entries(UNICODE_REPLACEMENTS)) {
        if (original !== 'b' && original !== '#') { // Handle 'b' and '#' separately for precise pitch replacement
            result = result.replace(new RegExp('\\\\' + original, 'g'), replacement);
        }
    }
    
    // Simple flat/sharp replacement - basic character replacement
    console.log('🎵 Before b/# replacement:', { input: result.slice(0, 30) });
    
    const beforeB = result;
    result = result.replace(/b/g, '♭'); // Replace all 'b' with flat symbol
    const afterB = result;
    
    const beforeSharp = result;
    result = result.replace(/#/g, '♯'); // Replace all '#' with sharp symbol
    const afterSharp = result;
    
    console.log('🎵 b/# replacement results:', {
        bChanged: beforeB !== afterB,
        sharpChanged: beforeSharp !== afterSharp,
        final: result.slice(0, 30)
    });
    
    return result;
}

// Convert Unicode back to standard characters for backend processing
export function convertUnicodeToStandard(text) {
    let result = text;
    
    // Simple direct replacements - more reliable than complex regex
    result = result.replace(/▬/g, '-');  // Black rectangle -> dash
    result = result.replace(/•/g, '.');  // Bullet -> dot
    result = result.replace(/┃/g, '|');  // Heavy vertical line -> pipe
    result = result.replace(/≋/g, '~');  // Triple tilde -> tilde
    result = result.replace(/♯/g, '#');  // Musical sharp -> hash
    result = result.replace(/♭/g, 'b');  // Musical flat -> b
    
    console.log('🔍 Unicode conversion debug:', {
        input: text,
        output: result,
        changed: text !== result,
        hasBarline: text.includes('┃'),
        hasDash: text.includes('▬'),
        hasDot: text.includes('•'),
        hasSharp: text.includes('♯'),
        hasFlat: text.includes('♭')
    });
    
    return result;
}

// Setup Unicode replacement functionality for input field
export function setupUnicodeInput(inputElement, fontSelectElement, useUnicodeFlag) {
    if (!inputElement) return;

    // Handle keydown events for immediate replacement - only for Unicode-capable fonts
    inputElement.addEventListener('keydown', function(e) {
        const currentFont = fontSelectElement?.value;
        const replacement = UNICODE_REPLACEMENTS[e.key];
        
        // Handle special case for 'b' - check if it should be a flat symbol
        if (e.key === 'b' && isUnicodeCapableFont(currentFont)) {
            const start = e.target.selectionStart;
            const value = e.target.value;
            const charBefore = start > 0 ? value[start - 1] : '';
            
            // Check if the previous character suggests this 'b' is a flat symbol
            if (/[1-7SsRrGgMmPpDdNnAaBbCcEeFf]/.test(charBefore)) {
                e.preventDefault();
                const end = e.target.selectionEnd;
                e.target.value = value.substring(0, start) + '♭' + value.substring(end);
                e.target.setSelectionRange(start + 1, start + 1);
                e.target.dispatchEvent(new Event('input', { bubbles: true }));
                return;
            }
        }
        
        // Only replace if font supports Unicode AND we have a replacement (excluding 'b')
        if (replacement && e.key !== 'b' && isUnicodeCapableFont(currentFont)) {
            e.preventDefault();
            const start = e.target.selectionStart;
            const end = e.target.selectionEnd;
            const value = e.target.value;
            
            e.target.value = value.substring(0, start) + replacement + value.substring(end);
            e.target.setSelectionRange(start + 1, start + 1);
            
            // Trigger input event to update parsing
            e.target.dispatchEvent(new Event('input', { bubbles: true }));
        }
        // If font doesn't support Unicode, let the original character be typed
    });
    
    // Handle input events for paste operations - font-aware
    inputElement.addEventListener('input', function(e) {
        // Only apply Unicode replacements if Unicode mode is enabled
        console.log('⌨️ Input event: checking Unicode mode:', { 
            useUnicode: useUnicodeFlag.current, 
            inputValue: e.target.value.slice(0, 30) 
        });
        
        if (!useUnicodeFlag.current) {
            console.log('⌨️ Input event: Unicode mode OFF, skipping replacements');
            return;
        }
        
        console.log('⌨️ Input event: Unicode mode ON, proceeding with replacements');
        
        const currentFont = fontSelectElement?.value;
        const originalValue = e.target.value;
        const newValue = applyUnicodeReplacements(originalValue, currentFont);
        
        if (originalValue !== newValue) {
            console.log('⌨️ Input event: Unicode replacements applied:', { originalValue, newValue });
            const start = e.target.selectionStart;
            const end = e.target.selectionEnd;
            e.target.value = newValue;
            e.target.setSelectionRange(start, end);
        }
    });
}