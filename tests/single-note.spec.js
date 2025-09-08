const { test, expect } = require('@playwright/test');

test.describe('Single Note Rendering', () => {
  test('renders single "1" as quarter note', async ({ page }) => {
    // Navigate to the web app
    await page.goto('http://localhost:3000');
    
    // Input a single "1"
    await page.fill('textarea', '1');
    
    // Click process button
    await page.click('button:has-text("Process")');
    
    // Wait for processing
    await page.waitForTimeout(1000);
    
    // Check for errors
    const errorMessage = await page.locator('.error-message').textContent().catch(() => null);
    if (errorMessage) {
      console.error('Error found:', errorMessage);
    }
    expect(errorMessage).toBeNull();
    
    // Check that VexFlow canvas has content
    const canvasHasContent = await page.locator('#vexflow-canvas.has-content').isVisible();
    expect(canvasHasContent).toBe(true);
    
    // Get the generated VexFlow JavaScript
    const vexflowOutput = await page.locator('#vexflow-output').textContent();
    console.log('VexFlow Output:', vexflowOutput);
    
    // Check that it contains a quarter note duration
    expect(vexflowOutput).toContain("duration: 'q'");
    
    // Check console for debug messages
    const consoleLogs = [];
    page.on('console', msg => consoleLogs.push(msg.text()));
    
    // Re-process to capture console logs
    await page.click('button:has-text("Process")');
    await page.waitForTimeout(500);
    
    console.log('Console logs:', consoleLogs);
  });
});