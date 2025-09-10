const { test, expect } = require('@playwright/test');

test('visual test of tala rendering', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Test with tala 0
    const testInput1 = `      0
C D E |`;
    
    console.log('Testing tala 0 above barline...');
    await page.fill('#input-text', testInput1);
    await page.waitForTimeout(2000);
    
    // Take screenshot for visual verification
    await page.screenshot({ path: 'test-results/tala-0-test.png', fullPage: true });
    console.log('Screenshot saved: test-results/tala-0-test.png');
    
    // Test with tala 3
    const testInput2 = `      3
C D E |`;
    
    console.log('Testing tala 3 above barline...');
    await page.fill('#input-text', testInput2);
    await page.waitForTimeout(2000);
    
    await page.screenshot({ path: 'test-results/tala-3-test.png', fullPage: true });
    console.log('Screenshot saved: test-results/tala-3-test.png');
    
    // Test with multiple talas
    const testInput3 = `0   1   2   3
C | D | E | F |`;
    
    console.log('Testing multiple talas...');
    await page.fill('#input-text', testInput3);
    await page.waitForTimeout(2000);
    
    await page.screenshot({ path: 'test-results/tala-multiple-test.png', fullPage: true });
    console.log('Screenshot saved: test-results/tala-multiple-test.png');
});