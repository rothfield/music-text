const { test, expect } = require('@playwright/test');

test('Unicode toggle should change font', async ({ page }) => {
  // Listen for console messages and errors
  page.on('console', msg => console.log('BROWSER:', msg.text()));
  page.on('pageerror', error => console.log('PAGE ERROR:', error.message));
  
  // Start by going to the file:// path to load the webapp locally
  await page.goto('file:///home/john/projects/music-text/webapp/index.html');
  
  // Wait for the page to load
  await page.waitForLoadState('networkidle');
  
  // Wait a bit for JavaScript to initialize
  await page.waitForTimeout(1000);
  
  // Check if unicode toggle exists
  const unicodeToggle = page.locator('#unicode-toggle');
  await expect(unicodeToggle).toBeVisible();
  
  // Get the input textarea
  const inputTextarea = page.locator('#input-text');
  await expect(inputTextarea).toBeVisible();
  
  // Click the Fonts button to show font configuration
  const fontsButton = page.locator('#fonts-button');
  await expect(fontsButton).toBeVisible();
  await fontsButton.click();
  
  // Get the font select dropdown (now visible)
  const fontSelect = page.locator('#font-select');
  await expect(fontSelect).toBeVisible();
  
  // Test that unicode toggle is initially checked (should be default)
  await expect(unicodeToggle).toBeChecked();
  
  // Get initial font family
  const initialFont = await inputTextarea.evaluate(el => window.getComputedStyle(el).fontFamily);
  console.log('Initial font:', initialFont);
  
  // Toggle unicode OFF
  await unicodeToggle.click();
  await page.waitForTimeout(100); // Small delay for font switching
  
  // Check that it's now unchecked
  await expect(unicodeToggle).not.toBeChecked();
  
  // Get font after toggle
  const fontAfterToggleOff = await inputTextarea.evaluate(el => window.getComputedStyle(el).fontFamily);
  console.log('Font after toggle OFF:', fontAfterToggleOff);
  
  // Toggle unicode back ON
  await unicodeToggle.click();
  await page.waitForTimeout(100); // Small delay for font switching
  
  // Check that it's checked again
  await expect(unicodeToggle).toBeChecked();
  
  // Get font after toggle back on
  const fontAfterToggleOn = await inputTextarea.evaluate(el => window.getComputedStyle(el).fontFamily);
  console.log('Font after toggle ON:', fontAfterToggleOn);
  
  // The font should change when toggling
  if (initialFont !== fontAfterToggleOff) {
    console.log('✅ Font changed when toggled OFF');
  } else {
    console.log('⚠️ Font did NOT change when toggled OFF');
  }
  
  // Test input with musical characters
  await inputTextarea.fill('|1-2#b');
  await page.waitForTimeout(500);
  
  // Check that the input contains unicode characters when toggle is on
  const inputValue = await inputTextarea.inputValue();
  console.log('Input value with unicode ON:', inputValue);
  
  // Toggle off again and check if input changes back to standard characters
  await unicodeToggle.click();
  await page.waitForTimeout(500);
  
  const inputValueOff = await inputTextarea.inputValue();
  console.log('Input value with unicode OFF:', inputValueOff);
  
  // Font select dropdown should reflect the font changes
  const selectedFontOff = await fontSelect.inputValue();
  console.log('Selected font when unicode OFF:', selectedFontOff);
  
  // Toggle back on
  await unicodeToggle.click();
  await page.waitForTimeout(500);
  
  const selectedFontOn = await fontSelect.inputValue();
  console.log('Selected font when unicode ON:', selectedFontOn);
  
  // Fonts should be different between unicode on/off states
  expect(selectedFontOff).not.toBe(selectedFontOn);
});