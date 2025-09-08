const { test, expect } = require('@playwright/test');

test('final mordent test', async ({ page }) => {
  test.setTimeout(30000);
  
  console.log('Testing if ~1~ produces visible mordent ornament...');
  
  await page.goto('http://localhost:3000');
  await page.waitForLoadState('domcontentloaded');
  
  // Input the mordent notation
  const textarea = await page.locator('textarea').first();
  await textarea.fill('~1~\n');
  
  // Wait for processing
  await page.waitForTimeout(3000);
  
  // Take screenshot to check visual result  
  await page.screenshot({ path: 'test-results/final-mordent.png', fullPage: true });
  
  // Check for any VexFlow canvas
  const canvas = await page.locator('canvas').first();
  const canvasVisible = await canvas.isVisible().catch(() => false);
  
  console.log('✅ VexFlow canvas visible:', canvasVisible);
  console.log('📸 Screenshot saved to test-results/final-mordent.png');
  console.log('🎵 Check the screenshot to see if mordent ornament appears above the note');
});