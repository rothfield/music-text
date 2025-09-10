// Storage Module
// Centralized localStorage operations with consistent error handling

import { STORAGE_KEYS } from './config.js';

export class Storage {
    static save(key, value) {
        try {
            localStorage.setItem(key, value);
        } catch (e) {
            console.warn(`Failed to save ${key} to localStorage:`, e);
        }
    }

    static load(key, defaultValue = '') {
        try {
            return localStorage.getItem(key) || defaultValue;
        } catch (e) {
            console.warn(`Failed to load ${key} from localStorage:`, e);
            return defaultValue;
        }
    }

    static remove(key) {
        try {
            localStorage.removeItem(key);
        } catch (e) {
            console.warn(`Failed to remove ${key} from localStorage:`, e);
        }
    }

    static loadBoolean(key, defaultValue = false) {
        try {
            const value = localStorage.getItem(key);
            return value !== null ? value === 'true' : defaultValue;
        } catch (e) {
            console.warn(`Failed to load boolean ${key} from localStorage:`, e);
            return defaultValue;
        }
    }

    static saveBoolean(key, value) {
        this.save(key, value.toString());
    }

    static loadFloat(key, defaultValue = 0) {
        try {
            const value = localStorage.getItem(key);
            return value !== null ? parseFloat(value) : defaultValue;
        } catch (e) {
            console.warn(`Failed to load float ${key} from localStorage:`, e);
            return defaultValue;
        }
    }

    static saveFloat(key, value) {
        this.save(key, value.toString());
    }

    static loadInt(key, defaultValue = 0) {
        try {
            const value = localStorage.getItem(key);
            return value !== null ? parseInt(value) : defaultValue;
        } catch (e) {
            console.warn(`Failed to load int ${key} from localStorage:`, e);
            return defaultValue;
        }
    }

    static saveInt(key, value) {
        this.save(key, value.toString());
    }
}

// Specific storage functions for the music text app
export function saveInputText(text) {
    Storage.save(STORAGE_KEYS.INPUT_TEXT, text);
}

export function loadInputText() {
    return Storage.load(STORAGE_KEYS.INPUT_TEXT);
}

export function saveActiveTab(tabName) {
    Storage.save(STORAGE_KEYS.ACTIVE_TAB, tabName);
}

export function loadActiveTab() {
    return Storage.load(STORAGE_KEYS.ACTIVE_TAB, 'pest');
}

// Font preference storage
export function saveFontFamily(fontFamily) {
    Storage.save('music-text-font-family', fontFamily);
}

export function loadFontFamily() {
    return Storage.load('music-text-font-family');
}

export function saveFontSize(size) {
    Storage.saveInt('music-text-font-size', size);
}

export function loadFontSize() {
    return Storage.loadInt('music-text-font-size');
}

export function saveLetterSpacing(spacing) {
    Storage.saveFloat('music-text-letter-spacing', spacing);
}

export function loadLetterSpacing() {
    return Storage.loadFloat('music-text-letter-spacing');
}

export function saveLineHeight(height) {
    Storage.saveFloat('music-text-line-height', height);
}

export function loadLineHeight() {
    return Storage.loadFloat('music-text-line-height');
}

export function saveUnicodePreference(useUnicode) {
    Storage.saveBoolean('music-text-use-unicode', useUnicode);
}

export function loadUnicodePreference() {
    return Storage.loadBoolean('music-text-use-unicode', true);
}

export function saveControlsHidden(hidden) {
    if (hidden) {
        Storage.save('music-text-controls-hidden', 'true');
    } else {
        Storage.remove('music-text-controls-hidden');
    }
}

export function loadControlsHidden() {
    return Storage.load('music-text-controls-hidden') === 'true';
}