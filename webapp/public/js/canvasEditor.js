/**
 * Canvas Editor Module - WYSIWYG Music Notation Editor
 * Canvas-based visual and text editor for music notation
 */

import { DocumentModel, DocumentPersistence } from './documentModel.js';

// Helper function for throttling
function throttle(func, limit) {
    let inThrottle;
    return function(...args) {
        if (!inThrottle) {
            func.apply(this, args);
            inThrottle = true;
            setTimeout(() => inThrottle = false, limit);
        }
    };
}

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

        // Document-first architecture
        this.document = new DocumentModel(); // Primary document model
        this.persistence = new DocumentPersistence('musicTextDocument');

        // Legacy text-based support (for backward compatibility)
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

        // UUID-based selection tracking (now connected to document model)
        this.selectedUuids = new Set(); // Set of UUIDs for selected elements
        this.elementUuidMap = new Map(); // Map of UUID -> element data

        // Coordinate tracking from SVG
        this.elementCoordinates = [];
        this.characterPositions = {};

        // Stored SVG data for cursor blinking
        this.lastSvgContent = null;
        this.lastSvgImage = null;
        this.cursorBlinkState = true;

        // Event handlers
        this.onContentChange = null;
        this.onSelectionChange = null;

        this.throttledSubmitToServer = throttle(this.submitToServer, 50); // 50ms throttle for smoother updates
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

        // Load saved state from local storage
        const hasLoadedState = this.loadFromLocalStorage();

        // Initialize canvas with placeholder message if nothing loaded
        if (!hasLoadedState) {
            this.clearCanvas();
        }

        // Start cursor blinking
        this.startCursorBlink();

        console.log('✅ Canvas Editor initialized', hasLoadedState ? '(loaded from localStorage)' : '(fresh start)');
        return this;
    }

    // Setup event listeners for canvas
    setupEventListeners() {
        // Make canvas focusable and handle keyboard input
        this.canvas.setAttribute('tabindex', '0');
        this.canvas.style.cursor = 'text'; // Always show text cursor
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

        this.canvas.addEventListener('dblclick', (e) => {
            this.handleCanvasDoubleClick(e);
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

        // Prevent context menu on right-click to avoid interfering with selection
        this.canvas.addEventListener('contextmenu', (e) => {
            e.preventDefault();
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
        } else if (e.key === 'ArrowUp' || e.key === 'ArrowDown') {
            e.preventDefault();
            this.moveCursorVertical(e.key === 'ArrowUp' ? -1 : 1);
        } else if (e.key === 'Enter') {
            e.preventDefault();
            this.handleEnterKey();
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
        if (e.ctrlKey || e.metaKey || e.key === 'Escape' || e.key === 'Enter') {
            return; // Ignore special keys (Enter is handled in handleKeyDown)
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

            // Update UUID selection from character position
            this.updateUuidSelectionFromCharacters();

            this.resetCursorBlink();

            // Save to local storage
            this.saveToLocalStorage();

            // Re-render with new cursor position
            this.submitToServer(this.textContent);

            // Notify selection change
            if (this.onSelectionChange) {
                this.onSelectionChange({
                    start: this.selection.start,
                    end: this.selection.end,
                    uuids: Array.from(this.selectedUuids)
                });
            }
        }

        // Prevent text selection outside canvas
        e.preventDefault();
    }

    // Handle mouse up for ending text selection
    handleCanvasMouseUp(e) {
        if (this.isSelecting) {
            this.isSelecting = false;

            // Update UUID selection from final character selection
            this.updateUuidSelectionFromCharacters();

            // A final update to ensure the selection is correctly rendered
            this.submitToServer(this.textContent);

            // Notify selection change if there's a callback
            if (this.onSelectionChange) {
                this.onSelectionChange({
                    start: this.selection.start,
                    end: this.selection.end,
                    uuids: Array.from(this.selectedUuids)
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

        // First, try to use precise character positions from SVG metadata
        if (this.characterPositions && Object.keys(this.characterPositions).length > 0) {
            // Account for SVG transform translate(20, 60)
            const adjustedX = scaledX - 20;
            const adjustedY = scaledY - 60;

            // Find bounds and organize positions by line
            let topMostY = Number.MAX_VALUE;
            let bottomMostY = Number.MIN_VALUE;

            // Group positions by Y coordinate (lines)
            const lineMap = new Map(); // Y -> array of {pos, x}

            for (const [posStr, coords] of Object.entries(this.characterPositions)) {
                const pos = parseInt(posStr);
                const xCoord = typeof coords === 'number' ? coords : coords.x;
                const yCoord = typeof coords === 'number' ? 0 : coords.y;

                // Track topmost and bottommost Y positions
                if (yCoord < topMostY) {
                    topMostY = yCoord;
                }
                if (yCoord > bottomMostY) {
                    bottomMostY = yCoord;
                }

                // Group by Y coordinate
                if (!lineMap.has(yCoord)) {
                    lineMap.set(yCoord, []);
                }
                lineMap.get(yCoord).push({pos, x: xCoord});
            }

            // Sort each line's positions by x coordinate
            for (const [y, positions] of lineMap) {
                positions.sort((a, b) => a.x - b.x);
            }

            // If click is above all content (more than 30 pixels above the first line), position at start
            if (adjustedY < topMostY - 30) {
                console.log(`Click above content: adjustedY(${adjustedY.toFixed(1)}) < topMost(${topMostY}) - 30 -> cursor 0`);
                return 0;
            }

            // If click is below all content (more than 30 pixels below the last line), position at end
            if (adjustedY > bottomMostY + 30) {
                console.log(`Click below content: adjustedY(${adjustedY.toFixed(1)}) > bottomMost(${bottomMostY}) + 30 -> cursor ${this.textContent.length}`);
                return this.textContent.length;
            }

            // Find the closest line
            let closestLineY = null;
            let minYDistance = Number.MAX_VALUE;

            for (const y of lineMap.keys()) {
                const yDist = Math.abs(y - adjustedY);
                if (yDist < minYDistance) {
                    minYDistance = yDist;
                    closestLineY = y;
                }
            }

            // If we found a line within reasonable distance
            if (closestLineY !== null && minYDistance < 30) {
                const linePositions = lineMap.get(closestLineY);

                if (linePositions && linePositions.length > 0) {
                    const firstPosInLine = linePositions[0];
                    const lastPosInLine = linePositions[linePositions.length - 1];

                    // If click is to the left of the first position on this line
                    if (adjustedX < firstPosInLine.x - 20) {
                        console.log(`Click left of line: adjusted(${adjustedX.toFixed(1)}) -> cursor ${firstPosInLine.pos}`);
                        return firstPosInLine.pos;
                    }

                    // If click is to the right of the last position on this line
                    if (adjustedX > lastPosInLine.x + 20) {
                        console.log(`Click right of line: adjusted(${adjustedX.toFixed(1)}) -> cursor ${lastPosInLine.pos}`);
                        return lastPosInLine.pos;
                    }

                    // Find closest position on this line
                    let closestPos = firstPosInLine.pos;
                    let minXDistance = Number.MAX_VALUE;

                    for (const {pos, x} of linePositions) {
                        const xDist = Math.abs(x - adjustedX);
                        if (xDist < minXDistance) {
                            minXDistance = xDist;
                            closestPos = pos;
                        }
                    }

                    console.log(`Click on line: adjusted(${adjustedX.toFixed(1)}, ${adjustedY.toFixed(1)}) -> cursor ${closestPos}`);
                    return Math.min(closestPos, this.textContent.length);
                }
            }

            // Fallback: return 0 if nothing matched
            console.log(`Click: no match found, defaulting to cursor 0`);
            return 0;
        }

        // Fallback to approximate calculation if no precise positions available
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

        console.log(`Click: display(${x.toFixed(1)}, ${y.toFixed(1)}) -> scaled(${scaledX.toFixed(1)}, ${scaledY.toFixed(1)}) -> line ${lineIndex}, char ${charIndex} -> cursor ${cursorPosition} (fallback)`);

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
                this.throttledSubmitToServer(this.textContent);
            }
        }

        // Always use text cursor since this is a text editor
        this.canvas.style.cursor = 'text';
    }

    // Insert character at cursor position (document-first)
    insertCharacter(char) {
        // Update text content (legacy format)
        this.textContent = this.textContent.slice(0, this.cursorPosition) + char + this.textContent.slice(this.cursorPosition);
        this.cursorPosition++;
        this.selection.start = this.cursorPosition;
        this.selection.end = this.cursorPosition;
        this.lines = this.textContent.split('\n');
        this.isDirty = true;

        // Update document model with new text
        this.document.cacheFormat('music_text', this.textContent);
        this.document.setCursor(null, this.cursorPosition);

        // Reset cursor blink
        this.resetCursorBlink();

        // Save to local storage
        this.saveToLocalStorage();

        // Submit to server for real-time canvas SVG generation
        this.submitToServer(this.textContent);

        if (this.onContentChange) {
            this.onContentChange(this.textContent);
        }

        // Notify selection change
        if (this.onSelectionChange) {
            this.onSelectionChange({
                start: this.selection.start,
                end: this.selection.end
            });
        }
    }

    // Handle backspace (document-first)
    handleBackspace() {
        if (this.cursorPosition > 0) {
            // Update text content (legacy format)
            this.textContent = this.textContent.slice(0, this.cursorPosition - 1) + this.textContent.slice(this.cursorPosition);
            this.cursorPosition--;
            this.selection.start = this.cursorPosition;
            this.selection.end = this.cursorPosition;
            this.lines = this.textContent.split('\n');
            this.isDirty = true;

            // Update document model with new text
            this.document.cacheFormat('music_text', this.textContent);
            this.document.setCursor(null, this.cursorPosition);

            // Reset cursor blink
            this.resetCursorBlink();

            // Save to local storage
            this.saveToLocalStorage();

            // Submit to server for real-time canvas SVG generation
            this.submitToServer(this.textContent);

            if (this.onContentChange) {
                this.onContentChange(this.textContent);
            }

            // Notify selection change
            if (this.onSelectionChange) {
                this.onSelectionChange({
                    start: this.selection.start,
                    end: this.selection.end
                });
            }
        }
    }

    // Handle delete (document-first)
    handleDelete() {
        if (this.cursorPosition < this.textContent.length) {
            // Update text content (legacy format)
            this.textContent = this.textContent.slice(0, this.cursorPosition) + this.textContent.slice(this.cursorPosition + 1);
            this.selection.start = this.cursorPosition;
            this.selection.end = this.cursorPosition;
            this.lines = this.textContent.split('\n');
            this.isDirty = true;

            // Update document model with new text
            this.document.cacheFormat('music_text', this.textContent);
            this.document.setCursor(null, this.cursorPosition);

            // Reset cursor blink
            this.resetCursorBlink();

            // Save to local storage
            this.saveToLocalStorage();

            // Submit to server for real-time canvas SVG generation
            this.submitToServer(this.textContent);

            if (this.onContentChange) {
                this.onContentChange(this.textContent);
            }

            // Notify selection change
            if (this.onSelectionChange) {
                this.onSelectionChange({
                    start: this.selection.start,
                    end: this.selection.end
                });
            }
        }
    }

    // Handle Enter key - insert newline
    handleEnterKey() {
        // Insert newline at cursor position
        this.textContent = this.textContent.slice(0, this.cursorPosition) + '\n' + this.textContent.slice(this.cursorPosition);
        this.cursorPosition++; // Move cursor after the newline
        this.selection.start = this.cursorPosition;
        this.selection.end = this.cursorPosition;
        this.lines = this.textContent.split('\n');
        this.isDirty = true;

        // Reset cursor blink
        this.resetCursorBlink();

        // Save to local storage
        this.saveToLocalStorage();

        // Submit to server for real-time canvas SVG generation
        this.submitToServer(this.textContent);

        if (this.onContentChange) {
            this.onContentChange(this.textContent);
        }

        // Notify selection change
        if (this.onSelectionChange) {
            this.onSelectionChange({
                start: this.selection.start,
                end: this.selection.end
            });
        }
    }

    // Move cursor
    moveCursor(direction) {
        this.cursorPosition = Math.max(0, Math.min(this.textContent.length, this.cursorPosition + direction));
        this.selection.start = this.cursorPosition;
        this.selection.end = this.cursorPosition;
        this.resetCursorBlink();

        // Save to local storage
        this.saveToLocalStorage();

        // Submit to server to update cursor position
        this.submitToServer(this.textContent);

        // Notify selection change
        if (this.onSelectionChange) {
            this.onSelectionChange({
                start: this.selection.start,
                end: this.selection.end
            });
        }
    }

    // Move cursor vertically (up or down lines)
    moveCursorVertical(direction) {
        if (!this.characterPositions || Object.keys(this.characterPositions).length === 0) {
            return; // No coordinate data available
        }

        // Get current position coordinates
        const currentCoords = this.characterPositions[this.cursorPosition];
        if (!currentCoords) {
            return; // Current position not tracked
        }

        const currentX = typeof currentCoords === 'number' ? currentCoords : currentCoords.x;
        const currentY = typeof currentCoords === 'number' ? 0 : currentCoords.y;

        // Find target Y coordinate (line above or below)
        const targetY = currentY + (direction * 60); // Assuming 60px line height

        // Find the position on the target line closest to current X
        let bestPosition = this.cursorPosition;
        let bestDistance = Number.MAX_VALUE;

        for (const [posStr, coords] of Object.entries(this.characterPositions)) {
            const pos = parseInt(posStr);
            const x = typeof coords === 'number' ? coords : coords.x;
            const y = typeof coords === 'number' ? 0 : coords.y;

            // Look for positions on the target line (within 15px tolerance)
            if (Math.abs(y - targetY) < 15) {
                const xDistance = Math.abs(x - currentX);
                if (xDistance < bestDistance) {
                    bestDistance = xDistance;
                    bestPosition = pos;
                }
            }
        }

        // If we found a position on the target line, move there
        if (bestPosition !== this.cursorPosition) {
            this.cursorPosition = bestPosition;
            this.selection.start = this.cursorPosition;
            this.selection.end = this.cursorPosition;
            this.resetCursorBlink();

            // Save to local storage
            this.saveToLocalStorage();

            // Submit to server to update cursor position
            this.submitToServer(this.textContent);

            // Notify selection change
            if (this.onSelectionChange) {
                this.onSelectionChange({
                    start: this.selection.start,
                    end: this.selection.end
                });
            }
        }
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

    // Submit to server for canvas SVG generation (document-first)
    async submitToServer(inputText) {
        if (!inputText.trim()) {
            this.clearCanvas();
            return;
        }

        try {
            // Prepare request data with both legacy and document-first information
            const requestData = {
                // Legacy text-based format (required for now)
                input_text: inputText,
                notation_type: this.detectNotationSystem(inputText),
                cursor_position: this.cursorPosition,
                selection_start: this.selection.start !== this.selection.end ? this.selection.start : null,
                selection_end: this.selection.start !== this.selection.end ? this.selection.end : null,

                // Document-first data (when available)
                selected_uuids: Array.from(this.selectedUuids),
                document_model: this.document.elements.size > 0 ? this.document.toJSON() : null
            };

            // Use the canvas SVG API with document-first support
            const response = await fetch('/api/canvas-svg', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(requestData)
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
            // Extract UUID data from SVG elements
            this.elementUuidMap.clear();
            const elementsWithUuids = svgElement.querySelectorAll('[data-note-id]');
            elementsWithUuids.forEach(element => {
                const uuid = element.getAttribute('data-note-id');
                const charStart = parseInt(element.getAttribute('data-char-start') || '0');
                const charEnd = parseInt(element.getAttribute('data-char-end') || '0');
                const x = parseFloat(element.getAttribute('x') || '0');
                const y = parseFloat(element.getAttribute('y') || '0');
                const elementType = element.getAttribute('data-element-type') || 'unknown';

                this.elementUuidMap.set(uuid, {
                    uuid,
                    charStart,
                    charEnd,
                    x,
                    y,
                    elementType,
                    element: element
                });
            });

            console.log('Extracted UUID data:', {
                uuids: this.elementUuidMap.size
            });

            // Extract coordinate metadata from the SVG
            const metadata = svgElement.querySelector('metadata#coordinate-data');
            if (metadata) {
                try {
                    const coordinateData = JSON.parse(metadata.textContent);
                    this.elementCoordinates = coordinateData.elements || [];
                    this.characterPositions = coordinateData.characterPositions || {};

                    console.log('Extracted coordinate data:', {
                        elements: this.elementCoordinates.length,
                        charPositions: Object.keys(this.characterPositions).length
                    });
                } catch (e) {
                    console.error('Failed to parse coordinate metadata:', e);
                }
            }

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
        // First try to find element using precise SVG coordinates
        if (this.elementCoordinates && this.elementCoordinates.length > 0) {
            // Get the actual canvas display dimensions vs internal dimensions
            const rect = this.canvas.getBoundingClientRect();
            const scaleX = this.canvas.width / rect.width;
            const scaleY = this.canvas.height / rect.height;

            // Scale click coordinates to match internal canvas coordinates
            const scaledX = x * scaleX - 20; // Account for SVG transform
            const scaledY = y * scaleY - 60; // Account for SVG transform

            return this.elementCoordinates.find(element =>
                scaledX >= element.x && scaledX <= element.x + element.width &&
                scaledY >= element.y - element.height && scaledY <= element.y + 5
            );
        }

        // Fallback to noteElements if no SVG coordinates
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
    // DEPRECATED: Use document.getCachedFormat('music_text') instead
    getValue() {
        console.warn('getValue() is deprecated. Use document.getCachedFormat("music_text") for document-first architecture.');
        return this.textContent;
    }

    // DEPRECATED: Use document operations instead of direct text manipulation
    setValue(content, cursorPos = null) {
        console.warn('setValue() is deprecated. Use document operations for document-first architecture.');

        // Legacy support: update document cache when text is set
        this.document.cacheFormat('music_text', content);
        this.textContent = content;
        this.lines = content.split('\n');

        // If cursor position provided, use it; otherwise keep current (clamped)
        if (cursorPos !== null) {
            this.cursorPosition = Math.min(cursorPos, content.length);
            this.selection.start = this.cursorPosition;
            this.selection.end = this.cursorPosition;
        } else {
            this.cursorPosition = Math.min(this.cursorPosition, content.length);
        }

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

    // New semantic command method for document-first operations
    async executeSemanticCommand(commandType, parameters = {}) {
        if (this.selectedUuids.size === 0) {
            throw new Error('No elements selected for semantic command');
        }

        try {
            const requestData = {
                command_type: commandType,
                target_uuids: Array.from(this.selectedUuids),
                parameters: parameters
            };

            const response = await fetch('/api/semantic-command', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(requestData)
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const result = await response.json();

            if (!result.success) {
                throw new Error(result.message || 'Semantic command failed');
            }

            console.log('Semantic command executed:', result);

            // Re-render to show changes (server maintains document state)
            await this.submitToServer(this.textContent);

            return result;

        } catch (error) {
            console.error('Semantic command failed:', error);
            throw error;
        }
    }

    // Generic method to apply text transformations via API (legacy)
    async applyTransformation(endpoint, transformData) {
        try {
            // Add current text, character-based selection, and UUID-based selection
            const requestData = {
                text: this.textContent,
                selection_start: this.selection.start,
                selection_end: this.selection.end,
                cursor_position: this.cursorPosition,
                selected_uuids: Array.from(this.selectedUuids), // Include UUID selection
                ...transformData
            };

            const response = await fetch(endpoint, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(requestData)
            });

            if (!response.ok) {
                throw new Error(`API request failed: ${response.status}`);
            }

            const result = await response.json();

            // Apply the transformed text and selection
            if (result.text !== undefined) {
                this.textContent = result.text;
                this.lines = result.text.split('\n');
                this.isDirty = true;
            }

            // Update cursor and selection if provided
            if (result.selection_start !== undefined) {
                this.selection.start = result.selection_start;
            }
            if (result.selection_end !== undefined) {
                this.selection.end = result.selection_end;
            }
            if (result.cursor_position !== undefined) {
                this.cursorPosition = result.cursor_position;
            } else if (result.selection_end !== undefined) {
                this.cursorPosition = result.selection_end;
            }

            // Save and submit
            this.saveToLocalStorage();
            this.submitToServer(this.textContent);

            // Trigger callbacks
            if (this.onContentChange) {
                this.onContentChange(this.textContent);
            }
            if (this.onSelectionChange) {
                this.onSelectionChange({
                    start: this.selection.start,
                    end: this.selection.end
                });
            }

            this.focus();
            return result;

        } catch (error) {
            console.error('Transformation failed:', error);
            throw error;
        }
    }

    setSelection(start, end, silent = false) {
        this.selection.start = Math.max(0, Math.min(this.textContent.length, start));
        this.selection.end = Math.max(0, Math.min(this.textContent.length, end));
        this.cursorPosition = this.selection.end;
        this.resetCursorBlink();

        // Notify selection change unless silent flag is set
        if (!silent && this.onSelectionChange) {
            this.onSelectionChange({
                start: this.selection.start,
                end: this.selection.end
            });
        }
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
        const musicalIndicators = /[|\-ҳои'SRGMPDNsrgmpdnCDEFGAB1-7]/;
        return musicalIndicators.test(lineText);
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

    // Save editor state to local storage (document-first)
    saveToLocalStorage() {
        try {
            // Update document model with current UI state
            this.document.setCursor(null, this.cursorPosition);
            this.document.setSelection(Array.from(this.selectedUuids));
            this.document.ui_state.editor_mode = this.currentMode;

            // Cache the current text representation
            this.document.cacheFormat('music_text', this.textContent);

            // Save document model
            const saved = this.persistence.saveDocument(this.document);
            if (saved) {
                console.log('Document saved to localStorage:', this.document.getStats());
            }

            // Legacy backup save (for compatibility during transition)
            const editorState = {
                textContent: this.textContent,
                cursorPosition: this.cursorPosition,
                selectionStart: this.selection.start,
                selectionEnd: this.selection.end,
                timestamp: Date.now()
            };
            localStorage.setItem('musicTextEditorState', JSON.stringify(editorState));

        } catch (e) {
            console.error('Failed to save to localStorage:', e);
        }
    }

    // Load editor state from local storage (document-first)
    loadFromLocalStorage() {
        try {
            // Try to load document model first
            const loadedDocument = this.persistence.loadDocument();
            if (loadedDocument) {
                this.document = loadedDocument;

                // Restore UI state from document
                const musicText = this.document.getCachedFormat('music_text') || '';
                this.textContent = musicText;
                this.cursorPosition = this.document.ui_state.selection.cursor_position;
                this.currentMode = this.document.ui_state.editor_mode || 'text';

                // Restore UUID-based selection
                this.selectedUuids = new Set(this.document.ui_state.selection.selected_uuids);

                // Update legacy character-based selection for compatibility
                this.updateCharacterSelectionFromUuids();
                this.lines = this.textContent.split('\n');

                console.log('Loaded document from localStorage:', this.document.getStats());

                // Update backing text area if it exists
                const backingTextArea = document.getElementById('backing-text-output');
                if (backingTextArea) {
                    backingTextArea.value = this.textContent;
                    backingTextArea.selectionStart = this.selection.start;
                    backingTextArea.selectionEnd = this.selection.end;
                }

                // Submit to server to render the loaded content
                if (this.textContent) {
                    this.submitToServer(this.textContent);
                }

                return true;
            }

            // Fallback to legacy format
            const savedState = localStorage.getItem('musicTextEditorState');
            if (savedState) {
                const state = JSON.parse(savedState);
                this.textContent = state.textContent || '';
                this.cursorPosition = Math.min(state.cursorPosition || 0, this.textContent.length);
                this.selection.start = Math.min(state.selectionStart || this.cursorPosition, this.textContent.length);
                this.selection.end = Math.min(state.selectionEnd || this.cursorPosition, this.textContent.length);
                this.lines = this.textContent.split('\n');

                // Create new document from loaded text
                if (this.textContent) {
                    DocumentModel.fromMusicText(this.textContent).then(doc => {
                        this.document = doc;
                        this.document.setCursor(null, this.cursorPosition);
                    });
                }

                console.log('Loaded legacy format from localStorage:', {
                    contentLength: this.textContent.length,
                    cursor: this.cursorPosition,
                    selection: [this.selection.start, this.selection.end],
                    age: Date.now() - (state.timestamp || 0)
                });

                // Update backing text area
                const backingTextArea = document.getElementById('backing-text-output');
                if (backingTextArea) {
                    backingTextArea.value = this.textContent;
                    backingTextArea.selectionStart = this.selection.start;
                    backingTextArea.selectionEnd = this.selection.end;
                }

                // Submit to server to render the loaded content
                if (this.textContent) {
                    this.submitToServer(this.textContent);
                }

                return true;
            }

        } catch (e) {
            console.error('Failed to load from localStorage:', e);
        }
        return false;
    }

    // Clear local storage (both document and legacy)
    clearLocalStorage() {
        try {
            // Clear document model
            this.persistence.clearDocument();

            // Clear legacy format
            localStorage.removeItem('musicTextEditorState');

            // Reset current document
            this.document = new DocumentModel();

            console.log('Cleared localStorage (document + legacy)');
        } catch (e) {
            console.error('Failed to clear localStorage:', e);
        }
    }

    // UUID-based selection methods for document-first operations

    // Select elements by their UUIDs
    selectByUuids(uuids) {
        this.selectedUuids.clear();
        if (Array.isArray(uuids)) {
            uuids.forEach(uuid => this.selectedUuids.add(uuid));
        } else {
            this.selectedUuids.add(uuids);
        }

        // Update character-based selection for backward compatibility
        this.updateCharacterSelectionFromUuids();

        console.log('Selected UUIDs:', Array.from(this.selectedUuids));
    }

    // Get currently selected UUIDs
    getSelectedUuids() {
        return Array.from(this.selectedUuids);
    }

    // Convert UUID selection to character indices for backward compatibility
    updateCharacterSelectionFromUuids() {
        if (this.selectedUuids.size === 0) {
            this.selection.start = this.cursorPosition;
            this.selection.end = this.cursorPosition;
            return;
        }

        let minStart = Number.MAX_VALUE;
        let maxEnd = Number.MIN_VALUE;

        for (const uuid of this.selectedUuids) {
            const element = this.elementUuidMap.get(uuid);
            if (element) {
                minStart = Math.min(minStart, element.charStart);
                maxEnd = Math.max(maxEnd, element.charEnd);
            }
        }

        if (minStart !== Number.MAX_VALUE && maxEnd !== Number.MIN_VALUE) {
            this.selection.start = minStart;
            this.selection.end = maxEnd;
        }
    }

    // Convert character selection to UUIDs
    updateUuidSelectionFromCharacters() {
        this.selectedUuids.clear();

        if (this.selection.start === this.selection.end) {
            return; // No selection
        }

        for (const [uuid, element] of this.elementUuidMap) {
            // Check if element overlaps with selection
            if (element.charEnd > this.selection.start && element.charStart < this.selection.end) {
                this.selectedUuids.add(uuid);
            }
        }

        console.log('Updated UUID selection from characters:', Array.from(this.selectedUuids));
    }

    // Get selection as both UUIDs and character indices
    getSelectionBoth() {
        return {
            uuids: Array.from(this.selectedUuids),
            characterRange: {
                start: this.selection.start,
                end: this.selection.end
            }
        };
    }
}