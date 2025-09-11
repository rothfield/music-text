const { test, expect } = require('@playwright/test');

test('debug header bar functionality', async ({ page }) => {
    // Capture console messages
    const messages = [];
    page.on('console', msg => messages.push(msg.text()));
    
    await page.goto('http://localhost:3000');
    await page.waitForSelector('#input-text');
    
    // Check initial header state
    const initialDetectedText = await page.locator('#detected-systems').textContent();
    console.log('Initial detected systems text:', initialDetectedText);
    
    // Check if header elements exist
    const unicodeToggle = await page.locator('#unicode-toggle').count();
    const fontsButton = await page.locator('#fonts-button').count();
    const detectedSystems = await page.locator('#detected-systems').count();
    
    console.log('Header elements found:');
    console.log('- Unicode toggle:', unicodeToggle);
    console.log('- Fonts button:', fontsButton);
    console.log('- Detected systems span:', detectedSystems);
    
    // Enter some test input
    await page.fill('#input-text', '|1 2 3|');
    await page.waitForTimeout(2000); // Wait for processing
    
    // Check if detected systems updated
    const updatedDetectedText = await page.locator('#detected-systems').textContent();
    console.log('Updated detected systems text:', updatedDetectedText);
    
    // Test Unicode toggle
    console.log('Testing Unicode toggle...');
    const toggleBefore = await page.locator('#unicode-toggle').isChecked();
    await page.click('#unicode-toggle');
    await page.waitForTimeout(500);
    const toggleAfter = await page.locator('#unicode-toggle').isChecked();
    console.log('Unicode toggle before:', toggleBefore, 'after:', toggleAfter);
    
    // Test Fonts button
    console.log('Testing Fonts button...');
    const fontConfigBefore = await page.locator('#font-config').isVisible();
    await page.click('#fonts-button');
    await page.waitForTimeout(500);
    const fontConfigAfter = await page.locator('#font-config').isVisible();
    console.log('Font config visible before:', fontConfigBefore, 'after:', fontConfigAfter);
    
    // Show relevant console messages
    console.log('=== RELEVANT CONSOLE MESSAGES ===');
    messages.forEach((msg, i) => {
        if (msg.includes('notation systems') || msg.includes('Unicode') || msg.includes('Font') || msg.includes('ERROR') || msg.includes('❌')) {
            console.log(`${i}: ${msg}`);
        }
    });
    
    // Test passes if elements exist (basic functionality check)
    expect(unicodeToggle).toBeGreaterThan(0);
    expect(fontsButton).toBeGreaterThan(0);
    expect(detectedSystems).toBeGreaterThan(0);
});