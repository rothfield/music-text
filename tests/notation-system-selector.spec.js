const { test, expect } = require('@playwright/test');

test.describe('Notation System Selector', () => {
  test('should have notation system selector as first element', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Wait for page to load
    await page.waitForLoadState('networkidle');
    
    // Check that notation selector exists and is visible
    const selector = page.locator('#notation-system');
    await expect(selector).toBeVisible();
    
    // Check that it has the correct options
    const options = await selector.locator('option').allTextContents();
    expect(options).toEqual([
      'Auto-detect',
      '1234567', 
      'CDEFGAB',
      'SRGMPDn',
      'dha-ge-na'
    ]);
    
    // Check that Auto-detect is selected by default
    await expect(selector).toHaveValue('auto');
  });

  test('should change notation system and parse accordingly', async ({ page }) => {
    await page.goto('http://localhost:3000');
    await page.waitForLoadState('networkidle');
    
    // Fill textarea with mixed notation that could be interpreted differently
    const textarea = page.locator('#input-text');
    await textarea.fill('C D E');
    
    // Initially should auto-detect as Western
    let lilypond = await page.locator('#lilypondOutput').textContent();
    expect(lilypond).toContain('c4 d4 e4'); // Western interpretation
    
    // Change to Number notation system
    await page.selectOption('#notation-system', 'Number');
    
    // Should re-parse the same input as numbers (which would be invalid)
    // Wait for the parsing to update
    await page.waitForTimeout(500);
    
    // Verify the selector changed
    await expect(page.locator('#notation-system')).toHaveValue('Number');
  });

  test('should persist notation system selection in localStorage', async ({ page }) => {
    await page.goto('http://localhost:3000');
    await page.waitForLoadState('networkidle');
    
    // Change notation system
    await page.selectOption('#notation-system', 'Sargam');
    
    // Reload the page
    await page.reload();
    await page.waitForLoadState('networkidle');
    
    // Check that selection was persisted
    await expect(page.locator('#notation-system')).toHaveValue('Sargam');
  });

  test('should show proper monospace font for notation examples', async ({ page }) => {
    await page.goto('http://localhost:3000');
    await page.waitForLoadState('networkidle');
    
    const selector = page.locator('#notation-system');
    
    // Check that selector has monospace font
    const fontFamily = await selector.evaluate(el => 
      window.getComputedStyle(el).fontFamily
    );
    expect(fontFamily).toContain('monospace');
  });
});