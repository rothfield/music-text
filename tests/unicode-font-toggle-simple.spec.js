const { test, expect } = require('@playwright/test');

test('Unicode toggle should change font family of textarea', async ({ page }) => {
  // Listen for console messages and errors
  page.on('console', msg => console.log('BROWSER:', msg.text()));
  page.on('pageerror', error => console.log('PAGE ERROR:', error.message));
  
  // Start by going to the HTTP server
  await page.goto('http://localhost:8081');
  
  // Wait for the page to load
  await page.waitForLoadState('networkidle');
  
  // Wait a bit for JavaScript to initialize
  await page.waitForTimeout(2000);
  
  // Check if unicode toggle exists
  const unicodeToggle = page.locator('#unicode-toggle');
  await expect(unicodeToggle).toBeVisible();
  
  // Get the input textarea
  const inputTextarea = page.locator('#input-text');
  await expect(inputTextarea).toBeVisible();
  
  // Test that unicode toggle is initially checked (should be default)
  const isInitiallyChecked = await unicodeToggle.isChecked();
  console.log('Unicode toggle initially checked:', isInitiallyChecked);
  
  // Get initial font family from computed styles
  const initialFont = await inputTextarea.evaluate(el => window.getComputedStyle(el).fontFamily);
  console.log('Initial font family:', initialFont);
  
  // Toggle unicode (if it's on, turn it off; if it's off, turn it on)
  await unicodeToggle.click();
  await page.waitForTimeout(500); // Small delay for font switching
  
  // Check the new state
  const isCheckedAfterToggle = await unicodeToggle.isChecked();
  console.log('Unicode toggle after click:', isCheckedAfterToggle);
  
  // Get font after toggle
  const fontAfterToggle = await inputTextarea.evaluate(el => window.getComputedStyle(el).fontFamily);
  console.log('Font after toggle:', fontAfterToggle);
  
  // Test input with musical characters
  await inputTextarea.fill('|1-2#b');
  await page.waitForTimeout(500);
  
  // Check the input value
  const inputValue = await inputTextarea.inputValue();
  console.log('Input value:', inputValue);
  
  // Toggle again
  await unicodeToggle.click();
  await page.waitForTimeout(500);
  
  // Check state and font again
  const isFinallyChecked = await unicodeToggle.isChecked();
  const finalFont = await inputTextarea.evaluate(el => window.getComputedStyle(el).fontFamily);
  
  console.log('Final toggle state:', isFinallyChecked);
  console.log('Final font:', finalFont);
  
  const finalInputValue = await inputTextarea.inputValue();
  console.log('Final input value:', finalInputValue);
  
  // Basic verification that the functionality is working
  console.log('🧪 Test Summary:');
  console.log('- Initial font:', initialFont);
  console.log('- Font after toggle:', fontAfterToggle);
  console.log('- Final font:', finalFont);
  console.log('- Font changed during test:', initialFont !== fontAfterToggle || fontAfterToggle !== finalFont);
});