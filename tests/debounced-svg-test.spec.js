const { test, expect } = require('@playwright/test');

test.describe('Debounced SVG Generation', () => {
    test('should only generate SVG when LilyPond PNG tab is selected with debouncing', async ({ page }) => {
        // Navigate to the app
        await page.goto('http://localhost:3000');
        
        // Wait for page to load
        await page.waitForSelector('#input');
        
        // Enter test notation
        await page.fill('#input', '1-2-3');
        
        // Wait for auto-parse to complete
        await page.waitForTimeout(1000);
        
        // Verify we're on AST tab initially
        await expect(page.locator('.tab.active[data-tab="ast"]')).toBeVisible();
        
        // Click on LilyPond PNG tab
        await page.click('.tab[data-tab="lilypond-png"]');
        
        // Verify the tab is now active
        await expect(page.locator('.tab.active[data-tab="lilypond-png"]')).toBeVisible();
        
        // Check that debounced message appears
        await expect(page.locator('#lilypond-png-output')).toContainText('⏳ Generating SVG in 5 seconds...');
        
        // Wait for the 5-second debounce + generation time
        await page.waitForTimeout(7000);
        
        // Check that SVG was generated (should show an image or success message)
        const svgOutput = page.locator('#lilypond-png-output');
        await expect(svgOutput.locator('img, .success, .warning')).toBeVisible();
        
        // Test tab switching doesn't regenerate immediately
        await page.click('.tab[data-tab="ast"]');
        await page.click('.tab[data-tab="lilypond-png"]');
        
        // Should use cached result (no "Generating..." message)
        await expect(page.locator('#lilypond-png-output')).not.toContainText('⏳ Generating SVG in 5 seconds...');
        
        console.log('✅ Debounced SVG generation test completed successfully');
    });
    
    test('should regenerate SVG when input changes', async ({ page }) => {
        // Navigate to the app
        await page.goto('http://localhost:3000');
        
        // Enter initial notation
        await page.fill('#input', '1-2');
        
        // Go to LilyPond PNG tab
        await page.click('.tab[data-tab="lilypond-png"]');
        
        // Wait for initial generation
        await page.waitForTimeout(7000);
        
        // Change input
        await page.fill('#input', '3-4-5');
        
        // Go back to LilyPond PNG tab
        await page.click('.tab[data-tab="ast"]');
        await page.click('.tab[data-tab="lilypond-png"]');
        
        // Should show debounce message since input changed
        await expect(page.locator('#lilypond-png-output')).toContainText('⏳ Generating SVG in 5 seconds...');
        
        console.log('✅ SVG regeneration on input change test completed successfully');
    });
});