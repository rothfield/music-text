// Playwright tests focused on text-editor behavior over content lines only
// Validates: caret placement, typing, backspace, arrow navigation, and mouse selection highlighting

import { test, expect } from '@playwright/test';

const waitForSvg = async (page) => {
  await page.waitForSelector('#svg-container svg', { state: 'attached' });
};

const getTextContent = async (page) => {
  return await page.evaluate(() => window.appInstance?.canvasEditor?.textContent || '');
};

const typeAndWait = async (page, text) => {
  await page.keyboard.type(text);
  // Debounce wait + render
  await page.waitForTimeout(200);
};

test.describe('SVG Editor Content Lines - Text Editing Core', () => {
  test.beforeEach(async ({ page, baseURL }) => {
    await page.goto(baseURL || '/');
    await page.waitForSelector('#svg-container');
  });

  test('caret: click on content line places insertion point', async ({ page }) => {
    // Seed minimal valid content so SVG renders content lines
    await page.evaluate(() => {
      const editor = window.appInstance.canvasEditor;
      editor.textContent = 'S R G M\nS - | N';
      editor.cursorPosition = editor.textContent.length;
      editor.submitToServer();
    });
    await waitForSvg(page);

    const svgBox = await page.locator('#svg-container svg').boundingBox();
    // Click likely within first content line (roughly top-left quadrant)
    await page.mouse.click(svgBox.x + 80, svgBox.y + 60);

    // Caret should render
    await expect(page.locator('#svg-container svg line#client-cursor')).toBeVisible();
  });

  test('typing and backspace edit only content lines', async ({ page }) => {
    // Start with a single content line
    await page.evaluate(() => {
      const editor = window.appInstance.canvasEditor;
      editor.textContent = 'S R | G M';
      editor.cursorPosition = 0;
      editor.submitToServer();
    });
    await waitForSvg(page);

    // Focus container and type
    await page.click('#svg-container');
    await typeAndWait(page, 'S');

    let txt = await getTextContent(page);
    expect(txt.startsWith('SS')).toBeTruthy();

    // Backspace removes last char
    await page.keyboard.press('Backspace');
    await page.waitForTimeout(150);
    txt = await getTextContent(page);
    expect(txt.startsWith('S ')).toBeTruthy();
  });

  test('arrow keys navigate across content lines (Left/Right/Up/Down)', async ({ page }) => {
    await page.evaluate(() => {
      const editor = window.appInstance.canvasEditor;
      editor.textContent = 'S R G M\n1 2 3 4\nA B | C D';
      editor.cursorPosition = editor.textContent.length;
      editor.submitToServer();
    });
    await waitForSvg(page);

    // Focus and move left a few times
    await page.click('#svg-container');
    await page.keyboard.press('ArrowLeft');
    await page.keyboard.press('ArrowLeft');
    await page.waitForTimeout(100);

    const posAfterLeft = await page.evaluate(() => window.appInstance.canvasEditor.cursorPosition);
    expect(typeof posAfterLeft).toBe('number');

    // Move up to previous content line
    await page.keyboard.press('ArrowUp');
    await page.waitForTimeout(150);
    const posAfterUp = await page.evaluate(() => window.appInstance.canvasEditor.cursorPosition);
    expect(typeof posAfterUp).toBe('number');
    expect(posAfterUp).toBeLessThan(posAfterLeft);

    // Move down to next content line
    await page.keyboard.press('ArrowDown');
    await page.waitForTimeout(150);
    const posAfterDown = await page.evaluate(() => window.appInstance.canvasEditor.cursorPosition);
    expect(typeof posAfterDown).toBe('number');
  });

  test('mouse drag selects content-line elements and highlights them', async ({ page }) => {
    await page.evaluate(() => {
      const editor = window.appInstance.canvasEditor;
      editor.textContent = 'S - R | G M N';
      editor.cursorPosition = 0;
      editor.submitToServer();
    });
    await waitForSvg(page);

    const svg = page.locator('#svg-container svg');
    const box = await svg.boundingBox();
    // Drag horizontally across likely content glyphs
    await page.mouse.move(box.x + 60, box.y + 58);
    await page.mouse.down();
    await page.mouse.move(box.x + 260, box.y + 58);
    await page.mouse.up();
    await page.waitForTimeout(120);

    // Some content elements should be highlighted
    const selectedCount = await page.locator('#svg-container svg .svg-selected').count();
    expect(selectedCount).toBeGreaterThan(0);
  });
});

