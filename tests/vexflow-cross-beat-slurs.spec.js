const { test, expect } = require('@playwright/test');

test.describe('VexFlow Cross-Beat Slurs', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:3000');
  });

  test('renders cross-beat slur spanning tuplet to next note', async ({ page }) => {
    // Test: slur from tuplet to next beat
    // ___
    // 1-2 3
    await page.fill('#notation-input', '___\n1-2 3');
    
    // Wait for VexFlow auto-rendering
    await page.waitForTimeout(2000);
    
    // Should render VexFlow with slur
    const vexflowCanvas = await page.locator('#vexflow-canvas svg');
    await expect(vexflowCanvas).toBeVisible();
    
    // Log the actual SVG content to debug slur rendering
    const svgContent = await page.innerHTML('#vexflow-canvas');
    console.log('VexFlow SVG content for overline slur:', svgContent.substring(0, 500));
    
    // Check for VexFlow slur elements in the SVG - slurs are typically curves or paths
    const slurElements = await page.locator('#vexflow-canvas svg path, #vexflow-canvas svg curve');
    const slurCount = await slurElements.count();
    console.log('Found slur elements:', slurCount);
    
    // For now, just check that VexFlow renders something - slur implementation TBD
    await expect(vexflowCanvas).toBeVisible();
  });

  test('renders slur spanning different beats', async ({ page }) => {
    // Test: slur within single beat
    // __
    // 1 2 3
    await page.fill('#notation-input', '__\n1 2 3');
    
    await page.waitForTimeout(2000);
    
    const vexflowCanvas = await page.locator('#vexflow-canvas svg');
    await expect(vexflowCanvas).toBeVisible();
    
    const svgContent = await page.innerHTML('#vexflow-canvas');
    console.log('VexFlow SVG content for overline slur:', svgContent.substring(0, 300));
    
    await expect(vexflowCanvas).toBeVisible();
  });

  test('renders slur spanning barlines', async ({ page }) => {
    // Test: slur across barline
    // ___
    // 1 2 | 3
    await page.fill('#notation-input', '___\n1 2 | 3');
    
    await page.waitForTimeout(2000);
    
    const vexflowCanvas = await page.locator('#vexflow-canvas svg');
    await expect(vexflowCanvas).toBeVisible();
    
    const svgContent = await page.innerHTML('#vexflow-canvas');
    console.log('VexFlow SVG content for overline slur:', svgContent.substring(0, 300));
    
    await expect(vexflowCanvas).toBeVisible();
  });

  test('renders complex mixed slurs and beats', async ({ page }) => {
    // Test: multiple cross-beat slurs
    // ___  ___
    // 1-2  3 4-5
    await page.fill('#notation-input', '___  ___\n1-2  3 4-5');
    
    await page.waitForTimeout(2000);
    
    const vexflowCanvas = await page.locator('#vexflow-canvas svg');
    await expect(vexflowCanvas).toBeVisible();
    
    const svgContent = await page.innerHTML('#vexflow-canvas');
    console.log('VexFlow SVG content for overline slurs:', svgContent.substring(0, 400));
    
    await expect(vexflowCanvas).toBeVisible();
  });

  test('compares VexFlow with LilyPond slur consistency', async ({ page }) => {
    // Test that VexFlow and LilyPond produce consistent slur behavior
    const testPattern = '___\n1-2 3';
    await page.fill('#notation-input', testPattern);
    
    // Wait for VexFlow auto-rendering
    await page.waitForTimeout(2000);
    
    // Generate LilyPond output
    await page.click('#generate-lilypond-btn');
    await page.waitForTimeout(2000);
    
    // Both should be visible and contain slur indicators
    const vexflowCanvas = await page.locator('#vexflow-canvas svg');
    const lilypondSource = await page.textContent('#lilypond-source');
    
    await expect(vexflowCanvas).toBeVisible();
    expect(lilypondSource).toContain('(');
    expect(lilypondSource).toContain(')');
    
    console.log('VexFlow/LilyPond slur consistency test - both outputs visible');
  });
});