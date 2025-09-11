const { test, expect } = require('@playwright/test');

test('SVG auto-generation on tab click', async ({ page }) => {
    // Capture console messages
    const messages = [];
    page.on('console', msg => messages.push(msg.text()));
    
    await page.goto('http://localhost:3000');
    await page.waitForSelector('#input-text');
    
    // Enter some test input
    await page.fill('#input-text', '|1 2 3|');
    await page.waitForTimeout(1000);
    
    // Click on SVG tab
    await page.click('#svg-tab-btn');
    await page.waitForTimeout(3000); // Wait for auto-generation
    
    // Check if auto-generation message appeared in console
    const autoGenMessage = messages.find(msg => msg.includes('Auto-generating SVG on first tab visit'));
    
    console.log('=== CONSOLE MESSAGES ===');
    messages.forEach((msg, i) => {
        if (msg.includes('Auto-generating') || msg.includes('SVG')) {
            console.log(`${i}: ${msg}`);
        }
    });
    
    // Check if SVG content was generated (not just placeholder text)
    const svgContent = await page.locator('#svg-content').textContent();
    const hasRealContent = !svgContent.includes('LilyPond SVG rendering will appear here');
    
    console.log('SVG Content Preview:', svgContent.substring(0, 100) + '...');
    console.log('Has real content (not placeholder):', hasRealContent);
    console.log('Found auto-generation message:', !!autoGenMessage);
    
    // Test should pass if we found the auto-generation message
    expect(!!autoGenMessage).toBe(true);
});