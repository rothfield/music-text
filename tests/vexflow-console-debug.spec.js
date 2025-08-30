const { test, expect } = require('@playwright/test');

test.describe('VexFlow Console Debug', () => {
  test('debug cross-beat slur console output', async ({ page }) => {
    // Capture console messages
    const consoleMessages = [];
    page.on('console', msg => {
      if (msg.text().includes('VexFlow:') || msg.text().includes('Drawing slur')) {
        consoleMessages.push(msg.text());
      }
    });
    
    await page.goto('http://localhost:3000');
    
    // Test cross-beat slur pattern
    await page.fill('#notation-input', '(1-2 3)');
    await page.waitForTimeout(3000);
    
    // Log all VexFlow console messages
    console.log('=== VexFlow Console Messages ===');
    for (const msg of consoleMessages) {
      console.log(msg);
    }
    console.log('=== End Console Messages ===');
    
    // Check if VexFlow rendered
    const vexflowContainer = await page.locator('#vexflow-canvas');
    const content = await vexflowContainer.innerHTML();
    const hasVexFlow = content.includes('svg');
    
    console.log('VexFlow rendered:', hasVexFlow);
    expect(hasVexFlow).toBe(true);
  });
  
  test('debug simple slur pattern', async ({ page }) => {
    const consoleMessages = [];
    page.on('console', msg => {
      if (msg.text().includes('VexFlow:') || msg.text().includes('Drawing slur')) {
        consoleMessages.push(msg.text());
      }
    });
    
    await page.goto('http://localhost:3000');
    
    // Test simple slur pattern  
    await page.fill('#notation-input', '(1 2) 3');
    await page.waitForTimeout(3000);
    
    console.log('=== Simple Slur Console Messages ===');
    for (const msg of consoleMessages) {
      console.log(msg);
    }
    console.log('=== End Console Messages ===');
  });
});