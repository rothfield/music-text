// @ts-check
const { test, expect } = require('@playwright/test');

test('test modern UI functionality and mobile responsiveness', async ({ page }) => {
  // Test desktop view first
  await page.setViewportSize({ width: 1366, height: 768 });
  await page.goto('/');
  await page.waitForLoadState('networkidle');
  
  // Check that modern UI elements are present
  await expect(page.locator('.hero')).toBeVisible();
  await expect(page.locator('.card')).toBeVisible();
  await expect(page.locator('#notation-input')).toBeVisible();
  await expect(page.locator('#vexflow-canvas')).toBeVisible();
  
  // Test VexFlow rendering
  const textarea = page.locator('#notation-input');
  await textarea.clear();
  await textarea.fill('1-2-3');
  
  // Wait for WASM to initialize and render
  await page.waitForTimeout(2000);
  
  // Check if VexFlow canvas has content
  const canvas = page.locator('#vexflow-canvas');
  await expect(canvas).toHaveClass(/has-content/);
  
  // Take desktop screenshot
  await page.screenshot({ 
    path: 'modern-ui-desktop.png', 
    fullPage: true 
  });
  
  // Test mobile view
  await page.setViewportSize({ width: 375, height: 667 });
  await page.waitForTimeout(500);
  
  // Check mobile responsiveness
  await expect(page.locator('.hero')).toBeVisible();
  await expect(page.locator('.card')).toBeVisible();
  await expect(textarea).toBeVisible();
  
  // Take mobile screenshot
  await page.screenshot({ 
    path: 'modern-ui-mobile.png', 
    fullPage: true 
  });
  
  // Test tablet view
  await page.setViewportSize({ width: 768, height: 1024 });
  await page.waitForTimeout(500);
  
  await page.screenshot({ 
    path: 'modern-ui-tablet.png', 
    fullPage: true 
  });
  
  console.log('✅ Modern UI tests completed successfully');
});