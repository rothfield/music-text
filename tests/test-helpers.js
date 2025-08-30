const { test: baseTest, expect } = require('@playwright/test');

/**
 * Extended test that automatically fails on console errors
 * Every test using this will auto-fail if console errors occur
 */
const test = baseTest.extend({
  page: async ({ page }, use, testInfo) => {
    const consoleErrors = [];
    
    // Capture console errors
    page.on('console', msg => {
      if (msg.type() === 'error') {
        consoleErrors.push(msg.text());
      }
    });
    
    // Capture JavaScript errors/exceptions  
    page.on('pageerror', error => {
      consoleErrors.push(`JavaScript Error: ${error.message}`);
    });
    
    // Use the page for the test
    await use(page);
    
    // After test completes, check for errors
    if (consoleErrors.length > 0) {
      console.log('\n=== CONSOLE ERRORS DETECTED - TEST AUTO-FAILED ===');
      for (const error of consoleErrors) {
        console.log(`ERROR: ${error}`);
      }
      console.log('=== END CONSOLE ERRORS ===\n');
      
      // Attach errors to test report
      await testInfo.attach('console-errors', {
        contentType: 'text/plain',
        body: consoleErrors.join('\n')
      });
      
      throw new Error(`Test failed due to ${consoleErrors.length} console error(s): ${consoleErrors.join('; ')}`);
    }
  },
});

module.exports = { test, expect };