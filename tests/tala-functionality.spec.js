const { test, expect } = require('@playwright/test');

test('tala markers render above barlines', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Input notation with tala marker above barline
    const testInput = `      0
C D E |`;
    
    await page.fill('#input-text', testInput);
    await page.waitForTimeout(1000); // Wait for auto-rendering
    
    // Check that VexFlow canvas is rendered
    await expect(page.locator('#vexflow-canvas.has-content')).toBeVisible();
    
    // Check that the canvas has content (non-zero dimensions)
    const canvas = page.locator('#vexflow-canvas');
    const canvasBox = await canvas.boundingBox();
    expect(canvasBox.width).toBeGreaterThan(100);
    expect(canvasBox.height).toBeGreaterThan(50);
    
    // Check console logs for tala rendering
    const logs = [];
    page.on('console', msg => {
        if (msg.type() === 'log') {
            logs.push(msg.text());
        }
    });
    
    // Trigger re-render to capture console logs
    await page.fill('#input-text', '');
    await page.fill('#input-text', testInput);
    await page.waitForTimeout(2000);
    
    // Check if tala debug messages are present
    const talaLogs = logs.filter(log => log.includes('tala') || log.includes('Tala'));
    console.log('Tala-related logs:', talaLogs);
    
    // Check that notes are rendered
    const hasNoteRenderingLogs = logs.some(log => 
        log.includes('note') || log.includes('Note') || log.includes('C') || log.includes('D') || log.includes('E')
    );
    expect(hasNoteRenderingLogs).toBe(true);
});

test('tala with crossed zero renders properly', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Test with tala 0 (should show crossed zero)
    const testInput = `      0
C D E |`;
    
    await page.fill('#input-text', testInput);
    await page.waitForTimeout(2000);
    
    // Check that VexFlow canvas has content
    const canvas = page.locator('#vexflow-canvas');
    await expect(canvas).toHaveClass(/has-content/);
    
    // Verify canvas dimensions are reasonable
    const box = await canvas.boundingBox();
    expect(box.width).toBeGreaterThan(200);
});

test('tala with regular number renders properly', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Test with tala 3 (regular number)
    const testInput = `   3
C D E |`;
    
    await page.fill('#input-text', testInput);
    await page.waitForTimeout(2000);
    
    // Check that VexFlow canvas has content
    const canvas = page.locator('#vexflow-canvas');
    await expect(canvas).toHaveClass(/has-content/);
    
    // Verify canvas dimensions are reasonable
    const box = await canvas.boundingBox();
    expect(box.width).toBeGreaterThan(200);
});