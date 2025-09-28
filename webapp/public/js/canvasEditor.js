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
        // No longer tracking lines or cursor - document elements are source of truth
        this.isDirty = false;

        // Document-first architecture
        this.document = null; // Document must come from server or localStorage
        this.persistence = new DocumentPersistence('musicTextDocument');
        this.isCreatingDocument = false; // Flag to prevent duplicate document creation
        this.pendingEditCommands = []; // Queue for edit commands while document is being created

        // UUID-based selection for document-first architecture
        this.selectedUuids = new Set(); // Set of UUIDs for selected elements
        this.cursorUuid = null; // UUID where cursor is positioned

        // Minimal client-side caret
        this.caretIndex = 0;
        this.caretTimer = null;
        this.caretVisible = true;

        // Selection state for mouse interaction
        this.isSelecting = false;
        this.selectionStart = 0;

        // Visual elements
        this.noteElements = [];
        this.selectedElement = null;

        // UUID selection already initialized above
        this.elementUuidMap = new Map(); // Map of UUID -> element data

        // Coordinate tracking from SVG
        this.elementCoordinates = [];
        this.characterPositions = {};

        // Stored SVG data for cursor blinking (disabled)
        this.lastSvgContent = null;
        this.lastSvgImage = null;
        // this.cursorBlinkState = true; // (disabled)

        // Event handlers
        this.onContentChange = null;
        this.onSelectionChange = null;

        // Deprecated: draw/render throttles removed to prevent unintended server calls during UI-only ops
        this.draw = () => {};
        this.throttledSave = throttle(this.saveToLocalStorage.bind(this), 500);
        this.drawNow = () => {};

        // Guard to prevent accidental server sync during UI-only interactions
        this.suspendServerSync = false;

        // One-time formats fetch guard after a document is loaded
        this.initialFormatsFetched = false;
        // Debug overlay for character bounding boxes
        this.debugCharBoxes = true;

        // Visual caret override for empty lines (no anchors)
        this.emptyLineCaret = null; // { x, y }
    }

    // Initialize the SVG editor (formerly canvas editor)
    init(containerId = 'canvasEditor') {
        const container = document.getElementById(containerId);
        if (!container) {
            throw new Error(`Container element '${containerId}' not found`);
        }

        this.svgContainer = document.getElementById('svg-container');

        if (!this.svgContainer) {
            throw new Error('SVG container element not found');
        }

        // Remove canvas-specific code - no more 2D context needed
        this.setupEventListeners();
        this.setupToolbarListeners();

        // Set initial mode to text
        this.switchToTextMode();

        // Load saved state from local storage
        const hasLoadedState = this.loadFromLocalStorage();
        // After loading from localStorage, request formats once if a document exists
        if (hasLoadedState && this.document && this.document.documentUUID) {
            this.afterDocumentLoaded();
        }

        // Initialize canvas with placeholder message if nothing loaded
        if (!hasLoadedState) {
            this.clearCanvas();
        }


        console.log('✅ Canvas Editor initialized', hasLoadedState ? '(loaded from localStorage)' : '(fresh start)');
        return this;
    }

    // Call after setting this.document (from localStorage, import, or file/open)
    async afterDocumentLoaded() {
        try {
            if (!this.document || !this.document.documentUUID) return;
            if (this.initialFormatsFetched) return;

            // Ensure we are not in a suspended UI interaction
            const prevSuspend = this.suspendServerSync;
            this.suspendServerSync = false;

            const response = await fetch(`/api/documents/${this.document.documentUUID}`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    document: this.document.toJSON(),
                    notation_type: this.notationType || 'number'
                })
            });
            if (!response.ok) {
                console.error('Initial formats fetch failed:', response.status);
                this.suspendServerSync = prevSuspend;
                return;
            }
            const result = await response.json();

            if (result.document) {
                this.document = DocumentModel.fromJSON(result.document);
            }
            if (result.formats) {
                if (result.formats.editor_svg) {
                    this.renderEditorSvg(result.formats.editor_svg);
                }
                if (window.UI && window.UI.updateFormatsFromBackend) {
                    window.UI.updateFormatsFromBackend(result.formats);
                }
                this.updateDocumentTabDisplay();
            }

            this.initialFormatsFetched = true;
            this.suspendServerSync = prevSuspend;
        } catch (e) {
            console.error('afterDocumentLoaded error:', e);
        }
    }

    // Setup event listeners for SVG container
    setupEventListeners() {
        // SVG container is already focusable via tabindex in HTML
        this.svgContainer.style.cursor = 'pointer'; // Show clickable cursor
        this.svgContainer.addEventListener('keydown', (e) => {
            this.handleKeyDown(e);
        });

        // Keypress handled via document updates

        // SVG events for text selection and navigation
        this.svgContainer.addEventListener('mousedown', (e) => {
            // Suspend server sync during pointer interaction
            this.suspendServerSync = true;
            this.svgContainer.focus(); // Focus container on click
            this.handleSvgMouseDown(e);
        });

        this.svgContainer.addEventListener('dblclick', (e) => {
            this.handleSvgDoubleClick(e);
        });

        // Add keyboard event listener for arrow/home/end keys with selection (Shift)
        this.svgContainer.addEventListener('keydown', (e) => {
            if (this.handleNavigationKeys(e)) {
                e.preventDefault();
            } else {
                this.handleKeydown(e);
            }
        });

        this.svgContainer.addEventListener('mousemove', (e) => {
            this.handleSvgMouseMove(e);
        });

        this.svgContainer.addEventListener('mouseup', (e) => {
            this.handleSvgMouseUp(e);
            // Re-enable server sync after pointer interaction
            this.suspendServerSync = false;
        });

        // Handle mouse leaving container during selection
        this.svgContainer.addEventListener('mouseleave', (e) => {
            this.handleSvgMouseUp(e);
            this.suspendServerSync = false;
        });

        // Prevent context menu on right-click to avoid interfering with selection
        this.svgContainer.addEventListener('contextmenu', (e) => {
            e.preventDefault();
        });

        // Track scroll position for viewport state
        this.svgContainer.addEventListener('scroll', (e) => {
            if (this.document) {
                this.document.setViewport({
                    scroll_x: this.svgContainer.scrollLeft,
                    scroll_y: this.svgContainer.scrollTop
                });
            }
            // Don't save on every scroll event (too frequent)
            // Use throttled save instead
            this.throttledSave();
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
        this.svgContainer.focus();
        this.render();
        if (typeof this.postRenderCaret === 'function') this.postRenderCaret();
    }

    // Switch to visual editing mode
    switchToVisualMode() {
        this.currentMode = 'visual';
        document.getElementById('canvasEditor').className = 'canvas-editor visual-mode';
        document.getElementById('textMode').classList.remove('active');
        document.getElementById('visualMode').classList.add('active');
        this.svgContainer.focus();
        this.render();
        if (typeof this.postRenderCaret === 'function') this.postRenderCaret();
    }

    // Handle key events
    handleKeyDown(e) {
        if (e.key === 'Backspace') {
            e.preventDefault();
            this.handleBackspace();
        } else if (e.key === 'Delete') {
            e.preventDefault();
            this.handleDelete();
        } else if (e.key === 'Enter') {
            e.preventDefault();
            this.handleEnterKey();
        } else if (e.key === 'Escape') {
            this.selectedElement = null;
            this.render();
        } else if (!e.ctrlKey && !e.metaKey && e.key.length === 1) {
            // Handle regular character input
            e.preventDefault();
            this.insertCharacter(e.key);
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

    // Handle Arrow/Home/End with optional Shift selection; returns true if handled
    handleNavigationKeys(e) {
        const key = e.key;
        const shift = e.shiftKey;
        const handled = ['ArrowLeft','ArrowRight','ArrowUp','ArrowDown','Home','End'].includes(key);
        if (!handled) return false;

        // Build anchors from current SVG
        const anchors = this.buildAnchors();
        if (anchors.length === 0) return true; // swallow key if nothing to navigate

        const lastIdx = anchors[anchors.length - 1].idx;
        const { sourceX, sourceY } = this.findCurrentPosition(anchors);
        let newCaret = this.document.ui_state.selection.cursor_position || 0;

        switch (key) {
            case 'ArrowLeft':
                newCaret = Math.max(0, newCaret - 1);
                break;
            case 'ArrowRight':
                newCaret = Math.min(lastIdx + 1, newCaret + 1);
                break;
            case 'Home': {
                const line = this.findLineAtY(anchors, sourceY);
                if (line && line.nodes.length) {
                    newCaret = line.nodes[0].idx;
                    this.emptyLineCaret = null;
                } else {
                    // empty line: keep caret visually at left margin
                    this.extractSvgTransform();
                    this.emptyLineCaret = { x: (this.svgTransformX || 20), y: sourceY };
                }
                break;
            }
            case 'End': {
                const line = this.findLineAtY(anchors, sourceY);
                if (line && line.nodes.length) {
                    newCaret = line.nodes[line.nodes.length - 1].idx + 1;
                    this.emptyLineCaret = null;
                } else {
                    // empty line end == start
                    this.extractSvgTransform();
                    this.emptyLineCaret = { x: (this.svgTransformX || 20), y: sourceY };
                }
                break;
            }
            case 'ArrowUp': {
                const above = this.findAdjacentLine(anchors, sourceY, -1);
                if (above && above.nodes.length) {
                    newCaret = this.findNearestOnLine(above.nodes, sourceX).idx;
                    this.emptyLineCaret = null;
                } else if (above) {
                    // empty line above
                    this.extractSvgTransform();
                    this.emptyLineCaret = { x: (this.svgTransformX || 20), y: above.y };
                }
                break;
            }
            case 'ArrowDown': {
                const below = this.findAdjacentLine(anchors, sourceY, +1);
                if (below && below.nodes.length) {
                    newCaret = this.findNearestOnLine(below.nodes, sourceX).idx;
                    this.emptyLineCaret = null;
                } else if (below) {
                    this.extractSvgTransform();
                    this.emptyLineCaret = { x: (this.svgTransformX || 20), y: below.y };
                }
                break;
            }
        }

        // Update selection anchor if Shift
        if (!shift) {
            this.selection_anchor = null;
            this.document.ui_state.selection.selected_uuids = [];
        } else {
            if (this.selection_anchor == null) this.selection_anchor = this.document.ui_state.selection.cursor_position || 0;
            // Character-range selection visualization can be added; keep UUID selection intact for now
        }

        this.document.ui_state.selection.cursor_position = newCaret;
        if (typeof this.postRenderCaret === 'function') this.postRenderCaret();
        return true;
    }

    // Build ordered anchors from [data-char-index] nodes
    buildAnchors() {
        const svg = this.currentSvg;
        if (!svg) return [];
        const content = svg.querySelector('.canvas-content') || svg;
        const nodes = Array.from(content.querySelectorAll('[data-char-index]'));
        const anchors = nodes.map(n => {
            const bb = n.getBBox();
            return { idx: +(n.getAttribute('data-char-index') || '0'), x: bb.x, right: bb.x + bb.width, y: bb.y, node: n };
        }).sort((a,b) => a.idx - b.idx);
        return anchors;
    }

    // Find current visual position from caret index
    findCurrentPosition(anchors) {
        const caret = this.document.ui_state?.selection?.cursor_position ?? 0;
        // Exact
        const exact = anchors.find(a => a.idx === caret);
        if (exact) return { sourceX: exact.x, sourceY: exact.y };
        // Char before caret
        let before = null;
        for (let i = anchors.length - 1; i >= 0; i--) {
            if (anchors[i].idx < caret) { before = anchors[i]; break; }
        }
        if (before) return { sourceX: before.right, sourceY: before.y };
        // Fallback: first
        return { sourceX: anchors[0].x, sourceY: anchors[0].y };
    }

    // Group anchors by visual lines (y within tolerance)
    groupLines(anchors) {
        const TOL = 12;
        const lines = [];
        for (const a of anchors) {
            let line = lines.find(l => Math.abs(l.y - a.y) < TOL);
            if (!line) { line = { y: a.y, nodes: [] }; lines.push(line); }
            line.nodes.push(a);
        }
        lines.forEach(l => l.nodes.sort((a,b)=>a.x - b.x));
        lines.sort((a,b)=>a.y - b.y);
        return lines;
    }

    findLineAtY(anchors, y) {
        const lines = this.groupLines(anchors);
        let best = null, bestDy = Infinity;
        for (const l of lines) {
            const dy = Math.abs(l.y - y);
            if (dy < bestDy) { bestDy = dy; best = l; }
        }
        return best;
    }

    findAdjacentLine(anchors, y, dir) {
        const lines = this.groupLines(anchors);
        const sorted = lines.map(l=>l.y).sort((a,b)=>a-b);
        const current = this.findLineAtY(anchors, y);
        if (!current) return null;
        const i = sorted.indexOf(current.y);
        const j = i + (dir < 0 ? -1 : 1);
        if (j < 0 || j >= sorted.length) return null;
        const targetY = sorted[j];
        return lines.find(l => l.y === targetY) || null;
    }

    findNearestOnLine(nodes, x) {
        let best = nodes[0], bestDx = Math.abs(nodes[0].x - x);
        for (const n of nodes) {
            const dx = Math.abs(n.x - x);
            if (dx < bestDx) { bestDx = dx; best = n; }
        }
        return best;
    }


    // Handle mouse down for starting text selection
    handleSvgMouseDown(e) {
        if (!this.document) return;

        const rect = this.svgContainer.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;

        if (e.target.classList.contains('line-hitbox')) {
            const lineGroup = e.target.parentElement;
            const chars = Array.from(lineGroup.querySelectorAll('[data-char-index]')).map(c => {
                const bbox = c.getBBox();
                return {
                    el: c,
                    index: parseInt(c.getAttribute('data-char-index'), 10),
                    x: bbox.x,
                    width: bbox.width
                };
            }).sort((a, b) => a.index - b.index);

            if (chars.length > 0) {
                const firstChar = chars[0];
                const lastChar = chars[chars.length - 1];
                const clickX = x - (this.svgTransformX || 0);

                if (clickX < firstChar.x) {
                    this.document.ui_state.selection.cursor_position = firstChar.index;
                } else if (clickX > lastChar.x + lastChar.width) {
                    this.document.ui_state.selection.cursor_position = lastChar.index + 1;
                } else {
                    // Find the nearest character to snap to
                    let closest = null;
                    let minDistance = Infinity;
                    for (const char of chars) {
                        const midPoint = char.x + char.width / 2;
                        const distance = Math.abs(clickX - midPoint);
                        if (distance < minDistance) {
                            minDistance = distance;
                            closest = char;
                        }
                    }
                    if (closest) {
                        const midPoint = closest.x + closest.width / 2;
                        this.document.ui_state.selection.cursor_position = clickX < midPoint ? closest.index : closest.index + 1;
                    }
                }
            } else {
                const anchor = lineGroup.querySelector('[data-char-index]');
                if (anchor) {
                    this.document.ui_state.selection.cursor_position = parseInt(anchor.getAttribute('data-char-index'), 10);
                }
            }
        } else {
            const clickedChar = this.findCharacterAtPoint(x, y);
            if (clickedChar) {
                const charIndex = parseInt(clickedChar.getAttribute('data-char-index') || '0', 10);
                this.setCursorPosition(charIndex);
            } else {
                const calculatedPosition = this.calculateCursorPositionFromClick(x, y);
                if (calculatedPosition !== null) {
                    this.setCursorPosition(calculatedPosition);
                }
            }
        }

        this.isSelecting = true;
        this.selectionStart = this.document?.ui_state.selection.cursor_position || 0;
        this.selectedUuids.clear();
        if (this.document?.ui_state?.selection) {
            this.document.ui_state.selection.selected_uuids = [];
            this.document.ui_state.selection.cursor_uuid = null;
        }
        this.updateClientSideSelection();
        this.updateCaretFromDocument();
        this.postRenderCaret();
        e.preventDefault();
    }

    // Handle mouse up for ending text selection
    handleSvgMouseUp(e) {
        if (this.isSelecting) {
            this.isSelecting = false;

            // Update UUID selection from final character selection
            this.updateUuidSelectionFromCharacters();

            // Update UI state locally with selection
            if (this.document && this.document.ui_state && this.document.ui_state.selection) {
                this.document.ui_state.selection.selected_uuids = Array.from(this.selectedUuids);
            }

            // Console logging for selection testing
            const selectedText = ''; // Document elements are source of truth
            console.log('🖱️ Mouse Selection Complete:', {
                characterSelection: {
                    // UUID-based selection
                },
                selectedText: `"${selectedText}"`, 
                uuidSelection: {
                    count: this.selectedUuids.size,
                    uuids: Array.from(this.selectedUuids)
                },
                elementMapping: Array.from(this.selectedUuids).map(uuid => ({
                    uuid,
                    element: this.elementUuidMap.get(uuid)
                }))
            });

            // Apply client-side selection highlighting immediately (no server call)
            this.updateClientSideSelection();
            this.updateCaretFromDocument();
            if (typeof this.postRenderCaret === 'function') this.postRenderCaret();

            // Notify selection change if there's a callback
            if (this.onSelectionChange) {
                this.onSelectionChange({
                    // UUID-based selection
                    uuids: Array.from(this.selectedUuids)
                });
            }
        }
    }

    // Handle double click for selecting beats or words
    handleSvgDoubleClick(e) {
        const rect = this.svgContainer.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;

        // Calculate cursor position from click coordinates
        const clickPosition = this.calculateCursorPositionFromClick(x, y);
        if (clickPosition !== null) {
            const selection = this.selectBeatOrWordAt(clickPosition);
            if (selection) {
                // Selection is UUID-based, not position-based
                this.document.ui_state.selection.cursor_position = 0;

                // Update UUID selection from character selection
                this.updateUuidSelectionFromCharacters();

                    this.saveToLocalStorage();
                // Apply client-side selection highlighting immediately (double-click)
                this.updateClientSideSelection();
                this.initializeVisualCursor();

                // Notify selection change
                if (this.onSelectionChange) {
                    this.onSelectionChange({
                        // UUID-based selection,
                        uuids: Array.from(this.selectedUuids)
                    });
                }
            }
        }

        // Prevent text selection outside canvas
        e.preventDefault();
    }

    // Calculate cursor position from click coordinates
    calculateCursorPositionFromClick(x, y) {
        // Document-first: position based on elements
        return 0;

        // For SVG, coordinates are 1:1 with the container - no scaling needed
        const rect = this.svgContainer.getBoundingClientRect();

        // SVG coordinates are direct - no scaling transformation needed
        const scaledX = x;
        const scaledY = y;

        // First, try to use precise character positions from SVG metadata
        if (this.characterPositions && Object.keys(this.characterPositions).length > 0) {
            // Account for SVG transform (dynamically extracted)
            const adjustedX = scaledX - (this.svgTransformX || 20);
            const adjustedY = scaledY - (this.svgTransformY || 20);

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
                console.log(`Click below content: adjustedY(${adjustedY.toFixed(1)}) > bottomMost(${bottomMostY}) + 30`);
                return this.document.content?.length || 0;
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
                    return Math.min(closestPos, this.document.content?.length || 0);
                }
            }

            // Fallback: return 0 if nothing matched
            console.log(`Click: no match found, defaulting to cursor 0`);
            return 0;
        }

        // Fallback to approximate calculation if no precise positions available
        const charWidth = 12; // Character width based on font size
        const lineHeight = 60; // Line spacing from SVG renderer
        const leftMargin = this.svgTransformX || 20; // Left margin from SVG transform (dynamically extracted)
        const topMargin = this.svgTransformY || 20;  // Top margin from SVG transform (dynamically extracted)

        // Calculate which line was clicked (using scaled SVG coordinates)
        let lineIndex = Math.floor((scaledY - topMargin) / lineHeight);
        lineIndex = Math.max(0, lineIndex);

        // Calculate character position within line
        let charIndex = Math.floor((scaledX - leftMargin) / charWidth);
        charIndex = Math.max(0, charIndex);

        // Convert line and character index to absolute cursor position
        const lines = []; // Document-first: no text tracking
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
        return Math.min(cursorPosition, this.document.content?.length || 0);
    }

    // Handle canvas mouse move events for hover and text selection
    handleSvgMouseMove(e) {
        const rect = this.svgContainer.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;

        // Handle text selection during drag
        if (this.isSelecting) {
            const dragPosition = this.calculateCursorPositionFromClick(x, y);
            if (dragPosition !== null) {
                // Update selection range
                // UUID-based selection during drag
                // TODO: Select UUIDs between start and drag positions
                this.document.ui_state.selection.cursor_position = dragPosition;

                // Console logging for drag selection testing
                if (this.selectedUuids.size > 0) {
                    const selectedText = ''; // Document elements are source of truth
                    console.log('🔄 Dragging Selection:', {
                        from: this.selectionStart,
                        to: dragPosition,
                        selection: {uuids: Array.from(this.selectedUuids)},
                        selectedText: `"${selectedText}"`
                    });
                }

                // Update visuals locally only
                if (typeof this.postRenderCaret === 'function') this.postRenderCaret();
            }
        }

        // Always use pointer cursor to show it's clickable
        this.svgContainer.style.cursor = 'pointer';
    }

    // Insert character via document update
    async insertCharacter(char) {
        // Build editing command for server
        const editCommand = {
            type: 'insert_text',
            position: this.document?.ui_state?.selection?.cursor_position || 0,
            text: char
        };

        await this.applyEditCommand(editCommand);
    }

    // Handle backspace via document update
    async handleBackspace() {
        const cursorPos = this.document?.ui_state?.selection?.cursor_position || 0;
        if (cursorPos > 0 || this.selectedUuids.size > 0) {
            const editCommand = {
                type: 'delete_text',
                position: cursorPos,
                direction: 'backward',
                selected_uuids: Array.from(this.selectedUuids)
            };

            await this.applyEditCommand(editCommand);
        }
    }

    // Handle delete via document update
    async handleDelete() {
        const cursorPos = this.document?.ui_state?.selection?.cursor_position || 0;
        const contentLength = this.document?.content?.length || 0;
        if (cursorPos < contentLength || this.selectedUuids.size > 0) {
            const editCommand = {
                type: 'delete_text',
                position: cursorPos,
                direction: 'forward',
                selected_uuids: Array.from(this.selectedUuids)
            };

            await this.applyEditCommand(editCommand);
        }
    }

    // Apply edit command via PUT to server
    async applyEditCommand(editCommand) {
        console.log('applyEditCommand called with:', editCommand);
        console.log('Current document state:', this.document ? `UUID: ${this.document.documentUUID}` : 'null');
        console.trace('Call stack for applyEditCommand');

        // If no document exists, create new document with initial content
        if (!this.document || !this.document.documentUUID) {
            // If already creating a document, queue this command
            if (this.isCreatingDocument) {
                console.log('Document creation in progress, queueing command...');
                this.pendingEditCommands.push(editCommand);
                return;
            }

            this.isCreatingDocument = true;
            this.pendingEditCommands = [editCommand]; // Start with current command
            console.log('No document, creating new document with edit command:', editCommand);

            // Wait a tiny bit to collect rapid keystrokes
            await new Promise(resolve => setTimeout(resolve, 50));

            // Build initial text from all queued insert commands
            let initialText = '';
            for (const cmd of this.pendingEditCommands) {
                if (cmd.type === 'insert_text') {
                    initialText += cmd.text || '';
                }
            }
            console.log('Creating document with initial text:', initialText);

            try {
                // Use from-text endpoint to properly parse the initial text
                const response = await fetch('/api/documents/from-text', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        music_text: initialText,
                        notation_type: this.notationType || 'number'
                    })
                });

                if (response.ok) {
                    const result = await response.json();
                    if (result.success && result.document) {
                        // Create document from server response (includes UUID)
                        this.document = DocumentModel.fromJSON(result.document);
                        console.log('Created new document with UUID:', this.document.documentUUID);
                        console.log('Document after creation:', this.document);
                        console.log('Document cursor position after creation:', this.document.ui_state.selection.cursor_position);
                        console.log('Document elements:', this.document.elements);

                        // Verify the document was properly created
                        if (!this.document.documentUUID) {
                            console.error('ERROR: Document created but no UUID!');
                        } else {
                            console.log('✅ Document successfully set with UUID:', this.document.documentUUID);
                        }

                        // Update document tab display
                        this.updateDocumentTabDisplay();

                        // Cursor position is already in document.ui_state

                        // Update all format tabs from server response
                        if (result.formats) {
                            // Update editor SVG
                            if (result.formats.editor_svg) {
                                this.renderEditorSvg(result.formats.editor_svg);
                            }

                            // Update all format tabs using the unified function
                            if (window.UI && window.UI.updateFormatsFromBackend) {
                                window.UI.updateFormatsFromBackend(result.formats);
                            }
                        }

                        // Save to localStorage now that we have a UUID
                        this.saveToLocalStorage();

                        // Update UI
                        if (window.UI && window.UI.updateDocumentStatus) {
                            window.UI.updateDocumentStatus();
                        }

                        // Clear the queue - all text has been included in the document
                        this.pendingEditCommands = [];
                        this.isCreatingDocument = false;

                        // All queued commands have been handled in the initial document creation
                        return;
                    }
                } else {
                    console.error('Failed to create document:', response.status);
                    this.pendingEditCommands = [];
                    this.isCreatingDocument = false;
                    return;
                }
            } catch (error) {
                console.error('Failed to create document on server:', error);
                this.pendingEditCommands = [];
                this.isCreatingDocument = false;
                return;
            } finally {
                this.isCreatingDocument = false;
                this.pendingEditCommands = [];
            }
        }

        try {
            // Prepare document for server
            const docData = this.document.toJSON();

            // Send edit command to server via PUT
            const response = await fetch(`/api/documents/${this.document.documentUUID}`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    document: docData,
                    edit_command: editCommand,
                    notation_type: this.notationType || 'number'
                })
            });

            if (response.ok) {
                const result = await response.json();
                if (result.success && result.document) {
                    // Update document from server response
                    this.document.fromJSON(result.document);
                    this.isDirty = true;

                    // Update document tab display
                    this.updateDocumentTabDisplay();

                    // Cursor position is already updated in document.ui_state

                    // Update all format tabs from server response
                    if (result.formats) {
                        // Update editor SVG
                        if (result.formats.editor_svg) {
                            this.renderEditorSvg(result.formats.editor_svg);
                        }

                        // Update all format tabs using the unified function
                        if (window.UI && window.UI.updateFormatsFromBackend) {
                            window.UI.updateFormatsFromBackend(result.formats);
                        }
                    }

                    // Save and notify
                    this.saveToLocalStorage();
                    if (this.onContentChange) {
                        this.onContentChange('');
                    }
                    if (this.onSelectionChange) {
                        this.onSelectionChange({
                            // UUID-based selection
                        });
                    }
                }
            } else {
                // Handle error response
                const errorText = await response.text();
                console.error('PUT request failed:', response.status, errorText);

                // Try to parse as JSON for detailed error
                try {
                    const errorJson = JSON.parse(errorText);
                    console.error('Server error details:', errorJson);
                } catch {
                    console.error('Server error (raw):', errorText);
                }
            }
        } catch (error) {
            console.error('Failed to apply edit command:', error);
        }
    }

    // Handle Enter key - document-first architecture
    handleEnterKey() {
        this.insertCharacter('\n');
    }

    // Move cursor
    moveCursor(direction) {
        this.document.ui_state.selection.cursor_position = Math.max(0, Math.min(this.document.content?.length || 0, this.document.ui_state.selection.cursor_position + direction));
        // Clear UUID selection
        // this.resetCursorBlink(); // Disabled blinking cursor

        // Keep changes local (no server call)
        this.document.ui_state.selection.selected_uuids = [];

        // Do not persist on arrow move; keep client-side only

        // Update client-side cursor (no server call needed)
        if (typeof this.postRenderCaret === 'function') this.postRenderCaret(); else this.initializeVisualCursor();

        // Notify selection change
        if (this.onSelectionChange) {
            this.onSelectionChange({
                // UUID-based selection
            });
        }
    }

    // Move cursor vertically (up or down lines)
    moveCursorVertical(direction) {
        if (!this.characterPositions || Object.keys(this.characterPositions).length === 0) {
            return; // No coordinate data available
        }

        // Get current position coordinates
        const currentCoords = this.characterPositions[this.document.ui_state.selection.cursor_position];
        if (!currentCoords) {
            return; // Current position not tracked
        }

        const currentX = typeof currentCoords === 'number' ? currentCoords : currentCoords.x;
        const currentY = typeof currentCoords === 'number' ? 0 : currentCoords.y;

        // Find target Y coordinate (line above or below)
        const targetY = currentY + (direction * 60); // Assuming 60px line height

        // Find the position on the target line closest to current X
        let bestPosition = this.document.ui_state.selection.cursor_position;
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
        if (bestPosition !== this.document.ui_state.selection.cursor_position) {
            this.document.ui_state.selection.cursor_position = bestPosition;
            // Clear selection when clicking

            // Keep changes local (no server call)
            this.document.ui_state.selection.selected_uuids = [];

            // Do not persist on vertical move; keep client-side only

            // Update client-side cursor (no server call needed)
            if (typeof this.postRenderCaret === 'function') this.postRenderCaret(); else this.initializeVisualCursor();

            // Notify selection change
            if (this.onSelectionChange) {
                this.onSelectionChange({
                    // UUID-based selection
                });
            }
        }
    }


    // Send document to server for processing and update all tabs
    // In local-first architecture, we send the document we have
    async fetchDocumentByUUID(documentUUID) {
        if (this.suspendServerSync) {
            const err = new Error('fetchDocumentByUUID() called during suspended UI interaction');
            console.error(err);
            throw err;
        }
        try {
            // Send our document to server for processing
            const response = await fetch(`/api/documents/${documentUUID}`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    document: this.document.toJSON(),
                    notation_type: this.notationType || 'number'
                })
            });

            if (!response.ok) {
                throw new Error(`Failed to fetch document: ${response.status}`);
            }

            const result = await response.json();

            // Update local document with server data
            if (result.document) {
                // Update document model with server version
                this.document = DocumentModel.fromJSON(result.document);

                // Update text content if available from server
                // Server might store music_text in different places
                if (result.document.music_text) {
                    // Document-first: no text tracking
                }

                // Update editor state from document UI state
                if (this.document.ui_state) {
                    // cursor_position already in document.ui_state.selection
                    this.selectedUuids = new Set(this.document.ui_state.selection.selected_uuids || []);
                    // Note: currentMode is editor-specific, not document-specific
                }
            }

            // Update all UI tabs with formats
            if (result.formats) {
                if (result.formats.editor_svg) {
                    this.renderEditorSvg(result.formats.editor_svg);
                }

                // Update all format tabs using the new unified function
                if (window.UI && window.UI.updateFormatsFromBackend) {
                    window.UI.updateFormatsFromBackend(result.formats);
                }

                // Update document tab with the document JSON
                if (window.UI) {
                    // Update document status
                    if (window.UI.updateDocumentStatus) {
                        window.UI.updateDocumentStatus();
                    }

                    // Update document display
                    const documentOutput = document.getElementById('document-output');
                    if (documentOutput && result.document) {
                        documentOutput.textContent = JSON.stringify(result.document, null, 2);
                    }
                    if (result.formats.svg && window.UI.updateSVGSourceOutput) {
                        window.UI.updateSVGSourceOutput({
                            success: true,
                            editor_svg: result.formats.svg
                        });
                    }
                    if (result.formats.vexflow && window.UI.updateVexFlowOutput) {
                        window.UI.updateVexFlowOutput({
                            success: true,
                            vexflow: result.formats.vexflow
                        });
                    }
                }
            }

            return result;

        } catch (error) {
            console.error('Failed to fetch document by UUID:', error);
            throw error;
        }
    }

    // Render document using RESTful document API
    async renderDocument() {
        if (this.suspendServerSync) {
            const err = new Error('renderDocument() called during suspended UI interaction');
            console.error(err);
            throw err;
        }
        try {
            let documentData;

            // If we don't have a document UUID, create a new document on the server
            if (!this.document.documentUUID) {
                const createResponse = await fetch('/api/documents', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({
                        // Document-first: no music_text
                        metadata: this.document.metadata
                    })
                });

                if (!createResponse.ok) {
                    throw new Error(`Failed to create document: ${createResponse.status}`);
                }

                const createResult = await createResponse.json();
                this.document.fromJSON(createResult.document);
                documentData = createResult.document;

                // Update local document with server-generated data
                if (createResult.formats && createResult.formats.editor_svg) {
                    this.renderEditorSvg(createResult.formats.editor_svg);
                    return;
                }
            }

            // In local-first architecture, send document to server for rendering
            // Use PUT without edit command to just get formats
            const response = await fetch(`/api/documents/${this.document.documentUUID}`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    document: this.document.toJSON(),
                    notation_type: this.notationType || 'number'
                })
            });

            if (response.ok) {
                const result = await response.json();

                // Update local document with server data
                if (result.document) {
                    this.document.fromJSON(result.document);
                    this.updateDocumentTabDisplay();
                }

                // Render the SVG if available
                if (result.formats && result.formats.editor_svg) {
                    this.renderEditorSvg(result.formats.editor_svg);
                } else if (result.formats && result.formats.svg) {
                    this.renderEditorSvg(result.formats.svg);
                } else {
                    this.renderError('No SVG format available');
                }
            } else {
                const error = await response.text();
                console.error('Server error:', error);
                this.renderError(`Server error: ${response.status}`);
            }

        } catch (error) {
            console.error('Document operation failed:', error);
            this.renderError('Request failed: ' + error.message);
        }
    }

    // Legacy method - redirect to document-based rendering
    async submitToServer(inputText) {
        if (!inputText || !inputText.trim()) {
            this.clearCanvas();
            return;
        }

        // Update local text cache
        // Document-first: no text tracking

        // Use the new document-based rendering
        return this.renderDocument();
    }

    // Render SVG content in the canvas
    renderEditorSvg(svgContent) {
        console.log('renderEditorSvg called with content length:', svgContent ? svgContent.length : 0);

        // Create a temporary container to hold the SVG
        const tempDiv = document.createElement('div');
        tempDiv.innerHTML = svgContent;
        const svgElement = tempDiv.querySelector('svg');

        if (svgElement) {
            console.log('SVG element found, rendering to container');
            // Extract UUID data from SVG elements (both Notes and Beats)
            this.elementUuidMap.clear();

            // Handle Note UUIDs
            const noteElements = svgElement.querySelectorAll('[data-note-id]');
            noteElements.forEach(element => {
                const uuid = element.getAttribute('data-note-id');
                const charStart = parseInt(element.getAttribute('data-char-start') || '0');
                const charEnd = parseInt(element.getAttribute('data-char-end') || '0');
                const x = parseFloat(element.getAttribute('x') || '0');
                const y = parseFloat(element.getAttribute('y') || '0');
                const elementType = element.getAttribute('data-element-type') || 'note';

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

            // Handle Beat UUIDs
            const beatElements = svgElement.querySelectorAll('[data-beat-id]');
            beatElements.forEach(element => {
                const uuid = element.getAttribute('data-beat-id');
                const charStart = parseInt(element.getAttribute('data-char-start') || '0');
                const charEnd = parseInt(element.getAttribute('data-char-end') || '0');
                const x = parseFloat(element.getAttribute('x') || '0');
                const y = parseFloat(element.getAttribute('y') || '0');
                const elementType = element.getAttribute('data-element-type') || 'beat';

                // Only add Beat UUIDs if not already mapped by a Note UUID
                if (!this.elementUuidMap.has(uuid)) {
                    this.elementUuidMap.set(uuid, {
                        uuid,
                        charStart,
                        charEnd,
                        x,
                        y,
                        elementType,
                        element: element
                    });
                }
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

            // Clear existing content and insert SVG directly into DOM
            this.svgContainer.innerHTML = '';

            // Clone the SVG element and insert directly
            const svgClone = svgElement.cloneNode(true);

            // Remove server-rendered cursor to avoid duplicates
            const serverCursor = svgClone.querySelector('#svg-cursor');
            if (serverCursor) {
                serverCursor.remove();
            }

            // Ensure SVG has proper dimensions and styling
            svgClone.style.width = '100%';
            svgClone.style.height = 'auto';
            svgClone.style.display = 'block';

            // Insert SVG directly into the container
            this.svgContainer.appendChild(svgClone);

            // Store reference to the SVG for future manipulation
            this.currentSvg = svgClone;

            // Draw debug boxes for each character if enabled
            if (this.debugCharBoxes) {
                this.drawDebugCharBoxes();
            }

            // Initialize caret on freshly inserted SVG
            if (typeof this.postRenderCaret === 'function') {
                this.postRenderCaret();
            } else {
                this.initializeVisualCursor();
            }

            // Reapply debug boxes after caret placement in case caret changed DOM order
            if (this.debugCharBoxes) {
                this.drawDebugCharBoxes();
            }

            // Extract transform values from SVG
            this.extractSvgTransform();

            // Update cursor coordinates from UUID-based element positions
            this.updateCursorFromUUIDs();

            // Apply any existing selection highlighting
            this.updateClientSideSelection();

            // Ensure caret present after other DOM changes
            if (typeof this.postRenderCaret === 'function') {
                this.postRenderCaret();
            } else {
                this.initializeVisualCursor();
            }
        } else {
            console.warn('No SVG element found in server response');
            this.renderError('Invalid SVG response');
        }
    }

    // Draw a debug rectangle around each character-like element for visual verification
    drawDebugCharBoxes() {
        try {
            if (!this.currentSvg) return;
            const root = this.currentSvg;

            // Remove any existing debug boxes
            root.querySelectorAll('[data-debug="char-bbox"]').forEach(n => n.remove());

            // Prefer drawing inside the main content group so transforms apply
            const contentGroup = root.querySelector('.canvas-content') || root;

            // Candidate selectors for char-like elements
            const selectors = [
                '[data-char-index]',
                '.char',
                '.note-char',
                '.dash-char',
                '.lyrics-char',
                '.text-char',
                'text'
            ];
            const seen = new Set();
            for (const sel of selectors) {
                const list = contentGroup.querySelectorAll(sel);
                for (const el of list) {
                    if (seen.has(el)) continue;
                    seen.add(el);
                    if (typeof el.getBBox !== 'function') continue;
                    const bbox = el.getBBox();
                    if (!bbox || bbox.width <= 0 || bbox.height <= 0) continue;

                    const rect = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
                    rect.setAttribute('x', String(bbox.x));
                    rect.setAttribute('y', String(bbox.y));
                    rect.setAttribute('width', String(bbox.width));
                    rect.setAttribute('height', String(bbox.height));
                    rect.setAttribute('data-debug', 'char-bbox');
                    rect.setAttribute('class', 'char-bbox-debug');
                    rect.setAttribute('fill', 'none');
                    rect.setAttribute('stroke', '#e53935');
                    rect.setAttribute('stroke-width', '0.8');
                    rect.setAttribute('opacity', '0.6');
                    rect.style.pointerEvents = 'none';
                    contentGroup.appendChild(rect);
                }
            }
        } catch (e) {
            console.warn('drawDebugCharBoxes error:', e);
        }
    }

    // Update cursor coordinates from UUID-based element positions
    updateCursorFromUUIDs() {
        const cursorUUID = this.document.ui_state.selection.cursor_uuid;
        if (!cursorUUID) {
            // No UUID cursor, position at start
            this.cursorX = this.svgTransformX || 20;
            this.cursorY = this.svgTransformY || 20;
            return;
        }

        // Find the element with this UUID in the SVG
        const targetElement = this.currentSvg?.querySelector(`[data-note-id="${cursorUUID}"], [data-beat-id="${cursorUUID}"]`);
        if (targetElement) {
            // Get element position directly from SVG attributes
            const x = parseFloat(targetElement.getAttribute('x')) || 0;
            const y = parseFloat(targetElement.getAttribute('y')) || 0;
            const width = parseFloat(targetElement.getAttribute('data-width')) || 12;

            // Position cursor after this element
            this.cursorX = x + width;
            this.cursorY = y;

            console.log('Updated cursor coordinates from UUID:', {
                uuid: cursorUUID.slice(0, 8),
                x: this.cursorX,
                y: this.cursorY
            });
        } else {
            console.warn('Could not find element for cursor UUID:', cursorUUID);
            this.positionCursorAtEnd();
        }
    }

    // Position cursor at end of content
    positionCursorAtEnd() {
        // Position at end of the last rendered element
        const lastElement = this.currentSvg?.querySelector('[data-note-id]:last-of-type, [data-beat-id]:last-of-type');
        if (lastElement) {
            const rect = lastElement.getBoundingClientRect();
            const svgRect = this.currentSvg.getBoundingClientRect();
            this.cursorX = (rect.right - svgRect.left);
            this.cursorY = (rect.top - svgRect.top) + rect.height;
        } else {
            // Default position at start
            this.cursorX = this.svgTransformX || 20;
            this.cursorY = this.svgTransformY || 20;
        }
    }

    // Extract SVG transform values for coordinate calculations
    extractSvgTransform() {
        if (!this.currentSvg) return;

        // Find the main content group with transform
        const contentGroup = this.currentSvg.querySelector('.canvas-content');
        if (contentGroup) {
            const transform = contentGroup.getAttribute('transform');
            if (transform) {
                // Parse translate(x, y) values
                const match = transform.match(/translate\(([^,]+),\s*([^)]+)\)/);
                if (match) {
                    this.svgTransformX = parseFloat(match[1]);
                    this.svgTransformY = parseFloat(match[2]);
                    console.log('📐 Extracted SVG transform:', { x: this.svgTransformX, y: this.svgTransformY });
                    return;
                }
            }
        }

        // Fallback to default values if not found
        this.svgTransformX = 20;
        this.svgTransformY = 20;
        console.warn('⚠️ Could not extract SVG transform, using defaults');
    }

    // Apply client-side selection highlighting to SVG elements
    updateClientSideSelection() {
        if (!this.currentSvg) return;

        // If no selection, just ensure caret is visible and return (no server refresh)
        if (this.selectedUuids.size === 0) {
            if (typeof this.postRenderCaret === 'function') this.postRenderCaret();
            return;
        }

        // Clear ALL potential selection highlighting (brute force approach)
        this.currentSvg.querySelectorAll('[data-note-id], [data-beat-id]').forEach(el => {
            el.classList.remove('svg-selected');
            // Force style reset to ensure visual clearing
            el.style.fill = '';
            el.style.stroke = '';
            el.style.background = '';
            el.style.strokeWidth = '';
        });

        // Apply selection to elements with matching UUIDs
        this.selectedUuids.forEach(uuid => {
            // Select by data-beat-id (beats)
            const beatElements = this.currentSvg.querySelectorAll(`[data-beat-id="${uuid}"]`);
            beatElements.forEach(el => el.classList.add('svg-selected'));

            // Select by data-note-id (notes)
            const noteElements = this.currentSvg.querySelectorAll(`[data-note-id="${uuid}"]`);
            noteElements.forEach(el => el.classList.add('svg-selected'));
        });
    }

    // Add or update client-side cursor in SVG (element-based positioning)
    updateClientSideCursor() {
        if (!this.currentSvg) return;

        // Remove existing cursor
        const existingCursor = this.currentSvg.querySelector('#client-cursor');
        if (existingCursor) {
            existingCursor.remove();
        }

        // Use stored cursor coordinates if available
        let x, y;
        if (this.cursorX !== undefined && this.cursorY !== undefined) {
            x = this.cursorX;
            y = this.cursorY;
        } else {
            // Fallback to default position
            x = this.svgTransformX || 20;
            y = this.svgTransformY || 20;
        }

        // Cursor visualization disabled
        /*
        // Create cursor line element
        const cursor = document.createElementNS('http://www.w3.org/2000/svg', 'line');
        cursor.setAttribute('id', 'client-cursor');
        cursor.setAttribute('x1', x);
        cursor.setAttribute('x2', x);
        cursor.setAttribute('y1', y - 10);
        cursor.setAttribute('y2', y + 15);
        cursor.setAttribute('class', 'svg-cursor');

        // Add cursor to SVG
        this.currentSvg.appendChild(cursor);
        */
    }

    // Find SVG element at current cursor position
    findElementAtCursorPosition() {
        if (!this.elementUuidMap || this.elementUuidMap.size === 0) return null;

        // Find element that contains or is closest to cursor position
        let bestElement = null;
        let bestDistance = Infinity;

        for (const [uuid, elementData] of this.elementUuidMap) {
            // Check if cursor is within this element's character range
            if (this.document.ui_state.selection.cursor_position >= elementData.charStart && this.document.ui_state.selection.cursor_position <= elementData.charEnd) {
                return {
                    uuid,
                    x: elementData.x,
                    y: elementData.y,
                    width: elementData.element ? elementData.element.getBoundingClientRect().width / 4 : 12, // Rough width
                    element: elementData.element
                };
            }

            // Track closest element if cursor is not within any element
            const distance = Math.abs(this.document.ui_state.selection.cursor_position - elementData.charStart);
            if (distance < bestDistance) {
                bestDistance = distance;
                bestElement = elementData;
            }
        }

        // Return closest element if found
        if (bestElement) {
            return {
                uuid: bestElement.uuid,
                x: bestElement.x,
                y: bestElement.y,
                width: bestElement.element ? bestElement.element.getBoundingClientRect().width / 4 : 12,
                element: bestElement.element
            };
        }

        return null;
    }

    // Find character element at a specific point (simple .char element detection)
    findCharacterAtPoint(x, y) {
        if (!this.currentSvg) return null;

        // Account for the SVG transform when comparing coordinates
        const adjustedX = x - (this.svgTransformX || 20);
        const adjustedY = y - (this.svgTransformY || 20);

        // Look for character elements with semantic classes at the click point
        const selector = '.char, .note-char, .rest-char, .barline-char, .dash-char, ' +
                        '.breath-char, .lyrics-char, .text-char, .unknown-char, ' +
                        '.upper-char, .lower-char, .whitespace-char';
        const charElements = this.currentSvg.querySelectorAll(selector);
        for (const char of charElements) {
            // Get element position directly from attributes
            const elemX = parseFloat(char.getAttribute('x') || '0');
            const elemY = parseFloat(char.getAttribute('y') || '0');
            const elemWidth = parseFloat(char.getAttribute('data-width')) || 12;
            const elemHeight = 20; // Default font size

            // Check if click is within character bounds
            // Note: Y coordinate needs adjustment for text baseline
            if (adjustedX >= elemX && adjustedX <= elemX + elemWidth &&
                adjustedY >= elemY - elemHeight && adjustedY <= elemY + 5) {
                return char;
            }
        }
        return null;
    }

    // Find content line element at a specific point (only notes/beats from content lines)
    findContentLineElementAtPoint(x, y) {
        if (!this.currentSvg) return null;

        // Look for all content line elements, excluding decorative ones
        const allElements = this.currentSvg.querySelectorAll('text, rect');

        // Use negative selector - exclude octave markers, slurs, and other decorations
        const clickableElements = Array.from(allElements).filter(element => {
            const classList = element.className.baseVal || '';

            // Exclude decorative elements by class
            return !classList.includes('canvas-octave-upper') &&
                   !classList.includes('canvas-octave-lower') &&
                   !classList.includes('slur') &&
                   !classList.includes('marker');
        });

        for (const element of clickableElements) {
            const bbox = element.getBBox();
            const x1 = bbox.x;
            const y1 = bbox.y;
            const x2 = bbox.x + bbox.width;
            const y2 = bbox.y + bbox.height;

            if (x >= x1 && x <= x2 && y >= y1 && y <= y2) {
                // Debug: log what we found
                const uuid = element.getAttribute('data-note-id') || element.getAttribute('data-beat-id');
                const elementType = element.getAttribute('data-element-type');
                console.log('📍 Clicked element:', {
                    tagName: element.tagName,
                    className: element.className.baseVal,
                    uuid,
                    elementType,
                    hasCharStart: element.hasAttribute('data-char-start'),
                    charStart: element.getAttribute('data-char-start')
                });

                return {
                    element,
                    uuid,
                    x: x1,
                    y: y1,
                    width: bbox.width,
                    height: bbox.height,
                    elementType
                };
            }
        }

        return null;
    }

    // Position cursor at a specific SVG element
    positionCursorAtElement(elementInfo, clickX, clickY) {
        // Find the element data in our UUID map
        const elementData = this.elementUuidMap.get(elementInfo.uuid);
        if (elementData) {
            // Determine if click was closer to start or end of element
            const elementCenter = elementInfo.x + (elementInfo.width / 2);
            if (clickX < elementCenter) {
                // Clicked on left side - position at start
                this.document.ui_state.selection.cursor_position = elementData.charStart;
                this.cursorX = elementInfo.x;
            } else {
                // Clicked on right side - position at end
                this.document.ui_state.selection.cursor_position = elementData.charEnd;
                this.cursorX = elementInfo.x + elementInfo.width;
            }
            this.cursorY = elementInfo.y;
        }
    }

    // Position cursor at a specific point in empty space
    positionCursorAtPoint(x, y) {
        // Store cursor coordinates for rendering
        this.cursorX = x;
        this.cursorY = y;

        // For empty space, we can estimate character position or use 0
        this.document.ui_state.selection.cursor_position = 0; // Or estimate based on position
    }

    // Render error message in SVG container
    renderError(errorMessage) {
        this.svgContainer.innerHTML = '';

        // Create SVG with error message
        const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
        svg.setAttribute('width', '100%');
        svg.setAttribute('height', '100%');
        svg.style.background = '#fafafa';

        const text = document.createElementNS('http://www.w3.org/2000/svg', 'text');
        text.setAttribute('x', '50%');
        text.setAttribute('y', '50%');
        text.setAttribute('text-anchor', 'middle');
        text.setAttribute('dominant-baseline', 'middle');
        text.setAttribute('fill', '#e74c3c');
        text.setAttribute('font-family', 'monospace');
        text.setAttribute('font-size', '14px');
        text.textContent = 'Error: ' + errorMessage;

        svg.appendChild(text);
        this.svgContainer.appendChild(svg);
        this.currentSvg = svg;
    }

    // Clear SVG container
    clearCanvas() {
        this.svgContainer.innerHTML = '';

        // Create placeholder SVG
        const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
        svg.setAttribute('width', '100%');
        svg.setAttribute('height', '100%');
        svg.style.background = '#fafafa';
        svg.style.border = '1px solid #dddddd';

        const text = document.createElementNS('http://www.w3.org/2000/svg', 'text');
        text.setAttribute('x', '50%');
        text.setAttribute('y', '50%');
        text.setAttribute('text-anchor', 'middle');
        text.setAttribute('dominant-baseline', 'middle');
        text.setAttribute('fill', '#999');
        text.setAttribute('font-family', 'monospace');
        text.setAttribute('font-size', '14px');
        text.textContent = 'Type music notation to begin...';

        svg.appendChild(text);
        this.svgContainer.appendChild(svg);
        this.currentSvg = svg;
    }

    // Render cursor at current position (now handled by updateClientSideCursor)
    renderCursor() {
        // Legacy method - now handled by updateClientSideCursor()
        // This method updates the SVG cursor (blink timer disabled)
        this.initializeVisualCursor();
    }

    // Calculate cursor X position based on text content
    calculateCursorX() {
        if ((this.document.content?.length || 0) === 0 || this.document.ui_state.selection.cursor_position === 0) {
            return 20; // Left margin
        }

        // Get text up to cursor position
        // Document-first: calculate from elements
        const textToCursor = '';
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

        if ((this.document.content?.length || 0) === 0 || this.document.ui_state.selection.cursor_position === 0) {
            return topMargin; // Top position for empty text or start
        }

        // Count lines up to cursor
        // Document-first: calculate from elements
        const textToCursor = '';
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
            const rect = this.svgContainer.getBoundingClientRect();
            const scaleX = this.canvas.width / rect.width;
            const scaleY = this.canvas.height / rect.height;

            // Scale click coordinates to match internal canvas coordinates
            const scaledX = x * scaleX - (this.svgTransformX || 20); // Account for SVG transform
            const scaledY = y * scaleY - (this.svgTransformY || 20); // Account for SVG transform

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
        // Document-first: insert elements instead of text
        console.log('Insert text in document-first mode:', text);
        this.document.ui_state.selection.cursor_position += text.length;
        // Clear UUID selection
        // Document-first: no line tracking
        this.isDirty = true;


        // Do not trigger server re-render here; keep local

        if (this.onContentChange) {
            this.onContentChange('');
        }
    }

    deleteSelectedElement() {
        if (!this.selectedElement) return;

        // This would require more sophisticated text manipulation
        // For now, just clear the selection
        this.selectedElement = null;
        // No-op for serverless selection removal
    }

    // Apply octave adjustments
    applyOctaveAdjustment(type) {
        // This would modify the text based on the selected adjustment
        console.log('Applying octave adjustment:', type);
    }

    // Utility methods for compatibility with existing code
    // Get current text content
    getValue() {
        return ''; // Document-first: no text tracking
    }

    // Copy the current selection to the server-side clipboard
    async copySelection() {
        const selection = this.getSelection();
        if (selection.uuids.length === 0) {
            UI.setStatus('No selection to copy.', 'error');
            return;
        }

        const editCommand = {
            type: 'copy_selection',
            selection_start: selection.characterRange.start,
            selection_end: selection.characterRange.end,
        };

        await this.applyEditCommand(editCommand);
        UI.setStatus('Selection copied.', 'success');
    }

    // Paste from the server-side clipboard
    async paste() {
        const editCommand = {
            type: 'paste',
            position: this.document.ui_state.selection.cursor_position,
            selection_start: this.selection.start,
            selection_end: this.selection.end,
        };

        await this.applyEditCommand(editCommand);
        UI.setStatus('Pasted from clipboard.', 'success');
    }

    // DEPRECATED: Use document operations instead of direct text manipulation
    setValue(content, cursorPos = null) {
        console.warn('setValue() is deprecated. Use document operations for document-first architecture.');

        // Document-first: no text tracking
        console.warn('setValue() called - ignoring in document-first mode');

        // If cursor position provided, use it; otherwise keep current (clamped)
        if (cursorPos !== null) {
            this.document.ui_state.selection.cursor_position = Math.min(cursorPos, content.length);
            // Clear selection when clicking
        } else {
            this.document.ui_state.selection.cursor_position = Math.min(this.document.ui_state.selection.cursor_position, content.length);
        }

        this.isDirty = true;
    }

    focus() {
        this.svgContainer.focus();
    }

    getCursorPosition() {
        return {
            // UUID-based selection
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
            await this.draw('');

            return result;

        } catch (error) {
            console.error('Semantic command failed:', error);
            throw error;
        }
    }

    // Execute document transformation using the transform endpoint
    async executeTransform(transformType, parameters = {}) {
        if (this.selectedUuids.size === 0) {
            throw new Error('No elements selected for transformation');
        }

        try {
            // Use the stored document (maintains UUID consistency)
            if (!this.document) {
                throw new Error('No document available for transformation');
            }

            const requestData = {
                command_type: transformType,
                document: this.document, // Use stored document with consistent UUIDs
                target_uuids: Array.from(this.selectedUuids),
                parameters: parameters
            };

            console.log('Executing transform:', {
                type: transformType,
                targets: Array.from(this.selectedUuids),
                parameters: parameters
            });

            const response = await fetch('/api/documents/transform', {
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
                throw new Error(result.message || 'Transform failed');
            }

            console.log('Transform executed successfully:', result);

            // Update the document with the transformed version
            if (result.document) {
                // Store the transformed document - following Document Serialization Pattern
                this.document = result.document;

                // Re-render the canvas with the transformed document
                await this.renderFromDocument();
            }

            return result;

        } catch (error) {
            console.error('Transform failed:', error);
            throw error;
        }
    }

    // Render canvas directly from document structure (Document Serialization Pattern)
    async renderFromDocument() {
        if (!this.document) {
            this.clearCanvas();
            return;
        }

        try {
            // Prepare document-first request data
            const requestData = {
                // Send the document structure directly
                document_model: this.document,
                selected_uuids: Array.from(this.selectedUuids),
                cursor_position: this.document.ui_state.selection.cursor_position,

                // Legacy fields for compatibility (may not be needed)
                input_text: "", // No text input, using document
                notation_type: "sargam" // Default notation system
            };

            // Use the canvas SVG API with document-first approach
            const response = await fetch('/api/canvas-svg', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(requestData)
            });

            if (response.ok) {
                const svgContent = await response.text();
                this.renderEditorSvg(svgContent);

                console.log('Rendered canvas from document structure');
            } else {
                console.error('Failed to render from document:', response.status, response.statusText);
            }

        } catch (error) {
            console.error('Error rendering from document:', error);
        }
    }

    // Generic method to apply text transformations via API (legacy)
    async applyTransformation(endpoint, transformData) {
        try {
            // Add current text, character-based selection, and UUID-based selection
            const requestData = {
                text: '', // Document-first: no text tracking
                selected_uuids: Array.from(this.selectedUuids),
                cursor_position: this.document.ui_state.selection.cursor_position,
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
                // Document-first: no text tracking
                this.isDirty = true;
            }

            // Update cursor and selection if provided
            if (result.selection_start !== undefined) {
                // UUID-based selection
            }
            if (result.selection_end !== undefined) {
                // UUID-based selection
            }
            // Cursor position is already in document.ui_state
            if (result.selection_end !== undefined) {
                this.document.ui_state.selection.cursor_position = result.selection_end;
            }

            // Save and submit
            this.saveToLocalStorage();
            this.draw();

            // Trigger callbacks
            if (this.onContentChange) {
                this.onContentChange('');
            }
            if (this.onSelectionChange) {
                this.onSelectionChange({
                    // UUID-based selection
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
        // UUID-based selection - not numeric positions
        console.warn('setSelection called with numeric positions - ignoring');
        this.document.ui_state.selection.cursor_position = 0;
        // this.resetCursorBlink(); // Disabled blinking cursor

        // Notify selection change unless silent flag is set
        if (!silent && this.onSelectionChange) {
            this.onSelectionChange({
                // UUID-based selection
            });
        }
    }

    // Update selection tracking
    updateSelection() {
        if (this.onSelectionChange) {
            this.onSelectionChange({
                // UUID-based selection
            });
        }
    }

    // Check if text should be converted to uppercase for sargam
    shouldConvertToSargamUppercase() {
        const text = this.textContent;
        const lines = text.split('\n');
        const cursorPos = this.document.ui_state.selection.cursor_position;

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

    // Update the document tab display
    updateDocumentTabDisplay() {
        const documentOutput = document.getElementById('document-output');
        if (documentOutput && this.document) {
            documentOutput.textContent = JSON.stringify(this.document.toJSON(), null, 2);
        }
    }

    // Save editor state to local storage (document-first)
    saveToLocalStorage() {
        try {
            // Skip saving if no document or document has no UUID (not yet created on server)
            if (!this.document || !this.document.documentUUID) {
                console.log('Skipping localStorage save - no document or no UUID yet');
                return;
            }

            // Update document model UI state locally only
            if (this.document.ui_state && this.document.ui_state.selection) {
                this.document.ui_state.selection.selected_uuids = Array.from(this.selectedUuids);
                // Keep cursor_uuid null for client-only caret
            }

            // Note: text content is sent to server, not cached in document

            // Save document model
            const saved = this.persistence.saveDocument(this.document);
            if (saved) {
                console.log('Document saved to localStorage:', this.document.getStats());
            }

            // Update the document tab display
            this.updateDocumentTabDisplay();

            // Legacy backup save removed - using document-first approach only

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
                if (loadedDocument.documentUUID) {
                    // Valid document with UUID
                    this.document = loadedDocument;

                    // Restore UI state from document
                    // Text might be stored directly or need to be fetched from server
                    // Document-first: no text tracking
                    // cursor_position already in document.ui_state.selection

                    // Restore UUID-based selection
                    this.selectedUuids = new Set(this.document.ui_state.selection.selected_uuids);

                    // Update legacy character-based selection for compatibility
                    this.updateCharacterSelectionFromUuids();
                    // Document-first: no line tracking

                    console.log('Loaded document from localStorage:', this.document.getStats());

                    // Update UI to show document UUID
                    console.log('Document UUID loaded from localStorage:', this.document.documentUUID);
                    if (window.UI && window.UI.updateDocumentStatus) {
                        window.UI.updateDocumentStatus();
                    }

                    // In local-first architecture, render the document we have
                    // Don't fetch from server - we already have the document
                    if (this.document.documentUUID) {
                        console.log('Rendering document from localStorage:', this.document.documentUUID);
                        // Update document tab display
                        this.updateDocumentTabDisplay();
                        // Request standard representations once
                        this.afterDocumentLoaded();
                    } else {
                        // If no document UUID, clear the canvas
                        this.clearCanvas();
                    }

                    return true;
                } else {
                    // Invalid document without UUID - clear it
                    console.warn('Clearing invalid document from localStorage (no UUID)');
                    this.persistence.clearDocument();
                    return false;
                }
            } else {
                return false;
            }

            // No legacy fallback - using document-first approach only

        } catch (e) {
            console.error('Failed to load from localStorage:', e);
        }
        return false;
    }

    // Handle keyboard input (arrow keys for cursor movement)
    handleKeydown(e) {
        switch(e.key) {
            case 'ArrowLeft':
                e.preventDefault();
                this.moveCursorLeft();
                break;
            case 'ArrowRight':
                e.preventDefault();
                this.moveCursorRight();
                break;
            case 'ArrowUp':
                e.preventDefault();
                this.moveCursorUp();
                break;
            case 'ArrowDown':
                e.preventDefault();
                this.moveCursorDown();
                break;
        }
    }

    // Get all content-line elements (char class) in order
    getContentElements() {
        if (!this.currentSvg) return [];
        // Select all character elements with semantic classes
        const selector = '.char, .note-char, .rest-char, .barline-char, .dash-char, ' +
                        '.breath-char, .lyrics-char, .text-char, .unknown-char, ' +
                        '.upper-char, .lower-char, .whitespace-char';
        return Array.from(this.currentSvg.querySelectorAll(selector));
    }

    // Find current cursor element index
    getCurrentCursorIndex() {
        const elements = this.getContentElements();
        const currentUUID = this.document.ui_state.selection.cursor_uuid;

        if (!currentUUID) return -1;

        return elements.findIndex(el =>
            el.getAttribute('data-source-uuid') === currentUUID
        );
    }

    // Move cursor to previous position
    moveCursorLeft() {
        if (!this.document) return;

        const currentPos = this.document.ui_state.selection.cursor_position || 0;
        if (currentPos > 0) {
            this.setCursorPosition(currentPos - 1);
        }
    }

    // Move cursor to next position
    moveCursorRight() {
        if (!this.document) return;

        const currentPos = this.document.ui_state.selection.cursor_position || 0;
        // Allow cursor to go one position past the last character
        const elements = this.getContentElements();
        if (currentPos <= elements.length) {
            this.setCursorPosition(currentPos + 1);
        }
    }

    // Set cursor to a specific position (standard approach)
    setCursorPosition(position) {
        if (!this.document) return;

        // Update document state
        this.document.ui_state.selection.cursor_position = position;

        // Update visual position
        const coords = this.getCoordinatesForPosition(position);
        if (coords) {
            // Ensure cursor exists
            if (!this.cursorElement) {
                this.createCursorElement(coords.x, coords.y);
            } else {
                // Just move existing cursor
                this.updateCursorPosition(coords.x, coords.y);
            }
        }

        console.log(`Cursor position: ${position}`);
    }

    // Get (x,y) coordinates for a cursor position
    getCoordinatesForPosition(position) {
        if (!this.currentSvg) return null;

        console.log(`getCoordinatesForPosition called with position: ${position}`);

        // Standard approach: position N means "after character N-1, before character N"
        // Position 0 = before all characters
        // Position 1 = after char 0, before char 1
        // Position length = after all characters

        const elements = this.getContentElements();
        console.log(`Found ${elements.length} content elements`);

        // Default coordinates - use reasonable defaults for visibility
        let x = 20;  // Some left margin
        let y = 60;  // Reasonable baseline position

        if (elements.length === 0) {
            console.log('No elements found, using default position');
            return { x: 20, y: 60 };
        }

        if (position === 0) {
            // Before first character
            const firstEl = elements[0];
            if (firstEl) {
                x = parseFloat(firstEl.getAttribute('x') || '0');
                y = parseFloat(firstEl.getAttribute('y') || '20');
                console.log(`Position 0: first element at x=${x}, y=${y}`);
            }
        } else if (position > 0 && position <= elements.length) {
            // After character at index (position - 1)
            const charEl = elements[position - 1];
            if (charEl) {
                x = parseFloat(charEl.getAttribute('x') || '0');
                y = parseFloat(charEl.getAttribute('y') || '20');

                // Add character width to position cursor after it
                const width = parseFloat(charEl.getAttribute('data-width') || '12');
                x += width;
                console.log(`Position ${position}: after char ${position-1} at x=${x}, y=${y}`);
            }
        } else if (position > elements.length) {
            // Past last character - position at end
            const lastEl = elements[elements.length - 1];
            if (lastEl) {
                x = parseFloat(lastEl.getAttribute('x') || '0');
                y = parseFloat(lastEl.getAttribute('y') || '20');
                const width = parseFloat(lastEl.getAttribute('data-width') || '12');
                x += width;
                console.log(`Position ${position}: past last char at x=${x}, y=${y}`);
            }
        }

        console.log(`Returning coordinates: x=${x}, y=${y}`);
        return { x, y };
    }

    // Initialize visual cursor as a separate SVG element
    initializeVisualCursor() {
        if (!this.currentSvg || !this.document) return;

        console.log('initializeVisualCursor called');
        console.log('currentSvg:', this.currentSvg);
        console.log('document cursor position:', this.document.ui_state.selection.cursor_position);

        // DEBUG: Comment out test cursor for now
        // this.addTestCursor();

        // Prefer new caret hook if available
        if (typeof this.postRenderCaret === 'function') {
            this.postRenderCaret();
            return;
        }
        // Fallback: set cursor to current position
        const cursorPos = this.document.ui_state.selection.cursor_position || 0;
        this.setCursorPosition(cursorPos);
    }

    // Sync caret index from document UI state
    updateCaretFromDocument() {
        try {
            const pos = this.document && this.document.ui_state && this.document.ui_state.selection
                ? this.document.ui_state.selection.cursor_position
                : 0;
            if (typeof pos === 'number' && !Number.isNaN(pos)) {
                this.caretIndex = pos;
            }
        } catch (_) {
            // noop
        }
    }

    // DEBUG: Add a test cursor to verify visibility
    addTestCursor() {
        if (!this.currentSvg) return;

        // Remove any existing test cursor
        const existing = this.currentSvg.querySelector('#test-cursor-debug');
        if (existing) existing.remove();

        // Add a big red rectangle as a test cursor
        const rect = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
        rect.setAttribute('id', 'test-cursor-debug');
        rect.setAttribute('x', '50');
        rect.setAttribute('y', '50');
        rect.setAttribute('width', '5');
        rect.setAttribute('height', '30');
        rect.setAttribute('fill', 'red');
        rect.setAttribute('opacity', '0.8');

        // Add to root SVG
        this.currentSvg.appendChild(rect);
        console.log('Added test cursor rectangle at (50, 50)');
    }

    // Create a cursor element in the SVG
    createCursorElement(x, y) {
        if (!this.currentSvg) return;

        // Remove any existing cursor first
        this.removeCursorElement();

        // Find the content group to add cursor to
        const contentGroup = this.currentSvg.querySelector('.canvas-content');
        if (!contentGroup) {
            console.warn('No .canvas-content group found in SVG');
            // Try to add cursor to the main SVG instead
            const svgRoot = this.currentSvg;

            // Account for the transform on the content group (usually translate(20, 20))
            const adjustedX = x + 20; // Add the transform offset
            const adjustedY = y + 20; // Add the transform offset

            const cursor = document.createElementNS('http://www.w3.org/2000/svg', 'line');
            cursor.setAttribute('id', 'text-cursor');
            cursor.setAttribute('class', 'svg-cursor');
            cursor.setAttribute('x1', adjustedX.toString());
            cursor.setAttribute('y1', (adjustedY - 20).toString()); // Above baseline
            cursor.setAttribute('x2', adjustedX.toString());
            cursor.setAttribute('y2', (adjustedY + 5).toString());  // Below baseline
            cursor.setAttribute('stroke', '#ff0000');  // Red for visibility
            cursor.setAttribute('stroke-width', '2');
            cursor.setAttribute('opacity', '1');
            cursor.style.animation = 'cursor-blink 1s infinite';

            svgRoot.appendChild(cursor);
            this.cursorElement = cursor;
            console.log('Created cursor in SVG root at adjusted position');
            return;
        }

        console.log(`Creating cursor in content group at (${x}, ${y})`);

        // Create cursor as a rectangle for better visibility
        const cursor = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
        cursor.setAttribute('id', 'text-cursor');
        cursor.setAttribute('class', 'svg-cursor');
        cursor.setAttribute('x', (x - 1).toString());  // Slightly offset for centering
        cursor.setAttribute('y', (y - 15).toString()); // Above baseline
        cursor.setAttribute('width', '2');
        cursor.setAttribute('height', '20');
        cursor.setAttribute('fill', '#000000');  // Black for visibility
        cursor.setAttribute('opacity', '1');

        // Add blinking animation with inline style
        cursor.style.animation = 'cursor-blink 1s infinite';

        // Ensure cursor is on top by appending last
        contentGroup.appendChild(cursor);

        // Store reference
        this.cursorElement = cursor;

        console.log('Cursor element created:', cursor);
        console.log('Cursor parent:', cursor.parentNode);
        console.log('Cursor attributes:', {
            x: cursor.getAttribute('x'),
            y: cursor.getAttribute('y'),
            width: cursor.getAttribute('width'),
            height: cursor.getAttribute('height'),
            fill: cursor.getAttribute('fill'),
            opacity: cursor.getAttribute('opacity')
        });
    }

    // Remove the cursor element
    removeCursorElement() {
        if (this.cursorElement && this.cursorElement.parentNode) {
            this.cursorElement.parentNode.removeChild(this.cursorElement);
            this.cursorElement = null;
        }

        // Also remove any existing cursor by ID
        const existingCursor = this.currentSvg?.querySelector('#text-cursor');
        if (existingCursor && existingCursor.parentNode) {
            existingCursor.parentNode.removeChild(existingCursor);
        }
    }

    // Update cursor position without recreating it
    updateCursorPosition(x, y) {
        if (this.cursorElement) {
            // Check if it's a rect (new style) or line (old style)
            if (this.cursorElement.tagName === 'rect') {
                this.cursorElement.setAttribute('x', (x - 1).toString());
                this.cursorElement.setAttribute('y', (y - 15).toString());
            } else {
                // Old line style
                this.cursorElement.setAttribute('x1', x.toString());
                this.cursorElement.setAttribute('y1', (y - 15).toString());
                this.cursorElement.setAttribute('x2', x.toString());
                this.cursorElement.setAttribute('y2', (y + 5).toString());
            }
        } else {
            this.createCursorElement(x, y);
        }
    }

    // Robust caret placement using char bboxes, appended to content group
    postRenderCaret() {
        if (!this.currentSvg) return;
        const svg = this.currentSvg;
        const content = svg.querySelector('.canvas-content') || svg;

        // Remove previous caret and line indicator
        const prevCaret = content.querySelector('#caret');
        if (prevCaret) prevCaret.remove();
        const prevIndicator = content.querySelector('#line-indicator');
        if (prevIndicator) prevIndicator.remove();

        const caretIndex = this.document?.ui_state?.selection?.cursor_position ?? 0;
        const pos = this.characterPositions[caretIndex];

        let caretX = this.svgTransformX || 20;
        let caretY = this.svgTransformY || 20;

        if (this.emptyLineCaret) {
            caretX = this.emptyLineCaret.x;
            caretY = this.emptyLineCaret.y;
        } else if (pos) {
            caretX = pos.x;
            caretY = pos.y;
        } else if (caretIndex > 0 && this.characterPositions[caretIndex - 1]) {
            // Position after the previous character
            const prevPos = this.characterPositions[caretIndex - 1];
            const prevNode = content.querySelector(`[data-char-index="${caretIndex - 1}"]`);
            const width = prevNode ? (parseFloat(prevNode.getAttribute('data-width')) || 12) : 12;
            caretX = prevPos.x + width;
            caretY = prevPos.y;
        }

        // Draw line indicator
        const indicator = document.createElementNS('http://www.w3.org/2000/svg', 'text');
        indicator.setAttribute('id', 'line-indicator');
        indicator.setAttribute('x', -15);
        indicator.setAttribute('y', caretY);
        indicator.setAttribute('class', 'line-indicator');
        indicator.textContent = '>';
        content.appendChild(indicator);

        // Draw caret
        const caret = document.createElementNS('http://www.w3.org/2000/svg', 'line');
        caret.setAttribute('id', 'caret');
        caret.setAttribute('x1', caretX);
        caret.setAttribute('y1', caretY - 15);
        caret.setAttribute('x2', caretX);
        caret.setAttribute('y2', caretY + 5);
        caret.setAttribute('class', 'caret');
        content.appendChild(caret);

        this.updateLineHighlight();
    }

    updateLineHighlight() {
        if (!this.currentSvg) return;
        const svg = this.currentSvg;
        const contentGroup = svg.querySelector('.canvas-content') || svg;

        // Remove previous highlight box
        const prevHighlight = contentGroup.querySelector('#current-line-highlight');
        if (prevHighlight) prevHighlight.remove();

        const caretIndex = this.document?.ui_state?.selection?.cursor_position ?? 0;
        let targetChar;

        if (this.characterPositions[caretIndex]) {
            targetChar = svg.querySelector(`[data-char-index="${caretIndex}"]`);
        } else if (caretIndex > 0 && this.characterPositions[caretIndex - 1]) {
            targetChar = svg.querySelector(`[data-char-index="${caretIndex - 1}"]`);
        }

        if (targetChar) {
            const lineGroup = targetChar.closest('g[class$="-line"]');
            if (lineGroup) {
                const chars = lineGroup.querySelectorAll('[data-char-index]');
                if (chars.length > 0) {
                    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
                    chars.forEach(c => {
                        const bbox = c.getBBox();
                        minX = Math.min(minX, bbox.x);
                        minY = Math.min(minY, bbox.y);
                        maxX = Math.max(maxX, bbox.x + bbox.width);
                        maxY = Math.max(maxY, bbox.y + bbox.height);
                    });

                    const highlightRect = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
                    highlightRect.setAttribute('id', 'current-line-highlight');
                    highlightRect.setAttribute('x', minX - 5);
                    highlightRect.setAttribute('y', minY - 5);
                    highlightRect.setAttribute('width', (maxX - minX) + 10);
                    highlightRect.setAttribute('height', (maxY - minY) + 10);
                    highlightRect.setAttribute('fill', 'none');
                    highlightRect.setAttribute('stroke', 'black');
                    highlightRect.setAttribute('stroke-width', '1');
                    highlightRect.setAttribute('stroke-dasharray', '2,2');
                    highlightRect.setAttribute('pointer-events', 'none');
                    contentGroup.insertBefore(highlightRect, contentGroup.firstChild);
                }
            }
        }
    }

    // Legacy compatibility - redirect to new implementation
    addCursorRectangle(element) {
        // No longer modifies elements - cursor is separate
        this.initializeVisualCursor();
    }

    // Legacy compatibility - redirect to new implementation
    removeCursorRectangle(element = null) {
        // No longer modifies elements - cursor is separate
        // Just ensure cursor element is properly positioned
    }

    // Move cursor up to previous line
    moveCursorUp() {
        const elements = this.getContentElements();
        if (elements.length === 0) return;

        // Find current cursor element
        let currentIndex = -1;
        for (let i = 0; i < elements.length; i++) {
            if (elements[i].classList.contains('cursor')) {
                currentIndex = i;
                break;
            }
        }

        if (currentIndex < 0) return;

        const currentElement = elements[currentIndex];
        const currentY = parseFloat(currentElement.getAttribute('y')) || 0;
        const currentX = parseFloat(currentElement.getAttribute('x')) || 0;

        // Find element on previous line (lower Y value) closest to current X
        let bestIndex = -1;
        let bestDistance = Number.MAX_VALUE;

        for (let i = 0; i < elements.length; i++) {
            const element = elements[i];
            const y = parseFloat(element.getAttribute('y')) || 0;
            const x = parseFloat(element.getAttribute('x')) || 0;

            // Look for elements on previous line (Y coordinate less than current)
            if (y < currentY - 30) { // Line height threshold
                const xDistance = Math.abs(x - currentX);
                if (xDistance < bestDistance) {
                    bestDistance = xDistance;
                    bestIndex = i;
                }
            }
        }

        // Move to the best match if found
        if (bestIndex >= 0) {
            this.removeCursorRectangle(elements[currentIndex]);
            elements[currentIndex].classList.remove('cursor');

            elements[bestIndex].classList.add('cursor');
            this.addCursorRectangle(elements[bestIndex]);
            console.log(`Moved cursor up to element ${bestIndex}`);
        }
    }

    // Move cursor down to next line
    moveCursorDown() {
        const elements = this.getContentElements();
        if (elements.length === 0) return;

        // Find current cursor element
        let currentIndex = -1;
        for (let i = 0; i < elements.length; i++) {
            if (elements[i].classList.contains('cursor')) {
                currentIndex = i;
                break;
            }
        }

        if (currentIndex < 0) return;

        const currentElement = elements[currentIndex];
        const currentY = parseFloat(currentElement.getAttribute('y')) || 0;
        const currentX = parseFloat(currentElement.getAttribute('x')) || 0;

        // Find element on next line (higher Y value) closest to current X
        let bestIndex = -1;
        let bestDistance = Number.MAX_VALUE;

        for (let i = 0; i < elements.length; i++) {
            const element = elements[i];
            const y = parseFloat(element.getAttribute('y')) || 0;
            const x = parseFloat(element.getAttribute('x')) || 0;

            // Look for elements on next line (Y coordinate greater than current)
            if (y > currentY + 30) { // Line height threshold
                const xDistance = Math.abs(x - currentX);
                if (xDistance < bestDistance) {
                    bestDistance = xDistance;
                    bestIndex = i;
                }
            }
        }

        // Move to the best match if found
        if (bestIndex >= 0) {
            this.removeCursorRectangle(elements[currentIndex]);
            elements[currentIndex].classList.remove('cursor');

            elements[bestIndex].classList.add('cursor');
            this.addCursorRectangle(elements[bestIndex]);
            console.log(`Moved cursor down to element ${bestIndex}`);
        }
    }

    // Set cursor to a specific element
    setCursorToElement(element) {
        if (!element) return;

        // Get UUID from char element
        const uuid = element.getAttribute('data-source-uuid');
        const charIndex = element.getAttribute('data-char-index');

        if (uuid) {
            // Update document cursor UUID
            this.document.ui_state.selection.cursor_uuid = uuid;

            // Update visual cursor position
            this.updateCursorFromUUIDs();
            this.initializeVisualCursor();

            console.log('Moved cursor to char:', uuid.slice(0, 8), 'index:', charIndex);
        }
    }

    // Switch to a specific document by UUID and redraw
    switchToDocument(documentUUID) {
        try {
            const loadedDocument = this.persistence.loadDocumentByUUID(documentUUID);
            if (!loadedDocument) {
                console.error('Document not found:', documentUUID);
                return false;
            }

            // Set the loaded document as current
            this.document = loadedDocument;

            // Update editor state from document
            const musicText = this.document.getCachedFormat('music_text') || '';
            // Document-first: no text tracking
            // cursor_position already in document.ui_state.selection
            this.selectedUuids = new Set(this.document.ui_state.selection.selected_uuids);
            // Document-first: no line tracking

            // Update UI
            if (window.UI && window.UI.updateDocumentStatus) {
                window.UI.updateDocumentStatus();
            }

            // Draw the document immediately when switching
            if (this.textContent) {
                console.log('Switching to document and drawing:', documentUUID.slice(0, 8));
                this.drawNow(this.textContent);
            } else {
                this.clearCanvas();
            }

            return true;
        } catch (error) {
            console.error('Failed to switch to document:', error);
            return false;
        }
    }

    // Clear local storage (both document and legacy)
    clearLocalStorage() {
        try {
            // Clear document model
            this.persistence.clearDocument();

            // Legacy format no longer used

            // Reset current document
            this.document = null;

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
            // Clear selection when clicking
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
            // UUID-based selection, not numeric
        }
    }

    // Convert character selection to UUIDs
    updateUuidSelectionFromCharacters() {
        this.selectedUuids.clear();

        if (this.selectedUuids.size === 0) {
            console.log('🔍 UUID Selection: No character selection, clearing UUIDs');
            return; // No selection
        }

        console.log('🔍 UUID Selection Mapping:', {
            uuids: Array.from(this.selectedUuids),
            selectedCount: this.selectedUuids.size,
            availableElements: this.elementUuidMap.size
        });

        const overlappingElements = [];
        for (const [uuid, element] of this.elementUuidMap) {
            // Check if element overlaps with selection
            if (this.selectedUuids.has(element.uuid)) {
                this.selectedUuids.add(uuid);
                overlappingElements.push({
                    uuid,
                    charRange: { start: element.charStart, end: element.charEnd },
                    text: this.textContent.slice(element.charStart, element.charEnd),
                    element: element
                });
            }
        }

        console.log('🔍 UUID Selection Results:', {
            foundElements: overlappingElements.length,
            selectedUuids: Array.from(this.selectedUuids),
            elementDetails: overlappingElements
        });
    }

    // Get selection as both UUIDs and character indices
    getSelectionBoth() {
        return {
            uuids: Array.from(this.selectedUuids),
            characterRange: {
                // UUID-based selection
            }
        };
    }

    // Select beat or word at the given character position
    selectBeatOrWordAt(position) {
        if (!this.textContent || position < 0 || position >= this.textContent.length) {
            return null;
        }

        // Find the line containing this position
        const lines = []; // Document-first: no text tracking
        let currentPos = 0;
        let lineText = '';
        let lineStartPos = 0;
        let positionInLine = 0;

        for (let i = 0; i < lines.length; i++) {
            const lineLength = lines[i].length;
            if (currentPos + lineLength >= position) {
                lineText = lines[i];
                lineStartPos = currentPos;
                positionInLine = position - currentPos;
                break;
            }
            currentPos += lineLength + 1; // +1 for newline
        }

        // Determine if this is a content line (contains musical notation)
        const isContentLine = this.isContentLine(lineText);

        if (isContentLine) {
            // Select beat for musical content lines
            return this.selectBeatAt(lineText, lineStartPos, positionInLine);
        } else {
            // Select word for non-musical lines (directives, text)
            return this.selectWordAt(lineText, lineStartPos, positionInLine);
        }
    }

    // Select a beat at the given position within a content line
    selectBeatAt(lineText, lineStartPos, positionInLine) {
        // Beat delimiters: space, barline, start/end of line
        const beatDelimiters = /[ |]/;

        // Find the start of the current beat
        let beatStart = positionInLine;
        while (beatStart > 0 && !beatDelimiters.test(lineText[beatStart - 1])) {
            beatStart--;
        }

        // Find the end of the current beat
        let beatEnd = positionInLine;
        while (beatEnd < lineText.length && !beatDelimiters.test(lineText[beatEnd])) {
            beatEnd++;
        }

        // Skip leading/trailing whitespace within the beat bounds
        while (beatStart < beatEnd && lineText[beatStart] === ' ') {
            beatStart++;
        }
        while (beatEnd > beatStart && lineText[beatEnd - 1] === ' ') {
            beatEnd--;
        }

        // If we found a valid beat, return the selection
        if (beatStart < beatEnd) {
            return {
                start: lineStartPos + beatStart,
                end: lineStartPos + beatEnd,
                type: 'beat'
            };
        }

        return null;
    }

    // Select a word at the given position within a text line
    selectWordAt(lineText, lineStartPos, positionInLine) {
        // Word delimiters: space, common punctuation
        const wordDelimiters = /[ \t:,;.!?]/;

        // Find the start of the current word
        let wordStart = positionInLine;
        while (wordStart > 0 && !wordDelimiters.test(lineText[wordStart - 1])) {
            wordStart--;
        }

        // Find the end of the current word
        let wordEnd = positionInLine;
        while (wordEnd < lineText.length && !wordDelimiters.test(lineText[wordEnd])) {
            wordEnd++;
        }

        // Skip leading/trailing whitespace within the word bounds
        while (wordStart < wordEnd && lineText[wordStart] === ' ') {
            wordStart++;
        }
        while (wordEnd > wordStart && lineText[wordEnd - 1] === ' ') {
            wordEnd--;
        }

        // If we found a valid word, return the selection
        if (wordStart < wordEnd) {
            return {
                start: lineStartPos + wordStart,
                end: lineStartPos + wordEnd,
                type: 'word'
            };
        }

        return null;
    }
}