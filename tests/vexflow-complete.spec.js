const { test, expect } = require('@playwright/test');

test('VexFlow comprehensive notation test', async ({ page }) => {
    const messages = [];
    page.on('console', msg => messages.push(msg.text()));
    
    await page.goto('http://localhost:3000');
    await page.waitForSelector('#notation-input');
    
    // Test regular note
    await page.fill('#notation-input', '1');
    await page.waitForTimeout(1000);
    
    let sharpMessages = messages.filter(m => m.includes('Note keys:'));
    console.log('Test 1 - Regular note "1":');
    sharpMessages.forEach(msg => console.log('  ', msg));
    
    // Test sharp note
    messages.length = 0; // Clear messages
    await page.fill('#notation-input', '1#');
    await page.waitForTimeout(1000);
    
    sharpMessages = messages.filter(m => m.includes('Note keys:'));
    console.log('\nTest 2 - Sharp note "1#":');
    sharpMessages.forEach(msg => console.log('  ', msg));
    
    // Test flat note
    messages.length = 0;
    await page.fill('#notation-input', '3b');
    await page.waitForTimeout(1000);
    
    sharpMessages = messages.filter(m => m.includes('Note keys:'));
    console.log('\nTest 3 - Flat note "3b":');
    sharpMessages.forEach(msg => console.log('  ', msg));
    
    // Test sequence
    messages.length = 0;
    await page.fill('#notation-input', '1 2# 3 4b');
    await page.waitForTimeout(1000);
    
    sharpMessages = messages.filter(m => m.includes('Note keys:'));
    console.log('\nTest 4 - Mixed sequence "1 2# 3 4b":');
    sharpMessages.forEach(msg => console.log('  ', msg));
    
    expect(messages.length).toBeGreaterThan(0);
});