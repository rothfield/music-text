const { test } = require('@playwright/test');

test('capture VexFlow beaming screenshot', async ({ page }) => {
  // Navigate to the page
  await page.goto('http://localhost:3000', { waitUntil: 'networkidle' });
  
  // Wait a bit for any dynamic content
  await page.waitForTimeout(3000);
  
  // Enter SSSS pattern
  await page.fill('#input-text', '| SSSS | RRRR |\n| 1111 | 2222 |');
  
  // Wait for rendering
  await page.waitForTimeout(2000);
  
  // Take full page screenshot
  await page.screenshot({ path: 'test_output/vexflow_beaming_full.png', fullPage: true });
  
  // Try to capture just the VexFlow area if it exists
  try {
    const vexflow = page.locator('#live-vexflow-notation');
    await vexflow.screenshot({ path: 'test_output/vexflow_beaming_area.png' });
  } catch (e) {
    console.log('Could not capture VexFlow area specifically');
  }
});