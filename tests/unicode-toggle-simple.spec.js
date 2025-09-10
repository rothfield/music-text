const { test, expect } = require('@playwright/test');

test('simple Unicode toggle test with |Bb', async ({ page }) => {
    // Capture console messages
    const messages = [];
    page.on('console', msg => {
        const text = msg.text();
        console.log(`BROWSER: ${text}`);
        messages.push(text);
    });
    
    await page.goto('http://localhost:3000');
    await page.waitForSelector('#input-text');
    await page.waitForTimeout(1000);
    
    // Test with valid barlined input
    await page.fill('#input-text', '|Bb');
    
    console.log('=== TESTING WITH |Bb ===');
    console.log('Initial text:', await page.inputValue('#input-text'));
    console.log('Unicode toggle checked:', await page.isChecked('#unicode-toggle'));
    
    // Toggle Unicode OFF
    await page.click('#unicode-toggle');
    await page.waitForTimeout(300);
    
    console.log('After toggle OFF:', await page.inputValue('#input-text'));
    
    // Toggle Unicode ON
    await page.click('#unicode-toggle');  
    await page.waitForTimeout(300);
    
    console.log('After toggle ON:', await page.inputValue('#input-text'));
    
    // Test assertions
    expect(await page.isVisible('#unicode-toggle')).toBe(true);
});