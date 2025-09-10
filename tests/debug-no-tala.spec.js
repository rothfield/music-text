const { test, expect } = require('@playwright/test');

test('debug same input without tala to verify fix', async ({ page }) => {
    const logs = [];
    const errors = [];
    
    page.on('console', msg => {
        logs.push(`${msg.type()}: ${msg.text()}`);
    });
    
    page.on('pageerror', error => {
        errors.push(`Page Error: ${error.message}`);
    });
    
    await page.goto('http://localhost:3000');
    
    // Same single note but no tala
    const testInput = `C`;
    
    console.log('Testing without tala:', testInput);
    await page.fill('#input-text', testInput);
    await page.waitForTimeout(2000);
    
    console.log('=== Width Debug Logs ===');
    logs.filter(log => log.includes('minWidth') || log.includes('VexFlow Debug')).forEach(log => console.log(log));
    
    const canvas = page.locator('#vexflow-canvas');
    const canvasExists = await canvas.count() > 0;
    const hasContent = canvasExists ? await canvas.getAttribute('class') : 'none';
    console.log('Canvas exists:', canvasExists, 'class:', hasContent);
    
    const hasNaN = logs.some(log => log.includes('NaN')) || errors.some(error => error.includes('NaN'));
    console.log('Has NaN:', hasNaN);
});