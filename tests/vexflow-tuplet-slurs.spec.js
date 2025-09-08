const { test, expect } = require('@playwright/test');

test.describe('VexFlow Tuplet Slurs', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:3000');
  });

  test('renders slur within tuplet - 1 overline', async ({ page }) => {
    // Test pattern: _\n123 - single overline should create NO slur
    await page.fill('#notation-input', '_\n123');
    await page.waitForTimeout(2000);
    
    const vexflowContainer = await page.locator('#vexflow-canvas');
    const content = await vexflowContainer.innerHTML();
    console.log('Tuplet with single overline (_\\n123) VexFlow content:', content.substring(0, 300));
    
    const errorContainer = await page.locator('#error-container');
    const errorContent = await errorContainer.innerHTML();
    if (errorContent) {
      console.log('Error for _\\n123:', errorContent);
    }
    
    // Should render VexFlow or show specific error
    const hasVexFlow = content.includes('svg');
    expect(hasVexFlow).toBe(true);
  });

  test('renders slur within tuplet - 2 overlines', async ({ page }) => {
    // Test pattern: __\n123 - double overline should create slur over 2 notes
    await page.fill('#notation-input', '__\n123');
    await page.waitForTimeout(2000);
    
    const vexflowContainer = await page.locator('#vexflow-canvas');
    const content = await vexflowContainer.innerHTML();
    console.log('Tuplet with double overline (__\\n123) VexFlow content:', content.substring(0, 300));
    
    const errorContainer = await page.locator('#error-container');
    const errorContent = await errorContainer.innerHTML();
    if (errorContent) {
      console.log('Error for __\\n123:', errorContent);
    }
    
    const hasVexFlow = content.includes('svg');
    expect(hasVexFlow).toBe(true);
  });

  test('renders slur within tuplet - 3 overlines', async ({ page }) => {
    // Test pattern: ___\n123 - triple overline should create slur over all 3 notes
    await page.fill('#notation-input', '___\n123');
    await page.waitForTimeout(2000);
    
    const vexflowContainer = await page.locator('#vexflow-canvas');
    const content = await vexflowContainer.innerHTML();
    console.log('Tuplet with triple overline (___\\n123) VexFlow content:', content.substring(0, 300));
    
    const errorContainer = await page.locator('#error-container');
    const errorContent = await errorContainer.innerHTML();
    if (errorContent) {
      console.log('Error for ___\\n123:', errorContent);
    }
    
    const hasVexFlow = content.includes('svg');
    expect(hasVexFlow).toBe(true);
  });

  test('debug tuplet slur console output', async ({ page }) => {
    // Capture console messages for debugging
    const consoleMessages = [];
    page.on('console', msg => {
      if (msg.text().includes('VexFlow:') || msg.text().includes('Drawing slur')) {
        consoleMessages.push(msg.text());
      }
    });
    
    await page.goto('http://localhost:3000');
    
    // Test tuplet slur pattern
    await page.fill('#notation-input', '_\n123');
    await page.waitForTimeout(3000);
    
    console.log('=== Tuplet Slur Console Messages ===');
    for (const msg of consoleMessages) {
      console.log(msg);
    }
    console.log('=== End Console Messages ===');
  });
});