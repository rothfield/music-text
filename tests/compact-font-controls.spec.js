const { test, expect } = require('@playwright/test');

test('Compact font controls should work in status bar', async ({ page }) => {
  // Listen for console messages and errors
  page.on('console', msg => console.log('BROWSER:', msg.text()));
  page.on('pageerror', error => console.log('PAGE ERROR:', error.message));
  
  // Start by going to the HTTP server
  // Use configured baseURL from Playwright config
  await page.goto('/');
  
  // Wait for the page to load
  await page.waitForLoadState('networkidle');
  
  // Wait a bit for JavaScript to initialize
  await page.waitForTimeout(2000);
  
  // Check if all compact controls are visible in status bar
  const fontSelect = page.locator('#font-select');
  await expect(fontSelect).toBeVisible();
  
  const fontSizeSlider = page.locator('#font-size');
  await expect(fontSizeSlider).toBeVisible();
  
  const fontSizeValue = page.locator('#font-size-value');
  await expect(fontSizeValue).toBeVisible();
  
  const inputTextarea = page.locator('#input-text');
  await expect(inputTextarea).toBeVisible();
  
  // Test font family change
  console.log('Testing font family change...');
  await fontSelect.selectOption("'Consolas', monospace");
  await page.waitForTimeout(300);
  
  const fontAfterChange = await inputTextarea.evaluate(el => window.getComputedStyle(el).fontFamily);
  console.log('Font after change:', fontAfterChange);
  expect(fontAfterChange).toContain('Consolas');
  
  // Test font size change
  console.log('Testing font size change...');
  const initialSize = await fontSizeValue.textContent();
  console.log('Initial size:', initialSize);
  
  // Move slider to a different value
  await fontSizeSlider.fill('14');
  await page.waitForTimeout(300);
  
  const newSizeDisplay = await fontSizeValue.textContent();
  const actualFontSize = await inputTextarea.evaluate(el => window.getComputedStyle(el).fontSize);
  
  console.log('New size display:', newSizeDisplay);
  console.log('Actual font size:', actualFontSize);
  
  expect(newSizeDisplay).toBe('14px');
  expect(actualFontSize).toBe('14px');
  
  // Test unicode toggle still works with compact layout
  const unicodeToggle = page.locator('#unicode-toggle');
  await expect(unicodeToggle).toBeVisible();
  
  // Add some text to test unicode functionality with the current (non-unicode) font
  await inputTextarea.fill('|1-2#b');
  await page.waitForTimeout(500);
  
  // With Consolas (non-unicode font), should show standard characters
  const inputValueWithNonUnicodeFont = await inputTextarea.inputValue();
  console.log('Input value with non-unicode font:', inputValueWithNonUnicodeFont);
  
  // Should contain standard characters since Consolas is not unicode-capable
  expect(inputValueWithNonUnicodeFont).toBe('|1-2#b');
  
  // Now switch to a unicode-capable font and test unicode replacement
  console.log('Testing unicode with JuliaMono...');
  await fontSelect.selectOption("'JuliaMono', monospace");
  await page.waitForTimeout(500);
  
  // Clear and re-enter text to trigger unicode replacement
  await inputTextarea.fill('');
  await inputTextarea.fill('|1-2#b');
  await page.waitForTimeout(500);
  
  const inputValueWithUnicodeFont = await inputTextarea.inputValue();
  console.log('Input value with unicode font:', inputValueWithUnicodeFont);
  
  // Should contain unicode characters since JuliaMono is unicode-capable
  expect(inputValueWithUnicodeFont).toContain('┃');
  expect(inputValueWithUnicodeFont).toContain('▬');
  expect(inputValueWithUnicodeFont).toContain('♯');
  expect(inputValueWithUnicodeFont).toContain('♭');
  
  console.log('✅ All compact font controls working correctly!');
});
