/**
 * LocalStorage Management Module
 * Handles all localStorage operations for the Music Text application
 */

export const LocalStorage = {
    // Input text management
    saveInputText(text) {
        try {
            localStorage.setItem('music-text-input', text);
        } catch (error) {
            console.warn('Failed to save input to localStorage:', error);
        }
    },

    loadInputText() {
        try {
            return localStorage.getItem('music-text-input') || '';
        } catch (error) {
            console.warn('Failed to load input from localStorage:', error);
            return '';
        }
    },

    // Cursor position management
    saveCursorPosition(start, end) {
        try {
            localStorage.setItem('music-text-cursor', JSON.stringify({start, end}));
        } catch (error) {
            console.warn('Failed to save cursor position to localStorage:', error);
        }
    },

    loadCursorPosition() {
        try {
            const saved = localStorage.getItem('music-text-cursor');
            const result = saved ? JSON.parse(saved) : {start: 0, end: 0};
            console.log('📁 Loaded cursor position from storage:', result);
            return result;
        } catch (error) {
            console.warn('Failed to load cursor position from localStorage:', error);
            return {start: 0, end: 0};
        }
    },

    // Active tab management
    saveActiveTab(tabName) {
        try {
            localStorage.setItem('music-text-active-tab', tabName);
        } catch (error) {
            console.warn('Failed to save active tab to localStorage:', error);
        }
    },

    loadActiveTab() {
        try {
            return localStorage.getItem('music-text-active-tab') || 'vexflow';
        } catch (error) {
            console.warn('Failed to load active tab from localStorage:', error);
            return 'vexflow';
        }
    },

    // Font preference management
    saveFontPreference(fontClass) {
        try {
            localStorage.setItem('music-text-font-preference', fontClass);
        } catch (error) {
            console.warn('Failed to save font preference to localStorage:', error);
        }
    },

    loadFontPreference() {
        try {
            return localStorage.getItem('music-text-font-preference') || 'font-default';
        } catch (error) {
            console.warn('Failed to load font preference from localStorage:', error);
            return 'font-default';
        }
    },

    // Clear all data
    clearAll() {
        try {
            localStorage.removeItem('music-text-input');
            localStorage.removeItem('music-text-cursor');
            localStorage.removeItem('music-text-active-tab');
            localStorage.removeItem('music-text-font-preference');
        } catch (error) {
            console.warn('Failed to clear localStorage:', error);
        }
    }
};