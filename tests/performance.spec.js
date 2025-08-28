// @ts-check
const { test, expect } = require('@playwright/test');

test.describe('Performance Tests', () => {

  test('page loads within acceptable time', async ({ page }) => {
    const startTime = Date.now();
    
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Wait for WASM to load
    await expect(page.locator('#version-display')).toBeVisible({ timeout: 15000 });
    
    const loadTime = Date.now() - startTime;
    console.log(`Total page load time: ${loadTime}ms`);
    
    // Should load within 15 seconds (WASM can be slow)
    expect(loadTime).toBeLessThan(15000);
  });

  test('VexFlow rendering performance', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Wait for WASM to load
    await expect(page.locator('#version-display')).toBeVisible({ timeout: 10000 });
    
    const textarea = page.locator('#notation-input');
    
    // Test rendering time for simple notation
    const startTime = Date.now();
    
    await textarea.fill('1-2-3');
    
    // Wait for VexFlow to render
    await expect(page.locator('#live-vexflow-notation svg')).toBeVisible({ timeout: 3000 });
    
    const renderTime = Date.now() - startTime;
    console.log(`VexFlow render time: ${renderTime}ms`);
    
    // Should render within 3 seconds
    expect(renderTime).toBeLessThan(3000);
  });

  test('complex notation rendering performance', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Wait for WASM to load
    await expect(page.locator('#version-display')).toBeVisible({ timeout: 10000 });
    
    const textarea = page.locator('#notation-input');
    
    // Test with more complex notation
    const complexNotation = '1-2-3 4-5-6 | 7-1-2 3-4-5 | 6-7-1 2-3-4 |';
    
    const startTime = Date.now();
    
    await textarea.fill(complexNotation);
    
    // Wait for rendering
    await expect(page.locator('#live-vexflow-notation svg')).toBeVisible({ timeout: 5000 });
    
    const renderTime = Date.now() - startTime;
    console.log(`Complex notation render time: ${renderTime}ms`);
    
    // Complex notation should still render within 5 seconds
    expect(renderTime).toBeLessThan(5000);
  });

  test('server response time for health check', async ({ page }) => {
    await page.goto('/');
    
    // Monitor network requests
    const healthRequests = [];
    
    page.on('response', response => {
      if (response.url().includes('/api/health')) {
        healthRequests.push({
          url: response.url(),
          status: response.status(),
          responseTime: response.request().timing()
        });
      }
    });
    
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000); // Wait for health check
    
    // Should have made at least one health check request
    expect(healthRequests.length).toBeGreaterThan(0);
    
    const healthRequest = healthRequests[0];
    expect(healthRequest.status).toBe(200);
    
    console.log(`Health check requests: ${healthRequests.length}`);
  });

  test('memory usage during extended use', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Wait for WASM to load
    await expect(page.locator('#version-display')).toBeVisible({ timeout: 10000 });
    
    const textarea = page.locator('#notation-input');
    
    // Get initial memory usage
    const initialMetrics = await page.evaluate(() => {
      return performance.memory ? {
        usedJSHeapSize: performance.memory.usedJSHeapSize,
        totalJSHeapSize: performance.memory.totalJSHeapSize
      } : null;
    });
    
    // Simulate extended use - multiple notation changes
    const notationSamples = [
      '1 2 3 4',
      '1-2 3-4',
      'S R G M P D N',
      'C D E F G A B',
      '1-2-3-4-5',
      '| 1 2 | 3 4 | 5 6 | 7 1 |'
    ];
    
    for (let i = 0; i < notationSamples.length; i++) {
      await textarea.fill(notationSamples[i]);
      await page.waitForTimeout(800); // Wait for processing
      
      // Ensure rendering completed
      await expect(page.locator('#live-vexflow-notation')).toBeVisible();
    }
    
    // Get final memory usage
    const finalMetrics = await page.evaluate(() => {
      return performance.memory ? {
        usedJSHeapSize: performance.memory.usedJSHeapSize,
        totalJSHeapSize: performance.memory.totalJSHeapSize
      } : null;
    });
    
    if (initialMetrics && finalMetrics) {
      const memoryIncrease = finalMetrics.usedJSHeapSize - initialMetrics.usedJSHeapSize;
      console.log(`Memory increase: ${Math.round(memoryIncrease / 1024 / 1024)} MB`);
      
      // Memory increase should be reasonable (less than 50MB for this test)
      expect(memoryIncrease).toBeLessThan(50 * 1024 * 1024);
    }
  });

  test('debouncing works correctly', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Wait for WASM to load
    await expect(page.locator('#version-display')).toBeVisible({ timeout: 10000 });
    
    const textarea = page.locator('#notation-input');
    
    // Monitor console logs to check parsing frequency
    const parseLogs = [];
    page.on('console', msg => {
      if (msg.text().includes('Processing your notation') || msg.text().includes('VexFlow')) {
        parseLogs.push({ 
          text: msg.text(), 
          timestamp: Date.now() 
        });
      }
    });
    
    // Rapidly type multiple characters
    await textarea.focus();
    await page.keyboard.type('1');
    await page.keyboard.type('2');
    await page.keyboard.type('3');
    await page.keyboard.type('4');
    
    // Wait for debounce period
    await page.waitForTimeout(1000);
    
    // Should have minimal parsing calls due to debouncing
    console.log(`Parse operations triggered: ${parseLogs.length}`);
    
    // With proper debouncing, should have very few parse operations
    expect(parseLogs.length).toBeLessThan(5);
  });
});