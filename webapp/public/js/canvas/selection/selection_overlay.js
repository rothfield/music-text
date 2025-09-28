export class SelectionOverlay {
  constructor(renderer) {
    this.renderer = renderer; // SvgRenderer
  }

  updateCaret(posMetrics) {
    // posMetrics: { x, y, height }
    this.renderer.clearOverlays();
    if (!posMetrics) return;
    this.renderer.drawCaret(posMetrics.x, posMetrics.y, posMetrics.height);
  }

  updateSelection(rects = []) {
    // rects: [{x,y,w,h}]
    for (const r of rects) this.renderer.drawSelectionRect(r.x, r.y, r.w, r.h);
  }
}

