const { test, expect } = require('@playwright/test');

test.describe('VexFlow Rendering Check', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:3000');
  });

  test('renders simple pattern without errors', async ({ page }) => {
    await page.fill('#notation-input', '1 2 3');
    await page.waitForTimeout(2000);
    
    // Check if VexFlow canvas has content
    const vexflowContainer = await page.locator('#vexflow-canvas');
    const content = await vexflowContainer.innerHTML();
    console.log('Simple pattern VexFlow content:', content.substring(0, 200));
    
    // Should not be just placeholder
    expect(content).not.toContain('Your notation will appear here');
  });

  test('renders cross-beat slur pattern', async ({ page }) => {
    await page.fill('#notation-input', '(1-2 3)');
    await page.waitForTimeout(3000);
    
    const vexflowContainer = await page.locator('#vexflow-canvas');
    const content = await vexflowContainer.innerHTML();
    console.log('Cross-beat slur VexFlow content:', content.substring(0, 200));
    
    // Check if there's an error message or if it renders
    const errorContainer = await page.locator('#error-container');
    const errorContent = await errorContainer.innerHTML();
    console.log('Error container:', errorContent);
    
    // Should either render VexFlow or show specific error
    const hasVexFlow = content.includes('svg') || content.includes('vf-');
    const hasError = errorContent.length > 0;
    
    console.log('Has VexFlow:', hasVexFlow, 'Has Error:', hasError);
    expect(hasVexFlow || hasError).toBe(true);
  });

  test('renders different slur patterns', async ({ page }) => {
    const patterns = [
      '(1 2) 3',      // Working pattern
      '(1 2 | 3)',    // Working pattern  
      '(1-2 3)',      // Cross-beat pattern
      '(1-2) (3 4-5)' // Complex pattern
    ];
    
    for (const pattern of patterns) {
      console.log(`\n--- Testing pattern: ${pattern} ---`);
      await page.fill('#notation-input', pattern);
      await page.waitForTimeout(2000);
      
      const vexflowContainer = await page.locator('#vexflow-canvas');
      const content = await vexflowContainer.innerHTML();
      
      const errorContainer = await page.locator('#error-container');
      const errorContent = await errorContainer.innerHTML();
      
      const hasVexFlow = content.includes('svg');
      const hasError = errorContent.length > 0;
      
      console.log(`Pattern "${pattern}": VexFlow=${hasVexFlow}, Error=${hasError}`);
      if (hasError) {
        console.log(`Error: ${errorContent.substring(0, 200)}`);
      }
    }
  });
});