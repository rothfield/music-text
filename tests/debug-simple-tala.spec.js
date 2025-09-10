const { test, expect } = require('@playwright/test');

test('debug simplest possible tala case', async ({ page }) => {
    const logs = [];
    const errors = [];
    
    page.on('console', msg => {
        logs.push(`${msg.type()}: ${msg.text()}`);
    });
    
    page.on('pageerror', error => {
        errors.push(`Page Error: ${error.message}`);
    });
    
    await page.goto('http://localhost:3000');
    
    // Extremely simple case: just one note with tala
    const testInput = `0
C`;
    
    console.log('Testing simplest tala case:', testInput);
    await page.fill('#input-text', testInput);
    await page.waitForTimeout(2000);
    
    console.log('=== All Logs ===');
    logs.forEach(log => console.log(log));
    
    console.log('=== All Errors ===');
    errors.forEach(error => console.log(error));
    
    const hasNaN = logs.some(log => log.includes('NaN')) || errors.some(error => error.includes('NaN'));
    console.log('Has NaN:', hasNaN);
});