const { test, expect } = require('@playwright/test');

test('debug tala rendering issues', async ({ page }) => {
    // Capture console logs and errors
    const logs = [];
    const errors = [];
    
    page.on('console', msg => {
        logs.push(`${msg.type()}: ${msg.text()}`);
        if (msg.type() === 'error') {
            errors.push(msg.text());
        }
    });
    
    page.on('pageerror', error => {
        errors.push(`Page Error: ${error.message}`);
    });
    
    await page.goto('http://localhost:3000');
    
    // Input notation with tala marker above barline
    const testInput = `      0
C D E |`;
    
    console.log('Filling input with:', testInput);
    await page.fill('#notation-input', testInput);
    
    // Wait longer for processing
    await page.waitForTimeout(3000);
    
    // Check if canvas element exists
    const canvas = page.locator('#vexflow-canvas');
    const canvasExists = await canvas.count() > 0;
    console.log('Canvas element exists:', canvasExists);
    
    if (canvasExists) {
        const classes = await canvas.getAttribute('class');
        console.log('Canvas classes:', classes);
        
        const innerHTML = await canvas.innerHTML();
        console.log('Canvas content length:', innerHTML.length);
        console.log('Canvas content preview:', innerHTML.substring(0, 200));
    }
    
    // Print all logs and errors
    console.log('=== Console Logs ===');
    logs.forEach(log => console.log(log));
    
    console.log('=== Errors ===');
    errors.forEach(error => console.log(error));
    
    // Simple assertion
    expect(canvasExists).toBe(true);
});