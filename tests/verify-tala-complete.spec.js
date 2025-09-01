const { test, expect } = require('@playwright/test');

test('verify complete tala implementation', async ({ page }) => {
    const logs = [];
    
    page.on('console', msg => {
        logs.push(msg.text());
    });
    
    await page.goto('http://localhost:3000');
    
    // Test with tala marker
    const testInput = `  0
C | D`;
    
    console.log('Testing input:', testInput.replace('\n', '\\n'));
    await page.fill('#notation-input', testInput);
    await page.waitForTimeout(2000);
    
    // Check for tala-related logs
    const barnoteLogs = logs.filter(log => log.includes('BarNote'));
    const talaLogs = logs.filter(log => log.includes('tala') && !log.includes('None'));
    const drawingLogs = logs.filter(log => log.includes('Drawing tala'));
    
    console.log('\n=== BarNote Logs ===');
    barnoteLogs.forEach(log => console.log(log));
    
    console.log('\n=== Tala Storage Logs ===');
    talaLogs.forEach(log => console.log(log));
    
    console.log('\n=== Tala Drawing Logs ===');
    drawingLogs.forEach(log => console.log(log));
    
    // Take screenshot to verify visual rendering
    await page.screenshot({ path: 'test-results/tala-complete-test.png' });
    console.log('\nScreenshot saved to test-results/tala-complete-test.png');
    
    // Check if tala was actually set on BarNote
    const hasTalaSet = logs.some(log => log.includes('tala = 0'));
    console.log('\nTala was set on BarNote:', hasTalaSet);
});