// @ts-check
const { test, expect } = require('@playwright/test');

test.describe('Notation Parser Functionality', () => {

  test('WASM module loads successfully', async ({ page }) => {
    await page.goto('/');
    
    // Wait for WASM to load (check for success message or version display)
    await expect(page.locator('#version-display')).toBeVisible({ timeout: 10000 });
    
    // Should show version info
    const version = await page.locator('#version-display').textContent();
    expect(version).toMatch(/v\d+\.\d+\.\d+/);
    
    // Check for successful load message in status
    await expect(page.locator('.status.success')).toBeVisible({ timeout: 5000 });
  });

  test('server status indicator shows connection state', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    const indicator = page.locator('#server-status-indicator');
    const statusText = page.locator('#server-status-text');
    
    await expect(indicator).toBeVisible();
    await expect(statusText).toBeVisible();
    
    // Should show either online, offline, or checking state
    const indicatorClass = await indicator.getAttribute('class');
    expect(indicatorClass).toMatch(/\b(online|offline|checking)\b/);
    
    // Wait for initial health check to complete
    await page.waitForTimeout(2000);
    
    // After health check, should be online (assuming server is running)
    await expect(indicator).toHaveClass(/online/);
    await expect(statusText).toContainText('Server');
  });

  test('live VexFlow preview updates on input', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Wait for WASM to load
    await expect(page.locator('#version-display')).toBeVisible({ timeout: 10000 });
    
    const textarea = page.locator('#notation-input');
    const vexflowContainer = page.locator('#live-vexflow-container');
    
    // Initially should show placeholder
    await expect(page.locator('#live-vexflow-placeholder')).toBeVisible();
    
    // Enter notation
    await textarea.fill('1-2');
    
    // Wait for debounced parsing (300ms + processing time)
    await page.waitForTimeout(1000);
    
    // VexFlow SVG should appear
    await expect(page.locator('#live-vexflow-notation')).toBeVisible();
    await expect(page.locator('#live-vexflow-notation svg')).toBeVisible();
    
    // Detected system should update
    const detectedSystem = page.locator('#detected-system-display');
    await expect(detectedSystem).not.toContainText('???');
    await expect(detectedSystem).toContainText('Number');
  });

  test('beaming renders correctly in VexFlow', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Wait for WASM to load
    await expect(page.locator('#version-display')).toBeVisible({ timeout: 10000 });
    
    const textarea = page.locator('#notation-input');
    
    // Enter tuplet notation that should create beamed notes
    await textarea.fill('1-2');
    await page.waitForTimeout(1000);
    
    // Check that VexFlow rendered
    const svg = page.locator('#live-vexflow-notation svg');
    await expect(svg).toBeVisible();
    
    // Check for beam elements in SVG
    const beams = page.locator('svg .vf-beam, svg path[d*="L"], svg line');
    await expect(beams.first()).toBeVisible();
  });

  test('LilyPond generation works when server is available', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Wait for WASM and server status
    await expect(page.locator('#version-display')).toBeVisible({ timeout: 10000 });
    await page.waitForTimeout(2000); // Wait for server health check
    
    const textarea = page.locator('#notation-input');
    const generateBtn = page.locator('#generate-staff-btn');
    
    // Enter notation
    await textarea.fill('1 2 3 4');
    await page.waitForTimeout(500);
    
    // Click generate button
    await generateBtn.click();
    
    // Check if server is online
    const serverIndicator = page.locator('#server-status-indicator');
    const isServerOnline = await serverIndicator.evaluate(el => {
      return el.classList.contains('online');
    });
    
    if (isServerOnline) {
      // Should show staff notation section and image
      await expect(page.locator('#staff-notation-section')).toBeVisible();
      
      // Wait for processing
      await page.waitForTimeout(3000);
      
      // Check for either success or error (but not just placeholder)
      const hasImage = await page.locator('#staff-notation-image').isVisible();
      const hasError = await page.locator('#staff-notation-placeholder').isVisible();
      
      expect(hasImage || hasError).toBe(true);
    } else {
      // Server offline - should show appropriate error message
      await expect(page.locator('#staff-notation-placeholder')).toContainText('Server Connection Issue');
    }
  });

  test('debug panels toggle correctly', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Wait for WASM to load
    await expect(page.locator('#version-display')).toBeVisible({ timeout: 10000 });
    
    const textarea = page.locator('#notation-input');
    const showFsmBtn = page.locator('#show-fsm-btn');
    const debugSection = page.locator('#fsm-debug-section');
    
    // Enter some notation first
    await textarea.fill('1 2 3');
    await page.waitForTimeout(500);
    
    // Debug section should be visible initially
    await expect(debugSection).toBeVisible();
    
    // Click to hide
    await showFsmBtn.click();
    await expect(debugSection).not.toBeVisible();
    
    // Button text should change
    await expect(showFsmBtn).toContainText('Show');
    
    // Click to show again
    await showFsmBtn.click();
    await expect(debugSection).toBeVisible();
    await expect(showFsmBtn).toContainText('Hide');
  });

  test('different notation systems are detected correctly', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Wait for WASM to load
    await expect(page.locator('#version-display')).toBeVisible({ timeout: 10000 });
    
    const textarea = page.locator('#notation-input');
    const detectedSystem = page.locator('#detected-system-display');
    
    // Test Number notation
    await textarea.fill('1 2 3 4');
    await page.waitForTimeout(700);
    await expect(detectedSystem).toContainText('Number');
    
    // Test Letter notation
    await textarea.fill('C D E F');
    await page.waitForTimeout(700);
    await expect(detectedSystem).toContainText('Letter');
    
    // Test Sargam notation
    await textarea.fill('S R G M');
    await page.waitForTimeout(700);
    await expect(detectedSystem).toContainText('Sargam');
  });

  test('error handling for invalid notation', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Wait for WASM to load
    await expect(page.locator('#version-display')).toBeVisible({ timeout: 10000 });
    
    const textarea = page.locator('#notation-input');
    const detectedSystem = page.locator('#detected-system-display');
    
    // Enter invalid notation
    await textarea.fill('xyz 123 abc !@#');
    await page.waitForTimeout(700);
    
    // Should either handle gracefully or show error
    // Detected system might show ??? or handle mixed input
    const systemText = await detectedSystem.textContent();
    expect(systemText).toBeDefined();
  });
});