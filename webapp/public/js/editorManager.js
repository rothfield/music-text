/**
 * Editor Manager Module
 * Handles code editor initialization and integration with existing app
 */

import { UI } from './ui.js';

export class EditorManager {
    constructor() {
        this.editor = null;
        this.container = null;
        this.tooltip = null;
        this.initTooltip();
    }

    // Initialize CodeMirror editor
    init(containerId = 'musicInput') {
        this.container = document.getElementById(containerId);
        if (!this.container) {
            throw new Error(`Container element '${containerId}' not found`);
        }

        // Check if CodeMirror is available
        if (!window.CodeMirror) {
            throw new Error('CodeMirror library not loaded');
        }

        // Create editor instance
        this.editor = window.CodeMirror(this.container, {
            mode: 'text/plain', // Start with plain text, we'll add custom mode later
            lineNumbers: false,
            lineWrapping: true,
            theme: 'default',
            placeholder: 'Enter music notation like: |S R G M|',
            autofocus: true,
            viewportMargin: Infinity, // Auto-grow height
            extraKeys: {
                // Tab to insert actual tab character
                'Tab': function(cm) {
                    cm.replaceSelection('\t');
                }
            }
        });

        // Set initial size - let CSS handle minimum height
        this.editor.setSize(null, 'auto');

        // Add textarea-like interface methods to the container for compatibility
        this.addCompatibilityMethods();

        // Add Sargam notation key handlers
        this.editor.on('keydown', function(cm, event) {
            // Handle Sargam notation keys that should be auto-capitalized
            if ((event.key === 's' || event.key === 'p') && !event.ctrlKey && !event.altKey && !event.metaKey) {
                event.preventDefault();
                const char = event.key;
                const upperChar = char.toUpperCase();

                if (EditorManager.shouldConvertToSargamUppercase(cm)) {
                    cm.replaceSelection(upperChar);
                } else {
                    cm.replaceSelection(char);
                }
                return;
            }
        });

        console.log('✅ Editor initialized with Sargam key handlers');

        // Set up tooltip event handling after editor is ready
        setTimeout(() => this.setupTooltipHandlers(), 100);

        return this.editor;
    }

    // Initialize tooltip element
    initTooltip() {
        this.tooltip = document.createElement('div');
        this.tooltip.className = 'music-tooltip';
        this.tooltip.style.cssText = `
            position: absolute;
            background: rgba(0, 0, 0, 0.9);
            color: white;
            padding: 6px 10px;
            border-radius: 6px;
            font-size: 13px;
            font-weight: normal;
            white-space: nowrap;
            z-index: 10000;
            pointer-events: none;
            opacity: 0;
            transition: opacity 0.2s ease-in-out;
            box-shadow: 0 2px 8px rgba(0,0,0,0.3);
            border: 1px solid rgba(255,255,255,0.2);
        `;
        document.body.appendChild(this.tooltip);
    }

    // Set up tooltip event handlers
    setupTooltipHandlers() {
        if (!this.editor) return;

        const editorElement = this.editor.getWrapperElement();
        let tooltipTimeout = null;

        // Mouse enter handler with minimal delay
        editorElement.addEventListener('mouseenter', (e) => {
            if (e.target.matches('.cm-music-note[data-tooltip]')) {
                // Clear any existing timeout
                if (tooltipTimeout) {
                    clearTimeout(tooltipTimeout);
                }
                // Small delay to avoid interfering with quick cursor movements
                tooltipTimeout = setTimeout(() => {
                    this.showTooltip(e.target, e.target.dataset.tooltip, e);
                }, 150); // Reduced to 150ms
            }
        }, true);

        // Mouse leave handler
        editorElement.addEventListener('mouseleave', (e) => {
            if (e.target.matches('.cm-music-note[data-tooltip]')) {
                // Clear timeout if mouse leaves before tooltip shows
                if (tooltipTimeout) {
                    clearTimeout(tooltipTimeout);
                    tooltipTimeout = null;
                }
                this.hideTooltip();
            }
        }, true);

        // Hide tooltip on any interaction to avoid cursor interference
        const hideTooltipOnInteraction = () => {
            if (tooltipTimeout) {
                clearTimeout(tooltipTimeout);
                tooltipTimeout = null;
            }
            this.hideTooltip();
        };

        editorElement.addEventListener('click', hideTooltipOnInteraction);
        editorElement.addEventListener('keydown', hideTooltipOnInteraction);
        editorElement.addEventListener('focus', hideTooltipOnInteraction);
        editorElement.addEventListener('scroll', hideTooltipOnInteraction);
    }

    // Show tooltip
    showTooltip(element, text, event) {
        if (!text) return;

        this.tooltip.textContent = text;
        this.tooltip.style.opacity = '1';
        this.tooltip.style.display = 'block';

        if (event) {
            this.positionTooltip(event);
        }
    }

    // Hide tooltip
    hideTooltip() {
        this.tooltip.style.opacity = '0';
        setTimeout(() => {
            if (this.tooltip.style.opacity === '0') {
                this.tooltip.style.display = 'none';
            }
        }, 200);
    }

    // Position tooltip relative to mouse/element
    positionTooltip(event) {
        const rect = event.target.getBoundingClientRect();
        const tooltipRect = this.tooltip.getBoundingClientRect();

        // Position above the element, centered
        let left = rect.left + (rect.width / 2) - (tooltipRect.width / 2);
        let top = rect.top - tooltipRect.height - 8;

        // Ensure tooltip stays within viewport
        const margin = 10;
        if (left < margin) {
            left = margin;
        } else if (left + tooltipRect.width > window.innerWidth - margin) {
            left = window.innerWidth - tooltipRect.width - margin;
        }

        if (top < margin) {
            // If no room above, show below
            top = rect.bottom + 8;
        }

        this.tooltip.style.left = left + 'px';
        this.tooltip.style.top = top + 'px';
    }

    // Add textarea-compatible methods to container element
    addCompatibilityMethods() {
        const container = this.container;
        const editor = this.editor;

        // Add value getter/setter
        Object.defineProperty(container, 'value', {
            get: () => editor.getValue(),
            set: (value) => {
                const cursor = editor.getCursor();
                editor.setValue(value);
                editor.setCursor(cursor);
            }
        });

        // Add selection methods
        Object.defineProperty(container, 'selectionStart', {
            get: () => {
                const cursor = editor.getCursor('from');
                return editor.indexFromPos(cursor);
            }
        });

        Object.defineProperty(container, 'selectionEnd', {
            get: () => {
                const cursor = editor.getCursor('to');
                return editor.indexFromPos(cursor);
            }
        });

        // Add setSelectionRange method
        container.setSelectionRange = (start, end) => {
            const startPos = editor.posFromIndex(start);
            const endPos = editor.posFromIndex(end);
            editor.setSelection(startPos, endPos);
        };

        // Add focus method
        container.focus = () => {
            editor.focus();
        };

        // Add event listener compatibility
        const originalAddEventListener = container.addEventListener;
        container.addEventListener = function(type, listener, options) {
            if (type === 'input') {
                // Map input events from editor
                editor.on('change', (instance, changeObj) => {
                    // Create a synthetic event object
                    const syntheticEvent = {
                        target: container,
                        type: 'input'
                    };
                    listener(syntheticEvent);
                });
            } else if (type === 'keyup' || type === 'mouseup' || type === 'click') {
                // Map cursor position events
                editor.on('cursorActivity', () => {
                    const syntheticEvent = {
                        target: container,
                        type: type
                    };
                    listener(syntheticEvent);
                });
            } else {
                // Fall back to original addEventListener for other events
                originalAddEventListener.call(this, type, listener, options);
            }
        };
    }

    // Highlight lines based on parse results
    highlightFromParseResult(parseResult) {
        if (!this.editor || !parseResult || !parseResult.success) return;
        
        const editor = this.editor;
        
        // Clear all existing line classes
        const lineCount = editor.lineCount();
        for (let i = 0; i < lineCount; i++) {
            const lineHandle = editor.getLineHandle(i);
            editor.removeLineClass(lineHandle, 'background', 'content-line');
            editor.removeLineClass(lineHandle, 'background', 'directive-line');
            editor.removeLineClass(lineHandle, 'background', 'lyrics-line');
            editor.removeLineClass(lineHandle, 'background', 'text-line');
        }
        
        // Highlight directive lines (Title, Composer, etc)
        if (parseResult.parsed_document && parseResult.parsed_document.directives) {
            parseResult.parsed_document.directives.forEach(directive => {
                if (directive.source && directive.source.position) {
                    const lineNum = directive.source.position.line - 1; // Convert to 0-based
                    if (lineNum >= 0 && lineNum < lineCount) {
                        const lineHandle = editor.getLineHandle(lineNum);
                        editor.addLineClass(lineHandle, 'background', 'directive-line');
                    }
                }
            });
        }
        
        // Highlight content lines and lyrics from staves
        if (parseResult.parsed_document && parseResult.parsed_document.staves) {
            parseResult.parsed_document.staves.forEach(stave => {
                // Content line (music notation)
                if (stave.source && stave.source.position) {
                    const contentLineNum = stave.source.position.line - 1; // Convert to 0-based
                    if (contentLineNum >= 0 && contentLineNum < lineCount) {
                        const lineHandle = editor.getLineHandle(contentLineNum);
                        editor.addLineClass(lineHandle, 'background', 'content-line');
                    }
                }
                
                // Lyrics lines
                if (stave.lyrics_lines) {
                    stave.lyrics_lines.forEach(lyricsLine => {
                        if (lyricsLine.source && lyricsLine.source.position) {
                            const lyricsLineNum = lyricsLine.source.position.line - 1; // Convert to 0-based
                            if (lyricsLineNum >= 0 && lyricsLineNum < lineCount) {
                                const lineHandle = editor.getLineHandle(lyricsLineNum);
                                editor.addLineClass(lineHandle, 'background', 'lyrics-line');
                            }
                        }
                    });
                }
            });
        }
    }

    // Get the CodeMirror instance
    getEditor() {
        return this.editor;
    }

    // Set editor content
    setValue(content) {
        if (this.editor) {
            this.editor.setValue(content);
        }
    }

    // Get editor content
    getValue() {
        return this.editor ? this.editor.getValue() : '';
    }

    // Focus the editor
    focus() {
        if (this.editor) {
            this.editor.focus();
        }
    }

    // Set cursor position
    setCursor(line, ch) {
        if (this.editor) {
            this.editor.setCursor(line, ch);
        }
    }

    // Set selection range
    setSelection(start, end) {
        if (this.editor) {
            const startPos = this.editor.posFromIndex(start);
            const endPos = this.editor.posFromIndex(end);
            this.editor.setSelection(startPos, endPos);
        }
    }

    // Get cursor position
    getCursorPosition() {
        if (!this.editor) return { start: 0, end: 0 };
        
        const from = this.editor.getCursor('from');
        const to = this.editor.getCursor('to');
        
        return {
            start: this.editor.indexFromPos(from),
            end: this.editor.indexFromPos(to)
        };
    }


    // Return to original input content
    returnToOriginal() {
        if (!this.editor || !this._originalContent) return;
        
        // Switch back to plain text mode
        this.editor.setOption('mode', 'text/plain');
        
        // Restore original content
        this.editor.setValue(this._originalContent);
        
        // Remove visual indicators
        this.container.style.border = '';
        this.container.style.backgroundColor = '';
        this.container.style.cursor = '';
        this.container.onclick = null;
        
        // Clear stored content
        this._originalContent = null;
    }

    // Apply syntax highlighting using span styles from server
    applySpanStyles(spanStyles, parseResult = null) {
        if (!this.editor || !spanStyles) return;

        // Clear existing marks
        this.clearAllMarks();

        // Apply all styles from backend
        spanStyles.forEach((spanSpec) => {
            const pos = this.editor.posFromIndex(spanSpec.pos);
            const endPos = this.editor.posFromIndex(spanSpec.pos + spanSpec.length);

            // Build CSS string from styles object if it exists
            let cssString = '';
            if (spanSpec.styles) {
                cssString = Object.entries(spanSpec.styles)
                    .map(([key, value]) => `${key}: ${value}`)
                    .join('; ');
                if (cssString) cssString += '; ';
            }

            // Build markText options
            const markOptions = {
                className: spanSpec.classes.join(' '),
                css: cssString
            };

            // Add tooltip data attribute if title is available
            if (spanSpec.title) {
                markOptions.attributes = {
                    'data-tooltip': spanSpec.title
                };
            }

            this.editor.markText(pos, endPos, markOptions);
        });

        // Apply beat group styling if parse result available
        if (parseResult) {
            this.applyBeatGroupStyling(parseResult);
        }

        console.log('✅ Applied span styling for', spanStyles.length, 'positions');
    }


    // Set dynamic width using CSS custom properties
    setDynamicWidth(element, property) {
        // Calculate width based on surrounding elements or use a default formula
        const baseWidth = 1.2; // em
        const elementCount = this.getElementCount(element);
        const calculatedWidth = baseWidth * elementCount;

        element.style.setProperty(`--${property}`, `${calculatedWidth}em`);
    }

    // Helper to determine element count for width calculation
    getElementCount(element) {
        // This is a simplified approach - in practice you'd want to
        // calculate based on the actual span of the grouping
        return 2; // Default for now
    }

    // Apply beat group styling with precise width calculation
    applyBeatGroupStyling(parseResult) {
        if (!this.editor || !parseResult.rhythm_analyzed_document) return;

        const doc = parseResult.rhythm_analyzed_document;
        if (!doc.elements) return;

        doc.elements.forEach(element => {
            if (element.Stave && element.Stave.lines) {
                element.Stave.lines.forEach(line => {
                    if (line.Content) {
                        this.processBeatGroupsInLine(line.Content);
                    }
                });
            }
        });
    }

    // Process beat groups in a content line
    processBeatGroupsInLine(contentElements) {
        let beatGroupStart = null;
        let beatGroupElements = [];

        contentElements.forEach((element, index) => {
            if (element.Note && element.Note.in_beat_group) {
                const role = element.Note.beat_group;

                if (role === 'Start') {
                    beatGroupStart = element.Note.position;
                    beatGroupElements = [element];
                } else if (role === 'Middle' && beatGroupElements.length > 0) {
                    beatGroupElements.push(element);
                } else if (role === 'End' && beatGroupElements.length > 0) {
                    beatGroupElements.push(element);

                    // Apply beat group styling
                    this.applyBeatGroupArc(beatGroupStart, element.Note.position, beatGroupElements.length);

                    // Reset for next beat group
                    beatGroupStart = null;
                    beatGroupElements = [];
                }
            }
        });
    }

    // Apply beat group arc with calculated width
    applyBeatGroupArc(startPos, endPos, elementCount) {
        if (!startPos || !endPos) return;

        // Convert positions to editor coordinates
        const startCmPos = { line: startPos.row, ch: startPos.col - 1 }; // Convert to 0-based
        const endCmPos = { line: endPos.row, ch: endPos.col }; // End after the character

        // Get DOM coordinates for width calculation
        const startCoords = this.editor.charCoords(startCmPos, 'local');
        const endCoords = this.editor.charCoords(endCmPos, 'local');
        const arcWidth = Math.max(20, endCoords.left - startCoords.left); // Minimum 20px

        // Apply mark with dynamic width
        const mark = this.editor.markText(startCmPos, { line: startPos.row, ch: startPos.col }, {
            className: `beat-group-start beat-group-${elementCount}`,
            attributes: {
                'data-arc-width': arcWidth + 'px'
            }
        });

        // Apply CSS custom property for precise width
        setTimeout(() => {
            const markElement = mark.find()?.mark?.element;
            if (markElement) {
                markElement.style.setProperty('--beat-group-width', arcWidth + 'px');
            }
        }, 0);
    }

    // Clear all existing marks
    clearAllMarks() {
        if (!this.editor) return;

        const marks = this.editor.getAllMarks();
        marks.forEach(mark => mark.clear());
    }

    // Apply syntax highlighting using spans from server (legacy method)
    applySyntaxSpans(spans) {
        if (!this.editor || !spans) return;

        // Create custom mode based on the spans
        const customMode = this.createSpanBasedMode(spans);

        // Define the mode with editor
        window.CodeMirror.defineMode("music-syntax", function() {
            return customMode;
        });

        // Apply the custom mode
        this.editor.setOption('mode', 'music-syntax');

        console.log('✅ Applied syntax highlighting with', spans.length, 'spans');
    }

    // Create an editor mode based on syntax spans
    createSpanBasedMode(spans) {
        // Pre-calculate line offsets from editor content
        const editorContent = this.editor.getValue();
        const lines = editorContent.split('\n');
        const lineOffsets = [0];
        let offset = 0;
        for (let i = 0; i < lines.length - 1; i++) {
            offset += lines[i].length + 1; // +1 for newline character
            lineOffsets.push(offset);
        }

        return {
            token: function(stream, state) {
                // Initialize line tracking in state
                if (state.lineNumber === undefined) {
                    state.lineNumber = 0;
                }

                // Check if we're at the start of a new line
                if (stream.pos === 0 && stream.sol()) {
                    // Update line number only when we're at start of line
                    if (state.hasSeenLine) {
                        state.lineNumber++;
                    }
                    state.hasSeenLine = true;
                }

                // Calculate absolute position: line offset + stream position
                const lineOffset = lineOffsets[state.lineNumber] || 0;
                const absolutePos = lineOffset + stream.pos;
                const span = spans.find(t => absolutePos >= t.start && absolutePos < t.end);

                // Removed console logging for cleaner output

                if (span) {
                    // Consume one character
                    stream.next();
                    return `music-${span.type}`;
                }

                // Fallback - consume one character
                stream.next();
                return null;
            },
            startState: function() {
                return {};
            }
        };
    }

    // Check if text has at least one pitch or dash
    hasMoreThanOnePitchOrDash(text) {
        const pitches = text.match(/[SRGMPDNsrgmpdnCDEFGABcdefgab]/g) || [];
        const dashes = text.match(/-/g) || [];
        return (pitches.length + dashes.length) > 1;
    }

    // Static method to determine if s/p should be converted to S/P for Sargam notation
    static shouldConvertToSargamUppercase(cm) {
        const cursor = cm.getCursor();
        const lineText = cm.getLine(cursor.line);

        console.log('🔍 Sargam Detection Debug:');
        console.log('  Line text:', JSON.stringify(lineText));
        console.log('  Cursor position:', cursor);

        // Check if we're in a content line (contains music notation)
        const isContent = EditorManager.isContentLine(lineText);
        console.log('  Is content line:', isContent);
        if (!isContent) {
            return false;
        }

        // Detect notation system from existing content in the line
        const notationSystem = EditorManager.detectNotationSystem(lineText);
        console.log('  Detected notation system:', notationSystem);
        const shouldConvert = notationSystem === 'sargam';
        console.log('  Should convert to uppercase:', shouldConvert);

        return shouldConvert;
    }

    // Detect if a line is a content line (contains music notation)
    static isContentLine(lineText) {
        // Content lines typically contain:
        // - Barlines (|)
        // - Musical notes/pitches
        // - Dashes (-)
        // - Breath marks (')
        // Exclude directive lines (key: value format)
        if (lineText.includes(':') && !lineText.includes('|')) {
            return false; // Likely a directive line
        }

        // Look for musical content indicators
        const musicalIndicators = /[|\-'SRGMPDNsrgmpdnCDEFGAB1-7]/;
        return musicalIndicators.test(lineText);
    }

    // Detect notation system from line content
    static detectNotationSystem(lineText) {
        // Remove whitespace and barlines for analysis
        const content = lineText.replace(/[|\s\-']/g, '');
        console.log('    Cleaned content for analysis:', JSON.stringify(content));

        if (content.length === 0) {
            console.log('    Empty content - returning unknown');
            return 'unknown';
        }

        // Count occurrences of different notation systems
        const sargamChars = content.match(/[SRGMPDNsrgmpdn]/g) || [];
        const westernChars = content.match(/[CDEFGAB]/g) || [];
        const numberChars = content.match(/[1-7]/g) || [];

        console.log('    Sargam chars found:', sargamChars, 'count:', sargamChars.length);
        console.log('    Western chars found:', westernChars, 'count:', westernChars.length);
        console.log('    Number chars found:', numberChars, 'count:', numberChars.length);

        // Determine dominant system
        if (sargamChars.length > westernChars.length && sargamChars.length > numberChars.length) {
            console.log('    Detected: sargam (dominant)');
            return 'sargam';
        } else if (westernChars.length > numberChars.length) {
            console.log('    Detected: western (dominant)');
            return 'western';
        } else if (numberChars.length > 0) {
            console.log('    Detected: number (has numbers)');
            return 'number';
        }

        console.log('    Detected: unknown (no clear system)');
        return 'unknown';
    }

    // Toggle slur functionality for selected text
    toggleSlur() {
        if (!this.editor) {
            console.warn('Editor not initialized');
            return;
        }

        const selection = this.editor.getSelection();

        if (!selection || selection.trim() === '') {
            UI.setStatus('Please select text to add a slur', 'error');
            return;
        }

        // Check if selection has more than one pitch or dash
        if (!this.hasMoreThanOnePitchOrDash(selection)) {
            UI.setStatus('Selection must contain more than one pitch or dash', 'error');
            return;
        }

        const from = this.editor.getCursor('from');
        const to = this.editor.getCursor('to');

        // Check if selection already has slur marks
        const existingMarks = this.editor.findMarksAt(from).concat(this.editor.findMarksAt(to));
        const hasSlur = existingMarks.some(mark =>
            mark.className && (mark.className.includes('slur-start') || mark.className.includes('slur-end'))
        );

        if (hasSlur) {
            // Remove existing slur marks
            existingMarks.forEach(mark => {
                if (mark.className && (mark.className.includes('slur-start') || mark.className.includes('slur-end'))) {
                    mark.clear();
                }
            });
        } else {
            // Add new slur marks
            this.editor.markText(from, {line: from.line, ch: from.ch + 1}, {
                className: 'slur-start',
                title: 'Slur start'
            });

            this.editor.markText({line: to.line, ch: to.ch - 1}, to, {
                className: 'slur-end',
                title: 'Slur end'
            });
        }
    }
}