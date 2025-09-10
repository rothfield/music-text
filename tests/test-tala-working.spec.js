const { test, expect } = require('@playwright/test');

test('verify tala is working correctly', async ({ page }) => {
    const logs = [];
    
    page.on('console', msg => {
        const text = msg.text();
        logs.push(text);
        if (text.includes('BarNote') || text.includes('tala')) {
            console.log('LOG:', text);
        }
    });
    
    await page.goto('http://localhost:3000');
    
    // Test 1: Input WITH tala markers
    console.log('\n=== Test 1: WITH tala markers ===');
    const withTala = `    0       3
C D | E F | G A`;
    
    await page.fill('#input-text', withTala);
    await page.waitForTimeout(2000);
    
    // Clear logs for next test
    logs.length = 0;
    
    // Test 2: Input WITHOUT tala markers  
    console.log('\n=== Test 2: WITHOUT tala markers ===');
    const withoutTala = `C D | E F | G A`;
    
    await page.fill('#input-text', withoutTala);
    await page.waitForTimeout(2000);
    
    // Test 3: Mixed - some barlines with tala, some without
    console.log('\n=== Test 3: MIXED tala markers ===');
    const mixedTala = `    0           5
C D | E F | G A |`;
    
    await page.fill('#input-text', mixedTala);
    await page.waitForTimeout(2000);
});