// Lightweight state store with reducers/selectors for the canvas editor

export class EditorStore {
  constructor(initial = {}) {
    this.state = {
      document: null,
      uiState: null,
      mode: 'text',
      selection: { uuids: [], cursor_position: 0, cursor_uuid: null },
      ...initial,
    };
    this.listeners = new Set();
  }

  subscribe(listener) {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  getState() { return this.state; }

  notify() { this.listeners.forEach(l => l(this.state)); }

  dispatch(action) {
    switch (action.type) {
      case 'setDocument':
        this.state.document = action.document || null;
        this.state.uiState = action.document?.ui_state || null;
        break;
      case 'updateUiState':
        this.state.uiState = { ...(this.state.uiState || {}), ...(action.patch || {}) };
        if (this.state.document) this.state.document.ui_state = this.state.uiState;
        break;
      case 'setMode':
        this.state.mode = action.mode;
        break;
      case 'setCursor':
        if (!this.state.uiState) this.state.uiState = {};
        this.state.uiState.selection = {
          ...(this.state.uiState.selection || {}),
          cursor_position: action.position ?? this.state.uiState?.selection?.cursor_position ?? 0,
          cursor_uuid: action.uuid ?? null,
        };
        break;
      case 'setSelection':
        if (!this.state.uiState) this.state.uiState = {};
        this.state.uiState.selection = {
          ...(this.state.uiState.selection || {}),
          selected_uuids: action.uuids || [],
        };
        break;
      default:
        return; // no notify
    }
    this.notify();
  }
}

// Selectors
export const selectors = {
  document: (s) => s.document,
  uiState: (s) => s.uiState,
  cursorPosition: (s) => s.uiState?.selection?.cursor_position ?? 0,
  selectionUuids: (s) => s.uiState?.selection?.selected_uuids || [],
  mode: (s) => s.mode,
};

