const { test, expect } = require('@playwright/test');

test('mordent ornament rendering test', async ({ page }) => {
  // Go to the notation parser web interface
  await page.goto('http://localhost:3000');
  
  // Wait for the page to load
  await page.waitForLoadState('networkidle');
  
  // Input the mordent notation
  await page.fill('#input-text', '~1~');
  
  // Click render or wait for auto-rendering
  await page.click('#render-btn, #update-btn, button:has-text("Render"), button:has-text("Update")');
  
  // Wait for VexFlow to render
  await page.waitForTimeout(2000);
  
  // Take a screenshot to visually verify mordent rendering
  await page.screenshot({ path: 'test-results/mordent-rendering.png', fullPage: true });
  
  // Check console for mordent-related logs
  const consoleMessages = [];
  page.on('console', msg => consoleMessages.push(msg.text()));
  
  // Look for VexFlow canvas or SVG with rendered content
  const canvas = await page.locator('canvas, svg').first();
  await expect(canvas).toBeVisible();
  
  // Check if ornament was applied (look for console logs)
  await page.waitForTimeout(1000);
  const ornamentLogs = consoleMessages.filter(msg => 
    msg.includes('ornament') || msg.includes('mordent') || msg.includes('🎵')
  );
  
  console.log('Console messages about ornaments:', ornamentLogs);
  
  // Verify that VexFlow JSON contains ornament data
  const vexflowOutput = await page.locator('#vexflow-output, .vexflow-json, pre').textContent();
  expect(vexflowOutput).toContain('ornaments');
  expect(vexflowOutput).toContain('Mordent');
});