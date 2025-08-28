// @ts-check
const { test, expect } = require('@playwright/test');

test('show empty textarea with placeholder hints', async ({ page }) => {
  await page.goto('http://localhost:3000');
  await page.waitForLoadState('networkidle');
  
  // Wait for WASM to load
  await page.waitForTimeout(2000);
  
  // Ensure textarea is empty to show placeholder
  const textarea = page.locator('#notation-input');
  await textarea.clear();
  
  // Take screenshot showing placeholder
  await page.screenshot({ 
    path: 'empty-textarea-placeholder.png', 
    fullPage: true 
  });
  
  console.log('✅ Empty textarea with placeholder captured');
});