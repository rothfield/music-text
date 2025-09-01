const { test, expect } = require('@playwright/test');

test('test tala rendering with debug output', async ({ page }) => {
    const logs = [];
    
    page.on('console', msg => {
        const text = msg.text();
        logs.push(text);
        if (text.includes('tala') || text.includes('Tala')) {
            console.log('TALA LOG:', text);
        }
    });
    
    await page.goto('http://localhost:3000');
    
    // Simple test with tala 0
    const testInput = `      0
C D E |`;
    
    console.log('Testing tala 0 rendering...');
    await page.fill('#notation-input', testInput);
    await page.waitForTimeout(2000);
    
    // Check if tala drawing happened
    const talaDrawn = logs.some(log => log.includes('Tala') && log.includes('drawn'));
    console.log('Tala drawn:', talaDrawn);
    
    // Check console for tala-related logs
    const talaLogs = logs.filter(log => log.toLowerCase().includes('tala'));
    console.log('All tala-related logs:');
    talaLogs.forEach(log => console.log('  -', log));
    
    // Take screenshot
    await page.screenshot({ path: 'test-results/tala-render-test.png' });
    console.log('Screenshot saved to test-results/tala-render-test.png');
});