const { test, expect } = require('@playwright/test');

test('should save and restore basic cursor position', async ({ page }) => {
  // Use alternate port
  await page.goto('http://localhost:3001');
  
  // Clear localStorage first
  await page.evaluate(() => localStorage.clear());
  
  // Add some test content
  await page.fill('#input-text', '1 2 3 4 5');
  
  // Position cursor after "3 " (position 6)
  await page.click('#input-text');
  await page.keyboard.press('Home');
  for (let i = 0; i < 6; i++) {
    await page.keyboard.press('ArrowRight');
  }
  
  // Verify cursor position
  const cursorPos = await page.evaluate(() => {
    const input = document.getElementById('notation-input');
    return input.selectionStart;
  });
  expect(cursorPos).toBe(6);
  
  // Trigger a save by pressing a key
  await page.keyboard.press('ArrowLeft');
  await page.keyboard.press('ArrowRight');
  
  // Wait for save
  await page.waitForTimeout(300);
  
  // Check localStorage contains the position
  const saved = await page.evaluate(() => {
    const data = localStorage.getItem('musicNotationSettings');
    return data ? JSON.parse(data) : null;
  });
  
  console.log('Saved data:', saved);
  expect(saved).toBeTruthy();
  expect(saved.selectionStart).toBe(6);
  expect(saved.selectionEnd).toBe(6);
  
  // Reload the page
  await page.reload();
  await page.waitForLoadState('domcontentloaded');
  
  // Wait for restoration
  await page.waitForTimeout(500);
  
  // Check that cursor position was restored
  const restoredPos = await page.evaluate(() => {
    const input = document.getElementById('notation-input');
    return {
      start: input.selectionStart,
      end: input.selectionEnd,
      text: input.value
    };
  });
  
  console.log('Restored position:', restoredPos);
  expect(restoredPos.text).toBe('1 2 3 4 5');
  expect(restoredPos.start).toBe(6);
  expect(restoredPos.end).toBe(6);
});