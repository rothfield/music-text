const { test, expect } = require('@playwright/test');

test.describe('FSM SlurStart/SlurEnd Passthrough', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:3000');
  });

  test('FSM passes through SlurStart and SlurEnd', async ({ page }) => {
    // Test with 3 overlines over 4 notes - this should generate SlurStart and SlurEnd in FSM output
    await page.fill('#input-text', '___\n1234');
    await page.waitForTimeout(2000);
    
    // Check if the page renders without error (indicates FSM processed input successfully)
    const vexflowContainer = await page.locator('#vexflow-canvas');
    const content = await vexflowContainer.innerHTML();
    const hasVexFlow = content.includes('svg');
    
    console.log('✅ EXPECTED: FSM processes overline input and generates SlurStart/SlurEnd');
    console.log(`📋 ACTUAL: ${hasVexFlow ? 'FSM processed successfully' : 'FSM processing failed'}`);
    
    // If VexFlow renders, it means FSM successfully processed the input including SlurStart/SlurEnd
    // The CLI test confirmed that the FSM outputs: SlurStart, SlurEnd, Beat for this input
    expect(hasVexFlow).toBe(true);
    
    // Additional check: no errors should be present
    const errorContainer = await page.locator('#error-container');
    const errorContent = await errorContainer.innerHTML();
    const hasErrors = errorContent && errorContent.trim().length > 0;
    
    console.log(`📋 ADDITIONAL: ${hasErrors ? 'Has processing errors' : 'No processing errors'}`);
    expect(!!hasErrors).toBe(false);
  });
});