const { test, expect } = require('@playwright/test');

test('VexFlow beaming for SSSS pattern', async ({ page }) => {
  await page.goto('http://localhost:3000');
  
  // Wait for page to load
  await page.waitForTimeout(2000);
  
  // Enter SSSS pattern
  const textarea = page.locator('#notation-input');
  await textarea.fill('| SSSS | RRRR |');
  
  // Wait for rendering
  await page.waitForTimeout(2000);
  
  // Take screenshot of VexFlow output
  const vexflowContainer = page.locator('#vexflow-canvas');
  await vexflowContainer.screenshot({ path: 'test_output/vexflow_ssss_beaming.png' });
  
  // Also test with number notation
  await textarea.fill('| 1111 | 2222 |');
  await page.waitForTimeout(2000);
  await vexflowContainer.screenshot({ path: 'test_output/vexflow_1111_beaming.png' });
  
  // Check if SVG exists
  const svg = page.locator('#vexflow-canvas svg');
  await expect(svg).toBeVisible({ timeout: 5000 });
  
  // Look for beam elements in the SVG
  const beamPaths = await page.locator('#vexflow-canvas svg path').count();
  console.log(`Found ${beamPaths} path elements in SVG`);
  
  // Get the full SVG content for analysis
  const svgContent = await svg.innerHTML();
  const hasBeamClasses = svgContent.includes('vf-beam');
  console.log('SVG contains beam classes:', hasBeamClasses);
  console.log('SVG sample:', svgContent.substring(0, 500));
  
  // Test should FAIL if no beams are found
  expect(hasBeamClasses).toBe(true);
});