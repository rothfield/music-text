import { test, expect } from '@playwright/test';

test.describe('Visual verification of cleaned interface', () => {
    test('should show clean interface with copy buttons and no verbose descriptions', async ({ page }) => {
        await page.goto('http://localhost:8000');
        await page.waitForLoadState('networkidle');
        
        // Input notation to populate outputs
        await page.locator('#input').fill('1-2-3');
        await page.waitForTimeout(1500);
        
        // Take screenshot of the interface
        await page.screenshot({ 
            path: 'test-results/clean-interface-verification.png',
            fullPage: true 
        });
        
        // Verify key elements are present and clean
        await expect(page.locator('.copy-btn')).toHaveCount(8); // Should have multiple copy buttons
        await expect(page.locator('.output-description')).toHaveCount(0); // No verbose descriptions
        await expect(page.locator('.output-title')).toHaveCount(8); // Clean titles only
        
        // Verify no verbose text exists
        await expect(page.locator('text=Human-readable YAML format')).not.toBeVisible();
        await expect(page.locator('text=with proper indentation')).not.toBeVisible();
        await expect(page.locator('text=Pure pest parse tree')).not.toBeVisible();
        await expect(page.locator('text=before spatial analysis')).not.toBeVisible();
    });
});