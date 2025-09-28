import { EditorStore, selectors } from './state/store.js';
import { Persistence } from './io/persistence.js';
import { SvgRenderer } from './render/renderer_svg.js';
import { SelectionOverlay } from './selection/selection_overlay.js';
import { HitTester } from './hit_test/hit_test.js';
import { InputEvents } from './events/input.js';

export class CanvasController {
  constructor(containerId = 'svg-container') {
    try {
      console.log('CanvasController: Initializing with container:', containerId);

      this.container = document.getElementById(containerId);
      if (!this.container) {
        console.error('CanvasController: Container not found:', containerId);
        throw new Error(`Container ${containerId} not found`);
      } else {
        console.log('CanvasController: Container found successfully');
      }

      console.log('CanvasController: Creating EditorStore...');
      this.store = new EditorStore();

      console.log('CanvasController: Creating Persistence...');
      this.io = new Persistence('musicTextDocument');

      console.log('CanvasController: Creating SvgRenderer...');
      this.renderer = new SvgRenderer(this.container);

      console.log('CanvasController: Creating SelectionOverlay...');
      this.overlay = new SelectionOverlay(this.renderer);

      console.log('CanvasController: Creating HitTester...');
      this.hitTest = new HitTester(this.container);

      console.log('CanvasController: Creating InputEvents...');
      console.log('CanvasController: Container for InputEvents:', this.container);
      this.input = new InputEvents(this.container, (cmd) => this.handleCommand(cmd));

      console.log('CanvasController: Subscribing to store changes...');
      this.unsubscribe = this.store.subscribe(() => this.onStateChange());

      console.log('CanvasController: Constructor completed successfully');
    } catch (error) {
      console.error('CanvasController: Constructor failed:', error);
      throw error;
    }
  }

  init() {
    console.log('CanvasController: Calling input.bind()...');
    this.input.bind();
    console.log('CanvasController: input.bind() completed');
    const doc = this.io.loadLocal();
    if (doc) this.store.dispatch({ type: 'setDocument', document: doc });
    console.log('CanvasController: init() completed successfully');
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

    // Notify CanvasEditor of content changes
    if (this.onContentChange && state.document && typeof state.document.content === 'string') {
      this.onContentChange(state.document.content);
    }
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
        // Don't mutate content locally - send to server for processing
        console.log('CanvasController: InsertText command:', cmd.value);

        // Notify parent (CanvasEditor) to handle server communication
        if (this.onTextInput) {
          this.onTextInput({
            type: 'insert',
            value: cmd.value,
            position: selectors.cursorPosition(this.store.getState()),
            target_uuid: selectors.cursorUuid(this.store.getState())
          });
        }
        break;
      }
      case 'DeleteBackward': {
        // Don't mutate content locally - send to server for processing
        console.log('CanvasController: DeleteBackward command');

        // Notify parent (CanvasEditor) to handle server communication
        if (this.onTextInput) {
          this.onTextInput({
            type: 'deleteBackward',
            position: selectors.cursorPosition(this.store.getState())
          });
        }
        break;
      }
      case 'DeleteForward': {
        // Don't mutate content locally - send to server for processing
        console.log('CanvasController: DeleteForward command');

        // Notify parent (CanvasEditor) to handle server communication
        if (this.onTextInput) {
          this.onTextInput({
            type: 'deleteForward',
            position: selectors.cursorPosition(this.store.getState())
          });
        }
        break;
      }
      default:
        break;
    }
  }
}

