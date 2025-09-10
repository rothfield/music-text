const { test, expect } = require('@playwright/test');

test('debug simple notation without tala', async ({ page }) => {
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
    
    // Simple input without tala
    const testInput = `C D E |`;
    
    console.log('Filling input with:', testInput);
    await page.fill('#input-text', testInput);
    
    // Wait longer for processing
    await page.waitForTimeout(3000);
    
    // Check if canvas element exists
    const canvas = page.locator('#vexflow-canvas');
    const canvasExists = await canvas.count() > 0;
    console.log('Canvas element exists:', canvasExists);
    
    if (canvasExists) {
        const classes = await canvas.getAttribute('class');
        console.log('Canvas classes:', classes);
    }
    
    // Print relevant logs and errors
    console.log('=== Key Logs ===');
    logs.filter(log => log.includes('minWidth') || log.includes('Error') || log.includes('VexFlow')).forEach(log => console.log(log));
    
    console.log('=== Errors ===');
    errors.forEach(error => console.log(error));
    
    expect(canvasExists).toBe(true);
});