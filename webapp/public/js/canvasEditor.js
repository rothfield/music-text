/**
 * Canvas Editor Module - WYSIWYG Music Notation Editor
 * Canvas-based visual and text editor for music notation
 */

export class CanvasEditor {
    constructor() {
        this.canvas = null;
        this.ctx = null;
        this.textInput = null;
        this.currentMode = 'text'; // 'text' or 'visual'
        this.parseResult = null;
        this.cursor = { line: 0, char: 0 };
        this.lines = [''];
        this.isDirty = false;
        this.textContent = ''; // Store the current text content
        this.cursorPosition = 0; // Current cursor position in text
        this.selection = { start: 0, end: 0 }; // Selection range (position-based)
        this.cursorVisible = true; // For blinking cursor
        this.cursorBlinkInterval = null;

        // Selection state for mouse interaction
        this.isSelecting = false;
        this.selectionStart = 0;

        // Visual elements
        this.noteElements = [];
        this.selectedElement = null;

        // Event handlers
        this.onContentChange = null;
        this.onSelectionChange = null;
    }

    // Initialize the canvas editor
    init(containerId = 'canvasEditor') {
        const container = document.getElementById(containerId);
        if (!container) {
            throw new Error(`Container element '${containerId}' not found`);
        }

        this.canvas = document.getElementById('musicCanvas');

        if (!this.canvas) {
            throw new Error('Canvas element not found');
        }

        this.ctx = this.canvas.getContext('2d');
        this.setupEventListeners();
        this.setupToolbarListeners();

        // Set initial mode to text
        this.switchToTextMode();

        // Initialize canvas with placeholder message
        this.clearCanvas();

        // Start cursor blinking
        this.startCursorBlink();

        console.log('✅ Canvas Editor initialized');
        return this;
    }

    // Setup event listeners for canvas
    setupEventListeners() {
        // Make canvas focusable and handle keyboard input
        this.canvas.setAttribute('tabindex', '0');
        this.canvas.addEventListener('keydown', (e) => {
            this.handleKeyDown(e);
        });

        this.canvas.addEventListener('keypress', (e) => {
            this.handleKeyPress(e);
        });

        // Canvas events for text selection
        this.canvas.addEventListener('mousedown', (e) => {
            this.canvas.focus(); // Focus canvas on click
            this.handleCanvasMouseDown(e);
        });

        this.canvas.addEventListener('mousemove', (e) => {
            this.handleCanvasMouseMove(e);
        });

        this.canvas.addEventListener('mouseup', (e) => {
            this.handleCanvasMouseUp(e);
        });

        // Handle mouse leaving canvas during selection
        this.canvas.addEventListener('mouseleave', (e) => {
            this.handleCanvasMouseUp(e);
        });
    }

    // Setup toolbar event listeners
    setupToolbarListeners() {
        const textModeBtn = document.getElementById('textMode');
        const visualModeBtn = document.getElementById('visualMode');
        const insertNoteBtn = document.getElementById('insertNote');
        const insertBarlineBtn = document.getElementById('insertBarline');
        const insertGraceNoteBtn = document.getElementById('insertGraceNote');

        textModeBtn?.addEventListener('click', () => this.switchToTextMode());
        visualModeBtn?.addEventListener('click', () => this.switchToVisualMode());
        insertNoteBtn?.addEventListener('click', () => this.insertNote());
        insertBarlineBtn?.addEventListener('click', () => this.insertBarline());
        insertGraceNoteBtn?.addEventListener('click', () => this.insertGraceNote());
    }

    // Switch to text editing mode
    switchToTextMode() {
        this.currentMode = 'text';
        document.getElementById('canvasEditor').className = 'canvas-editor text-mode';
        document.getElementById('textMode').classList.add('active');
        document.getElementById('visualMode').classList.remove('active');
        this.canvas.focus();
        this.render();
    }

    // Switch to visual editing mode
    switchToVisualMode() {
        this.currentMode = 'visual';
        document.getElementById('canvasEditor').className = 'canvas-editor visual-mode';
        document.getElementById('textMode').classList.remove('active');
        document.getElementById('visualMode').classList.add('active');
        this.canvas.focus();
        this.render();
    }

    // Handle key events
    handleKeyDown(e) {
        if (e.key === 'Backspace') {
            e.preventDefault();
            this.handleBackspace();
        } else if (e.key === 'Delete') {
            e.preventDefault();
            this.handleDelete();
        } else if (e.key === 'ArrowLeft' || e.key === 'ArrowRight') {
            e.preventDefault();
            this.moveCursor(e.key === 'ArrowLeft' ? -1 : 1);
        } else if (e.key === 'Escape') {
            this.selectedElement = null;
            this.render();
        }

        // Handle other special keys for music notation
        if (e.ctrlKey || e.metaKey) {
            switch (e.key) {
                case 'l':
                    e.preventDefault();
                    this.applyOctaveAdjustment('lower-middle');
                    break;
                case 'ArrowUp':
                    e.preventDefault();
                    this.applyOctaveAdjustment('upper-middle');
                    break;
                case 'm':
                    e.preventDefault();
                    this.applyOctaveAdjustment('all-middle');
                    break;
            }
        }
    }

    // Handle key press events for text input
    handleKeyPress(e) {
        if (e.ctrlKey || e.metaKey || e.key === 'Escape') {
            return; // Ignore special keys
        }

        e.preventDefault();
        this.insertCharacter(e.key);
    }

    // Handle mouse down for starting text selection
    handleCanvasMouseDown(e) {
        const rect = this.canvas.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;

        // Calculate cursor position from click coordinates
        const clickPosition = this.calculateCursorPositionFromClick(x, y);
        if (clickPosition !== null) {
            // Start selection
            this.isSelecting = true;
            this.selectionStart = clickPosition;
            this.cursorPosition = clickPosition;
            this.selection.start = clickPosition;
            this.selection.end = clickPosition;
            this.resetCursorBlink();

            // Re-render with new cursor position
            this.submitToServer(this.textContent);
        }

        // Prevent text selection outside canvas
        e.preventDefault();
    }

    // Handle mouse up for ending text selection
    handleCanvasMouseUp(e) {
        if (this.isSelecting) {
            this.isSelecting = false;

            // Notify selection change if there's a callback
            if (this.onSelectionChange) {
                this.onSelectionChange({
                    start: this.selection.start,
                    end: this.selection.end
                });
            }
        }
    }

    // Calculate cursor position from click coordinates
    calculateCursorPositionFromClick(x, y) {
        if (!this.textContent) return 0;

        // Get the actual canvas display dimensions vs internal dimensions
        const rect = this.canvas.getBoundingClientRect();
        const scaleX = this.canvas.width / rect.width;
        const scaleY = this.canvas.height / rect.height;

        // Scale click coordinates to match internal canvas coordinates
        const scaledX = x * scaleX;
        const scaledY = y * scaleY;

        // SVG rendering parameters (these should match the SVG renderer)
        const charWidth = 12; // Character width based on font size
        const lineHeight = 60; // Line spacing from SVG renderer
        const leftMargin = 20; // Left margin from SVG transform translate(20, 60)
        const topMargin = 60;  // Top margin from SVG transform translate(20, 60)

        // Calculate which line was clicked (using scaled SVG coordinates)
        let lineIndex = Math.floor((scaledY - topMargin) / lineHeight);
        lineIndex = Math.max(0, lineIndex);

        // Calculate character position within line
        let charIndex = Math.floor((scaledX - leftMargin) / charWidth);
        charIndex = Math.max(0, charIndex);

        // Convert line and character index to absolute cursor position
        const lines = this.textContent.split('\n');
        let cursorPosition = 0;

        // Add characters from previous lines
        for (let i = 0; i < Math.min(lineIndex, lines.length); i++) {
            cursorPosition += lines[i].length + 1; // +1 for newline
        }

        // Add characters within the current line
        if (lineIndex < lines.length) {
            const currentLine = lines[lineIndex];
            cursorPosition += Math.min(charIndex, currentLine.length);
        }

        console.log(`Click: display(${x.toFixed(1)}, ${y.toFixed(1)}) -> scaled(${scaledX.toFixed(1)}, ${scaledY.toFixed(1)}) -> line ${lineIndex}, char ${charIndex} -> cursor ${cursorPosition}`);

        // Ensure cursor position is within bounds
        return Math.min(cursorPosition, this.textContent.length);
    }

    // Handle canvas mouse move events for hover and text selection
    handleCanvasMouseMove(e) {
        const rect = this.canvas.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;

        // Handle text selection during drag
        if (this.isSelecting) {
            const dragPosition = this.calculateCursorPositionFromClick(x, y);
            if (dragPosition !== null) {
                // Update selection range
                this.selection.start = Math.min(this.selectionStart, dragPosition);
                this.selection.end = Math.max(this.selectionStart, dragPosition);
                this.cursorPosition = dragPosition;

                // Re-render with updated selection (but don't update server constantly during drag)
                // TODO: Add visual selection highlighting in SVG
                console.log(`Selection: ${this.selection.start} to ${this.selection.end}`);
            }
        }

        // Handle visual element hover (existing functionality)
        const hoveredElement = this.findElementAt(x, y);
        this.canvas.style.cursor = this.isSelecting ? 'text' : (hoveredElement ? 'pointer' : 'text');
    }

    // Insert character at cursor position
    insertCharacter(char) {
        this.textContent = this.textContent.slice(0, this.cursorPosition) + char + this.textContent.slice(this.cursorPosition);
        this.cursorPosition++;
        this.selection.start = this.cursorPosition;
        this.selection.end = this.cursorPosition;
        this.lines = this.textContent.split('\n');
        this.isDirty = true;

        // Reset cursor blink
        this.resetCursorBlink();

        // Submit to server for real-time canvas SVG generation
        this.submitToServer(this.textContent);

        if (this.onContentChange) {
            this.onContentChange(this.textContent);
        }
    }

    // Handle backspace
    handleBackspace() {
        if (this.cursorPosition > 0) {
            this.textContent = this.textContent.slice(0, this.cursorPosition - 1) + this.textContent.slice(this.cursorPosition);
            this.cursorPosition--;
            this.selection.start = this.cursorPosition;
            this.selection.end = this.cursorPosition;
            this.lines = this.textContent.split('\n');
            this.isDirty = true;

            // Reset cursor blink
            this.resetCursorBlink();

            // Submit to server for real-time canvas SVG generation
            this.submitToServer(this.textContent);

            if (this.onContentChange) {
                this.onContentChange(this.textContent);
            }
        }
    }

    // Handle delete
    handleDelete() {
        if (this.cursorPosition < this.textContent.length) {
            this.textContent = this.textContent.slice(0, this.cursorPosition) + this.textContent.slice(this.cursorPosition + 1);
            this.selection.start = this.cursorPosition;
            this.selection.end = this.cursorPosition;
            this.lines = this.textContent.split('\n');
            this.isDirty = true;

            // Reset cursor blink
            this.resetCursorBlink();

            // Submit to server for real-time canvas SVG generation
            this.submitToServer(this.textContent);

            if (this.onContentChange) {
                this.onContentChange(this.textContent);
            }
        }
    }

    // Move cursor
    moveCursor(direction) {
        this.cursorPosition = Math.max(0, Math.min(this.textContent.length, this.cursorPosition + direction));
        this.selection.start = this.cursorPosition;
        this.selection.end = this.cursorPosition;
        this.resetCursorBlink();
    }

    // Start cursor blinking
    startCursorBlink() {
        if (this.cursorBlinkInterval) {
            clearInterval(this.cursorBlinkInterval);
        }

        this.cursorVisible = true;
        this.cursorBlinkInterval = setInterval(() => {
            this.cursorVisible = !this.cursorVisible;
            this.renderCursor();
        }, 530); // Standard cursor blink rate
    }

    // Stop cursor blinking
    stopCursorBlink() {
        if (this.cursorBlinkInterval) {
            clearInterval(this.cursorBlinkInterval);
            this.cursorBlinkInterval = null;
        }
        this.cursorVisible = false;
    }

    // Reset cursor blink (show cursor immediately and restart blink cycle)
    resetCursorBlink() {
        this.cursorVisible = true;
        this.renderCursor();
        this.startCursorBlink();
    }

    // Submit text to server for canvas SVG generation
    async submitToServer(inputText) {
        if (!inputText.trim()) {
            this.clearCanvas();
            return;
        }

        try {
            // Use the new canvas SVG API with cursor position support
            const response = await fetch('/api/canvas-svg', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    input_text: inputText,
                    notation_type: this.detectNotationSystem(inputText),  // Auto-detect notation system
                    cursor_position: this.cursorPosition,
                    selection_start: this.selection.start !== this.selection.end ? this.selection.start : null,
                    selection_end: this.selection.start !== this.selection.end ? this.selection.end : null
                })
            });

            if (response.ok) {
                const svgContent = await response.text();
                this.renderCanvasSvg(svgContent);

                // Also call the old parse endpoint for UI tab updates
                const parseResponse = await fetch(`/api/parse?input=${encodeURIComponent(inputText)}`);
                const parseResult = await parseResponse.json();
                this.updateParseResult(parseResult);

                // Update all UI tabs
                if (window.UI) {
                    if (window.UI.updatePipelineData) {
                        window.UI.updatePipelineData(parseResult);
                    }
                    if (window.UI.updateLilyPondOutput) {
                        window.UI.updateLilyPondOutput(parseResult);
                    }
                    if (window.UI.updateSourceOutput) {
                        window.UI.updateSourceOutput(parseResult);
                    }
                    if (window.UI.updateSVGSourceOutput) {
                        window.UI.updateSVGSourceOutput({ success: true, canvas_svg: svgContent });
                    }
                    if (window.UI.updateVexFlowOutput) {
                        window.UI.updateVexFlowOutput(parseResult);
                    }
                }
            } else {
                this.renderError(`Server error: ${response.status}`);
            }

        } catch (error) {
            console.error('Canvas SVG request failed:', error);
            this.renderError('Request failed: ' + error.message);
        }
    }

    // Render SVG content in the canvas
    renderCanvasSvg(svgContent) {
        // Create a temporary container to hold the SVG
        const tempDiv = document.createElement('div');
        tempDiv.innerHTML = svgContent;
        const svgElement = tempDiv.querySelector('svg');

        if (svgElement) {
            // Clear the canvas first
            this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);

            // Convert SVG to canvas using a temporary image
            const svgData = new XMLSerializer().serializeToString(svgElement);
            const svgBlob = new Blob([svgData], {type: 'image/svg+xml;charset=utf-8'});
            const url = URL.createObjectURL(svgBlob);

            const img = new Image();
            img.onload = () => {
                // Clear canvas again to ensure clean slate
                this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);

                // Draw SVG at full size to match canvas dimensions
                this.ctx.drawImage(img, 0, 0, this.canvas.width, this.canvas.height);
                URL.revokeObjectURL(url);

                // Render cursor on top of SVG
                this.renderCursor();
            };

            img.onerror = (error) => {
                console.error('Failed to load SVG as image:', error);
                URL.revokeObjectURL(url);
                this.renderError('Failed to render SVG');
            };

            img.src = url;
        } else {
            console.warn('No SVG element found in server response');
            this.renderError('Invalid SVG response');
        }
    }

    // Render error message on canvas
    renderError(errorMessage) {
        this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
        this.ctx.save();
        this.ctx.font = '14px monospace';
        this.ctx.fillStyle = '#e74c3c';
        this.ctx.textAlign = 'center';
        this.ctx.fillText('Error: ' + errorMessage, this.canvas.width / 2, this.canvas.height / 2);
        this.ctx.restore();

        // Render cursor on top of error
        this.renderCursor();
    }

    // Clear canvas
    clearCanvas() {
        this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);

        // Draw a placeholder background when canvas is empty
        this.ctx.save();
        this.ctx.fillStyle = '#fafafa';
        this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
        this.ctx.strokeStyle = '#dddddd';
        this.ctx.lineWidth = 1;
        this.ctx.strokeRect(0, 0, this.canvas.width, this.canvas.height);
        this.ctx.restore();

        // Show cursor for empty canvas
        this.renderCursor();
    }

    // Render cursor at current position
    renderCursor() {
        // Don't render cursor if we have server-rendered SVG content
        // The server SVG should handle cursor positioning
        if (this.textContent.trim()) {
            return; // Let server-side SVG handle cursor rendering
        }

        // Calculate cursor position for empty text
        const x = this.calculateCursorX();
        const y = this.calculateCursorY();

        // Only draw cursor if it should be visible and content is empty
        if (this.cursorVisible) {
            this.ctx.save();
            this.ctx.strokeStyle = '#e74c3c';
            this.ctx.fillStyle = '#e74c3c';
            this.ctx.lineWidth = 2;

            // Draw a filled rectangle for better visibility
            this.ctx.fillRect(x, y - 2, 2, 22);

            this.ctx.restore();
        }
    }

    // Calculate cursor X position based on text content
    calculateCursorX() {
        if (this.textContent.length === 0 || this.cursorPosition === 0) {
            return 20; // Left margin
        }

        // Get text up to cursor position
        const textToCursor = this.textContent.substring(0, this.cursorPosition);
        const lines = textToCursor.split('\n');
        const currentLineText = lines[lines.length - 1];

        // Measure text width
        this.ctx.font = '16px monospace';
        const textWidth = this.ctx.measureText(currentLineText).width;

        return 20 + textWidth; // Left margin + text width
    }

    // Calculate cursor Y position based on line number
    calculateCursorY() {
        // Always start at top with proper margin
        const lineHeight = 24;
        const topMargin = 40;

        if (this.textContent.length === 0 || this.cursorPosition === 0) {
            return topMargin; // Top position for empty text or start
        }

        // Count lines up to cursor
        const textToCursor = this.textContent.substring(0, this.cursorPosition);
        const lineCount = textToCursor.split('\n').length;

        return topMargin + (lineCount - 1) * lineHeight; // Top margin + line height
    }

    // Update parse result and re-render
    updateParseResult(parseResult) {
        this.parseResult = parseResult;
        this.extractNoteElements();
        if (this.currentMode === 'visual') {
            this.render();
        }
    }

    // Extract note elements from parse result for visual rendering
    extractNoteElements() {
        this.noteElements = [];
        if (!this.parseResult?.document?.elements) return;

        let yOffset = 50;
        this.parseResult.document.elements.forEach((stave, staveIndex) => {
            if (stave.Stave?.lines) {
                stave.Stave.lines.forEach((line) => {
                    if (line.Content) {
                        let xOffset = 50;
                        line.Content.forEach((element) => {
                            if (element.Note) {
                                this.noteElements.push({
                                    type: 'note',
                                    data: element.Note,
                                    x: xOffset,
                                    y: yOffset,
                                    width: 30,
                                    height: 30,
                                    id: `note-${staveIndex}-${this.noteElements.length}`
                                });
                                xOffset += 40;
                            } else if (element.Barline) {
                                this.noteElements.push({
                                    type: 'barline',
                                    data: element.Barline,
                                    x: xOffset,
                                    y: yOffset - 10,
                                    width: 3,
                                    height: 50,
                                    id: `barline-${staveIndex}-${this.noteElements.length}`
                                });
                                xOffset += 20;
                            }
                        });
                        yOffset += 80;
                    }
                });
            }
        });
    }

    // Render the canvas
    render() {
        if (!this.ctx) return;

        // Don't render anything here - let SVG rendering handle the content
        // Just ensure cursor is visible
        this.renderCursor();
    }

    // Render overlay for text mode
    renderTextModeOverlay() {
        // No overlay in text mode - just let the SVG render
        return;
    }

    // Render visual mode
    renderVisualMode() {
        // Draw staff lines
        this.drawStaffLines();

        // Draw musical elements
        this.noteElements.forEach(element => {
            this.drawElement(element);
        });

        // Highlight selected element
        if (this.selectedElement) {
            this.highlightElement(this.selectedElement);
        }
    }

    // Draw staff lines
    drawStaffLines() {
        this.ctx.strokeStyle = '#ccc';
        this.ctx.lineWidth = 1;

        for (let y = 60; y < this.canvas.height; y += 80) {
            // Draw 5 staff lines
            for (let i = 0; i < 5; i++) {
                const lineY = y + i * 10;
                this.ctx.beginPath();
                this.ctx.moveTo(30, lineY);
                this.ctx.lineTo(this.canvas.width - 30, lineY);
                this.ctx.stroke();
            }
        }
    }

    // Draw a musical element
    drawElement(element) {
        this.ctx.save();

        if (element.type === 'note') {
            // Draw note
            this.ctx.fillStyle = '#333';
            this.ctx.font = '18px serif';
            this.ctx.textAlign = 'center';
            this.ctx.fillText(element.data.pitch || 'S', element.x + 15, element.y + 20);

            // Draw octave dots if present
            if (element.data.octave_adjustment) {
                this.drawOctaveDots(element);
            }
        } else if (element.type === 'barline') {
            // Draw barline
            this.ctx.fillStyle = '#333';
            this.ctx.fillRect(element.x, element.y, element.width, element.height);
        }

        this.ctx.restore();
    }

    // Draw octave dots for a note
    drawOctaveDots(element) {
        const dots = element.data.octave_adjustment || 0;
        const dotSize = 2;
        const x = element.x + 15;

        if (dots > 0) {
            // Upper octave dots
            for (let i = 0; i < dots; i++) {
                this.ctx.beginPath();
                this.ctx.arc(x, element.y - 5 - (i * 5), dotSize, 0, Math.PI * 2);
                this.ctx.fill();
            }
        } else if (dots < 0) {
            // Lower octave dots
            for (let i = 0; i < Math.abs(dots); i++) {
                this.ctx.beginPath();
                this.ctx.arc(x, element.y + 35 + (i * 5), dotSize, 0, Math.PI * 2);
                this.ctx.fill();
            }
        }
    }

    // Highlight selected element
    highlightElement(element) {
        this.ctx.save();
        this.ctx.strokeStyle = '#007acc';
        this.ctx.lineWidth = 2;
        this.ctx.strokeRect(element.x - 2, element.y - 2, element.width + 4, element.height + 4);
        this.ctx.restore();
    }

    // Find element at coordinates
    findElementAt(x, y) {
        return this.noteElements.find(element =>
            x >= element.x && x <= element.x + element.width &&
            y >= element.y && y <= element.y + element.height
        );
    }

    // Toolbar actions
    insertNote() {
        this.insertText(' S');
    }

    insertBarline() {
        this.insertText(' |');
    }

    insertGraceNote() {
        const graceNote = prompt('Enter grace note:');
        if (graceNote) {
            this.insertText(`(${graceNote})`);
        }
    }

    // Helper method to insert text at cursor position
    insertText(text) {
        this.textContent = this.textContent.slice(0, this.cursorPosition) + text + this.textContent.slice(this.cursorPosition);
        this.cursorPosition += text.length;
        this.selection.start = this.cursorPosition;
        this.selection.end = this.cursorPosition;
        this.lines = this.textContent.split('\n');
        this.isDirty = true;

        // Reset cursor blink
        this.resetCursorBlink();

        // Submit to server for real-time canvas SVG generation
        this.submitToServer(this.textContent);

        if (this.onContentChange) {
            this.onContentChange(this.textContent);
        }
    }

    deleteSelectedElement() {
        if (!this.selectedElement) return;

        // This would require more sophisticated text manipulation
        // For now, just clear the selection
        this.selectedElement = null;
        this.render();
    }

    // Apply octave adjustments
    applyOctaveAdjustment(type) {
        // This would modify the text based on the selected adjustment
        console.log('Applying octave adjustment:', type);
    }

    // Utility methods for compatibility with existing code
    getValue() {
        return this.textContent;
    }

    setValue(content) {
        this.textContent = content;
        this.lines = content.split('\n');
        this.cursorPosition = Math.min(this.cursorPosition, content.length);
        this.isDirty = true;
        this.submitToServer(content);
    }

    focus() {
        this.canvas.focus();
    }

    getCursorPosition() {
        return {
            start: this.selection.start,
            end: this.selection.end
        };
    }

    setSelection(start, end) {
        this.selection.start = Math.max(0, Math.min(this.textContent.length, start));
        this.selection.end = Math.max(0, Math.min(this.textContent.length, end));
        this.cursorPosition = this.selection.end;
        this.resetCursorBlink();
    }

    // Update selection tracking
    updateSelection() {
        if (this.onSelectionChange) {
            this.onSelectionChange({
                start: this.selection.start,
                end: this.selection.end
            });
        }
    }

    // Check if text should be converted to uppercase for sargam
    shouldConvertToSargamUppercase() {
        const text = this.textContent;
        const lines = text.split('\n');
        const cursorPos = this.cursorPosition;

        // Find which line the cursor is on
        let currentPos = 0;
        for (let i = 0; i < lines.length; i++) {
            const lineLength = lines[i].length + 1; // +1 for newline
            if (currentPos + lineLength > cursorPos) {
                const lineText = lines[i];
                return this.isContentLine(lineText) && this.detectNotationSystem(lineText) === 'sargam';
            }
            currentPos += lineLength;
        }

        return false;
    }

    // Detect if line is a content line
    isContentLine(lineText) {
        if (lineText.includes(':') && !lineText.includes('|')) {
            return false; // Likely a directive line
        }
        const musicalIndicators = /[|\-'SRGMPDNsrgmpdnCDEFGAB1-7]/;
        return musicalIndicators.test(lineText);
    }

    // Handle keyboard key down events
    handleKeyDown(e) {
        // Handle special keys
        if (e.key === 'Backspace') {
            e.preventDefault();
            this.handleBackspace();
        } else if (e.key === 'Delete') {
            e.preventDefault();
            this.handleDelete();
        } else if (e.key === 'ArrowLeft') {
            e.preventDefault();
            this.moveCursor(-1);
        } else if (e.key === 'ArrowRight') {
            e.preventDefault();
            this.moveCursor(1);
        } else if (e.key === 'Enter') {
            e.preventDefault();
            this.insertText('\n');
        }
    }

    // Handle keyboard key press events
    handleKeyPress(e) {
        // Only handle printable characters
        if (e.key && e.key.length === 1 && !e.ctrlKey && !e.altKey && !e.metaKey) {
            e.preventDefault();
            this.insertText(e.key);
        }
    }

    // Handle backspace key
    handleBackspace() {
        if (this.cursorPosition > 0) {
            const beforeCursor = this.textContent.substring(0, this.cursorPosition - 1);
            const afterCursor = this.textContent.substring(this.cursorPosition);
            this.textContent = beforeCursor + afterCursor;
            this.cursorPosition--;
            this.isDirty = true;
            this.render();
            this.submitToServer(this.textContent);
        }
    }

    // Handle delete key
    handleDelete() {
        if (this.cursorPosition < this.textContent.length) {
            const beforeCursor = this.textContent.substring(0, this.cursorPosition);
            const afterCursor = this.textContent.substring(this.cursorPosition + 1);
            this.textContent = beforeCursor + afterCursor;
            this.isDirty = true;
            this.render();
            this.submitToServer(this.textContent);
        }
    }

    // Move cursor position
    moveCursor(delta) {
        this.cursorPosition = Math.max(0, Math.min(this.textContent.length, this.cursorPosition + delta));
        this.resetCursorBlink();
        this.render();
    }

    // Insert text at cursor position
    insertText(text) {
        const beforeCursor = this.textContent.substring(0, this.cursorPosition);
        const afterCursor = this.textContent.substring(this.cursorPosition);
        this.textContent = beforeCursor + text + afterCursor;
        this.cursorPosition += text.length;
        this.isDirty = true;
        this.render();
        this.submitToServer(this.textContent);
    }

    // Detect notation system
    detectNotationSystem(lineText) {
        const content = lineText.replace(/[|\s\-']/g, '');
        if (content.length === 0) return 'unknown';

        const sargamChars = content.match(/[SRGMPDNsrgmpdn]/g) || [];
        const westernChars = content.match(/[CDEFGAB]/g) || [];
        const numberChars = content.match(/[1-7]/g) || [];

        if (sargamChars.length > westernChars.length && sargamChars.length > numberChars.length) {
            return 'sargam';
        } else if (westernChars.length > numberChars.length) {
            return 'western';
        } else if (numberChars.length > 0) {
            return 'number';
        }
        return 'unknown';
    }
}