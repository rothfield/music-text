import { test, expect } from '@playwright/test';

test.describe('Copy buttons functionality', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto('http://localhost:8000');
        await page.waitForLoadState('networkidle');
    });

    test('should have copy buttons for all output sections', async ({ page }) => {
        // Input some notation to generate outputs
        await page.locator('#input').fill('1-2-3');
        await page.waitForTimeout(1000);
        
        // Check for copy buttons
        const copyButtons = await page.locator('.copy-btn').count();
        expect(copyButtons).toBeGreaterThan(0);
        
        // Verify specific copy buttons exist
        await expect(page.locator('button:has-text("📋 Copy")').first()).toBeVisible();
    });

    test('should show copy buttons exist and are clickable', async ({ page }) => {
        // Input some notation to generate outputs
        await page.locator('#input').fill('1-2-3');
        await page.waitForTimeout(1000);
        
        // Check that copy button is visible and clickable
        const copyBtn = page.locator('.copy-btn').first();
        await expect(copyBtn).toBeVisible();
        await expect(copyBtn).toContainText('📋 Copy');
        
        // Verify button is clickable (we skip clipboard test due to permissions)
        await expect(copyBtn).toBeEnabled();
    });

    test('should not have verbose descriptions anymore', async ({ page }) => {
        // Check that verbose descriptions are removed
        const descriptions = await page.locator('.output-description').count();
        expect(descriptions).toBe(0);
        
        // Verify that specific verbose text is gone
        await expect(page.locator('text=Human-readable YAML format with proper indentation')).not.toBeVisible();
        await expect(page.locator('text=Pure pest parse tree converted to AST')).not.toBeVisible();
    });

    test('should have clean output headers without verbose descriptions', async ({ page }) => {
        // Check that output titles exist in the DOM
        await expect(page.locator('.output-title:has-text("Parser Output (YAML)")')).toHaveCount(1);
        await expect(page.locator('.output-title:has-text("Raw AST (Before Spatial Processing)")')).toHaveCount(1);
        await expect(page.locator('.output-title:has-text("Raw JSON Response")')).toHaveCount(1);
        
        // Verify clean titles without verbose descriptions
        const titles = page.locator('.output-title');
        const count = await titles.count();
        expect(count).toBeGreaterThan(0);
    });
});