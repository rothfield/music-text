// Hit testing utilities: map x,y to character index using current SVG

export class HitTester {
  constructor(rootEl) {
    this.root = rootEl; // container holding the svg
  }

  // Returns nearest <text> with data-char-index or null
  elementAtPoint(x, y) {
    const svg = this.root?.querySelector('svg');
    if (!svg) return null;
    const el = document.elementFromPoint(x + svg.getBoundingClientRect().left, y + svg.getBoundingClientRect().top);
    if (!el) return null;
    if (el.getAttribute && el.hasAttribute('data-char-index')) return el;
    return el.closest?.('[data-char-index]') || null;
  }

  // Compute nearest char index by scanning siblings in same line group
  charIndexFromXY(x, y) {
    const svg = this.root?.querySelector('svg');
    if (!svg) return null;
    const pt = { x, y };
    const lineGroups = Array.from(svg.querySelectorAll('.line-group, [data-line-group="true"]'));
    // Fallback: search all chars
    const candidates = [];
    const chars = Array.from(svg.querySelectorAll('[data-char-index]'));
    for (const c of chars) {
      const bbox = c.getBBox?.();
      if (!bbox) continue;
      candidates.push({ el: c, index: parseInt(c.getAttribute('data-char-index'), 10) || 0, bbox });
    }
    if (candidates.length === 0) return 0;
    let best = candidates[0];
    let bestDist = Number.POSITIVE_INFINITY;
    for (const cand of candidates) {
      const midX = cand.bbox.x + cand.bbox.width / 2;
      const midY = cand.bbox.y + cand.bbox.height / 2;
      const dx = pt.x - midX;
      const dy = pt.y - midY;
      const d = Math.hypot(dx, dy);
      if (d < bestDist) { bestDist = d; best = cand; }
    }
    // Snap left/right of midpoint to before/after index
    const mid = best.bbox.x + best.bbox.width / 2;
    return pt.x < mid ? best.index : best.index + 1;
  }
}

