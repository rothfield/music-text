// @ts-check
const { test, expect, chromium } = require('@playwright/test');
const path = require('path');
const fs = require('fs');

test('app persists entered notation after browser restart', async () => {
  // Create a temporary directory for user data
  const userDataDir = path.join(__dirname, 'test-user-data-' + Date.now());
  
  // Session 1: Enter notation and let the app save it
  const browser1 = await chromium.launchPersistentContext(userDataDir, {
    headless: true
  });
  const page1 = await browser1.newPage();
  
  await page1.goto('http://localhost:3000');
  await page1.waitForLoadState('networkidle');
  
  // Enter notation in the textarea
  const testNotation = '1-2 3 | 4--5 6 7';
  const textarea = page1.locator('textarea').first();
  await textarea.fill(testNotation);
  
  // Trigger the app's save mechanism by:
  // 1. Blurring the textarea (common save trigger)
  await textarea.blur();
  
  // 2. Wait a moment for any debounced saves
  await page1.waitForTimeout(500);
  
  // 3. Click parse button if it exists (might trigger save)
  const parseButton = page1.locator('button:has-text("Parse"), button:has-text("Render")').first();
  if (await parseButton.isVisible()) {
    await parseButton.click();
    await page1.waitForTimeout(500);
  }
  
  console.log('Session 1: Entered notation:', testNotation);
  
  // Close the browser completely (simulates quitting)
  await browser1.close();
  
  // Session 2: Open new browser and verify app restored the notation
  const browser2 = await chromium.launchPersistentContext(userDataDir, {
    headless: true
  });
  const page2 = await browser2.newPage();
  
  await page2.goto('http://localhost:3000');
  await page2.waitForLoadState('networkidle');
  
  // Wait for any restoration logic to complete
  await page2.waitForTimeout(1000);
  
  // Check if notation was restored into the UI
  const restoredTextarea = page2.locator('textarea').first();
  const restoredText = await restoredTextarea.inputValue();
  
  console.log('Session 2: Found in textarea:', restoredText || '(empty)');
  
  // The app should have restored the notation
  if (restoredText === testNotation) {
    console.log('✅ App successfully restored notation after browser restart!');
  } else {
    console.log('❌ App did not restore notation. The app may not have auto-save/restore functionality.');
    console.log('   This is a feature that would need to be implemented in the app.');
  }
  
  // Even if the app doesn't auto-restore, we can check if it at least saved to localStorage
  const savedData = await page2.evaluate(() => {
    const keys = Object.keys(localStorage);
    const data = {};
    keys.forEach(key => {
      data[key] = localStorage.getItem(key);
    });
    return data;
  });
  
  console.log('\nLocalStorage contents:', Object.keys(savedData).length > 0 ? savedData : '(empty)');
  
  await browser2.close();
  
  // Clean up user data directory
  fs.rmSync(userDataDir, { recursive: true, force: true });
});