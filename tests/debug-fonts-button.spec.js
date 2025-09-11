const { test, expect } = require('@playwright/test');

test('debug fonts button specific issue', async ({ page }) => {
    // Capture console messages
    const messages = [];
    page.on('console', msg => messages.push(msg.text()));
    
    await page.goto('http://localhost:3000');
    await page.waitForSelector('#input-text');
    await page.waitForTimeout(1000); // Wait for app initialization
    
    // Check if font manager initialized
    console.log('=== FONT MANAGER CONSOLE MESSAGES ===');
    messages.forEach((msg, i) => {
        if (msg.includes('Font') || msg.includes('font') || msg.includes('FontManager') || msg.includes('init')) {
            console.log(`${i}: ${msg}`);
        }
    });
    
    // Check element states before clicking
    const fontsButtonExists = await page.locator('#fonts-button').count();
    const fontConfigExists = await page.locator('#font-config').count();
    const fontConfigInitialDisplay = await page.locator('#font-config').evaluate(el => getComputedStyle(el).display);
    
    console.log('Before click:');
    console.log('- Fonts button exists:', fontsButtonExists);
    console.log('- Font config exists:', fontConfigExists);
    console.log('- Font config display style:', fontConfigInitialDisplay);
    
    // Click the fonts button
    console.log('Clicking fonts button...');
    await page.click('#fonts-button');
    await page.waitForTimeout(500);
    
    // Check states after clicking
    const fontConfigAfterDisplay = await page.locator('#font-config').evaluate(el => getComputedStyle(el).display);
    const fontsButtonText = await page.locator('#fonts-button').textContent();
    
    console.log('After click:');
    console.log('- Font config display style:', fontConfigAfterDisplay);
    console.log('- Fonts button text:', fontsButtonText);
    
    // Try clicking again to see if toggle works
    await page.click('#fonts-button');
    await page.waitForTimeout(500);
    
    const fontConfigSecondDisplay = await page.locator('#font-config').evaluate(el => getComputedStyle(el).display);
    const fontsButtonSecondText = await page.locator('#fonts-button').textContent();
    
    console.log('After second click:');
    console.log('- Font config display style:', fontConfigSecondDisplay);
    console.log('- Fonts button text:', fontsButtonSecondText);
    
    // Test passes if elements exist
    expect(fontsButtonExists).toBeGreaterThan(0);
    expect(fontConfigExists).toBeGreaterThan(0);
});