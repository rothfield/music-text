const { test, expect } = require('@playwright/test');

test.describe('UUID-based insertion points', () => {
  test('should create empty document with UUID insertion point', async ({ page }) => {
    // Start the server
    await page.goto('http://localhost:3000');

    // Wait for the app to initialize
    await page.waitForSelector('.svg-container');

    // Wait specifically for SVG content to be loaded
    await page.waitForSelector('#svg-container svg', { timeout: 10000 });
    await page.waitForTimeout(3000); // Additional wait for document creation

    // Check that the SVG contains an element with data-source-uuid
    const svgContainer = await page.$('#svg-container');
    const svgContent = svgContainer ? await svgContainer.innerHTML() : '';
    console.log('SVG container found:', !!svgContainer);
    console.log('SVG content length:', svgContent.length);
    console.log('SVG content preview:', svgContent.substring(0, 2000));

    // Look for data-source-uuid attribute in the SVG
    const hasUuidElement = svgContent.includes('data-source-uuid');
    console.log('Has UUID element:', hasUuidElement);

    expect(hasUuidElement).toBe(true);

    // Try to find the specific insertion point element
    const uuidMatch = svgContent.match(/data-source-uuid="([^"]+)"/);
    if (uuidMatch) {
      const insertionUuid = uuidMatch[1];
      console.log('Found insertion UUID:', insertionUuid);

      // Verify it's a valid UUID format
      const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
      expect(insertionUuid).toMatch(uuidRegex);
    }
  });

  test('should use UUID for text insertion', async ({ page }) => {
    // First create a document
    const createResponse = await page.request.post('http://localhost:3000/api/documents', {
      data: {
        metadata: {}
      }
    });

    expect(createResponse.ok()).toBeTruthy();
    const createResult = await createResponse.json();
    console.log('Created document:', JSON.stringify(createResult, null, 2));

    // Extract the content line UUID from the created document
    const document = createResult.document;
    const stave = document.elements[0]?.Stave;
    const contentLine = stave?.lines[0]?.ContentLine;
    const contentLineUuid = contentLine?.id;

    console.log('Content line UUID:', contentLineUuid);
    expect(contentLineUuid).toBeDefined();

    // Try to insert text using the content line UUID
    const insertResponse = await page.request.post('http://localhost:3000/api/documents/transform', {
      data: {
        document: document,
        command_type: "insert_text",
        target_uuids: [],
        parameters: {
          text: "S",
          target_uuid: contentLineUuid,
          element_position: 0
        }
      }
    });

    const insertResult = await insertResponse.json();
    console.log('Insert result:', JSON.stringify(insertResult, null, 2));

    // Should succeed with UUID-based insertion
    expect(insertResult.success).toBe(true);

    // Check that the document was updated
    expect(insertResult.document).toBeDefined();
  });
});