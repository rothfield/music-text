// @ts-check
const { test, expect } = require('@playwright/test');

test('take full page screenshot', async ({ page }) => {
  await page.setViewportSize({ width: 1366, height: 768 });
  await page.goto('/');
  await page.waitForLoadState('networkidle');
  
  // Wait a moment for everything to load
  await page.waitForTimeout(2000);
  
  // Take a full page screenshot
  await page.screenshot({ 
    path: 'current-layout.png', 
    fullPage: true 
  });
  
  // Also take a focused screenshot of just the input area
  const inputSection = page.locator('.input-section');
  await inputSection.screenshot({ 
    path: 'input-section-current.png' 
  });
  
  // Get viewport width vs textarea width ratio
  const measurements = await page.evaluate(() => {
    const textarea = document.querySelector('#notation-input');
    const container = document.querySelector('.container');
    
    return {
      viewportWidth: window.innerWidth,
      textareaWidth: textarea ? textarea.offsetWidth : 0,
      containerWidth: container ? container.offsetWidth : 0,
      textareaWidthPercent: textarea ? (textarea.offsetWidth / window.innerWidth * 100).toFixed(1) : 0
    };
  });
  
  console.log('Visual measurements:', measurements);
  console.log(`Textarea uses ${measurements.textareaWidthPercent}% of viewport width`);
  
  // A wide textarea should use at least 80% of viewport width on desktop
  if (parseFloat(measurements.textareaWidthPercent) < 80) {
    console.log('❌ Textarea appears narrow - using less than 80% of viewport width');
  } else {
    console.log('✅ Textarea appears wide - using more than 80% of viewport width');
  }
});