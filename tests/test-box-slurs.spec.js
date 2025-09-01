const { test, expect } = require('@playwright/test');

test('Box drawing slurs render correctly', async ({ page }) => {
  await page.goto('http://localhost:3000');
  
  // Wait for the page to load completely
  await page.waitForSelector('.notation-input');
  
  // Input box drawing slur notation
  const boxSlurInput = `      ╭──╮
S R G M

      ____
1 2 3 4

      ╭───╮
P D N S`;

  await page.fill('.notation-input', boxSlurInput);
  
  // Wait for rendering to complete
  await page.waitForTimeout(1000);
  
  // Check that VexFlow canvas has content
  const canvas = page.locator('#vexflow-output .has-content');
  await expect(canvas).toBeVisible();
  
  // Check that slurs are rendered (look for slur paths in the SVG)
  const slurPaths = page.locator('#vexflow-output svg path');
  await expect(slurPaths).toHaveCount({ gte: 3 }); // Should have at least 3 slur paths
  
  // Take a screenshot for visual verification
  await page.screenshot({ 
    path: 'test_output/box-slurs-test.png',
    fullPage: true 
  });
});