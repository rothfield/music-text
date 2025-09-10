const { test, expect } = require('@playwright/test');

test('syllable support functionality', async ({ page }) => {
  await page.goto('http://localhost:3000');
  
  // Test input with lyrics
  const testInput = `1 2 3 4
hel- lo wor- ld`;
  
  // Enter the test input
  await page.fill('#input-text', testInput);
  await page.click('#parse-btn');
  
  // Wait for rendering to complete
  await page.waitForTimeout(2000);
  
  // Check that VexFlow canvas was rendered
  const canvas = page.locator('#vexflow-output canvas');
  await expect(canvas).toBeVisible();
  
  // Verify debug output shows syllables
  const debugOutput = await page.locator('#debug-output').textContent();
  expect(debugOutput).toContain('"syl":"hel-"');
  expect(debugOutput).toContain('"syl":"lo"');
  expect(debugOutput).toContain('"syl":"wor-"');
  expect(debugOutput).toContain('"syl":"ld"');
  
  // Check LilyPond output includes addlyrics
  const lilypondOutput = await page.locator('#lilypond-output').textContent();
  expect(lilypondOutput).toContain('\\addlyrics');
  expect(lilypondOutput).toContain('hel- lo wor- ld');
  expect(lilypondOutput).toContain('font-size = #-2');
  expect(lilypondOutput).toContain('font-shape = #\'italic');
  
  console.log('✅ Syllable functionality test passed');
});