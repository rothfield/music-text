const { test, expect } = require('@playwright/test');

test('Slur button functionality test', async ({ page }) => {
  await page.goto('http://localhost:3000/test_slur_button.html');
  
  // Wait for WASM initialization
  await page.waitForFunction(() => 
    document.getElementById('results').textContent.includes('WASM initialized')
  );
  
  // Test 1: Select text and toggle slur
  const textarea = page.locator('#test-input');
  
  // Clear and set test content
  await textarea.fill('S R G M P D N');
  
  // Select "R G M" (positions 2-8)
  await textarea.focus();
  await page.evaluate(() => {
    const textarea = document.getElementById('test-input');
    textarea.setSelectionRange(2, 8);
  });
  
  // Click toggle slur button
  await page.click('button:has-text("Toggle Slur")');
  
  // Check results
  const results = page.locator('#results');
  const resultsText = await results.textContent();
  
  // Verify the operation worked
  expect(resultsText).toContain('Text was modified');
  
  // Verify textarea content changed
  const newContent = await textarea.inputValue();
  expect(newContent).not.toBe('S R G M P D N');
  expect(newContent).toContain('_'); // Should contain slur characters
  
  // Test 2: Run automated tests
  await page.click('button:has-text("Run All Tests")');
  
  // Wait for tests to complete
  await page.waitForFunction(() => 
    document.getElementById('results').textContent.includes('All tests completed')
  );
  
  const finalResults = await results.textContent();
  expect(finalResults).toContain('All tests completed');
  
  // Verify at least some tests showed modifications
  expect(finalResults).toContain('Text was modified');
  
  console.log('Test results preview:');
  console.log(finalResults.substring(0, 500) + '...');
});