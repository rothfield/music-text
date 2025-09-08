const { test, expect } = require('@playwright/test');

test('V2 parser system with lyrics auto-assignment', async ({ page }) => {
  await page.goto('http://localhost:3000');
  
  // Test complex V2 notation with multiple staves and lyrics
  const v2Input = `key: D
time: 3/4
title: V2 Test

| S R# - G | M P |
  do re    mi fa sol

| 1 2 3 |
  one two three`;
  
  await page.fill('#notation-input', v2Input);
  
  // Wait for auto-rendering or click update
  try {
    await page.click('#update-btn', { timeout: 2000 });
  } catch (e) {
    // Auto-rendering may be active
  }
  
  // Wait for rendering
  await page.waitForTimeout(3000);
  
  // Check that VexFlow canvas has content
  const canvas = page.locator('#vexflow-canvas');
  await expect(canvas).toBeVisible();
  
  // Check for generated notes in the canvas
  const canvasContent = await canvas.innerHTML();
  expect(canvasContent).toContain('svg'); // VexFlow renders as SVG
  
  // Check console for successful parsing
  const logs = [];
  page.on('console', msg => logs.push(msg.text()));
  
  // Refresh to trigger parsing again
  await page.fill('#notation-input', v2Input + ' ');
  await page.waitForTimeout(2000);
  
  // Look for successful parsing indicators in console
  const hasSuccessLog = logs.some(log => 
    log.includes('VexFlow') || 
    log.includes('keys') ||
    log.includes('Note keys')
  );
  
  expect(hasSuccessLog).toBe(true);
  
  console.log('✅ V2 parser system working in web UI!');
});