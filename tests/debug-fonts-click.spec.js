const { test, expect } = require('@playwright/test');

test('debug fonts button click mechanics', async ({ page }) => {
    await page.goto('http://localhost:3000');
    await page.waitForSelector('#input-text');
    await page.waitForTimeout(1000);
    
    // Check CSS properties that might block clicks
    const buttonStyles = await page.evaluate(() => {
        const button = document.getElementById('fonts-button');
        const styles = getComputedStyle(button);
        return {
            pointerEvents: styles.pointerEvents,
            position: styles.position,
            zIndex: styles.zIndex,
            display: styles.display,
            visibility: styles.visibility,
            opacity: styles.opacity
        };
    });
    
    console.log('Button styles:', buttonStyles);
    
    // Check if button is actually clickable
    const isClickable = await page.locator('#fonts-button').isEnabled();
    const isVisible = await page.locator('#fonts-button').isVisible();
    
    console.log('Button state:', { isClickable, isVisible });
    
    // Try different clicking methods
    console.log('Attempting different click methods...');
    
    // Method 1: Regular click
    try {
        await page.click('#fonts-button', { timeout: 1000 });
        console.log('✅ Regular click succeeded');
    } catch (e) {
        console.log('❌ Regular click failed:', e.message);
    }
    
    // Method 2: Force click
    try {
        await page.click('#fonts-button', { force: true, timeout: 1000 });
        console.log('✅ Force click succeeded');
    } catch (e) {
        console.log('❌ Force click failed:', e.message);
    }
    
    // Method 3: JavaScript click
    try {
        await page.evaluate(() => {
            document.getElementById('fonts-button').click();
        });
        console.log('✅ JavaScript click succeeded');
    } catch (e) {
        console.log('❌ JavaScript click failed:', e.message);
    }
    
    // Check if any event listeners are actually attached
    const hasEventListeners = await page.evaluate(() => {
        const button = document.getElementById('fonts-button');
        // Try to detect if listeners are attached (this is somewhat limited)
        return {
            onclick: !!button.onclick,
            hasAttributes: button.hasAttributes(),
            getAttribute: button.getAttribute('onclick')
        };
    });
    
    console.log('Event listener detection:', hasEventListeners);
    
    expect(isClickable).toBe(true);
});