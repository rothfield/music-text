const { test, expect } = require('@playwright/test');

test('check tala in UI', async ({ page }) => {
  // Set up console logging
  const logs = [];
  page.on('console', msg => {
    logs.push(msg.text());
  });
  
  // Navigate to the web app
  await page.goto('http://localhost:3001');
  
  // Wait for page to load
  await page.waitForLoadState('networkidle');
  
  // Input the tala test
  const testInput = `+
S | R`;
  
  await page.fill('#input-text', testInput);
  await page.click('#parse-btn');
  
  // Wait for parsing to complete
  await page.waitForTimeout(2000);
  
  // Check for tala-related logs
  const talaLogs = logs.filter(log => 
    log.includes('tala') || log.includes('Tala') || log.includes('255') || log.includes('DEBUG: Assigning')
  );
  
  console.log('Tala-related logs:', talaLogs);
  
  // Check if LilyPond output contains the tala
  const lilypondOutput = await page.locator('#lilypond-output').textContent();
  console.log('LilyPond output contains markup:', lilypondOutput?.includes('markup'));
  
  // Take screenshot
  await page.screenshot({ path: '/tmp/ui_tala_test.png' });
});