const { test, expect } = require('@playwright/test');

test('web mordent test with correct format', async ({ page }) => {
  test.setTimeout(30000);
  
  await page.goto('http://localhost:3000');
  await page.waitForLoadState('domcontentloaded');
  
  // Input the correct format: ~ on line above note
  const textarea = await page.locator('textarea').first();
  await textarea.fill('~\n1');
  
  // Wait for processing
  await page.waitForTimeout(3000);
  
  // Take screenshot
  await page.screenshot({ path: 'test-results/web-mordent.png', fullPage: true });
  
  console.log('✅ Test completed - check server output for mordent');
});