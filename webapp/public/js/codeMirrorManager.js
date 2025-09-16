/**
 * CodeMirror Manager Module
 * Handles CodeMirror editor initialization and integration with existing app
 */

export class CodeMirrorManager {
    constructor() {
        this.editor = null;
        this.container = null;
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

        // Create CodeMirror instance
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

        console.log('✅ CodeMirror editor initialized');
        return this.editor;
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
                // Map input events from CodeMirror
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

    // Display XML content in the editor
    displayXML(xmlContent, originalContent = null) {
        if (!this.editor) return;
        
        // Store original content if provided
        if (originalContent !== null) {
            this._originalContent = originalContent;
        }
        
        // Try to switch to XML mode if available, otherwise stay in text mode
        try {
            this.editor.setOption('mode', 'xml');
        } catch (e) {
            console.warn('XML mode not available, using text mode');
        }
        
        // Set XML content
        this.editor.setValue(xmlContent);
        
        // Add a visual indicator that we're in XML mode
        this.container.style.border = '2px solid #4CAF50';
        this.container.style.backgroundColor = '#f8f9fa';
        this.container.title = 'Displaying XML representation - click to return to original input';
        
        // Make it clickable to return to original
        this.container.style.cursor = 'pointer';
        this.container.onclick = () => this.returnToOriginal();
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
        this.container.title = '';
        this.container.style.cursor = '';
        this.container.onclick = null;
        
        // Clear stored content
        this._originalContent = null;
    }

    // Apply syntax highlighting using character styles from server
    applyCharacterStyles(characterStyles, parseResult = null) {
        if (!this.editor || !characterStyles) return;

        // Clear existing marks
        this.clearAllMarks();

        // Apply each character style using CodeMirror marks
        characterStyles.forEach(style => {
            const pos = this.editor.posFromIndex(style.pos);
            const endPos = this.editor.posFromIndex(style.pos + 1);

            this.editor.markText(pos, endPos, {
                className: style.classes.join(' ')
            });
        });

        // Apply beat group styling if parse result available
        if (parseResult) {
            this.applyBeatGroupStyling(parseResult);
        }

        console.log('✅ Applied character styling for', characterStyles.length, 'positions');
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

        // Convert positions to CodeMirror coordinates
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

    // Apply syntax highlighting using tokens from server (legacy method)
    applySyntaxTokens(tokens) {
        if (!this.editor || !tokens) return;

        // Create custom mode based on the tokens
        const customMode = this.createTokenBasedMode(tokens);

        // Define the mode with CodeMirror
        window.CodeMirror.defineMode("music-syntax", function() {
            return customMode;
        });

        // Apply the custom mode
        this.editor.setOption('mode', 'music-syntax');

        console.log('✅ Applied syntax highlighting with', tokens.length, 'tokens');
    }

    // Create a CodeMirror mode based on syntax tokens
    createTokenBasedMode(tokens) {
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
                const token = tokens.find(t => absolutePos >= t.start && absolutePos < t.end);

                // Removed console logging for cleaner output

                if (token) {
                    // Consume one character
                    stream.next();
                    return `music-${token.token_type}`;
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
}