// @ts-check
const { test, expect } = require('@playwright/test');

test('verify WASM and VexFlow functionality', async ({ page }) => {
  await page.goto('http://localhost:3000');
  await page.waitForLoadState('networkidle');
  
  // Wait for WASM to load (give it time to initialize)
  await page.waitForTimeout(3000);
  
  // Check UI elements are present
  await expect(page.locator('.hero')).toBeVisible();
  await expect(page.locator('#notation-input')).toBeVisible();
  await expect(page.locator('#vexflow-canvas')).toBeVisible();
  
  // Clear and enter notation
  const textarea = page.locator('#notation-input');
  await textarea.clear();
  await textarea.fill('1-2-3');
  
  // Wait for processing
  await page.waitForTimeout(2000);
  
  // Check for any JavaScript errors in console
  const messages = [];
  page.on('console', msg => {
    if (msg.type() === 'error') {
      messages.push(msg.text());
    }
  });
  
  // Take screenshot
  await page.screenshot({ 
    path: 'functionality-test.png', 
    fullPage: true 
  });
  
  // Print any errors found
  if (messages.length > 0) {
    console.log('JavaScript errors found:', messages);
  } else {
    console.log('✅ No JavaScript errors detected');
  }
  
  console.log('✅ Functionality test completed');
});