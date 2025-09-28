const { test, expect } = require('@playwright/test');

test('Editor typing workflow: New → Type CDE → Check LilyPond', async ({ page }) => {
  console.log('🎹 Starting editor typing test...');

  // 1. Go to localhost:3000/
  await page.goto('http://localhost:3000/');
  await page.waitForLoadState('networkidle');
  console.log('✅ Step 1: Loaded localhost:3000/');

  // 2. Click "New" button
  await page.click('button:has-text("New")');
  console.log('✅ Step 2: Clicked New button');
  await page.waitForTimeout(1000);

  // 3. Click on svg-container (editor panel)
  await page.click('#svg-container');
  console.log('✅ Step 3: Clicked svg-container');
  await page.waitForTimeout(500);

  // 4. Type "CDE"
  await page.keyboard.type('CDE', { delay: 200 });
  console.log('✅ Step 4: Typed "CDE"');
  await page.waitForTimeout(2000); // Wait for API processing

  // 5. Check LilyPond source tab for "c8 d8 e8"
  await page.click('button:has-text("LilyPond")');
  console.log('✅ Step 5: Clicked LilyPond tab');
  await page.waitForTimeout(1000);

  // Get LilyPond source content
  const lilypondContent = await page.locator('#lilypond_src-output').textContent();
  console.log(`📝 LilyPond content: "${lilypondContent}"`);

  // Verify it contains c8 d8 e8 pattern
  expect(lilypondContent).toContain('c8 d8 e8');
  console.log('✅ Found expected pattern: c8 d8 e8');

  console.log('🎯 Editor typing test completed successfully!');
});