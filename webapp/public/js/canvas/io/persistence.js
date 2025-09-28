// IO and persistence abstraction wrapping localStorage and server calls

export class Persistence {
  constructor(storageKey = 'musicTextDocument') {
    this.storageKey = storageKey;
    this.initialFormatsFetched = false;
  }

  loadLocal() {
    try {
      const raw = localStorage.getItem(this.storageKey);
      return raw ? JSON.parse(raw) : null;
    } catch {
      return null;
    }
  }

  saveLocal(document) {
    try {
      localStorage.setItem(this.storageKey, JSON.stringify(document));
      return true;
    } catch {
      return false;
    }
  }

  async fetchFormatsOnce(documentUUID) {
    if (this.initialFormatsFetched || !documentUUID) return null;
    this.initialFormatsFetched = true;
    // Placeholder: integrate with existing formats endpoint if present
    return null;
  }

  async renderSvg(document, ui_state) {
    // Expect existing endpoint `/api/render/canvas-svg` or similar; fallback no-op
    try {
      const res = await fetch('/api/render/canvas-svg', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ document, ui_state }),
      });
      if (!res.ok) throw new Error('render failed');
      const data = await res.json();
      return data; // { svg, metrics? }
    } catch (e) {
      console.warn('renderSvg failed or endpoint missing', e);
      return { svg: '' };
    }
  }
}

