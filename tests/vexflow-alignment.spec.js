// @ts-check
const { test, expect } = require('@playwright/test');

test('test VexFlow alignment with actual content', async ({ page }) => {
  await page.setViewportSize({ width: 1366, height: 768 });
  await page.goto('/');
  await page.waitForLoadState('networkidle');
  
  // Wait for page to fully load
  await page.waitForTimeout(2000);
  
  // Enter notation to trigger VexFlow rendering
  const textarea = page.locator('#notation-input');
  await textarea.fill('1-2-3');
  
  // Wait for VexFlow to render
  await page.waitForTimeout(1000);
  
  // Wait for VexFlow SVG to appear
  await expect(page.locator('#live-vexflow-notation svg')).toBeVisible({ timeout: 5000 });
  
  // Take screenshot with actual VexFlow content
  await page.screenshot({ 
    path: 'vexflow-alignment.png', 
    fullPage: true 
  });
  
  // Get alignment measurements
  const measurements = await page.evaluate(() => {
    const textarea = document.querySelector('#notation-input');
    const vexflowContainer = document.querySelector('#live-vexflow-container');
    const vexflowSvg = document.querySelector('#live-vexflow-notation svg');
    
    const textareaRect = textarea.getBoundingClientRect();
    const containerRect = vexflowContainer.getBoundingClientRect();
    const svgRect = vexflowSvg ? vexflowSvg.getBoundingClientRect() : null;
    
    return {
      textareaLeft: textareaRect.left,
      textareaWidth: textareaRect.width,
      containerLeft: containerRect.left,
      containerWidth: containerRect.width,
      svgLeft: svgRect ? svgRect.left : null,
      svgWidth: svgRect ? svgRect.width : null,
      leftAlignmentDiff: Math.abs(textareaRect.left - containerRect.left),
      containerExpandsRight: containerRect.width > textareaRect.width
    };
  });
  
  console.log('VexFlow alignment measurements:', measurements);
  
  // Check left alignment (should be very close, within 5px)
  if (measurements.leftAlignmentDiff <= 5) {
    console.log('✅ VexFlow container left-aligned with textarea');
  } else {
    console.log(`❌ VexFlow container not aligned - ${measurements.leftAlignmentDiff}px difference`);
  }
  
  // Check if container can expand
  if (measurements.containerExpandsRight) {
    console.log('✅ VexFlow container can expand beyond textarea width');
  } else {
    console.log('❌ VexFlow container locked to textarea width');
  }
});