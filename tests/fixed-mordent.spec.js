const { test, expect } = require('@playwright/test');

test('mordent with newline', async ({ page }) => {
  test.setTimeout(30000);
  
  page.on('console', msg => {
    const text = msg.text();
    if (text.includes('🎵') || text.includes('ornament') || text.includes('Mordent')) {
      console.log('ORNAMENT:', text);
    }
  });
  
  await page.goto('http://localhost:3000');
  await page.waitForLoadState('domcontentloaded');
  
  // Input mordent with proper newline
  const textarea = await page.locator('textarea').first();
  await textarea.fill('~1~\n');
  
  // Wait for processing
  await page.waitForTimeout(3000);
  
  await page.screenshot({ path: 'test-results/mordent-fixed.png', fullPage: true });
  
  console.log('✅ Test completed - check screenshot');
});