// Test document-first clicking with UUIDs
import { test, expect } from '@playwright/test';

test.describe('Document UUID-based Click Handling', () => {
  test.beforeEach(async ({ page, baseURL }) => {
    await page.goto(baseURL || '/');
    await page.waitForSelector('#svg-container');
  });

  test('clicking on SVG elements selects by UUID', async ({ page }) => {
    // Create a document with known UUIDs
    await page.evaluate(() => {
      const editor = window.appInstance.canvasEditor;
      const doc = editor.document;

      // Clear and setup a simple document
      doc.elements.clear();
      doc.content = [];

      // Add some test elements with specific UUIDs
      const noteUUID1 = 'test-note-uuid-001';
      const noteUUID2 = 'test-note-uuid-002';
      const noteUUID3 = 'test-note-uuid-003';

      doc.addElement(noteUUID1, {
        type: 'note',
        content: 'S',
        properties: { pitch_code: 61, octave: 0 }
      });

      doc.addElement(noteUUID2, {
        type: 'note',
        content: 'R',
        properties: { pitch_code: 63, octave: 0 }
      });

      doc.addElement(noteUUID3, {
        type: 'note',
        content: 'G',
        properties: { pitch_code: 65, octave: 0 }
      });

      // Set the text representation
      editor.textContent = 'S R G';
      editor.cursorPosition = 0;

      // Trigger render
      return editor.submitToServer(editor.textContent);
    });

    // Wait for SVG to render
    await page.waitForSelector('#svg-container svg .char');
    await page.waitForTimeout(200);

    // Click on the first character
    const svgBox = await page.locator('#svg-container svg').boundingBox();
    await page.mouse.click(svgBox.x + 30, svgBox.y + 40); // Adjust for transform(20,20) + character position

    // Check that cursor position was updated (clicking on char at index 0 places cursor at that index)
    const cursorPos = await page.evaluate(() => {
      return window.appInstance.canvasEditor.cursorPosition;
    });
    expect(cursorPos).toBeLessThanOrEqual(1); // Cursor at or near first character

    // Click on the third character (G)
    await page.mouse.click(svgBox.x + 54, svgBox.y + 40); // Third character position

    const cursorPos2 = await page.evaluate(() => {
      return window.appInstance.canvasEditor.cursorPosition;
    });
    expect(cursorPos2).toBeGreaterThanOrEqual(2); // Should move to third position
  });

  test('document UUID persists through transformations', async ({ page }) => {
    // Setup initial document
    await page.evaluate(() => {
      const editor = window.appInstance.canvasEditor;
      editor.textContent = 'S R G M';
      editor.cursorPosition = 0;

      // Generate initial document with UUIDs
      editor.document.documentUUID = 'test-document-uuid-123';

      return editor.submitToServer(editor.textContent);
    });

    await page.waitForSelector('#svg-container svg');

    // Check document UUID is preserved
    const docUUID = await page.evaluate(() => {
      return window.appInstance.canvasEditor.document.documentUUID;
    });
    expect(docUUID).toBe('test-document-uuid-123');

    // Make a selection by clicking and dragging
    const svgBox = await page.locator('#svg-container svg').boundingBox();
    await page.mouse.move(svgBox.x + 30, svgBox.y + 40);
    await page.mouse.down();
    await page.mouse.move(svgBox.x + 60, svgBox.y + 40);
    await page.mouse.up();

    // Check selection was made
    const selection = await page.evaluate(() => {
      const editor = window.appInstance.canvasEditor;
      return {
        start: editor.selection.start,
        end: editor.selection.end,
        uuids: Array.from(editor.selectedUuids)
      };
    });

    expect(selection.end).toBeGreaterThan(selection.start);

    // Document UUID should still be the same
    const docUUIDAfter = await page.evaluate(() => {
      return window.appInstance.canvasEditor.document.documentUUID;
    });
    expect(docUUIDAfter).toBe('test-document-uuid-123');
  });

  test('clicking updates cursor position in document model', async ({ page }) => {
    await page.evaluate(() => {
      const editor = window.appInstance.canvasEditor;
      editor.textContent = 'S R | G M';
      editor.cursorPosition = 0;

      // Set a known document UUID
      editor.document.documentUUID = 'cursor-test-doc-456';

      return editor.submitToServer(editor.textContent);
    });

    await page.waitForSelector('#svg-container svg .char');

    // Click at different positions and verify document model updates
    const svgBox = await page.locator('#svg-container svg').boundingBox();

    // Click on third character (after 'R')
    await page.mouse.click(svgBox.x + 54, svgBox.y + 40);

    const modelState = await page.evaluate(() => {
      const editor = window.appInstance.canvasEditor;
      return {
        cursorPosition: editor.cursorPosition,
        uiStateCursor: editor.document.ui_state.selection.cursor_position,
        documentUUID: editor.document.documentUUID
      };
    });

    expect(modelState.cursorPosition).toBeGreaterThanOrEqual(2);
    expect(modelState.uiStateCursor).toBe(modelState.cursorPosition);
    expect(modelState.documentUUID).toBe('cursor-test-doc-456');
  });
});