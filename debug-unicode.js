const { chromium } = require('playwright');

(async () => {
  const browser = await chromium.launch({ headless: false });
  const page = await browser.newPage();
  
  // Enable console logs from browser
  page.on('console', msg => {
    console.log(`BROWSER: ${msg.type()}: ${msg.text()}`);
  });
  
  await page.goto('http://127.0.0.1:3000');
  await page.waitForLoadState('networkidle');
  
  console.log('Page loaded, waiting for JavaScript initialization...');
  await page.waitForTimeout(1000);
  
  // Clear any existing input
  await page.fill('#input-text', '');
  
  // Type 'Bb' to test 
  await page.fill('#input-text', 'Bb');
  
  console.log('Initial text value:', await page.inputValue('#input-text'));
  console.log('Unicode toggle checked:', await page.isChecked('#unicode-toggle'));
  
  // Toggle Unicode off
  console.log('Clicking Unicode toggle to turn OFF...');
  await page.click('#unicode-toggle');
  await page.waitForTimeout(500);
  
  console.log('After clicking toggle - checked:', await page.isChecked('#unicode-toggle'));
  console.log('Text value after toggle off:', await page.inputValue('#input-text'));
  
  // Toggle Unicode back on  
  console.log('Clicking Unicode toggle to turn ON...');
  await page.click('#unicode-toggle');
  await page.waitForTimeout(500);
  
  console.log('After clicking toggle again - checked:', await page.isChecked('#unicode-toggle'));
  console.log('Text value after toggle on:', await page.inputValue('#input-text'));
  
  console.log('Test completed. Waiting 3 seconds before closing...');
  await page.waitForTimeout(3000);
  await browser.close();
})().catch(console.error);