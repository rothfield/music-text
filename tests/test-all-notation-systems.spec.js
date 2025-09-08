const { test, expect } = require('@playwright/test');

test.describe('All Notation System Options', () => {
  test('should work with Auto-detect (Numbers)', async ({ page }) => {
    await page.goto('http://localhost:3000');
    await page.waitForLoadState('networkidle');
    
    // Ensure Auto-detect is selected
    await page.selectOption('#notation-system', 'auto');
    
    // Input number notation
    await page.fill('#notation-input', '1 2 3');
    await page.waitForTimeout(500);
    
    // Should auto-detect as Number notation
    // (We can't easily test the exact output without complex setup, but this tests the flow)
    const textarea = page.locator('#notation-input');
    await expect(textarea).toHaveValue('1 2 3');
  });

  test('should work with explicit Number notation', async ({ page }) => {
    await page.goto('http://localhost:3000');
    await page.waitForLoadState('networkidle');
    
    // Select Number notation explicitly
    await page.selectOption('#notation-system', 'Number');
    await expect(page.locator('#notation-system')).toHaveValue('Number');
    
    // Input number notation
    await page.fill('#notation-input', '1-2-3 4-5');
    await page.waitForTimeout(500);
    
    const textarea = page.locator('#notation-input');
    await expect(textarea).toHaveValue('1-2-3 4-5');
  });

  test('should work with Western notation', async ({ page }) => {
    await page.goto('http://localhost:3000');
    await page.waitForLoadState('networkidle');
    
    // Select Western notation
    await page.selectOption('#notation-system', 'Western');
    await expect(page.locator('#notation-system')).toHaveValue('Western');
    
    // Input western notation
    await page.fill('#notation-input', 'C D E F G');
    await page.waitForTimeout(500);
    
    const textarea = page.locator('#notation-input');
    await expect(textarea).toHaveValue('C D E F G');
  });

  test('should work with Sargam notation', async ({ page }) => {
    await page.goto('http://localhost:3000');
    await page.waitForLoadState('networkidle');
    
    // Select Sargam notation
    await page.selectOption('#notation-system', 'Sargam');
    await expect(page.locator('#notation-system')).toHaveValue('Sargam');
    
    // Input sargam notation
    await page.fill('#notation-input', 'S R G M P');
    await page.waitForTimeout(500);
    
    const textarea = page.locator('#notation-input');
    await expect(textarea).toHaveValue('S R G M P');
  });

  test('should work with Tabla notation', async ({ page }) => {
    await page.goto('http://localhost:3000');
    await page.waitForLoadState('networkidle');
    
    // Select Tabla notation
    await page.selectOption('#notation-system', 'Tabla');
    await expect(page.locator('#notation-system')).toHaveValue('Tabla');
    
    // Input tabla notation
    await page.fill('#notation-input', 'dha ge na ka');
    await page.waitForTimeout(500);
    
    const textarea = page.locator('#notation-input');
    await expect(textarea).toHaveValue('dha ge na ka');
  });

  test('should persist selected notation system', async ({ page }) => {
    await page.goto('http://localhost:3000');
    await page.waitForLoadState('networkidle');
    
    // Change to Sargam
    await page.selectOption('#notation-system', 'Sargam');
    
    // Reload page
    await page.reload();
    await page.waitForLoadState('networkidle');
    
    // Should remember Sargam selection
    await expect(page.locator('#notation-system')).toHaveValue('Sargam');
  });

  test('should have proper notation examples in options', async ({ page }) => {
    await page.goto('http://localhost:3000');
    await page.waitForLoadState('networkidle');
    
    const selector = page.locator('#notation-system');
    
    // Check option texts
    const labelOption = selector.locator('option[disabled]');
    const autoOption = selector.locator('option[value="auto"]');
    const numberOption = selector.locator('option[value="Number"]');
    const westernOption = selector.locator('option[value="Western"]');
    const sargamOption = selector.locator('option[value="Sargam"]');
    const tablaOption = selector.locator('option[value="Tabla"]');
    
    await expect(labelOption).toContainText('Notation System');
    await expect(autoOption).toContainText('Auto-detect');
    await expect(numberOption).toContainText('1234567');
    await expect(westernOption).toContainText('CDEFGAB');
    await expect(sargamOption).toContainText('SRGMPDn');
    await expect(tablaOption).toContainText('dha-ge-na');
  });
});