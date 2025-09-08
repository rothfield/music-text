const { test, expect } = require('@playwright/test');

test('mordent debug test', async ({ page }) => {
  test.setTimeout(30000);
  
  await page.goto('http://localhost:3000');
  await page.waitForLoadState('domcontentloaded');
  
  // Input the mordent
  const textarea = await page.locator('textarea').first();
  await textarea.fill('~1~\n');
  
  // Wait for processing and screenshot
  await page.waitForTimeout(3000);
  await page.screenshot({ path: 'test-results/debug-logs.png', fullPage: true });
  
  console.log('Check server output for 🎵 MORDENT DEBUG logs');
});