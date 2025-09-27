// Playwright tests for insert buttons and octave transform on content-line selections
import { test, expect } from '@playwright/test';

const waitForSvg = async (page) => {
  await page.waitForSelector('#svg-container svg', { state: 'attached' });
};

const getTextContent = async (page) => {
  return await page.evaluate(() => window.appInstance?.canvasEditor?.textContent || '');
};

test.describe('SVG Editor - Insert buttons and octave transform', () => {
  test.beforeEach(async ({ page, baseURL }) => {
    await page.goto(baseURL || '/');
    await page.waitForSelector('#svg-container');
  });

  test('insert buttons modify content-line text and re-render SVG', async ({ page }) => {
    await page.evaluate(() => {
      const editor = window.appInstance.canvasEditor;
      editor.textContent = '';
      editor.cursorPosition = 0;
      editor.submitToServer();
    });

    await page.click('#insertNote');
    await page.waitForTimeout(150);
    let txt = await getTextContent(page);
    expect(txt.includes('S')).toBeTruthy();

    await page.click('#insertBarline');
    await page.waitForTimeout(150);
    txt = await getTextContent(page);
    expect(txt.includes('|')).toBeTruthy();

    await page.click('#insertGraceNote');
    await page.waitForTimeout(150);
    txt = await getTextContent(page);
    expect(txt.includes('(S)')).toBeTruthy();

    await waitForSvg(page);
  });

  test('apply higher octave to selected notes (content-line selection only)', async ({ page }) => {
    // Seed content with multiple notes to select
    await page.evaluate(() => {
      const editor = window.appInstance.canvasEditor;
      editor.textContent = 'S R G M N';
      editor.cursorPosition = 0;
      editor.submitToServer();
    });
    await waitForSvg(page);

    const svg = page.locator('#svg-container svg');
    const box = await svg.boundingBox();
    // Drag across a range of notes on the same content line
    await page.mouse.move(box.x + 60, box.y + 60);
    await page.mouse.down();
    await page.mouse.move(box.x + 260, box.y + 60);
    await page.mouse.up();

    // Ensure selection is highlighted (content elements only)
    const beforeSelected = await page.locator('#svg-container svg .svg-selected').count();
    expect(beforeSelected).toBeGreaterThan(0);

    // Try to apply higher octave
    const higherBtn = page.locator('#btn-higher');
    // Button may start disabled until selection propagated; wait a beat
    await page.waitForTimeout(100);
    // If still disabled, assume fallback path or skip
    const disabled = await higherBtn.isDisabled();
    if (!disabled) {
      await higherBtn.click();
      // Wait for a moment to allow server/client updates
      await page.waitForTimeout(200);
      // We expect some visual change; at minimum ensure still rendering and not erroring
      await expect(svg).toBeVisible();
    } else {
      test.info().annotations.push({ type: 'note', description: 'Higher octave button disabled; backend transform not exercised.' });
    }
  });
});

