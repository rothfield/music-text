import { test, expect } from '@playwright/test';

test.describe('Empty line edge cases', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto('http://localhost:8000');
        await page.waitForLoadState('networkidle');
    });

    test('should handle single newline at start', async ({ page }) => {
        await page.locator('#input').fill('\n1-2-3');
        await page.waitForTimeout(1000);
        
        const output = await page.locator('#ast-output').textContent();
        expect(output).not.toContain('Error');
        expect(output).toContain('staves');
    });

    test('should handle only empty line', async ({ page }) => {
        await page.locator('#input').fill('\n');
        await page.waitForTimeout(1000);
        
        const output = await page.locator('#ast-output').textContent();
        expect(output).not.toContain('Error');
        // Should parse to empty staves
        expect(output).toContain('staves');
    });

    test('should handle only whitespace', async ({ page }) => {
        await page.locator('#input').fill('   ');
        await page.waitForTimeout(1000);
        
        const output = await page.locator('#ast-output').textContent();
        expect(output).not.toContain('Error');
    });

    test('should handle mixed whitespace and newlines', async ({ page }) => {
        await page.locator('#input').fill('  \n  \n  1-2-3');
        await page.waitForTimeout(1000);
        
        const output = await page.locator('#ast-output').textContent();
        expect(output).not.toContain('Error');
        expect(output).toContain('staves');
    });

    test('should handle carriage return + newline', async ({ page }) => {
        await page.locator('#input').fill('\r\n1-2-3');
        await page.waitForTimeout(1000);
        
        const output = await page.locator('#ast-output').textContent();
        expect(output).not.toContain('Error');
        expect(output).toContain('staves');
    });

    test('should handle tabs and spaces', async ({ page }) => {
        await page.locator('#input').fill('\t  \n1-2-3');
        await page.waitForTimeout(1000);
        
        const output = await page.locator('#ast-output').textContent();
        expect(output).not.toContain('Error');
        expect(output).toContain('staves');
    });
});