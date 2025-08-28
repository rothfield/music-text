// @ts-check
const { test, expect } = require('@playwright/test');

test.describe('Visual Layout Debug', () => {

  test('capture layout screenshots for analysis', async ({ page }) => {
    // Set a consistent large viewport
    await page.setViewportSize({ width: 1366, height: 768 });
    
    // Screenshot debug page
    await page.goto('/debug-layout.html');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000); // Let measurements update
    
    // Take full page screenshot of debug page
    await page.screenshot({ 
      path: 'debug-layout-full.png', 
      fullPage: true 
    });
    
    // Get measurements from debug page
    const measurements = await page.locator('#measurements').textContent();
    console.log('Debug page measurements:', measurements);
    
    // Screenshot main page  
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000); // Wait for WASM load
    
    // Take full page screenshot of main page
    await page.screenshot({ 
      path: 'main-page-full.png', 
      fullPage: true 
    });
    
    // Focus on just the input area
    const inputSection = page.locator('.input-section');
    await inputSection.screenshot({ path: 'input-section-only.png' });
    
    // Get actual textarea measurements from main page
    const textarea = page.locator('#notation-input');
    const textareaBox = await textarea.boundingBox();
    const sectionBox = await inputSection.boundingBox();
    
    console.log('Main page - Section width:', sectionBox?.width);
    console.log('Main page - Textarea width:', textareaBox?.width);
    console.log('Main page - Width difference:', (sectionBox?.width || 0) - (textareaBox?.width || 0));
    
    // Check computed styles
    const computedStyles = await textarea.evaluate((el) => {
      const styles = window.getComputedStyle(el);
      return {
        width: styles.width,
        boxSizing: styles.boxSizing,
        padding: styles.padding,
        border: styles.border,
        margin: styles.margin,
        maxWidth: styles.maxWidth,
        minWidth: styles.minWidth
      };
    });
    
    console.log('Computed styles:', computedStyles);
  });

  test('measure exact pixel differences', async ({ page }) => {
    await page.setViewportSize({ width: 1366, height: 768 });
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Get pixel measurements
    const measurements = await page.evaluate(() => {
      const body = document.body;
      const inputSection = document.querySelector('.input-section');
      const textarea = document.querySelector('#notation-input');
      
      return {
        bodyWidth: body.offsetWidth,
        bodyClientWidth: body.clientWidth,
        sectionWidth: inputSection?.offsetWidth,
        sectionClientWidth: inputSection?.clientWidth,
        textareaWidth: textarea?.offsetWidth,
        textareaClientWidth: textarea?.clientWidth,
        viewportWidth: window.innerWidth,
        scrollWidth: document.documentElement.scrollWidth
      };
    });
    
    console.log('Detailed measurements:', JSON.stringify(measurements, null, 2));
    
    // Check if there's unused space
    const unusedSpace = (measurements.sectionClientWidth || 0) - (measurements.textareaWidth || 0);
    console.log(`Unused space in section: ${unusedSpace}px`);
    
    if (unusedSpace > 20) {
      console.log('⚠️  Significant unused space detected!');
    }
  });
});