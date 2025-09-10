const { test, expect } = require('@playwright/test');

test('debug mordent rendering', async ({ page }) => {
  test.setTimeout(60000);
  
  const allLogs = [];
  
  // Capture all console messages
  page.on('console', msg => {
    const text = msg.text();
    allLogs.push(text);
    console.log('CONSOLE:', text);
  });
  
  // Capture errors
  page.on('pageerror', error => {
    console.log('PAGE ERROR:', error.message);
    allLogs.push('PAGE ERROR: ' + error.message);
  });
  
  await page.goto('http://localhost:3000');
  await page.waitForLoadState('domcontentloaded');
  
  // Input the mordent
  const textarea = await page.locator('textarea, #input-text, input').first();
  await textarea.fill('~1~');
  
  // Wait for processing
  await page.waitForTimeout(5000);
  
  // Check what JSON is being generated
  const jsonOutput = await page.locator('pre, .json-output, #vexflow-json').textContent().catch(() => 'No JSON found');
  console.log('Generated JSON:', jsonOutput);
  
  // Take screenshot
  await page.screenshot({ path: 'test-results/debug-mordent.png', fullPage: true });
  
  // Summary
  console.log('\n=== SUMMARY ===');
  console.log('Total console logs:', allLogs.length);
  
  const ornamentLogs = allLogs.filter(log => 
    log.toLowerCase().includes('ornament') || 
    log.toLowerCase().includes('mordent')
  );
  console.log('Ornament logs:', ornamentLogs.length);
  
  const errorLogs = allLogs.filter(log => 
    log.toLowerCase().includes('error') || 
    log.toLowerCase().includes('failed')
  );
  console.log('Error logs:', errorLogs.length);
  
  if (ornamentLogs.length > 0) {
    console.log('\nORNAMENT LOGS:');
    ornamentLogs.forEach(log => console.log('  ', log));
  }
  
  if (errorLogs.length > 0) {
    console.log('\nERROR LOGS:');
    errorLogs.forEach(log => console.log('  ', log));
  }
});