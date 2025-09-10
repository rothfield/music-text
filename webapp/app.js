let debounceTimer;
let svgDebounceTimer;
const STORAGE_KEYS = {
    INPUT_TEXT: 'music-text-parser-input',
    ACTIVE_TAB: 'music-text-parser-active-tab'
};

// Bootstrap handles tab switching automatically, but we still need to save active tab
function saveActiveTabFromBootstrap() {
    const activeTab = document.querySelector('.nav-link.active');
    if (activeTab) {
        const tabName = activeTab.id.replace('-tab-btn', '');
        saveActiveTab(tabName);
    }
}

// Unicode character replacements for better visual width
const UNICODE_REPLACEMENTS = {
    '-': '▬',  // Black rectangle for dashes
    '.': '•',  // Bullet for dots
    '|': '┃',  // Heavy vertical line for barlines
    '~': '≋',  // Triple tilde (U+224B) - represents ornament/mordent
    '#': '♯',  // Musical sharp sign (U+266F)
    'b': '♭'   // Musical flat sign (U+266D) - replaces b only in musical contexts
};

// Valid pitch patterns from server
let VALID_FLAT_PATTERNS = [];
let VALID_SHARP_PATTERNS = [];

// Fetch valid pitch patterns from server at startup
async function loadValidPitches() {
    console.log('🔄 Loading valid pitch patterns from server...');
    try {
        const response = await fetch('/api/valid-pitches');
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        const data = await response.json();
        VALID_FLAT_PATTERNS = data.flat_patterns || [];
        VALID_SHARP_PATTERNS = data.sharp_patterns || [];
        console.log('✅ Loaded valid pitch patterns:', { 
            flats: VALID_FLAT_PATTERNS.length, 
            sharps: VALID_SHARP_PATTERNS.length,
            flatSample: VALID_FLAT_PATTERNS.slice(0, 3),
            sharpSample: VALID_SHARP_PATTERNS.slice(0, 3)
        });
    } catch (error) {
        console.error('❌ Failed to load valid pitch patterns, falling back to regex:', error);
        // Fallback - will use the old regex method if server fails
        VALID_FLAT_PATTERNS = [];
        VALID_SHARP_PATTERNS = [];
    }
}

// Fonts that support proper Unicode monospace rendering
const UNICODE_CAPABLE_FONTS = [
    'JuliaMono',
    'JuliaMono Latin', 
    'DejaVu Sans Mono'
    // Note: Kurinto Mono removed - breaks monospace alignment for | character
    // Note: JuliaMono Latin likely doesn't have Unicode musical symbols - will fallback to ASCII
];

// On-demand font loading for fonts that need special handling
const ON_DEMAND_FONTS = {
    'Kurinto Mono': 'https://github.com/welai/kurinto/raw/master/fonts/ttf/KurintoMono-Regular.ttf'
};

// Track loaded fonts to avoid duplicate loading
const loadedFonts = new Set();

// Load font on demand
async function loadFontOnDemand(fontName) {
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

// Check if current font supports Unicode replacements
function isUnicodeCapableFont(fontFamily) {
    return UNICODE_CAPABLE_FONTS.some(font => 
        fontFamily.toLowerCase().includes(font.toLowerCase())
    );
}

// Apply Unicode replacements to input (for display) - only for Unicode-capable fonts
function applyUnicodeReplacements(text, fontFamily) {
    // Only apply Unicode replacements if font supports them
    if (!fontFamily || !isUnicodeCapableFont(fontFamily)) {
        return text; // Return original ASCII characters
    }
    
    let result = text;
    
    // Apply most replacements globally
    for (const [original, replacement] of Object.entries(UNICODE_REPLACEMENTS)) {
        if (original !== 'b' && original !== '#') { // Handle 'b' and '#' separately for precise pitch replacement
            result = result.replace(new RegExp('\\' + original, 'g'), replacement);
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
function convertUnicodeToStandard(text) {
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

// Add event listeners to Bootstrap tab buttons
document.addEventListener('DOMContentLoaded', async function() {
    // Simple Unicode replacement - no server patterns needed
    
    const tabButtons = document.querySelectorAll('.nav-link');
    tabButtons.forEach(button => {
        button.addEventListener('shown.bs.tab', saveActiveTabFromBootstrap);
    });
    
    // UI Elements
    const inputText = document.getElementById('input-text');
    const unicodeToggle = document.getElementById('unicode-toggle');
    console.log('🔍 DOM elements found:', { 
        inputText: !!inputText, 
        unicodeToggle: !!unicodeToggle,
        unicodeToggleId: unicodeToggle?.id,
        unicodeToggleChecked: unicodeToggle?.checked 
    });
    const fontsButton = document.getElementById('fonts-button');
    const fontConfig = document.getElementById('font-config');
    const fontSelect = document.getElementById('font-select');
    const fontSize = document.getElementById('font-size');
    const fontSizeValue = document.getElementById('font-size-value');
    const spacingSlider = document.getElementById('spacing-slider');
    const spacingValue = document.getElementById('spacing-value');
    const heightSlider = document.getElementById('height-slider');
    const heightValue = document.getElementById('height-value');
    const resetFontBtn = document.getElementById('reset-font-btn');
    const closeFontConfig = document.getElementById('close-font-config');
    
    // Fonts Button - Toggle font configuration visibility  
    if (fontsButton && fontConfig) {
        fontsButton.addEventListener('click', function() {
            if (fontConfig.style.display === 'none') {
                fontConfig.style.display = 'block';
                fontsButton.textContent = 'Hide';
            } else {
                fontConfig.style.display = 'none';
                fontsButton.textContent = 'Fonts...';
            }
        });
    }
    
    // Close Font Config Button
    if (closeFontConfig && fontConfig && fontsButton) {
        closeFontConfig.addEventListener('click', function() {
            fontConfig.style.display = 'none';
            fontsButton.textContent = 'Fonts...';
        });
    }
    
    // Unicode Toggle
    let useUnicode = true;
    
    if (unicodeToggle && inputText) {
        console.log('🎛️ Setting up Unicode toggle...');
        // Load saved Unicode preference
        try {
            const savedUnicode = localStorage.getItem('music-text-use-unicode');
            if (savedUnicode !== null) {
                useUnicode = savedUnicode === 'true';
                unicodeToggle.checked = useUnicode;
                console.log('📂 Loaded Unicode preference from localStorage:', useUnicode);
            } else {
                console.log('📂 No saved Unicode preference, using default:', useUnicode);
            }
        } catch (e) {
            console.warn('Failed to load Unicode preference:', e);
        }
        
        console.log('🔗 Attaching event listener to Unicode toggle...');
        unicodeToggle.addEventListener('change', function() {
            console.log('🎯 Unicode toggle clicked! New state:', this.checked, 'Previous state:', useUnicode);
            console.log('🎯 Input text before toggle:', inputText.value.slice(0, 30));
            useUnicode = this.checked;
            
            // Save to localStorage
            try {
                localStorage.setItem('music-text-use-unicode', useUnicode.toString());
                console.log('💾 Saved Unicode preference to localStorage:', useUnicode);
            } catch (e) {
                console.warn('Failed to save Unicode preference:', e);
            }
            
            // Refresh the display
            console.log('🔄 About to call refreshTextDisplay...');
            refreshTextDisplay();
            console.log('✅ refreshTextDisplay completed');
        });
        console.log('✅ Unicode toggle event listener attached successfully!');
    } else {
        console.error('❌ Unicode toggle setup failed:', { 
            unicodeToggle: !!unicodeToggle, 
            inputText: !!inputText 
        });
    }
    
    function refreshTextDisplay() {
        if (!inputText) {
            console.log('🔄 refreshTextDisplay: inputText is null/undefined');
            return;
        }
        
        console.log('🔄 refreshTextDisplay called:', { 
            useUnicode, 
            inputValue: inputText.value.slice(0, 50),
            inputLength: inputText.value.length 
        });
        
        const currentText = convertUnicodeToStandard(inputText.value);
        if (useUnicode) {
            const currentFont = inputText.style.fontFamily || (fontSelect ? fontSelect.value : "'JuliaMono', monospace");
            console.log('🎵 About to apply Unicode replacements:', { currentFont, currentText: currentText.slice(0, 50) });
            const displayText = applyUnicodeReplacements(currentText, currentFont);
            inputText.value = displayText;
            console.log('🎵 Unicode applied:', { result: displayText.slice(0, 50) });
        } else {
            inputText.value = currentText;
            console.log('🎵 Unicode OFF - using standard text:', { result: currentText.slice(0, 50) });
        }
        
        // Trigger parsing update
        inputText.dispatchEvent(new Event('input', { bubbles: true }));
    }
    
    // Font Family Selection
    if (fontSelect && inputText) {
        fontSelect.addEventListener('change', function() {
            const selectedFont = this.value;
            inputText.style.fontFamily = selectedFont;
            
            // Save to localStorage
            try {
                localStorage.setItem('music-text-font-family', selectedFont);
            } catch (e) {
                console.warn('Failed to save font preference:', e);
            }
        });
        
        // Load saved font family
        try {
            const savedFont = localStorage.getItem('music-text-font-family');
            if (savedFont) {
                fontSelect.value = savedFont;
                inputText.style.fontFamily = savedFont;
            }
        } catch (e) {
            console.warn('Failed to load saved font:', e);
        }
    }

    // Font Size
    if (fontSize && fontSizeValue && inputText) {
        fontSize.addEventListener('input', function() {
            const size = this.value + 'px';
            inputText.style.fontSize = size;
            fontSizeValue.textContent = size;
            
            // Save to localStorage
            try {
                localStorage.setItem('music-text-font-size', this.value);
            } catch (e) {
                console.warn('Failed to save font size:', e);
            }
        });
        
        // Load saved font size
        try {
            const savedSize = localStorage.getItem('music-text-font-size');
            if (savedSize) {
                fontSize.value = savedSize;
                inputText.style.fontSize = savedSize + 'px';
                fontSizeValue.textContent = savedSize + 'px';
            }
        } catch (e) {
            console.warn('Failed to load saved font size:', e);
        }
    }

    // Letter Spacing Slider
    if (spacingSlider && spacingValue && inputText) {
        spacingSlider.addEventListener('input', function() {
            const spacing = parseFloat(this.value);
            inputText.style.letterSpacing = spacing + 'em';
            spacingValue.textContent = spacing + 'em';
            
            // Save to localStorage
            try {
                localStorage.setItem('music-text-letter-spacing', spacing.toString());
            } catch (e) {
                console.warn('Failed to save letter spacing:', e);
            }
        });
        
        // Load saved letter spacing
        try {
            const savedSpacing = localStorage.getItem('music-text-letter-spacing');
            if (savedSpacing !== null) {
                const spacing = parseFloat(savedSpacing);
                spacingSlider.value = spacing;
                inputText.style.letterSpacing = spacing + 'em';
                spacingValue.textContent = spacing + 'em';
            }
        } catch (e) {
            console.warn('Failed to load saved letter spacing:', e);
        }
    }

    // Line Height Slider
    if (heightSlider && heightValue && inputText) {
        heightSlider.addEventListener('input', function() {
            const height = parseFloat(this.value);
            inputText.style.lineHeight = height + 'em';
            heightValue.textContent = height + 'em';
            
            // Save to localStorage
            try {
                localStorage.setItem('music-text-line-height', height.toString());
            } catch (e) {
                console.warn('Failed to save line height:', e);
            }
        });
        
        // Load saved line height
        try {
            const savedHeight = localStorage.getItem('music-text-line-height');
            if (savedHeight !== null) {
                const height = parseFloat(savedHeight);
                heightSlider.value = height;
                inputText.style.lineHeight = height + 'em';
                heightValue.textContent = height + 'em';
            }
        } catch (e) {
            console.warn('Failed to load saved line height:', e);
        }
    }

    // Reset Font Settings
    if (resetFontBtn) {
        resetFontBtn.addEventListener('click', function() {
            // Reset font family
            if (fontSelect) {
                fontSelect.value = "'JuliaMono', monospace";
                inputText.style.fontFamily = "'JuliaMono', monospace";
                localStorage.removeItem('music-text-font-family');
            }
            
            // Reset font size
            if (fontSize && fontSizeValue) {
                fontSize.value = '14';
                inputText.style.fontSize = '14px';
                fontSizeValue.textContent = '14px';
                localStorage.removeItem('music-text-font-size');
            }
            
            // Reset letter spacing
            if (spacingSlider && spacingValue) {
                spacingSlider.value = '0.1';
                inputText.style.letterSpacing = '0.1em';
                spacingValue.textContent = '0.1em';
                localStorage.removeItem('music-text-letter-spacing');
            }
            
            // Reset line height
            if (heightSlider && heightValue) {
                heightSlider.value = '1.6';
                inputText.style.lineHeight = '1.6em';
                heightValue.textContent = '1.6em';
                localStorage.removeItem('music-text-line-height');
            }
        });
    }
    
    if (fontSelect && inputText) {
        fontSelect.addEventListener('change', async function() {
            const selectedFont = this.value;
            
            // Extract font name for on-demand loading
            const fontName = selectedFont.replace(/['"]/g, '').split(',')[0];
            
            // Load font on demand if needed
            await loadFontOnDemand(fontName);
            
            inputText.style.fontFamily = selectedFont;
            
            // Refresh display with font-aware Unicode replacements
            const currentText = convertUnicodeToStandard(inputText.value);
            const displayText = applyUnicodeReplacements(currentText, selectedFont);
            inputText.value = displayText;
            
            // Save font choice to localStorage
            try {
                localStorage.setItem('music-text-font', selectedFont);
                console.log('Font changed to:', selectedFont, 'Unicode capable:', isUnicodeCapableFont(selectedFont));
            } catch (e) {
                console.warn('Failed to save font choice:', e);
            }
            
            // Trigger input event to refresh parsing with new display
            inputText.dispatchEvent(new Event('input', { bubbles: true }));
        });
        
        // Load saved font choice
        try {
            const savedFont = localStorage.getItem('music-text-font');
            if (savedFont) {
                fontSelect.value = savedFont;
                inputText.style.fontFamily = savedFont;
                console.log('Loaded saved font:', savedFont);
            }
        } catch (e) {
            console.warn('Failed to load saved font:', e);
        }
    }
    
    // Add slider controls functionality
    if (spacingSlider && spacingValue && inputText) {
        spacingSlider.addEventListener('input', function() {
            const spacing = parseFloat(this.value);
            inputText.style.letterSpacing = spacing + 'em';
            spacingValue.textContent = spacing + 'em';
            
            // Save to localStorage
            try {
                localStorage.setItem('music-text-horizontal-spacing', spacing.toString());
            } catch (e) {
                console.warn('Failed to save horizontal spacing:', e);
            }
        });
        
        // Load saved horizontal spacing
        try {
            const savedSpacing = localStorage.getItem('music-text-horizontal-spacing');
            if (savedSpacing !== null) {
                const spacing = parseFloat(savedSpacing);
                spacingSlider.value = spacing;
                inputText.style.letterSpacing = spacing + 'em';
                spacingValue.textContent = spacing + 'em';
            }
        } catch (e) {
            console.warn('Failed to load saved horizontal spacing:', e);
        }
    }
    
    if (heightSlider && heightValue && inputText) {
        heightSlider.addEventListener('input', function() {
            const lineHeight = parseFloat(this.value);
            inputText.style.lineHeight = lineHeight + 'em';
            heightValue.textContent = lineHeight + 'em';
            
            // Save to localStorage
            try {
                localStorage.setItem('music-text-vertical-spacing', lineHeight.toString());
            } catch (e) {
                console.warn('Failed to save vertical spacing:', e);
            }
        });
        
        // Load saved vertical spacing
        try {
            const savedLineHeight = localStorage.getItem('music-text-vertical-spacing');
            if (savedLineHeight !== null) {
                const lineHeight = parseFloat(savedLineHeight);
                heightSlider.value = lineHeight;
                inputText.style.lineHeight = lineHeight + 'em';
                heightValue.textContent = lineHeight + 'em';
            }
        } catch (e) {
            console.warn('Failed to load saved vertical spacing:', e);
        }
    }
    
    // Font size slider functionality
    if (fontSize && fontSizeValue && inputText) {
        fontSize.addEventListener('input', function() {
            const fontSize = parseInt(this.value);
            inputText.style.fontSize = fontSize + 'px';
            fontSizeValue.textContent = fontSize + 'px';
            
            // Save to localStorage
            try {
                localStorage.setItem('music-text-font-size', fontSize.toString());
            } catch (e) {
                console.warn('Failed to save font size:', e);
            }
        });
        
        // Load saved font size
        try {
            const savedFontSize = localStorage.getItem('music-text-font-size');
            if (savedFontSize !== null) {
                const fontSize = parseInt(savedFontSize);
                fontSize.value = fontSize;
                inputText.style.fontSize = fontSize + 'px';
                fontSizeValue.textContent = fontSize + 'px';
            }
        } catch (e) {
            console.warn('Failed to load saved font size:', e);
        }
    }
    
    // Font controls buttons (only if they exist)
    const resetDefaultsBtn = document.getElementById('reset-defaults-btn');
    const showControlsBtn = document.getElementById('show-controls-btn'); 
    const closeControlsBtn = document.getElementById('close-controls-btn');
    const fontControls = document.getElementById('font-controls');
    
    // Default values
    const DEFAULTS = {
        spacing: 0.1,
        height: 1.6,
        size: 10,
        font: "'JuliaMono', monospace"
    };
    
    if (resetDefaultsBtn) {
        resetDefaultsBtn.addEventListener('click', function() {
            // Reset all values to defaults
            if (spacingSlider) {
                spacingSlider.value = DEFAULTS.spacing;
                inputText.style.letterSpacing = DEFAULTS.spacing + 'em';
                spacingValue.textContent = DEFAULTS.spacing + 'em';
                localStorage.setItem('music-text-horizontal-spacing', DEFAULTS.spacing.toString());
            }
            
            if (heightSlider) {
                heightSlider.value = DEFAULTS.height;
                inputText.style.lineHeight = DEFAULTS.height + 'em';
                heightValue.textContent = DEFAULTS.height + 'em';
                localStorage.setItem('music-text-vertical-spacing', DEFAULTS.height.toString());
            }
            
            if (fontSize) {
                fontSize.value = DEFAULTS.size;
                inputText.style.fontSize = DEFAULTS.size + 'px';
                fontSizeValue.textContent = DEFAULTS.size + 'px';
                localStorage.setItem('music-text-font-size', DEFAULTS.size.toString());
            }
            
            if (fontSelect) {
                fontSelect.value = DEFAULTS.font;
                inputText.style.fontFamily = DEFAULTS.font;
                localStorage.setItem('music-text-font-family', DEFAULTS.font);
            }
        });
    }
    
    // Show/hide controls functionality
    if (closeControlsBtn) {
        closeControlsBtn.addEventListener('click', function() {
            if (fontControls) fontControls.style.display = 'none';
            if (showControlsBtn) showControlsBtn.style.display = 'inline-block';
            localStorage.setItem('music-text-controls-hidden', 'true');
        });
    }
    
    if (showControlsBtn) {
        showControlsBtn.addEventListener('click', function() {
            if (fontControls) fontControls.style.display = 'block';
            if (showControlsBtn) showControlsBtn.style.display = 'none';
            localStorage.removeItem('music-text-controls-hidden');
        });
    }
    
    // Check if controls should be hidden on load
    if (localStorage.getItem('music-text-controls-hidden') === 'true') {
        if (fontControls) fontControls.style.display = 'none';
        if (showControlsBtn) showControlsBtn.style.display = 'inline-block';
    }
    
    // Add Unicode replacement functionality to input
    if (inputText) {
        // Handle keydown events for immediate replacement - only for Unicode-capable fonts
        inputText.addEventListener('keydown', function(e) {
            const currentFont = e.target.style.fontFamily || fontSelect?.value;
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
        inputText.addEventListener('input', function(e) {
            // Only apply Unicode replacements if Unicode mode is enabled
            console.log('⌨️ Input event: checking Unicode mode:', { 
                useUnicode, 
                inputValue: e.target.value.slice(0, 30) 
            });
            
            if (!useUnicode) {
                console.log('⌨️ Input event: Unicode mode OFF, skipping replacements');
                return;
            }
            
            console.log('⌨️ Input event: Unicode mode ON, proceeding with replacements');
            
            const currentFont = e.target.style.fontFamily || fontSelect?.value;
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
});

function saveInputText(text) {
    try {
        localStorage.setItem(STORAGE_KEYS.INPUT_TEXT, text);
    } catch (e) {
        console.warn('Failed to save input text to localStorage:', e);
    }
}

function loadInputText() {
    try {
        return localStorage.getItem(STORAGE_KEYS.INPUT_TEXT) || '';
    } catch (e) {
        console.warn('Failed to load input text from localStorage:', e);
        return '';
    }
}

function saveActiveTab(tabName) {
    try {
        localStorage.setItem(STORAGE_KEYS.ACTIVE_TAB, tabName);
    } catch (e) {
        console.warn('Failed to save active tab to localStorage:', e);
    }
}

function loadActiveTab() {
    try {
        return localStorage.getItem(STORAGE_KEYS.ACTIVE_TAB) || 'pest';
    } catch (e) {
        console.warn('Failed to load active tab from localStorage:', e);
        return 'pest';
    }
}

function restoreActiveTab() {
    const savedTab = loadActiveTab();
    
    // Find the saved tab button and activate it using Bootstrap
    const tabButton = document.getElementById(savedTab + '-tab-btn');
    
    if (tabButton) {
        // Use Bootstrap's Tab API to activate the tab
        const tab = new bootstrap.Tab(tabButton);
        tab.show();
    } else {
        // Fallback to first tab if saved tab doesn't exist
        const firstButton = document.querySelector('.nav-link');
        if (firstButton) {
            const tab = new bootstrap.Tab(firstButton);
            tab.show();
        }
    }
}

function syntaxHighlight(json) {
    json = json.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
    return json.replace(/(\"(\\u[a-fA-F0-9]{4}|\\[^u]|[^\\\"])*\"(\s*:)?|\b(true|false|null)\b|-?\d+(?:\.\d*)?(?:[eE][+\-]?\d+)?)/g, function (match) {
        let cls = 'number';
        if (/^\"/.test(match)) {
            if (/:$/.test(match)) {
                cls = 'key';
            } else {
                cls = 'string';
            }
        } else if (/true|false/.test(match)) {
            cls = 'boolean';
        } else if (/null/.test(match)) {
            cls = 'null';
        }
        return '<span class="' + cls + '">' + match + '</span>';
    });
}

async function parseInput(input) {
    console.log('🚀 parseInput() called:', {
        inputLength: input.length,
        isEmpty: !input.trim(),
        firstLine: input.split('\n')[0],
        totalLines: input.split('\n').length,
        timestamp: new Date().toISOString()
    });
    
    const pestOutput = document.querySelector('#pest-tab .json-output');
    const documentOutput = document.querySelector('#document-tab .json-output');
    const processedOutput = document.querySelector('#processed-tab .json-output');
    const minimalLilyOutput = document.querySelector('#minimal-lily-tab .json-output');
    const fullLilyOutput = document.querySelector('#full-lily-tab .json-output');
    const svgOutput = document.getElementById('svg-content');
    const vexflowCanvas = document.getElementById('vexflow-canvas');
    const vexflowData = document.getElementById('vexflow-data');
    
    if (!input.trim()) {
        // Reset notation systems display
        const detectedSystemsSpan = document.getElementById('detected-systems');
        detectedSystemsSpan.textContent = 'Enter some music to see detected systems...';
        
        pestOutput.innerHTML = 'Type in the textarea above to see the raw PEST parse tree...';
        pestOutput.className = 'json-output p-3 loading';
        documentOutput.innerHTML = 'Parsed document structure will appear here...';
        documentOutput.className = 'json-output p-3 loading';
        processedOutput.innerHTML = 'Processed staves will appear here...';
        processedOutput.className = 'json-output p-3 loading';
        minimalLilyOutput.innerHTML = 'Minimal LilyPond notation will appear here...';
        minimalLilyOutput.className = 'json-output p-3 loading';
        fullLilyOutput.innerHTML = 'Full LilyPond score will appear here...';
        fullLilyOutput.className = 'json-output p-3 loading';
        svgOutput.innerHTML = 'LilyPond SVG rendering will appear here...';
        svgOutput.className = 'p-3 loading';
        if (vexflowData) vexflowData.innerHTML = 'VexFlow notation data will appear here...';
        if (vexflowCanvas) {
            const ctx = vexflowCanvas.getContext('2d');
            ctx.clearRect(0, 0, vexflowCanvas.width, vexflowCanvas.height);
            ctx.fillStyle = '#fafafa';
            ctx.fillRect(0, 0, vexflowCanvas.width, vexflowCanvas.height);
            ctx.fillStyle = '#666';
            ctx.font = '14px Arial';
            ctx.fillText('VexFlow canvas will render here...', 20, 100);
        }
        return;
    }
    
    try {
        // Set all outputs to loading
        pestOutput.innerHTML = 'Parsing...';
        pestOutput.className = 'tab-content json-output loading';
        documentOutput.innerHTML = 'Parsing...';
        documentOutput.className = 'tab-content json-output loading';
        processedOutput.innerHTML = 'Processing...';
        processedOutput.className = 'tab-content json-output loading';
        minimalLilyOutput.innerHTML = 'Generating...';
        minimalLilyOutput.className = 'tab-content json-output loading';
        fullLilyOutput.innerHTML = 'Generating...';
        fullLilyOutput.className = 'tab-content json-output loading';
        svgOutput.innerHTML = 'Rendering...';
        svgOutput.className = 'tab-content loading';
        if (vexflowData) vexflowData.innerHTML = 'Converting...';
        if (vexflowCanvas) {
            const ctx = vexflowCanvas.getContext('2d');
            ctx.clearRect(0, 0, vexflowCanvas.width, vexflowCanvas.height);
            ctx.fillStyle = '#fafafa';
            ctx.fillRect(0, 0, vexflowCanvas.width, vexflowCanvas.height);
            ctx.fillStyle = '#666';
            ctx.font = '14px Arial';
            ctx.fillText('Rendering VexFlow...', 20, 100);
        }
        
        // Convert Unicode characters back to standard characters for backend
        const standardInput = convertUnicodeToStandard(input);
        console.log('🔄 Converting Unicode to standard for backend:', {
            original: input.slice(0, 50) + (input.length > 50 ? '...' : ''),
            converted: standardInput.slice(0, 50) + (standardInput.length > 50 ? '...' : ''),
            hasUnicode: input !== standardInput
        });
        
        // Fetch all outputs from unified endpoint
        const apiUrl = `/api/parse?input=${encodeURIComponent(standardInput)}`;
        console.log('🔄 Making API request:', { 
            input: standardInput.slice(0, 100) + (standardInput.length > 100 ? '...' : ''),
            url: apiUrl,
            timestamp: new Date().toISOString()
        });
        
        const response = await fetch(apiUrl);
        console.log('📡 API Response received:', {
            status: response.status,
            ok: response.ok,
            headers: Object.fromEntries(response.headers.entries())
        });
        
        const data = await response.json();
        console.log('📋 Parsed API data:', {
            success: data.success,
            hasError: !!data.error,
            error: data.error?.slice(0, 200),
            detectedSystems: data.detected_notation_systems,
            outputsGenerated: {
                pest: !!data.pest_output,
                document: !!data.parsed_document,
                lily: !!data.minimal_lilypond,
                vexflow: !!data.vexflow
            }
        });
        
        if (data.success) {
            console.log('✅ Processing successful API response');
            
            // Update detected notation systems display
            const detectedSystemsSpan = document.getElementById('detected-systems');
            if (data.detected_notation_systems && data.detected_notation_systems.length > 0) {
                detectedSystemsSpan.textContent = data.detected_notation_systems.join(', ');
                console.log('🎵 Updated notation systems display:', data.detected_notation_systems);
            } else {
                detectedSystemsSpan.textContent = 'None detected';
                console.log('⚠️ No notation systems detected');
            }
            
            // PEST Output
            if (data.pest_output) {
                const jsonString = JSON.stringify(data.pest_output, null, 2);
                pestOutput.innerHTML = '<div class="syntax-highlight">' + syntaxHighlight(jsonString) + '</div>';
                pestOutput.className = 'tab-content json-output active';
            } else {
                pestOutput.innerHTML = 'No PEST output available';
                pestOutput.className = 'tab-content json-output loading active';
            }
            
            // Document Structure
            if (data.parsed_document) {
                const docJsonString = JSON.stringify(data.parsed_document, null, 2);
                documentOutput.innerHTML = '<div class="syntax-highlight">' + syntaxHighlight(docJsonString) + '</div>';
                documentOutput.className = 'tab-content json-output';
            } else {
                documentOutput.innerHTML = 'No document structure available';
                documentOutput.className = 'tab-content json-output loading';
            }
            
            // Processed Staves
            if (data.processed_staves) {
                const processedJsonString = JSON.stringify(data.processed_staves, null, 2);
                processedOutput.innerHTML = '<div class="syntax-highlight">' + syntaxHighlight(processedJsonString) + '</div>';
                processedOutput.className = 'tab-content json-output';
            } else {
                processedOutput.innerHTML = 'No processed staves available';
                processedOutput.className = 'tab-content json-output loading';
            }
            
            // Minimal LilyPond
            if (data.minimal_lilypond) {
                minimalLilyOutput.innerHTML = '<pre style="white-space: pre-wrap;">' + data.minimal_lilypond + '</pre>';
                minimalLilyOutput.className = 'tab-content json-output';
            } else {
                minimalLilyOutput.innerHTML = 'No minimal LilyPond available';
                minimalLilyOutput.className = 'tab-content json-output loading';
            }
            
            // Full LilyPond
            if (data.full_lilypond) {
                fullLilyOutput.innerHTML = '<pre style="white-space: pre-wrap;">' + data.full_lilypond + '</pre>';
                fullLilyOutput.className = 'tab-content json-output';
            } else {
                fullLilyOutput.innerHTML = 'No full LilyPond available';
                fullLilyOutput.className = 'tab-content json-output loading';
            }
            
            // SVG Output
            if (data.lilypond_svg) {
                svgOutput.innerHTML = data.lilypond_svg;
                svgOutput.className = 'tab-content';
            } else {
                svgOutput.innerHTML = 'No SVG available';
                svgOutput.className = 'tab-content loading';
            }
            
            // VexFlow - Enhanced rendering with professional features
            if (data.vexflow) {
                // console.log('🎼 Rendering enhanced VexFlow output:', {
                //     hasVexflowData: !!data.vexflow,
                //     staves: data.vexflow.staves?.length,
                //     hasAdvancedFeatures: data.vexflow.staves?.some(s => s.notes?.some(n => n.type === 'Tuplet' || n.type === 'SlurStart'))
                // });
                
                const vexflowOutput = document.getElementById('vexflow-output');
                
                // Create container for VexFlow rendering
                vexflowOutput.innerHTML = `
                    <div class="vexflow-professional">
                        <div class="text-muted mb-2">Professional VexFlow Rendering with Advanced Features</div>
                        <div id="vexflow-notation" style="width: 100%; min-height: 200px; border: 1px solid #ddd; background: #fafafa;"></div>
                        <div class="mt-2">
                            <button id="toggle-vexflow-data" class="btn btn-sm btn-outline-secondary">Show JSON Data</button>
                        </div>
                        <div id="vexflow-data" class="json-output mt-2" style="display: none; max-height: 300px; overflow-y: auto;"></div>
                    </div>
                `;
                
                // Render with enhanced VexFlow renderer
                if (window.VexFlowRenderer) {
                    window.VexFlowRenderer.renderVexFlowNotation(data.vexflow, 'vexflow-notation')
                        .then(success => {
                            if (success) {
                                // console.log('✅ Enhanced VexFlow rendering completed');
                            } else {
                                console.warn('⚠️ VexFlow rendering had issues');
                            }
                        })
                        .catch(error => {
                            console.error('🚨 VexFlow rendering failed:', error);
                            // VexFlow error handling - display in canvas area if needed
                        });
                } else {
                    console.warn('⚠️ VexFlowRenderer not available, loading...');
                    // Try to load the renderer and retry
                    loadVexFlowRenderer().then(() => {
                        if (window.VexFlowRenderer) {
                            window.VexFlowRenderer.renderVexFlowNotation(data.vexflow, 'vexflow-notation');
                        }
                    });
                }
                
                // Setup JSON data toggle
                const vexJsonString = JSON.stringify(data.vexflow, null, 2);
                const vexflowDataDiv = document.getElementById('vexflow-data');
                vexflowDataDiv.innerHTML = '<div class="syntax-highlight">' + syntaxHighlight(vexJsonString) + '</div>';
                
                // Toggle VexFlow data visibility (if toggle button exists)
                const toggleButton = document.getElementById('toggle-vexflow-data');
                if (toggleButton) {
                    toggleButton.addEventListener('click', function() {
                        const dataDiv = document.getElementById('vexflow-data');
                        const button = this;
                    if (dataDiv.style.display === 'none') {
                        dataDiv.style.display = 'block';
                        button.textContent = 'Hide JSON Data';
                    } else {
                        dataDiv.style.display = 'none';
                        button.textContent = 'Show JSON Data';
                    }
                    });
                }
                
            } else {
                // console.log('⚠️ No VexFlow data available');
                const vexflowOutput = document.getElementById('vexflow-output');
                vexflowOutput.innerHTML = '<div class="text-muted">No VexFlow data available - check parser output</div>';
            }
            
        } else {
            console.log('❌ API returned error response:', {
                success: data.success,
                error: data.error,
                errorLength: data.error?.length,
                timestamp: new Date().toISOString()
            });
            
            // Show error in notation systems display
            const detectedSystemsSpan = document.getElementById('detected-systems');
            detectedSystemsSpan.textContent = 'Parse error';
            console.log('🔴 Updated notation systems display with: Parse error');
            
            const errorMsg = '<div class="error">Parse Error: ' + (data.error || 'Unknown error') + '</div>';
            pestOutput.innerHTML = errorMsg;
            pestOutput.className = 'tab-content json-output active';
            documentOutput.innerHTML = errorMsg;
            documentOutput.className = 'tab-content json-output';
            processedOutput.innerHTML = errorMsg;
            processedOutput.className = 'tab-content json-output';
            minimalLilyOutput.innerHTML = errorMsg;
            minimalLilyOutput.className = 'tab-content json-output';
            fullLilyOutput.innerHTML = errorMsg;
            fullLilyOutput.className = 'tab-content json-output';
            svgOutput.innerHTML = errorMsg;
            svgOutput.className = 'tab-content';
            if (vexflowData) vexflowData.innerHTML = errorMsg;
            console.log('📄 Updated all output tabs with error message');
        }
        
    } catch (error) {
        console.error('🚨 Network/JavaScript error caught:', {
            message: error.message,
            name: error.name,
            stack: error.stack,
            timestamp: new Date().toISOString()
        });
        
        // Show network error in notation systems display  
        const detectedSystemsSpan = document.getElementById('detected-systems');
        detectedSystemsSpan.textContent = 'Network error';
        console.log('🔴 Updated notation systems display with: Network error');
        
        const errorMsg = '<div class="error">Network Error: ' + error.message + '</div>';
        pestOutput.innerHTML = errorMsg;
        pestOutput.className = 'tab-content json-output active';
        documentOutput.innerHTML = errorMsg;
        documentOutput.className = 'tab-content json-output';
        processedOutput.innerHTML = errorMsg;
        processedOutput.className = 'tab-content json-output';
        minimalLilyOutput.innerHTML = errorMsg;
        minimalLilyOutput.className = 'tab-content json-output';
        fullLilyOutput.innerHTML = errorMsg;
        fullLilyOutput.className = 'tab-content json-output';
        svgOutput.innerHTML = errorMsg;
        svgOutput.className = 'tab-content';
        if (vexflowData) vexflowData.innerHTML = errorMsg;
        console.log('📄 Updated all output tabs with network error message');
    }
}

// Check if SVG tab is currently active
function isSvgTabActive() {
    const svgTabButton = document.getElementById('svg-tab-btn');
    return svgTabButton && svgTabButton.classList.contains('active');
}

// Auto-expand textarea based on content
function autoExpandTextarea(textarea) {
    // Reset height to auto to get the correct scrollHeight
    textarea.style.height = 'auto';
    
    // Calculate the new height based on scrollHeight
    const newHeight = Math.max(60, textarea.scrollHeight); // Min height of 60px (about 3 rows)
    
    // Set the new height
    textarea.style.height = newHeight + 'px';
}

document.getElementById('input-text').addEventListener('input', function(e) {
    const inputValue = e.target.value;
    console.log('⌨️ Input event triggered:', {
        inputLength: inputValue.length,
        firstChars: inputValue.slice(0, 20),
        timestamp: new Date().toISOString(),
        svgTabActive: isSvgTabActive()
    });
    
    // Auto-expand textarea
    autoExpandTextarea(e.target);
    
    // Save input text to localStorage
    saveInputText(inputValue);
    
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
        console.log('⏰ Debounce timer triggered, calling parseInput');
        // Get the current value from textarea (which should have Unicode replacements)
        const currentValue = e.target.value;
        parseInput(currentValue);
    }, 1000); // Increased debounce to reduce API calls
    
    // If SVG tab is active, also trigger SVG generation with 3-second debounce
    if (isSvgTabActive()) {
        clearTimeout(svgDebounceTimer);
        svgDebounceTimer = setTimeout(() => {
            // console.log('🎵 SVG debounce timer triggered, generating SVG automatically');
            generateSvgFromLilypond();
        }, 3000); // 3-second debounce for SVG generation
    }
});

function drawVexFlowPlaceholder(canvas, input) {
    const ctx = canvas.getContext('2d');
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    
    // Background
    ctx.fillStyle = '#fafafa';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    
    // Staff lines
    ctx.strokeStyle = '#333';
    ctx.lineWidth = 1;
    const staffY = 100;
    const staffWidth = 500;
    const staffX = 50;
    
    for (let i = 0; i < 5; i++) {
        ctx.beginPath();
        ctx.moveTo(staffX, staffY + i * 10);
        ctx.lineTo(staffX + staffWidth, staffY + i * 10);
        ctx.stroke();
    }
    
    // Title
    ctx.fillStyle = '#333';
    ctx.font = 'bold 16px serif';
    ctx.fillText('VexFlow Notation (Demo)', 50, 30);
    
    // Input display
    ctx.fillStyle = '#666';
    ctx.font = '11px monospace';
    ctx.fillText('Input: ' + input.substring(0, 50), 50, 50);
    
    // Treble clef (simplified)
    ctx.fillStyle = '#333';
    ctx.font = '30px serif';
    ctx.fillText('𝄞', staffX + 10, staffY + 25);
    
    // Time signature
    ctx.font = '16px serif';
    ctx.fillText('4', staffX + 50, staffY + 10);
    ctx.fillText('4', staffX + 50, staffY + 25);
    
    // Notes based on input
    let noteX = staffX + 80;
    const notes = input.match(/[1-7A-G]/g) || ['1', '2', '3'];
    
    for (let i = 0; i < Math.min(notes.length, 8); i++) {
        const note = notes[i];
        let noteY = staffY + 20; // Default middle line (B)
        
        // Map notes to staff positions
        if (['1', 'C'].includes(note)) noteY = staffY + 50; // C below staff
        else if (['2', 'D'].includes(note)) noteY = staffY + 45; // D
        else if (['3', 'E'].includes(note)) noteY = staffY + 40; // E
        else if (['4', 'F'].includes(note)) noteY = staffY + 35; // F
        else if (['5', 'G'].includes(note)) noteY = staffY + 30; // G
        else if (['6', 'A'].includes(note)) noteY = staffY + 25; // A
        else if (['7', 'B'].includes(note)) noteY = staffY + 20; // B
        
        // Draw note head
        ctx.fillStyle = '#333';
        ctx.beginPath();
        ctx.ellipse(noteX, noteY, 6, 4, 0, 0, 2 * Math.PI);
        ctx.fill();
        
        // Draw stem
        ctx.beginPath();
        ctx.moveTo(noteX + 6, noteY);
        ctx.lineTo(noteX + 6, noteY - 25);
        ctx.lineWidth = 2;
        ctx.stroke();
        ctx.lineWidth = 1;
        
        // Ledger lines if needed
        if (noteY >= staffY + 45) {
            ctx.beginPath();
            ctx.moveTo(noteX - 8, staffY + 50);
            ctx.lineTo(noteX + 14, staffY + 50);
            ctx.stroke();
        }
        
        noteX += 50;
    }
    
    // Bar line
    if (input.includes('|')) {
        ctx.lineWidth = 2;
        ctx.beginPath();
        ctx.moveTo(noteX - 10, staffY);
        ctx.lineTo(noteX - 10, staffY + 40);
        ctx.stroke();
        ctx.lineWidth = 1;
    }
    
    // Footer
    ctx.fillStyle = '#999';
    ctx.font = '10px Arial';
    ctx.fillText('VexFlow-style rendering placeholder', 50, canvas.height - 20);
}

// Load VexFlow renderer dynamically
async function loadVexFlowRenderer() {
    if (window.VexFlowRenderer) return;
    
    try {
        const script = document.createElement('script');
        script.src = 'vexflow-renderer.js';
        script.async = true;
        
        return new Promise((resolve, reject) => {
            script.onload = () => {
                // console.log('✅ VexFlow renderer loaded');
                resolve();
            };
            script.onerror = () => {
                console.error('❌ Failed to load VexFlow renderer');
                reject(new Error('Failed to load VexFlow renderer'));
            };
            document.head.appendChild(script);
        });
    } catch (error) {
        console.error('🚨 Error loading VexFlow renderer:', error);
    }
}

// Initialize the application on page load
function initializeApp() {
    // Restore saved input text
    const savedInput = loadInputText();
    const inputElement = document.getElementById('input-text');
    if (savedInput && inputElement) {
        // Set value without triggering input event
        inputElement.value = savedInput;
        // Auto-expand textarea to fit restored content
        autoExpandTextarea(inputElement);
    }
    
    // Restore active tab
    restoreActiveTab();
    
    // Load VexFlow renderer
    loadVexFlowRenderer();
    
    // Only parse if there's actually saved input to avoid unnecessary API calls
    if (savedInput && savedInput.trim()) {
        parseInput(savedInput);
    }
}

// Initialize when DOM is loaded
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initializeApp);
} else {
    // DOM is already loaded
    initializeApp();
}

// SVG Generation function
async function generateSvgFromLilypond() {
    // console.log("🎵 generateSvgFromLilypond() called");
    
    // Get notation directly from input field
    const inputField = document.getElementById("input-text");
    if (!inputField || !inputField.value.trim()) {
        alert("Please enter music notation first.");
        return;
    }
    
    const notation = inputField.value.trim();
    
    // Convert Unicode characters back to standard characters for backend
    const standardNotation = convertUnicodeToStandard(notation);
    console.log('🔄 Converting Unicode to standard for SVG generation:', {
        original: notation.slice(0, 50) + (notation.length > 50 ? '...' : ''),
        converted: standardNotation.slice(0, 50) + (standardNotation.length > 50 ? '...' : ''),
        hasUnicode: notation !== standardNotation
    });
    
    // Update button state
    const button = document.getElementById("generate-svg-btn");
    const svgContent = document.getElementById("svg-content");
    
    button.disabled = true;
    button.textContent = "Generating...";
    svgContent.innerHTML = "<div class=\"text-muted\">Generating SVG from notation...</div>";
    
    try {
        const response = await fetch("/api/lilypond-svg", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                notation: standardNotation
            })
        });
        
        const result = await response.json();
        
        if (result.success && result.svg_content) {
            svgContent.innerHTML = result.svg_content;
            console.log("✅ SVG generated successfully");
        } else {
            svgContent.innerHTML = `<div class="alert alert-danger">SVG Generation Error: ${result.error || "Unknown error"}</div>`;
            console.error("❌ SVG generation failed:", result.error);
        }
    } catch (error) {
        console.error("🚨 Network error during SVG generation:", error);
        svgContent.innerHTML = `<div class="alert alert-danger">Network Error: ${error.message}</div>`;
    } finally {
        button.disabled = false;
        button.textContent = "Generate SVG";
    }
}
