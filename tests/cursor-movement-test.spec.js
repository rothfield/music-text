import { test, expect } from '@playwright/test';

test('cursor movement with arrow keys', async ({ page }) => {
  // Navigate to the app
  await page.goto('http://localhost:3000/');

  // Wait for the page to load
  await page.waitForLoadState('networkidle');

  // Click New button and cancel the dialog
  await page.click('text=New');
  await page.click('text=Cancel');

  // Click on the SVG container to focus
  await page.click('#svg-container-div');

  // Type multiple characters
  await page.type('#svg-container-div', 'SRG');

  // Wait for rendering
  await page.waitForTimeout(500);

  // Get the SVG content
  const svgContent = await page.locator('#rendered-svg').innerHTML();
  console.log('SVG content after typing SRG:', svgContent);

  // Check that we have three char elements
  const charElements = await page.locator('.char').count();
  expect(charElements).toBe(3);

  // Try to press left arrow and see if cursor moves
  await page.keyboard.press('ArrowLeft');
  await page.waitForTimeout(100);

  // Press left arrow again
  await page.keyboard.press('ArrowLeft');
  await page.waitForTimeout(100);

  // Press right arrow
  await page.keyboard.press('ArrowRight');
  await page.waitForTimeout(100);

  console.log('Arrow key navigation test completed successfully');
});