const { chromium } = require('playwright');

(async () => {
  const browser = await chromium.launch({ headless: false, slowMo: 1000 });
  const page = await browser.newPage();
  
  console.log('Opening http://localhost:3000...');
  await page.goto('http://localhost:3000');
  
  console.log('Waiting for page to load...');
  await page.waitForSelector('#input-text');
  
  console.log('Taking screenshot of initial state...');
  await page.screenshot({ path: 'initial-state.png' });
  
  console.log('Typing some input to trigger potential flashing...');
  await page.fill('#input-text', '1 2 3');
  
  console.log('Waiting to observe any flashing behavior...');
  await page.waitForTimeout(5000);
  
  console.log('Taking screenshot after input...');
  await page.screenshot({ path: 'after-input.png' });
  
  console.log('Keeping browser open for manual inspection...');
  await page.waitForTimeout(30000);
  
  await browser.close();
})();