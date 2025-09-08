const { test, expect } = require('@playwright/test');

test.describe('Cursor Position Storage', () => {
  test('should save and restore cursor position on page reload', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Clear any existing localStorage
    await page.evaluate(() => localStorage.clear());
    
    // Add test content
    const testInput = '1 2 3 4 5';
    await page.fill('#notation-input', testInput);
    
    // Position cursor in the middle (after "3 ")
    const textarea = page.locator('#notation-input');
    await textarea.click();
    await page.keyboard.press('Home');
    for (let i = 0; i < 6; i++) {
      await page.keyboard.press('ArrowRight'); // Position after "3 "
    }
    
    // Verify cursor position before reload
    const cursorPosBefore = await page.evaluate(() => {
      const input = document.getElementById('inputTextarea');
      return { start: input.selectionStart, end: input.selectionEnd };
    });
    expect(cursorPosBefore.start).toBe(6);
    expect(cursorPosBefore.end).toBe(6);
    
    // Trigger save by moving focus
    await page.keyboard.press('ArrowLeft');
    await page.keyboard.press('ArrowRight');
    
    // Wait for localStorage save
    await page.waitForTimeout(200);
    
    // Verify localStorage contains cursor position
    const savedData = await page.evaluate(() => {
      const data = localStorage.getItem('musicNotationSettings');
      return data ? JSON.parse(data) : null;
    });
    expect(savedData.selectionStart).toBe(6);
    expect(savedData.selectionEnd).toBe(6);
    
    // Reload page
    await page.reload();
    
    // Wait for page to load
    await page.waitForLoadState('domcontentloaded');
    await page.waitForTimeout(300);
    
    // Check that text was restored
    const restoredText = await textarea.inputValue();
    expect(restoredText).toBe('1 2 3 4 5');
    
    // Check that cursor position was restored
    const cursorPosAfter = await page.evaluate(() => {
      const input = document.getElementById('inputTextarea');
      return { start: input.selectionStart, end: input.selectionEnd };
    });
    expect(cursorPosAfter.start).toBe(6);
    expect(cursorPosAfter.end).toBe(6);
  });
  
  test('should save and restore text selection', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Clear localStorage
    await page.evaluate(() => localStorage.clear());
    
    // Add test content
    const testInput = 'hello world test';
    await page.fill('#notation-input', testInput);
    
    // Select "world" (characters 6-11)
    const textarea = page.locator('#notation-input');
    await textarea.click();
    await page.keyboard.press('Home');
    for (let i = 0; i < 6; i++) {
      await page.keyboard.press('ArrowRight');
    }
    for (let i = 0; i < 5; i++) {
      await page.keyboard.press('Shift+ArrowRight');
    }
    
    // Verify selection before reload
    const selectionBefore = await page.evaluate(() => {
      const input = document.getElementById('inputTextarea');
      return input.value.substring(input.selectionStart, input.selectionEnd);
    });
    expect(selectionBefore).toBe('world');
    
    // Trigger save
    await page.keyboard.press('ArrowLeft');
    await page.keyboard.press('Shift+ArrowRight');
    await page.waitForTimeout(200);
    
    // Reload page
    await page.reload();
    await page.waitForLoadState('domcontentloaded');
    await page.waitForTimeout(300);
    
    // Check selection was restored
    const selectionAfter = await page.evaluate(() => {
      const input = document.getElementById('inputTextarea');
      return input.value.substring(input.selectionStart, input.selectionEnd);
    });
    expect(selectionAfter).toBe('world');
  });
  
  test('should handle cursor position when text length changes', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Clear localStorage
    await page.evaluate(() => localStorage.clear());
    
    // Add initial content
    const initialText = '1 2 3';
    await page.fill('#notation-input', initialText);
    
    // Position cursor at the end
    const textarea = page.locator('#notation-input');
    await textarea.click();
    await page.keyboard.press('End');
    
    // Trigger save
    await page.keyboard.press('ArrowLeft');
    await page.keyboard.press('ArrowRight');
    await page.waitForTimeout(200);
    
    // Manually update localStorage with longer text (simulating user editing)
    await page.evaluate(() => {
      const settings = JSON.parse(localStorage.getItem('musicNotationSettings'));
      settings.notation = '1 2 3 4 5 6 7 8 9 10'; // Much longer
      settings.selectionStart = 50; // Beyond new text length
      settings.selectionEnd = 50;
      localStorage.setItem('musicNotationSettings', JSON.stringify(settings));
    });
    
    // Reload page
    await page.reload();
    await page.waitForLoadState('domcontentloaded');
    await page.waitForTimeout(300);
    
    // Check that cursor position was clamped to text length
    const finalPos = await page.evaluate(() => {
      const input = document.getElementById('inputTextarea');
      return { 
        start: input.selectionStart, 
        end: input.selectionEnd, 
        textLength: input.value.length 
      };
    });
    
    // Cursor should be at end of actual text, not beyond it
    expect(finalPos.start).toBeLessThanOrEqual(finalPos.textLength);
    expect(finalPos.end).toBeLessThanOrEqual(finalPos.textLength);
  });
});