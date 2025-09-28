import { EditorStore, selectors } from './state/store.js';
import { Persistence } from './io/persistence.js';
import { SvgRenderer } from './render/renderer_svg.js';
import { SelectionOverlay } from './selection/selection_overlay.js';
import { HitTester } from './hit_test/hit_test.js';
import { InputEvents } from './events/input.js';

export class CanvasController {
  constructor(containerId = 'svg-container') {
    this.container = document.getElementById(containerId);
    if (!this.container) throw new Error(`Container ${containerId} not found`);
    this.store = new EditorStore();
    this.io = new Persistence('musicTextDocument');
    this.renderer = new SvgRenderer(this.container);
    this.overlay = new SelectionOverlay(this.renderer);
    this.hitTest = new HitTester(this.container);
    this.input = new InputEvents(this.container, (cmd) => this.handleCommand(cmd));
    this.unsubscribe = this.store.subscribe(() => this.onStateChange());
  }

  init() {
    this.input.bind();
    const doc = this.io.loadLocal();
    if (doc) this.store.dispatch({ type: 'setDocument', document: doc });
    return this;
  }

  destroy() { this.input.unbind(); this.unsubscribe?.(); }

  async loadDocument(document) { this.store.dispatch({ type: 'setDocument', document }); }

  async render() {
    const state = this.store.getState();
    const { document, uiState } = state;
    if (!document) { this.renderer.mountContentSvg(''); return; }
    const res = await this.io.renderSvg(document, uiState);
    this.renderer.mountContentSvg(res.svg || '');
    // caret update could use metrics if provided; placeholder clears overlays
    this.overlay.updateCaret(null);
  }

  onStateChange() {
    const state = this.store.getState();
    this.io.saveLocal(state.document);
    // re-render content when document/ui changes
    this.render();
  }

  // High-level command handling
  handleCommand(cmd) {
    switch (cmd.type) {
      case 'PointerDown': {
        const idx = this.hitTest.charIndexFromXY(cmd.x, cmd.y);
        this.store.dispatch({ type: 'setCursor', position: idx });
        this.dragging = true;
        this.dragStart = idx;
        break;
      }
      case 'PointerMove': {
        if (!this.dragging) break;
        const idx = this.hitTest.charIndexFromXY(cmd.x, cmd.y);
        // In a later pass, compute selection rects and overlay
        this.store.dispatch({ type: 'setCursor', position: idx });
        break;
      }
      case 'PointerUp': {
        this.dragging = false;
        break;
      }
      case 'MoveCursor': {
        const pos = selectors.cursorPosition(this.store.getState());
        const delta = cmd.dir === 'left' ? -1 : cmd.dir === 'right' ? 1 : 0;
        this.store.dispatch({ type: 'setCursor', position: Math.max(0, pos + delta) });
        break;
      }
      case 'InsertText': {
        // Minimal local mutation to document text content if present
        const state = this.store.getState();
        const doc = state.document;
        if (doc && typeof doc.text_content === 'string') {
          const pos = selectors.cursorPosition(state);
          const before = doc.text_content.slice(0, pos);
          const after = doc.text_content.slice(pos);
          doc.text_content = before + cmd.value + after;
          this.store.dispatch({ type: 'setDocument', document: doc });
          this.store.dispatch({ type: 'setCursor', position: pos + cmd.value.length });
        }
        break;
      }
      case 'DeleteBackward': {
        const state = this.store.getState();
        const doc = state.document;
        if (doc && typeof doc.text_content === 'string') {
          const pos = selectors.cursorPosition(state);
          if (pos > 0) {
            doc.text_content = doc.text_content.slice(0, pos - 1) + doc.text_content.slice(pos);
            this.store.dispatch({ type: 'setDocument', document: doc });
            this.store.dispatch({ type: 'setCursor', position: pos - 1 });
          }
        }
        break;
      }
      case 'DeleteForward': {
        const state = this.store.getState();
        const doc = state.document;
        if (doc && typeof doc.text_content === 'string') {
          const pos = selectors.cursorPosition(state);
          doc.text_content = doc.text_content.slice(0, pos) + doc.text_content.slice(pos + 1);
          this.store.dispatch({ type: 'setDocument', document: doc });
        }
        break;
      }
      default:
        break;
    }
  }
}

