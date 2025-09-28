const { test, expect } = require('@playwright/test');

test('New button creates empty document', async ({ page }) => {
  console.log('🆕 Starting New button test...');

  // Set up API monitoring to verify document creation
  let documentCreated = false;
  let documentUUID = null;

  page.on('request', req => {
    if (req.url().includes('/api/documents') && req.method() === 'POST') {
      console.log('📡 API Call: Creating new document');
    }
  });

  page.on('response', async res => {
    if (res.url().includes('/api/documents') && res.status() === 200 && res.request().method() === 'POST') {
      try {
        const body = await res.json();
        if (body.success && body.document && body.document.documentUUID) {
          documentCreated = true;
          documentUUID = body.document.documentUUID;
          console.log(`✅ Document created with UUID: ${documentUUID}`);
        }
      } catch (e) {
        console.log('❌ Could not parse document creation response');
      }
    }
  });

  // 1. Go to localhost:3000/
  await page.goto('http://localhost:3000/');
  await page.waitForLoadState('networkidle');
  console.log('✅ Step 1: Loaded localhost:3000/');

  // 2. Verify initial state before clicking New
  console.log('🔍 Checking initial state...');

  // Check that svg-container exists
  await expect(page.locator('#svg-container')).toBeVisible();
  console.log('✅ SVG container is visible');

  // Check that tabs are present
  await expect(page.locator('#lilypond_src')).toBeVisible();
  console.log('✅ LilyPond tab is visible');

  // 3. Click "New" button
  await page.click('#newDocButton');
  console.log('✅ Step 2: Clicked New button');

  // 4. Wait for document creation API call
  await page.waitForTimeout(2000);

  // 5. Verify document was created via API
  expect(documentCreated).toBeTruthy();
  expect(documentUUID).toBeTruthy();
  console.log(`✅ Step 3: Document created successfully (UUID: ${documentUUID})`);

  // 6. Verify focus is set to svg-container after new button pressed
  const focusedElement = await page.evaluate(() => document.activeElement?.id);
  expect(focusedElement).toBe('svg-container');
  console.log('✅ Step 4: Focus is set to svg-container after new button press');

  // 7. Verify all tabs are functional
  await page.click('#lilypond_src');
  await expect(page.locator('#lilypond_src-output')).toBeVisible();
  console.log('✅ Step 5: LilyPond tab is functional');

  await page.click('#vexflow_svg');
  await expect(page.locator('#vexflow_svg-output')).toBeVisible();
  console.log('✅ Step 6: VexFlow tab is functional');

  await page.click('#editor_svg');
  await expect(page.locator('#svg-container')).toBeVisible();
  console.log('✅ Step 7: Editor tab is functional');

  // 8. Verify editor starts empty (ready for input)
  const editorContent = await page.evaluate(() => {
    return window.app?.canvasEditor?.document?.value || '';
  });

  expect(editorContent).toBe('');
  console.log('✅ Step 8: Editor starts with empty content');

  // 9. Verify document tab shows correct structure
  await page.click('#document');
  await expect(page.locator('#document-output')).toBeVisible();
  console.log('✅ Step 9: Document tab is accessible');

  // Parse document tab text content into POJO
  const documentTabContent = await page.locator('#document-output').textContent();
  console.log('📄 Document tab raw content:', documentTabContent.substring(0, 200) + '...');

  let documentPOJO;
  try {
    documentPOJO = JSON.parse(documentTabContent);
    console.log('✅ Successfully parsed document tab content as JSON');
  } catch (e) {
    throw new Error(`Failed to parse document tab content as JSON: ${e.message}\nContent: ${documentTabContent}`);
  }

  // Verify document POJO structure
  expect(documentPOJO).toBeTruthy();
  expect(documentPOJO.elements).toEqual([]);
  expect(documentPOJO.documentUUID).toBeTruthy();
  expect(typeof documentPOJO.documentUUID).toBe('string');
  console.log(`✅ Step 10: Document POJO has empty elements array and valid UUID: ${documentPOJO.documentUUID}`);

  console.log('🎯 New button test completed successfully!');
});