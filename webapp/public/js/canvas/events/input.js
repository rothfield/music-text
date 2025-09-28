import { throttle } from '../util/throttle.js';

export class InputEvents {
  constructor(targetEl, dispatch) {
    this.el = targetEl; // container with svg
    this.dispatch = dispatch; // function to send high-level commands
    this.bound = false;
    this.onMouseDown = this.onMouseDown.bind(this);
    this.onMouseMove = throttle(this.onMouseMove.bind(this), 16);
    this.onMouseUp = this.onMouseUp.bind(this);
    this.onKeyDown = this.onKeyDown.bind(this);
  }

  bind() {
    if (this.bound) return;
    console.log('InputEvents: Binding events to element:', this.el);
    console.log('InputEvents: Element exists:', !!this.el);
    this.bound = true;
    this.el.addEventListener('mousedown', this.onMouseDown);
    window.addEventListener('mousemove', this.onMouseMove);
    window.addEventListener('mouseup', this.onMouseUp);
    window.addEventListener('keydown', this.onKeyDown);
    console.log('InputEvents: Event binding completed');
  }

  unbind() {
    if (!this.bound) return;
    this.bound = false;
    this.el.removeEventListener('mousedown', this.onMouseDown);
    window.removeEventListener('mousemove', this.onMouseMove);
    window.removeEventListener('mouseup', this.onMouseUp);
    window.removeEventListener('keydown', this.onKeyDown);
  }

  onMouseDown(ev) {
    // Focus the container so it can receive keyboard events
    this.el.focus();
    const rect = this.el.getBoundingClientRect();
    this.dispatch({ type: 'PointerDown', x: ev.clientX - rect.left, y: ev.clientY - rect.top, shift: ev.shiftKey });
  }
  onMouseMove(ev) {
    const rect = this.el.getBoundingClientRect();
    this.dispatch({ type: 'PointerMove', x: ev.clientX - rect.left, y: ev.clientY - rect.top });
  }
  onMouseUp(ev) {
    const rect = this.el.getBoundingClientRect();
    this.dispatch({ type: 'PointerUp', x: ev.clientX - rect.left, y: ev.clientY - rect.top });
  }
  onKeyDown(ev) {
    // Check if the SVG container is focused
    if (document.activeElement !== this.el) {
      return; // Only handle keys when our container is focused
    }

    console.log('InputEvents: Key pressed:', ev.key);

    // Normalize a few key commands
    const map = {
      ArrowLeft: { type: 'MoveCursor', dir: 'left' },
      ArrowRight: { type: 'MoveCursor', dir: 'right' },
      ArrowUp: { type: 'MoveCursor', dir: 'up' },
      ArrowDown: { type: 'MoveCursor', dir: 'down' },
      Backspace: { type: 'DeleteBackward' },
      Delete: { type: 'DeleteForward' },
      Enter: { type: 'InsertText', value: '\n' },
    };
    const cmd = map[ev.key];
    if (cmd) {
      console.log('InputEvents: Dispatching command:', cmd);
      ev.preventDefault();
      this.dispatch(cmd);
      return;
    }
    if (ev.key.length === 1 && !ev.ctrlKey && !ev.metaKey && !ev.altKey) {
      console.log('InputEvents: Dispatching text insert:', ev.key);
      this.dispatch({ type: 'InsertText', value: ev.key });
      ev.preventDefault();
    }
  }
}

