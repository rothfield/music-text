/**
 * Font Manager Module
 * Handles font selection and management for the textarea
 */

import { LocalStorage } from './localStorage.js';
import { UI } from './ui.js';

export const FontManager = {
    // Available font classes
    FONT_CLASSES: [
        'font-default',
        'font-courier', 
        'font-source-code',
        'font-fira-code',
        'font-menlo'
    ],

    // Initialize font manager
    init() {
        this.loadSavedFont();
        this.setupFontSelector();
    },

    // Load saved font preference and apply it
    loadSavedFont() {
        const savedFont = LocalStorage.loadFontPreference();
        const musicInput = document.getElementById('musicInput');
        const fontSelect = document.getElementById('fontSelect');
        
        if (fontSelect) {
            fontSelect.value = savedFont;
        }
        
        if (musicInput) {
            // Remove all font classes first
            this.removeFontClasses(musicInput);
            // Add the saved font class
            musicInput.classList.add(savedFont);
        }
    },

    // Setup font selector event handler
    setupFontSelector() {
        const fontSelect = document.getElementById('fontSelect');
        if (fontSelect) {
            fontSelect.addEventListener('change', (e) => {
                this.changeFont(e.target.value);
            });
        }
    },

    // Change font family
    changeFont(fontClass) {
        if (!this.FONT_CLASSES.includes(fontClass)) {
            console.warn(`Invalid font class: ${fontClass}`);
            return;
        }

        const textarea = document.getElementById('musicInput');
        if (!textarea) {
            console.error('Music input textarea not found');
            return;
        }
        
        // Remove all existing font classes
        this.removeFontClasses(textarea);
        
        // Add the selected font class
        textarea.classList.add(fontClass);
        
        // Save the preference
        LocalStorage.saveFontPreference(fontClass);
        
        // Restore focus and cursor position
        UI.restoreFocusAndCursor();
        
        console.log('Font changed to:', fontClass);
    },

    // Remove all font classes from element
    removeFontClasses(element) {
        this.FONT_CLASSES.forEach(fontClass => {
            element.classList.remove(fontClass);
        });
    },

    // Get current font class
    getCurrentFont() {
        const textarea = document.getElementById('musicInput');
        if (!textarea) return 'font-default';

        for (const fontClass of this.FONT_CLASSES) {
            if (textarea.classList.contains(fontClass)) {
                return fontClass;
            }
        }
        
        return 'font-default';
    },

    // Reset to default font
    resetToDefault() {
        this.changeFont('font-default');
    },

    // Get font display name
    getFontDisplayName(fontClass) {
        const displayNames = {
            'font-default': 'Default Mono',
            'font-courier': 'Courier New',
            'font-source-code': 'Source Code Pro',
            'font-fira-code': 'Fira Code',
            'font-menlo': 'Menlo'
        };
        
        return displayNames[fontClass] || fontClass;
    },

    // Check if font is available
    isFontAvailable(fontClass) {
        return this.FONT_CLASSES.includes(fontClass);
    }
};