const { test, expect } = require('@playwright/test');

test('V2 WASM parser integration test', async ({ page }) => {
  await page.goto('http://localhost:3000');
  
  // Test the new V2 WASM function directly
  const v2TestInput = `key: C
time: 4/4

| S R G M |
  do re mi fa`;

  const result = await page.evaluate(async (input) => {
    // Wait for WASM to load
    await new Promise(resolve => {
      if (window.wasm && window.wasm.parse_notation_v2) {
        resolve();
      } else {
        const checkWasm = () => {
          if (window.wasm && window.wasm.parse_notation_v2) {
            resolve();
          } else {
            setTimeout(checkWasm, 100);
          }
        };
        checkWasm();
      }
    });
    
    // Call V2 parser function
    const result = window.wasm.parse_notation_v2(input);
    return {
      success: result.success,
      error: result.error_message,
      hasVexflowJs: result.vexflow_js && result.vexflow_js.length > 0,
      hasDocument: result.document && result.document.length > 0
    };
  }, v2TestInput);
  
  // Verify V2 parser worked
  expect(result.success).toBe(true);
  expect(result.error).toBeNull();
  expect(result.hasVexflowJs).toBe(true);
  expect(result.hasDocument).toBe(true);
  
  console.log('✅ V2 WASM parser integration successful!');
});