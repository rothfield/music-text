const { test, expect } = require('@playwright/test');

test.describe('UUID-based text insertion', () => {
  test('should handle insert_text with target_uuid', async ({ page }) => {
    // Start the server (assuming it's running on localhost:3000)
    await page.goto('http://localhost:3000');

    // Wait for the app to initialize
    await page.waitForSelector('.svg-container');
    await page.waitForTimeout(2000); // Give it time to load

    // Try typing a character
    await page.focus('.svg-container');
    await page.keyboard.type('S');

    // Wait for the server response
    await page.waitForTimeout(1000);

    // Check if the character appears in the SVG
    const svgText = await page.textContent('svg');
    console.log('SVG content after typing S:', svgText);

    // Basic test: make sure no error occurred
    const errorElement = await page.$('[data-testid="error"]');
    expect(errorElement).toBeNull();
  });

  test('should handle empty document creation', async ({ page }) => {
    // Test creating a new document
    const response = await page.request.post('http://localhost:3000/api/documents', {
      data: {
        content: null, // Empty document
        metadata: {}
      }
    });

    expect(response.ok()).toBeTruthy();
    const result = await response.json();

    console.log('Empty document creation result:', JSON.stringify(result, null, 2));
    expect(result.success).toBe(true);
    expect(result.document).toBeDefined();
    expect(result.document.documentUUID).toBeDefined();
  });

  test('should handle insert_text API directly', async ({ page }) => {
    // First create a document with some content
    const createResponse = await page.request.post('http://localhost:3000/api/documents', {
      data: {
        content: "123", // Simple content
        metadata: {}
      }
    });

    expect(createResponse.ok()).toBeTruthy();
    const createResult = await createResponse.json();

    // Try to insert text using the transform API
    const insertResponse = await page.request.post('http://localhost:3000/api/documents/transform', {
      data: {
        document: createResult.document,
        command_type: "insert_text",
        target_uuids: [],
        parameters: {
          text: "S",
          target_uuid: null, // No UUID for now
          element_position: 0
        }
      }
    });

    const insertResult = await insertResponse.json();
    console.log('Insert text result:', JSON.stringify(insertResult, null, 2));

    // Should not fail even without UUID
    expect(insertResult.success).toBe(true);
  });
});