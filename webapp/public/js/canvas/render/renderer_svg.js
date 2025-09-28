// Renderer for mounting server-provided SVG and managing overlay layers

export class SvgRenderer {
  constructor(rootEl) {
    this.root = rootEl; // container element with #svg-container semantics
    this.contentLayer = null;
    this.overlayLayer = null;
    this.debugLayer = null;
    this.ensureLayers();
  }

  ensureLayers() {
    if (!this.root) return;
    this.root.innerHTML = '';
    const svgNS = 'http://www.w3.org/2000/svg';
    const svg = document.createElementNS(svgNS, 'svg');
    svg.setAttribute('id', 'editor-svg');
    svg.setAttribute('width', '100%');
    svg.setAttribute('height', '100%');

    this.contentLayer = document.createElementNS(svgNS, 'g');
    this.contentLayer.setAttribute('data-layer', 'content');
    this.overlayLayer = document.createElementNS(svgNS, 'g');
    this.overlayLayer.setAttribute('data-layer', 'overlay');
    this.debugLayer = document.createElementNS(svgNS, 'g');
    this.debugLayer.setAttribute('data-layer', 'debug');

    svg.appendChild(this.contentLayer);
    svg.appendChild(this.overlayLayer);
    svg.appendChild(this.debugLayer);
    this.root.appendChild(svg);
  }

  mountContentSvg(svgString) {
    // Only replace the content layer to avoid flicker in overlays
    if (!this.contentLayer) this.ensureLayers();
    this.contentLayer.innerHTML = svgString || '';
  }

  clearOverlays() { if (this.overlayLayer) this.overlayLayer.innerHTML = ''; }

  drawCaret(x, y, height = 16) {
    if (!this.overlayLayer) return;
    const svgNS = 'http://www.w3.org/2000/svg';
    const line = document.createElementNS(svgNS, 'line');
    line.setAttribute('x1', String(x));
    line.setAttribute('y1', String(y - height));
    line.setAttribute('x2', String(x));
    line.setAttribute('y2', String(y));
    line.setAttribute('class', 'canvas-caret');
    this.overlayLayer.appendChild(line);
  }

  drawSelectionRect(x, y, w, h) {
    if (!this.overlayLayer) return;
    const svgNS = 'http://www.w3.org/2000/svg';
    const rect = document.createElementNS(svgNS, 'rect');
    rect.setAttribute('x', String(x));
    rect.setAttribute('y', String(y));
    rect.setAttribute('width', String(w));
    rect.setAttribute('height', String(h));
    rect.setAttribute('class', 'canvas-selection');
    this.overlayLayer.appendChild(rect);
  }
}

