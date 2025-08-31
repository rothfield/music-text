const { test, expect } = require('@playwright/test');

test('debug sharp notation in VexFlow', async ({ page }) => {
    // Listen for console messages
    const consoleMessages = [];
    page.on('console', msg => {
        consoleMessages.push(`${msg.type()}: ${msg.text()}`);
    });

    await page.goto('http://localhost:3000');
    
    // Wait for WASM to load
    await page.waitForSelector('#notation-input', { timeout: 10000 });
    
    // Enter 1# 
    await page.fill('#notation-input', '1#');
    
    // Click process
    await page.click('button:has-text("Process")');
    
    // Wait for processing
    await page.waitForTimeout(2000);
    
    // Log all console messages containing 'DEBUG'
    console.log('=== CONSOLE DEBUG MESSAGES ===');
    consoleMessages.filter(msg => msg.includes('DEBUG')).forEach(msg => {
        console.log(msg);
    });
    
    // Also log any messages with sharp notation
    consoleMessages.filter(msg => msg.includes('#') || msg.includes('cs')).forEach(msg => {
        console.log('SHARP/CS:', msg);
    });
    
    console.log('=== ALL CONSOLE MESSAGES ===');
    consoleMessages.forEach(msg => {
        console.log(msg);
    });
});