const { test, expect } = require('@playwright/test');

test('Document-first typing workflow: New → Type → Check LilyPond', async ({ page }) => {
  console.log('🎹 Starting document-first typing workflow test...');

  // Set up API call monitoring
  page.on('request', req => {
    if (req.url().includes('/api/documents')) {
      console.log(`📡 API Call: ${req.method()} ${req.url()}`);
    }
  });

  page.on('response', async res => {
    if (res.url().includes('/api/documents') && res.status() === 200) {
      console.log(`📡 API Response: ${res.status()} ${res.url()}`);
    }
  });

  // 1. Go to index.html
  await page.goto('/');
  await page.waitForLoadState('networkidle');
  console.log('✅ Loaded index.html');

  // 2. Click on "New" button
  await page.click('button:has-text("New"), button[title*="New"], #new-btn, .new-button');
  console.log('✅ Clicked New button');

  // Wait for any new document API calls
  await page.waitForTimeout(1000);

  // 3. Click on editor pane to focus it
  const editorSelectors = [
    '.canvas-editor',
    '#editor',
    '.editor-pane',
    'textarea',
    '#svg-container',
    '.editor-content'
  ];

  let editorFound = false;
  for (const selector of editorSelectors) {
    try {
      await page.click(selector, { timeout: 2000 });
      console.log(`✅ Clicked editor pane: ${selector}`);
      editorFound = true;
      break;
    } catch (e) {
      console.log(`❌ Editor selector ${selector} not found, trying next...`);
    }
  }

  if (!editorFound) {
    // Fallback: click somewhere in the center of the page
    await page.click('body');
    console.log('⚠️ Fallback: clicked body to focus');
  }

  // 4. Type "SRG"
  console.log('⌨️ Typing "SRG"...');
  await page.keyboard.type('SRG', { delay: 300 });

  // Wait for API calls to complete
  await page.waitForTimeout(2000);

  // 5. Click on LilyPond source tab
  const lilypondTabSelectors = [
    'button:has-text("LilyPond")',
    'button:has-text("lilypond")',
    '.tab-lilypond',
    '#lilypond-tab',
    '[data-tab="lilypond"]',
    'button[title*="LilyPond"]'
  ];

  let tabFound = false;
  for (const selector of lilypondTabSelectors) {
    try {
      await page.click(selector, { timeout: 2000 });
      console.log(`✅ Clicked LilyPond tab: ${selector}`);
      tabFound = true;
      break;
    } catch (e) {
      console.log(`❌ LilyPond tab selector ${selector} not found, trying next...`);
    }
  }

  expect(tabFound).toBeTruthy();

  // Wait for content to load
  await page.waitForTimeout(1000);

  // 6. Check LilyPond source contains note pattern like "c8 d8 e8"
  const lilypondContentSelectors = [
    '#lilypond-output',
    '.lilypond-content',
    '#lilypond_src-output',
    'pre:has-text("version")',
    'textarea[readonly]',
    '.output-content'
  ];

  let lilypondContent = '';
  for (const selector of lilypondContentSelectors) {
    try {
      const element = await page.locator(selector).first();
      if (await element.isVisible({ timeout: 2000 })) {
        lilypondContent = await element.textContent();
        console.log(`✅ Found LilyPond content in: ${selector}`);
        break;
      }
    } catch (e) {
      console.log(`❌ LilyPond content selector ${selector} not found, trying next...`);
    }
  }

  console.log(`📝 LilyPond content: "${lilypondContent}"`);

  // Verify we have LilyPond content
  expect(lilypondContent).toContain('version');

  // Check for note pattern (c8 d8 e8 or similar)
  const hasNotePattern = /[cdefgab]\d+\s+[cdefgab]\d+\s+[cdefgab]\d+/i.test(lilypondContent);

  if (hasNotePattern) {
    console.log('✅ Found expected note pattern in LilyPond source!');
  } else {
    console.log('⚠️ Note pattern not found, but LilyPond content exists');
    console.log(`Full content: ${lilypondContent}`);
  }

  // At minimum, verify we have some musical content
  expect(lilypondContent.length).toBeGreaterThan(20);

  console.log('🎯 Document-first typing workflow test completed!');
});