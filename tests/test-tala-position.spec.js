const { test, expect } = require('@playwright/test');

test('test tala positioning above barline', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Test different tala positions
    const tests = [
        { input: `      0\nC D E |`, name: 'tala-0-end' },
        { input: `0\n| C D E`, name: 'tala-0-start' },
        { input: `      2       5\nC D E | F G A |`, name: 'tala-2-5-multiple' },
    ];
    
    for (const test of tests) {
        console.log(`Testing: ${test.name}`);
        await page.fill('#input-text', test.input);
        await page.waitForTimeout(2000);
        
        // Take screenshot
        await page.screenshot({ 
            path: `test-results/${test.name}.png`,
            fullPage: false,
            clip: { x: 0, y: 100, width: 800, height: 300 }
        });
        console.log(`Screenshot saved: test-results/${test.name}.png`);
    }
});