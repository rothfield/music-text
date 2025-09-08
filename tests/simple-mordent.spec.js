const { test, expect } = require('@playwright/test');

test('simple mordent test', async ({ page }) => {
  // Set a longer timeout for this test
  test.setTimeout(60000);
  
  // Go to the notation parser
  await page.goto('http://localhost:3000');
  
  // Wait for the page to be ready
  await page.waitForLoadState('domcontentloaded');
  await page.waitForTimeout(2000);
  
  // Look for the input textarea
  const textarea = await page.locator('textarea, #notation-input, input[type="text"]').first();
  await textarea.fill('~\n1');
  
  // Wait and take screenshot to see the current state
  await page.waitForTimeout(3000);
  await page.screenshot({ path: 'test-results/mordent-test.png', fullPage: true });
  
  // Check console logs for ornament processing
  const logs = [];
  page.on('console', msg => {
    const text = msg.text();
    logs.push(text);
    if (text.includes('ornament') || text.includes('Mordent') || text.includes('🎵')) {
      console.log('ORNAMENT LOG:', text);
    }
  });
  
  // Trigger another render if needed
  await page.waitForTimeout(2000);
  
  // Look for VexFlow canvas
  const canvas = await page.locator('canvas, svg').first();
  if (await canvas.isVisible()) {
    console.log('✅ VexFlow canvas found');
  } else {
    console.log('❌ No VexFlow canvas found');
  }
  
  // Check if there are any ornament-related logs
  await page.waitForTimeout(1000);
  const ornamentLogs = logs.filter(msg => 
    msg.toLowerCase().includes('ornament') || 
    msg.toLowerCase().includes('mordent') ||
    msg.includes('🎵')
  );
  
  console.log('Total logs:', logs.length);
  console.log('Ornament-related logs:', ornamentLogs.length);
  ornamentLogs.forEach(log => console.log('  -', log));
});