const { test, expect } = require('@playwright/test');

test('check console for 1# sharp notation', async ({ page }) => {
    // Capture all console messages
    const messages = [];
    page.on('console', msg => messages.push(msg.text()));
    
    await page.goto('http://localhost:3000');
    await page.waitForSelector('#input-text');
    await page.type('#input-text', '1#');
    await page.waitForTimeout(2000); // Wait for auto-processing
    
    // Capture the full VexFlow JavaScript from the page
    const fullJS = await page.evaluate(() => {
        // Try to get it from various possible sources
        if (window.lastVexFlowJS) return window.lastVexFlowJS;
        if (window.vexflowOutput) return window.vexflowOutput;
        return 'JavaScript not found in global scope';
    });
    
    console.log('=== FULL GENERATED JAVASCRIPT ===');
    console.log(fullJS);
    console.log('=== END JAVASCRIPT ===');
    
    console.log('\n=== ALL CONSOLE MESSAGES ===');
    messages.forEach((msg, i) => {
        console.log(`${i}: ${msg}`);
    });
    
    // The test passes if we got some output
    expect(messages.length).toBeGreaterThan(0);
});