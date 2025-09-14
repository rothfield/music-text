/**
 * LocalStorage Management Module
 * Handles all localStorage operations for the Music Text application
 */

export const LocalStorage = {
    // Input text management
    saveInputText(text) {
        try {
            localStorage.setItem('musictext_input', text);
        } catch (e) {
            console.warn('Failed to save input text to localStorage:', e);
        }
    },

    loadInputText() {
        try {
            return localStorage.getItem('musictext_input') || '';
        } catch (e) {
            console.warn('Failed to load input text from localStorage:', e);
            return '';
        }
    },

    // Cursor position management
    saveCursorPosition(start, end) {
        try {
            localStorage.setItem('musictext_cursor', JSON.stringify({start, end}));
        } catch (e) {
            console.warn('Failed to save cursor position to localStorage:', e);
        }
    },

    loadCursorPosition() {
        try {
            const stored = localStorage.getItem('musictext_cursor');
            return stored ? JSON.parse(stored) : {start: 0, end: 0};
        } catch (e) {
            console.warn('Failed to load cursor position from localStorage:', e);
            return {start: 0, end: 0};
        }
    },

    // Active tab management
    saveActiveTab(tabName) {
        try {
            localStorage.setItem('musictext_active_tab', tabName);
        } catch (e) {
            console.warn('Failed to save active tab to localStorage:', e);
        }
    },

    loadActiveTab() {
        try {
            return localStorage.getItem('musictext_active_tab') || 'vexflow';
        } catch (e) {
            console.warn('Failed to load active tab from localStorage:', e);
            return 'vexflow';
        }
    },

    // Font preference management
    saveFontPreference(fontClass) {
        try {
            localStorage.setItem('musictext_font', fontClass);
        } catch (e) {
            console.warn('Failed to save font preference to localStorage:', e);
        }
    },

    loadFontPreference() {
        try {
            return localStorage.getItem('musictext_font') || 'font-default';
        } catch (e) {
            console.warn('Failed to load font preference from localStorage:', e);
            return 'font-default';
        }
    },

    // Clear all data
    clearAll() {
        try {
            localStorage.removeItem('musictext_input');
            localStorage.removeItem('musictext_cursor');
            localStorage.removeItem('musictext_active_tab');
            localStorage.removeItem('musictext_font');
        } catch (e) {
            console.warn('Failed to clear localStorage:', e);
        }
    }
};