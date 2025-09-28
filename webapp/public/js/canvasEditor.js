import { CanvasController } from './canvas/controller.js';

// Minimal adapter exposing the historical API surface
export class CanvasEditor {
  constructor() {
    this._controller = null;
  }

  init() {
    this._controller = new CanvasController('svg-container').init();
    window.musicApp = window.musicApp || {};
    window.musicApp.canvasEditor = this;
    return this;
  }

  async loadDocument(doc) {
    return this._controller?.loadDocument(doc);
  }
}

