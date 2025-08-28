// @ts-check
const { test, expect } = require('@playwright/test');

test('detailed width investigation', async ({ page }) => {
  await page.setViewportSize({ width: 1366, height: 768 });
  await page.goto('/');
  await page.waitForLoadState('networkidle');
  
  // Get detailed element measurements
  const details = await page.evaluate(() => {
    const body = document.body;
    const inputSection = document.querySelector('.input-section');
    const inputWrapper = document.querySelector('.input-wrapper');
    const textarea = document.querySelector('#notation-input');
    
    function getElementDetails(el, name) {
      if (!el) return { name, error: 'Element not found' };
      
      const computed = window.getComputedStyle(el);
      const rect = el.getBoundingClientRect();
      
      return {
        name,
        offsetWidth: el.offsetWidth,
        clientWidth: el.clientWidth,
        scrollWidth: el.scrollWidth,
        boundingWidth: rect.width,
        computedWidth: computed.width,
        paddingLeft: computed.paddingLeft,
        paddingRight: computed.paddingRight,
        borderLeft: computed.borderLeftWidth,
        borderRight: computed.borderRightWidth,
        marginLeft: computed.marginLeft,
        marginRight: computed.marginRight,
        boxSizing: computed.boxSizing,
        position: computed.position,
        left: rect.left,
        right: rect.right
      };
    }
    
    return {
      body: getElementDetails(body, 'body'),
      inputSection: getElementDetails(inputSection, '.input-section'),
      inputWrapper: getElementDetails(inputWrapper, '.input-wrapper'),
      textarea: getElementDetails(textarea, 'textarea')
    };
  });
  
  console.log('Detailed element analysis:');
  console.log(JSON.stringify(details, null, 2));
  
  // Calculate where the width is being lost
  const sectionPadding = parseInt(details.inputSection.paddingLeft) + parseInt(details.inputSection.paddingRight);
  const textareaPadding = parseInt(details.textarea.paddingLeft) + parseInt(details.textarea.paddingRight);
  const textareaBorder = parseInt(details.textarea.borderLeft) + parseInt(details.textarea.borderRight);
  const textareaMargin = parseInt(details.textarea.marginLeft) + parseInt(details.textarea.marginRight);
  
  console.log(`Section padding: ${sectionPadding}px`);
  console.log(`Textarea padding: ${textareaPadding}px`);
  console.log(`Textarea border: ${textareaBorder}px`);
  console.log(`Textarea margin: ${textareaMargin}px`);
  console.log(`Total textarea overhead: ${textareaPadding + textareaBorder + textareaMargin}px`);
  
  const expectedTextareaWidth = details.inputSection.offsetWidth - sectionPadding;
  const actualTextareaWidth = details.textarea.offsetWidth;
  const widthLoss = expectedTextareaWidth - actualTextareaWidth;
  
  console.log(`Expected textarea width: ${expectedTextareaWidth}px`);
  console.log(`Actual textarea width: ${actualTextareaWidth}px`);
  console.log(`Width loss: ${widthLoss}px`);
});