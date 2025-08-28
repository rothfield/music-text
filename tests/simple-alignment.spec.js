// @ts-check
const { test, expect } = require('@playwright/test');

test('check VexFlow container alignment', async ({ page }) => {
  await page.setViewportSize({ width: 1366, height: 768 });
  await page.goto('/');
  await page.waitForLoadState('networkidle');
  
  // Just check the layout alignment without waiting for WASM
  const measurements = await page.evaluate(() => {
    const textarea = document.querySelector('#notation-input');
    const vexflowContainer = document.querySelector('#live-vexflow-container');
    
    const textareaRect = textarea.getBoundingClientRect();
    const containerRect = vexflowContainer.getBoundingClientRect();
    
    return {
      textareaLeft: Math.round(textareaRect.left),
      textareaRight: Math.round(textareaRect.right), 
      textareaWidth: Math.round(textareaRect.width),
      containerLeft: Math.round(containerRect.left),
      containerRight: Math.round(containerRect.right),
      containerWidth: Math.round(containerRect.width),
      leftAlignmentDiff: Math.abs(Math.round(textareaRect.left - containerRect.left)),
      containerExpandsRight: containerRect.width >= textareaRect.width
    };
  });
  
  console.log('Layout alignment measurements:');
  console.log(`Textarea: left=${measurements.textareaLeft}, right=${measurements.textareaRight}, width=${measurements.textareaWidth}`);
  console.log(`VexFlow: left=${measurements.containerLeft}, right=${measurements.containerRight}, width=${measurements.containerWidth}`);
  console.log(`Left alignment difference: ${measurements.leftAlignmentDiff}px`);
  
  // Take screenshot
  await page.screenshot({ 
    path: 'vexflow-layout-check.png', 
    fullPage: true 
  });
  
  // Check alignment
  if (measurements.leftAlignmentDiff <= 5) {
    console.log('✅ VexFlow container left-aligned with textarea');
  } else {
    console.log(`❌ VexFlow container not aligned - ${measurements.leftAlignmentDiff}px difference`);
  }
  
  if (measurements.containerExpandsRight) {
    console.log('✅ VexFlow container can expand to match or exceed textarea width');
  } else {
    console.log('❌ VexFlow container is narrower than textarea');
  }
});