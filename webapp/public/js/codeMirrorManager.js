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
}