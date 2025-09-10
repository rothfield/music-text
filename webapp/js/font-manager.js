// Font Management Module
// Handle all font-related UI and logic

import { DEFAULT_FONT_SETTINGS } from './config.js';
import { 
    saveFontFamily, loadFontFamily, 
    saveFontSize, loadFontSize,
    saveLetterSpacing, loadLetterSpacing,
    saveLineHeight, loadLineHeight,
    saveControlsHidden, loadControlsHidden
} from './storage.js';
import { loadFontOnDemand, applyUnicodeReplacements, convertUnicodeToStandard } from './unicode-processor.js';

export class FontManager {
    constructor(inputElement) {
        this.inputElement = inputElement;
        this.fontSelect = null;
        this.fontSize = null;
        this.fontSizeValue = null;
        this.spacingSlider = null;
        this.spacingValue = null;
        this.heightSlider = null;
        this.heightValue = null;
        this.resetFontBtn = null;
        this.fontsButton = null;
        this.fontConfig = null;
        this.closeFontConfig = null;
    }

    init() {
        this.bindElements();
        this.setupFontControls();
        this.setupVisibilityToggle();
        this.loadSavedSettings();
        this.restoreControlsVisibility();
    }

    bindElements() {
        this.fontSelect = document.getElementById('font-select');
        this.fontSize = document.getElementById('font-size');
        this.fontSizeValue = document.getElementById('font-size-value');
        this.spacingSlider = document.getElementById('spacing-slider');
        this.spacingValue = document.getElementById('spacing-value');
        this.heightSlider = document.getElementById('height-slider');
        this.heightValue = document.getElementById('height-value');
        this.resetFontBtn = document.getElementById('reset-font-btn');
        this.fontsButton = document.getElementById('fonts-button');
        this.fontConfig = document.getElementById('font-config');
        this.closeFontConfig = document.getElementById('close-font-config');
    }

    setupFontControls() {
        this.setupFontFamily();
        this.setupFontSize();
        this.setupLetterSpacing();
        this.setupLineHeight();
        this.setupResetButton();
    }

    setupFontFamily() {
        if (!this.fontSelect || !this.inputElement) return;

        this.fontSelect.addEventListener('change', async (e) => {
            const selectedFont = e.target.value;
            
            // Extract font name for on-demand loading
            const fontName = selectedFont.replace(/['\"]/g, '').split(',')[0];
            
            // Load font on demand if needed
            await loadFontOnDemand(fontName);
            
            this.inputElement.style.fontFamily = selectedFont;
            
            // Save font choice
            saveFontFamily(selectedFont);
            console.log('Font changed to:', selectedFont);
            
            // Refresh display with font-aware Unicode replacements
            this.refreshTextDisplay();
            
            // Trigger input event to refresh parsing
            this.inputElement.dispatchEvent(new Event('input', { bubbles: true }));
        });
    }

    setupFontSize() {
        if (!this.fontSize || !this.fontSizeValue || !this.inputElement) return;

        this.fontSize.addEventListener('input', (e) => {
            const size = parseInt(e.target.value);
            this.inputElement.style.fontSize = size + 'px';
            this.fontSizeValue.textContent = size + 'px';
            saveFontSize(size);
        });
    }

    setupLetterSpacing() {
        if (!this.spacingSlider || !this.spacingValue || !this.inputElement) return;

        this.spacingSlider.addEventListener('input', (e) => {
            const spacing = parseFloat(e.target.value);
            this.inputElement.style.letterSpacing = spacing + 'em';
            this.spacingValue.textContent = spacing + 'em';
            saveLetterSpacing(spacing);
        });
    }

    setupLineHeight() {
        if (!this.heightSlider || !this.heightValue || !this.inputElement) return;

        this.heightSlider.addEventListener('input', (e) => {
            const height = parseFloat(e.target.value);
            this.inputElement.style.lineHeight = height + 'em';
            this.heightValue.textContent = height + 'em';
            saveLineHeight(height);
        });
    }

    setupResetButton() {
        if (!this.resetFontBtn) return;

        this.resetFontBtn.addEventListener('click', () => {
            this.resetToDefaults();
        });
    }

    setupVisibilityToggle() {
        // Fonts Button - Toggle font configuration visibility  
        if (this.fontsButton && this.fontConfig) {
            this.fontsButton.addEventListener('click', () => {
                if (this.fontConfig.style.display === 'none') {
                    this.fontConfig.style.display = 'block';
                    this.fontsButton.textContent = 'Hide';
                } else {
                    this.fontConfig.style.display = 'none';
                    this.fontsButton.textContent = 'Fonts...';
                }
            });
        }
        
        // Close Font Config Button
        if (this.closeFontConfig && this.fontConfig && this.fontsButton) {
            this.closeFontConfig.addEventListener('click', () => {
                this.fontConfig.style.display = 'none';
                this.fontsButton.textContent = 'Fonts...';
            });
        }
    }

    loadSavedSettings() {
        // Load saved font family
        const savedFont = loadFontFamily();
        if (savedFont && this.fontSelect) {
            this.fontSelect.value = savedFont;
            this.inputElement.style.fontFamily = savedFont;
        }

        // Load saved font size
        const savedSize = loadFontSize();
        if (savedSize && this.fontSize && this.fontSizeValue) {
            this.fontSize.value = savedSize;
            this.inputElement.style.fontSize = savedSize + 'px';
            this.fontSizeValue.textContent = savedSize + 'px';
        }

        // Load saved letter spacing
        const savedSpacing = loadLetterSpacing();
        if (savedSpacing !== 0 && this.spacingSlider && this.spacingValue) {
            this.spacingSlider.value = savedSpacing;
            this.inputElement.style.letterSpacing = savedSpacing + 'em';
            this.spacingValue.textContent = savedSpacing + 'em';
        }

        // Load saved line height
        const savedHeight = loadLineHeight();
        if (savedHeight !== 0 && this.heightSlider && this.heightValue) {
            this.heightSlider.value = savedHeight;
            this.inputElement.style.lineHeight = savedHeight + 'em';
            this.heightValue.textContent = savedHeight + 'em';
        }
    }

    restoreControlsVisibility() {
        if (loadControlsHidden()) {
            if (this.fontConfig) this.fontConfig.style.display = 'none';
            if (this.fontsButton) this.fontsButton.style.display = 'inline-block';
        }
    }

    resetToDefaults() {
        // Reset font family
        if (this.fontSelect) {
            this.fontSelect.value = DEFAULT_FONT_SETTINGS.font;
            this.inputElement.style.fontFamily = DEFAULT_FONT_SETTINGS.font;
            saveFontFamily(DEFAULT_FONT_SETTINGS.font);
        }
        
        // Reset font size
        if (this.fontSize && this.fontSizeValue) {
            this.fontSize.value = DEFAULT_FONT_SETTINGS.size;
            this.inputElement.style.fontSize = DEFAULT_FONT_SETTINGS.size + 'px';
            this.fontSizeValue.textContent = DEFAULT_FONT_SETTINGS.size + 'px';
            saveFontSize(DEFAULT_FONT_SETTINGS.size);
        }
        
        // Reset letter spacing
        if (this.spacingSlider && this.spacingValue) {
            this.spacingSlider.value = DEFAULT_FONT_SETTINGS.spacing;
            this.inputElement.style.letterSpacing = DEFAULT_FONT_SETTINGS.spacing + 'em';
            this.spacingValue.textContent = DEFAULT_FONT_SETTINGS.spacing + 'em';
            saveLetterSpacing(DEFAULT_FONT_SETTINGS.spacing);
        }
        
        // Reset line height
        if (this.heightSlider && this.heightValue) {
            this.heightSlider.value = DEFAULT_FONT_SETTINGS.height;
            this.inputElement.style.lineHeight = DEFAULT_FONT_SETTINGS.height + 'em';
            this.heightValue.textContent = DEFAULT_FONT_SETTINGS.height + 'em';
            saveLineHeight(DEFAULT_FONT_SETTINGS.height);
        }
    }

    refreshTextDisplay(useUnicode = true) {
        if (!this.inputElement) {
            console.log('🔄 refreshTextDisplay: inputElement is null/undefined');
            return;
        }
        
        console.log('🔄 refreshTextDisplay called:', { 
            useUnicode, 
            inputValue: this.inputElement.value.slice(0, 50),
            inputLength: this.inputElement.value.length 
        });
        
        const currentText = convertUnicodeToStandard(this.inputElement.value);
        if (useUnicode) {
            const currentFont = this.inputElement.style.fontFamily || (this.fontSelect ? this.fontSelect.value : DEFAULT_FONT_SETTINGS.font);
            console.log('🎵 About to apply Unicode replacements:', { currentFont, currentText: currentText.slice(0, 50) });
            const displayText = applyUnicodeReplacements(currentText, currentFont);
            this.inputElement.value = displayText;
            console.log('🎵 Unicode applied:', { result: displayText.slice(0, 50) });
        } else {
            this.inputElement.value = currentText;
            console.log('🎵 Unicode OFF - using standard text:', { result: currentText.slice(0, 50) });
        }
        
        // Trigger parsing update
        this.inputElement.dispatchEvent(new Event('input', { bubbles: true }));
    }
}