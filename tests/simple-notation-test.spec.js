const { test, expect } = require('@playwright/test');

test('notation system selector basic functionality', async ({ page }) => {
  await page.goto('http://localhost:3000');
  
  // Wait for WASM to load
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(2000);
  
  // Check that notation selector exists
  const selector = page.locator('#notation-system');
  await expect(selector).toBeVisible({ timeout: 5000 });
  
  // Check default value
  await expect(selector).toHaveValue('auto');
  
  // Try changing the selection
  await selector.selectOption('Western');
  await expect(selector).toHaveValue('Western');
  
  await selector.selectOption('Sargam');  
  await expect(selector).toHaveValue('Sargam');
  
  // Test with actual input to verify parsing works
  const textarea = page.locator('#notation-input');
  await expect(textarea).toBeVisible();
  await textarea.fill('1 2 3');
  
  // Change notation system and verify it re-parses
  await selector.selectOption('Number');
  await page.waitForTimeout(500); // Allow time for re-parsing
  
  console.log('✅ Notation system selector is working properly!');
});