const { test, expect } = require('@playwright/test');

test('editor workflow - new dialog cancel and type S', async ({ page }) => {
  // Navigate to localhost:3000
  await page.goto('http://localhost:3000/');

  // Handle the confirm dialog that appears when clicking "New"
  page.on('dialog', async dialog => {
    await dialog.dismiss(); // Press ESC to dismiss
  });

  // Click on "New" button
  await page.click('text=New');

  // Click on svg-container to focus it
  await page.click('#svg-container');

  // Wait a moment for focus
  await page.waitForTimeout(500);

  // Type 'S'
  await page.keyboard.type('S');

  // Wait for the SVG to update
  await page.waitForTimeout(1000);

  // Check what SVG content exists for debugging
  const svgContent = await page.locator('#svg-container svg').innerHTML();
  console.log('SVG content after typing S:', svgContent);

  // Look for any text element in SVG (more flexible check)
  await expect(page.locator('#svg-container svg text')).toContainText('S');
});