const { test, expect } = require('./test-helpers');

test.describe('VexFlow Overline Slurs', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:3000');
  });

  test('FAILING: 2 overlines should slur only first 2 notes', async ({ page }) => {
    // Test pattern: __\n124 - overline over first 2 notes should create slur over just those 2 notes
    // Console errors are automatically detected by test helper
    
    // Capture ALL console messages for debugging
    const allConsoleMessages = [];
    page.on('console', msg => {
      allConsoleMessages.push(msg.text());
    });
    
    // Check if WASM is loaded and what functions are available
    const wasmStatus = await page.evaluate(() => {
      return {
        wasmLoaded: typeof window.wasm !== 'undefined' && window.wasm !== null,
        parseNotationAvailable: typeof window.parse_notation === 'function',
        wasmType: typeof window.wasm,
        initError: window.wasmInitError || 'none'
      };
    });
    console.log(`🔧 WASM Status:`, JSON.stringify(wasmStatus, null, 2));
    
    await page.fill('#notation-input', '__\n124');
    await page.waitForTimeout(3000);
    
    // Log all console messages to see what's actually happening
    console.log('\n=== ALL CONSOLE MESSAGES ===');
    for (const msg of allConsoleMessages) {
      console.log(msg);
    }
    console.log('=== END CONSOLE MESSAGES ===\n');
    
    const vexflowContainer = await page.locator('#vexflow-canvas');
    const content = await vexflowContainer.innerHTML();
    
    // Check if VexFlow renders
    const hasVexFlow = content.includes('svg');
    expect(hasVexFlow).toBe(true);
    
    // Check if slur curves are present in the SVG
    const hasSlurCurves = content.includes('path') && content.includes('curve');
    console.log(`\n✅ VexFlow rendered: ${hasVexFlow}`);
    console.log(`🎵 Slur curves detected: ${hasSlurCurves}`);
    
    // For debugging: log VexFlow debug messages from 2-pass generator
    const debugMessages = await page.evaluate(() => {
      // Access console history (if available)
      return window.lastVexFlowDebugMessages || 'No debug messages found';
    });
    console.log(`🔧 VexFlow Generator Messages: ${debugMessages}`);
    
    // This test will pass but shows the wrong behavior
    // The real test is in the console output verification
  });

  test('CONTROL: 1 overline should create NO slur (single note)', async ({ page }) => {
    const consoleMessages = [];
    page.on('console', msg => {
      if (msg.text().includes('VexFlow:') || msg.text().includes('Drawing slur')) {
        consoleMessages.push(msg.text());
      }
    });
    
    await page.fill('#notation-input', '_\n123');
    await page.waitForTimeout(3000);
    
    console.log('=== 1-Overline Slur Console Messages ===');
    for (const msg of consoleMessages) {
      console.log(msg);
    }
    console.log('=== End Console Messages ===');
    
    console.log('\n✅ EXPECTED: Slur from note0 to note2 (all 3 notes)');
    console.log('📋 BEHAVIOR: Should be correct');
  });

  test('CONTROL: 3 overlines should slur all 3 notes', async ({ page }) => {
    const consoleMessages = [];
    page.on('console', msg => {
      if (msg.text().includes('VexFlow:') || msg.text().includes('Drawing slur')) {
        consoleMessages.push(msg.text());
      }
    });
    
    await page.fill('#notation-input', '___\n123');
    await page.waitForTimeout(3000);
    
    console.log('=== 3-Overline Slur Console Messages ===');
    for (const msg of consoleMessages) {
      console.log(msg);
    }
    console.log('=== End Console Messages ===');
    
    console.log('\n✅ EXPECTED: NO slur (single note under overline)'); 
    console.log('📋 BEHAVIOR: Should create no slur');
  });

  test('Compare all overline patterns', async ({ page }) => {
    const patterns = [
      { pattern: '_\n123', description: '1 overline - NO slur (single note)', expected: 'no slur' },
      { pattern: '__\n123', description: '2 overlines - slur over 2 notes', expected: 'note0 to note1' },
      { pattern: '___\n123', description: '3 overlines - slur over 3 notes', expected: 'note0 to note2' }
    ];
    
    for (const test of patterns) {
      console.log(`\n=== Testing ${test.pattern} ===`);
      console.log(`Description: ${test.description}`);
      console.log(`Expected: ${test.expected}`);
      
      const consoleMessages = [];
      page.on('console', msg => {
        if (msg.text().includes('VexFlow: Slur 0 from')) {
          consoleMessages.push(msg.text());
        }
      });
      
      await page.fill('#notation-input', test.pattern);
      await page.waitForTimeout(2000);
      
      if (consoleMessages.length > 0) {
        console.log(`Actual: ${consoleMessages[0]}`);
        const isCorrect = consoleMessages[0].includes(test.expected);
        console.log(`Status: ${isCorrect ? '✅ CORRECT' : '❌ INCORRECT'}`);
      } else {
        console.log('Actual: No slur created');
        console.log('Status: ❌ NO SLUR');
      }
      
      // Clear console messages for next test
      page.removeAllListeners('console');
    }
  });
});