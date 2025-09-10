const { test, expect } = require('@playwright/test');

test('debug Unicode toggle behavior with Bb', async ({ page }) => {
    // Capture all console messages
    const messages = [];
    page.on('console', msg => {
        const text = msg.text();
        console.log(`BROWSER: ${msg.type()}: ${text}`);
        messages.push(text);
    });
    
    await page.goto('http://localhost:3000');
    
    // Wait for page to load and initialize
    await page.waitForSelector('#input-text');
    await page.waitForTimeout(1000); // Wait for async initialization
    
    // Clear any existing input
    await page.fill('#input-text', '');
    
    // Type 'Bb' to test 
    await page.fill('#input-text', 'Bb');
    
    console.log('=== INITIAL STATE ===');
    console.log('Initial text value:', await page.inputValue('#input-text'));
    console.log('Unicode toggle checked:', await page.isChecked('#unicode-toggle'));
    
    // Toggle Unicode off
    console.log('=== TOGGLING UNICODE OFF ===');
    await page.click('#unicode-toggle');
    await page.waitForTimeout(500);
    
    console.log('After clicking toggle - checked:', await page.isChecked('#unicode-toggle'));
    console.log('Text value after toggle off:', await page.inputValue('#input-text'));
    
    // Toggle Unicode back on  
    console.log('=== TOGGLING UNICODE ON ===');
    await page.click('#unicode-toggle');
    await page.waitForTimeout(500);
    
    console.log('After clicking toggle again - checked:', await page.isChecked('#unicode-toggle'));
    console.log('Text value after toggle on:', await page.inputValue('#input-text'));
    
    // Log relevant console messages
    console.log('=== RELEVANT CONSOLE MESSAGES ===');
    const relevantMessages = messages.filter(msg => 
        msg.includes('Unicode') || 
        msg.includes('toggle') || 
        msg.includes('refresh') ||
        msg.includes('DOM elements') ||
        msg.includes('Setting up') ||
        msg.includes('patterns')
    );
    relevantMessages.forEach(msg => console.log(`FILTERED: ${msg}`));
    
    // Basic assertions to verify the test ran
    expect(await page.isVisible('#unicode-toggle')).toBe(true);
    expect(await page.isVisible('#input-text')).toBe(true);
});