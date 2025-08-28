const { test, expect } = require('@playwright/test');

test('VexFlow tie rendering for 22-', async ({ page }) => {
    // Navigate to the app
    await page.goto('http://localhost:3000');
    
    // Wait for WASM to load
    await page.waitForTimeout(2000);
    
    // Enter "22 -" notation
    const textarea = page.locator('#notation-input');
    await textarea.fill('22 -');
    
    // Wait for VexFlow rendering
    await page.waitForTimeout(1000);
    
    // Check that VexFlow canvas has content
    const canvas = page.locator('#vexflow-canvas');
    await expect(canvas).toHaveClass(/has-content/);
    
    // Check for SVG content (VexFlow renders as SVG)
    const svg = page.locator('#vexflow-canvas svg');
    await expect(svg).toBeVisible();
    
    // Log the VexFlow output for debugging
    const vexflowOutput = await page.evaluate(() => {
        const input = document.getElementById('notation-input');
        if (window.parse_notation && input.value) {
            const result = window.parse_notation(input.value);
            return result.vexflow_output;
        }
        return null;
    });
    
    console.log('VexFlow JSON output:', vexflowOutput);
    
    // Parse the VexFlow JSON to check for tied notes
    if (vexflowOutput) {
        const staves = JSON.parse(vexflowOutput);
        console.log('Parsed staves:', JSON.stringify(staves, null, 2));
        
        // Look for tied notes in the structure
        let foundTiedNote = false;
        for (const stave of staves) {
            for (const element of stave.notes) {
                if (element.Note && element.Note.tied) {
                    foundTiedNote = true;
                    console.log('Found tied note:', element.Note);
                    break;
                }
            }
        }
        
        // Verify that we found at least one tied note
        expect(foundTiedNote).toBe(true);
    }
    
    // Take a screenshot to visually verify the tie
    await page.screenshot({ 
        path: 'test-results/tie-rendering-22dash.png',
        fullPage: true
    });
});