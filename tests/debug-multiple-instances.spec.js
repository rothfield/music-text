const { test, expect } = require('@playwright/test');

test('debug multiple app instances', async ({ page }) => {
    // Capture console messages
    const messages = [];
    page.on('console', msg => messages.push(msg.text()));
    
    await page.goto('http://localhost:3000');
    await page.waitForSelector('#input-text');
    await page.waitForTimeout(1500); // Wait longer for full initialization
    
    // Count how many times FontManager was initialized
    const initCounts = messages.filter(msg => 
        msg.includes('🔤 FontManager init() called')
    ).length;
    
    const setupCounts = messages.filter(msg => 
        msg.includes('🔤 Adding event listener to fonts button')
    ).length;
    
    const musicAppCounts = messages.filter(msg => 
        msg.includes('✅ Music Text App initialized')
    ).length;
    
    console.log('=== INITIALIZATION COUNTS ===');
    console.log('FontManager init() called:', initCounts);
    console.log('Event listener added:', setupCounts);
    console.log('Music Text App initialized:', musicAppCounts);
    
    // Show all initialization messages
    console.log('=== ALL INITIALIZATION MESSAGES ===');
    messages.forEach((msg, i) => {
        if (msg.includes('FontManager') || msg.includes('Music Text App') || msg.includes('init')) {
            console.log(`${i}: ${msg}`);
        }
    });
    
    // Test passes if we can see the counts
    expect(initCounts).toBeGreaterThan(0);
});