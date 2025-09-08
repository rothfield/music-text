const { test, expect } = require('@playwright/test');

test('tala detection with number notation', async ({ page }) => {
  // Navigate to the web app
  await page.goto('http://localhost:3000');
  
  // Wait for page to load
  await page.waitForLoadState('networkidle');
  
  // Input the tala test
  const testInput = `0
1 2 3 4 | 5 6 7 1 |`;
  
  await page.fill('#input-text', testInput);
  await page.click('#parse-btn');
  
  // Wait for parsing to complete
  await page.waitForTimeout(1000);
  
  // Check console output for tala detection
  const logs = [];
  page.on('console', msg => {
    logs.push(msg.text());
  });
  
  // Re-parse to capture console logs
  await page.fill('#input-text', testInput);
  await page.click('#parse-btn');
  await page.waitForTimeout(1000);
  
  // Take screenshot for verification
  await page.screenshot({ path: '/tmp/tala_test_screenshot.png' });
  
  // Check if VexFlow canvas exists and has content
  const canvas = page.locator('#vexflow-output canvas');
  await expect(canvas).toBeVisible();
  
  // Print relevant console logs
  console.log('Console logs:', logs.filter(log => 
    log.includes('tala') || log.includes('Tala') || log.includes('Element')
  ));
});