import { test, expect } from '@playwright/test';

test('arrow key navigation POC', async ({ page }) => {
  // Navigate to the app
  await page.goto('http://localhost:3000/');

  // Wait for the page to load
  await page.waitForLoadState('networkidle');

  // Click New button and cancel the dialog
  await page.click('text=New');
  await page.click('text=Cancel');

  // Click on the SVG container to focus
  await page.click('#svg-container-div');

  // Type "SRG" to get multiple characters
  await page.type('#svg-container-div', 'SRG');

  // Wait for rendering
  await page.waitForTimeout(500);

  // Check that we have char elements
  const charElements = await page.locator('.char').count();
  console.log(`Found ${charElements} char elements`);
  expect(charElements).toBeGreaterThan(0);

  // Get all char elements and their UUIDs
  const chars = await page.locator('.char').all();
  const charData = [];
  for (let i = 0; i < chars.length; i++) {
    const uuid = await chars[i].getAttribute('data-source-uuid');
    const text = await chars[i].textContent();
    const charIndex = await chars[i].getAttribute('data-char-index');
    charData.push({ index: i, uuid, text, charIndex });
    console.log(`Char ${i}: "${text}" UUID: ${uuid?.slice(0, 8)} charIndex: ${charIndex}`);
  }

  // Test arrow key functionality by adding console logging
  await page.evaluate(() => {
    console.log('=== Testing Arrow Key Navigation ===');

    // Get all char elements
    const chars = document.querySelectorAll('.char');
    console.log(`Found ${chars.length} char elements in DOM`);

    // Add visual highlighting for debugging
    chars.forEach((char, index) => {
      char.style.outline = '1px solid red';
      char.setAttribute('data-test-index', index);
    });

    // Test clicking on each element
    chars.forEach((char, index) => {
      char.addEventListener('click', () => {
        console.log(`Clicked char ${index}: "${char.textContent}" UUID: ${char.getAttribute('data-source-uuid')?.slice(0, 8)}`);

        // Add visual feedback
        chars.forEach(c => c.style.backgroundColor = '');
        char.style.backgroundColor = 'yellow';
      });
    });

    // Focus the container and test arrow keys
    const container = document.getElementById('svg-container-div');
    if (container) {
      container.focus();
      console.log('Container focused for arrow key testing');
    }
  });

  // Test clicking on each character
  for (let i = 0; i < Math.min(chars.length, 3); i++) {
    console.log(`\nTesting click on char ${i}...`);
    await chars[i].click();
    await page.waitForTimeout(200);
  }

  // Test arrow keys
  console.log('\nTesting arrow keys...');
  await page.keyboard.press('ArrowLeft');
  await page.waitForTimeout(200);

  await page.keyboard.press('ArrowRight');
  await page.waitForTimeout(200);

  await page.keyboard.press('ArrowLeft');
  await page.waitForTimeout(200);

  console.log('Arrow key POC completed');
});