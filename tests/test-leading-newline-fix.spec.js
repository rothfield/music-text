import { test, expect } from '@playwright/test';

test.describe('Leading newline fix', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto('http://localhost:8000');
        await page.waitForLoadState('networkidle');
    });

    test('should parse "1-2-3" without leading newline error', async ({ page }) => {
        // Clear the input and type "1-2-3"
        await page.locator('#input').click();
        await page.locator('#input').fill('');
        await page.locator('#input').type('1-2-3');
        
        // Wait for processing to complete
        await page.waitForTimeout(1000);
        
        // Check that output does not contain a parsing error
        const output = await page.locator('#ast-output').textContent();
        expect(output).not.toContain('Error');
        expect(output).not.toContain('expected document');
    });

    test('should parse "1 2 3" (with spaces) correctly', async ({ page }) => {
        // Clear the input and type "1 2 3"
        await page.locator('#input').click();
        await page.locator('#input').fill('');
        await page.locator('#input').type('1 2 3');
        
        // Wait for processing to complete
        await page.waitForTimeout(1000);
        
        // Check that output does not contain a parsing error
        const output = await page.locator('#ast-output').textContent();
        expect(output).not.toContain('Error');
        expect(output).not.toContain('expected document');
    });

    test('should handle input with simulated leading newlines', async ({ page }) => {
        // Simulate what might happen with a textarea that has leading newlines
        await page.locator('#input').click();
        await page.locator('#input').fill('\n\n1-2-3');
        
        // Wait for processing to complete
        await page.waitForTimeout(1000);
        
        // Check that output does not contain a parsing error
        const output = await page.locator('#ast-output').textContent();
        expect(output).not.toContain('Error');
        expect(output).not.toContain('expected document');
        
        // Verify that parsing was successful
        expect(output).toContain('staves');
        expect(output).toContain('measures');
    });

    test('should handle input with empty lines between content', async ({ page }) => {
        // Test with empty lines in the middle
        await page.locator('#input').click();
        await page.locator('#input').fill('1-2-3\n\n\n4-5-6');
        
        // Wait for processing to complete
        await page.waitForTimeout(1000);
        
        // Check that output does not contain a parsing error
        const output = await page.locator('#ast-output').textContent();
        expect(output).not.toContain('Error');
        expect(output).not.toContain('expected document');
        
        // Verify that parsing was successful
        expect(output).toContain('staves');
        expect(output).toContain('measures');
    });
});