import { test, expect } from '@playwright/test';

test.describe('Actual empty line bug investigation', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto('http://localhost:8000');
        await page.waitForLoadState('networkidle');
    });

    test('investigate what exactly is still broken', async ({ page }) => {
        // Test the exact scenario that's still broken
        await page.locator('#input').click();
        await page.locator('#input').fill('\n123');
        
        // Wait for processing
        await page.waitForTimeout(2000);
        
        // Check what we get
        const output = await page.locator('#ast-output').textContent();
        console.log('Output for newline + 123:', output);
        
        // Take screenshot to see what's happening
        await page.screenshot({ 
            path: 'test-results/empty-line-bug-investigation.png',
            fullPage: true 
        });
        
        // Check if there's an error
        if (output.includes('Error') || output.includes('expected document')) {
            console.log('ERROR FOUND:', output);
            throw new Error('Still has empty line parsing error: ' + output.substring(0, 200));
        }
    });

    test('test different ways the empty line bug might manifest', async ({ page }) => {
        const testCases = [
            '\n123',           // newline + content
            '\n\n123',         // double newline + content  
            '   \n123',        // spaces + newline + content
            '\t\n123',         // tab + newline + content
            '\r\n123',         // carriage return + newline + content
        ];
        
        for (const testCase of testCases) {
            console.log('Testing case:', JSON.stringify(testCase));
            
            await page.locator('#input').click();
            await page.locator('#input').fill(''); // Clear first
            await page.locator('#input').fill(testCase);
            
            await page.waitForTimeout(1500);
            
            const output = await page.locator('#ast-output').textContent();
            
            if (output.includes('Error') || output.includes('expected document')) {
                console.log(`BROKEN CASE: ${JSON.stringify(testCase)}`);
                console.log(`OUTPUT: ${output.substring(0, 300)}`);
                throw new Error(`Case ${JSON.stringify(testCase)} still broken: ${output.substring(0, 200)}`);
            } else {
                console.log(`OK: ${JSON.stringify(testCase)}`);
            }
        }
    });
});