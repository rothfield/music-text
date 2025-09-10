// @ts-check
const { test, expect } = require('@playwright/test');

test.describe('Layout and Responsive Design', () => {
  
  test('page loads with correct title', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveTitle(/Notation Parser/);
    
    // Check main header is visible
    const header = page.locator('h1');
    await expect(header).toContainText('Text to Staff Notation');
  });

  test('textarea utilizes full available width', async ({ page }) => {
    await page.goto('/');
    
    // Wait for page to fully load
    await page.waitForLoadState('networkidle');
    
    const textarea = page.locator('#input-text');
    const inputSection = page.locator('.input-section');
    
    // Ensure elements are visible
    await expect(textarea).toBeVisible();
    await expect(inputSection).toBeVisible();
    
    // Get bounding boxes
    const textareaBox = await textarea.boundingBox();
    const sectionBox = await inputSection.boundingBox();
    
    expect(textareaBox).toBeTruthy();
    expect(sectionBox).toBeTruthy();
    
    // Calculate expected width accounting for section padding (15px on each side) 
    const expectedMinWidth = sectionBox.width - (15 * 2) - 10; // Extra margin for safety
    
    console.log(`Section width: ${sectionBox.width}px`);
    console.log(`Textarea width: ${textareaBox.width}px`);
    console.log(`Expected min width: ${expectedMinWidth}px`);
    
    // Textarea should utilize most of the available width
    expect(textareaBox.width).toBeGreaterThan(expectedMinWidth);
  });

  test('responsive layout on different screen sizes', async ({ page }) => {
    // Test desktop layout
    await page.setViewportSize({ width: 1366, height: 768 });
    await page.goto('/');
    
    const body = page.locator('body');
    const bodyBox = await body.boundingBox();
    
    // Body should not exceed max-width (1400px) and be centered
    expect(bodyBox.width).toBeLessThanOrEqual(1400);
    
    // Test tablet layout
    await page.setViewportSize({ width: 768, height: 1024 });
    await page.reload();
    
    const textarea = page.locator('#input-text');
    await expect(textarea).toBeVisible();
    
    // Test mobile layout
    await page.setViewportSize({ width: 375, height: 667 });
    await page.reload();
    
    await expect(textarea).toBeVisible();
    const textareaMobile = await textarea.boundingBox();
    
    // On mobile, textarea should still be usable (minimum width)
    expect(textareaMobile.width).toBeGreaterThan(300);
  });

  test('server status indicator is positioned correctly', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    const indicator = page.locator('#server-status-indicator');
    await expect(indicator).toBeVisible();
    
    // Should be fixed position in top right
    const indicatorBox = await indicator.boundingBox();
    const viewport = page.viewportSize();
    
    expect(indicatorBox.x + indicatorBox.width).toBeCloseTo(viewport.width - 20, 50); // Within 50px of expected position
    expect(indicatorBox.y).toBeCloseTo(20, 30); // Within 30px of top
  });

  test('font size is readable and appropriate', async ({ page }) => {
    await page.goto('/');
    
    const textarea = page.locator('#input-text');
    
    // Check computed font size
    const fontSize = await textarea.evaluate(el => {
      return window.getComputedStyle(el).fontSize;
    });
    
    // Should be 20px as set in CSS
    expect(fontSize).toBe('20px');
    
    // Check font family
    const fontFamily = await textarea.evaluate(el => {
      return window.getComputedStyle(el).fontFamily;
    });
    
    expect(fontFamily).toMatch(/courier/i);
  });

  test('layout sections are properly spaced', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check that main content sections are visible and properly spaced
    const header = page.locator('.header');
    const inputSection = page.locator('.input-section');
    const vexflowSection = page.locator('#live-vexflow-section');
    
    await expect(header).toBeVisible();
    await expect(inputSection).toBeVisible();
    await expect(vexflowSection).toBeVisible();
    
    // Sections should not overlap
    const headerBox = await header.boundingBox();
    const inputBox = await inputSection.boundingBox();
    const vexflowBox = await vexflowSection.boundingBox();
    
    // Input section should be below header
    expect(inputBox.y).toBeGreaterThan(headerBox.y + headerBox.height);
    
    // VexFlow section should be below input section  
    expect(vexflowBox.y).toBeGreaterThan(inputBox.y + inputBox.height);
  });

  test('box-sizing is applied correctly', async ({ page }) => {
    await page.goto('/');
    
    const textarea = page.locator('#input-text');
    
    // Check box-sizing property
    const boxSizing = await textarea.evaluate(el => {
      return window.getComputedStyle(el).boxSizing;
    });
    
    expect(boxSizing).toBe('border-box');
  });
});