const { test, expect } = require('@playwright/test');

test.describe('Slur Toggle Functionality', () => {
  test('should add continuous slur line over selected notes', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Input test notation
    const testInput = '1 2';
    await page.fill('#inputTextarea', testInput);
    
    // Select the entire line "1 2"
    const textarea = page.locator('#inputTextarea');
    await textarea.click();
    await textarea.selectText();
    
    // Verify selection
    const selectedText = await page.evaluate(() => {
      const textarea = document.getElementById('inputTextarea');
      return textarea.value.substring(textarea.selectionStart, textarea.selectionEnd);
    });
    expect(selectedText).toBe('1 2');
    
    // Click slur button
    await page.click('#slurButton');
    
    // Check that slur line was added
    const finalText = await textarea.inputValue();
    expect(finalText).toBe('___\n1 2');
    
    // Verify focus is back on textarea
    const focusedElement = await page.evaluate(() => document.activeElement.id);
    expect(focusedElement).toBe('inputTextarea');
  });
  
  test('should remove slur line when toggling existing slurs', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Input notation with existing slur
    const testInput = '___\n1 2';
    await page.fill('#inputTextarea', testInput);
    
    // Select the "1 2" line
    await page.locator('#inputTextarea').click();
    await page.keyboard.press('End'); // Go to end
    await page.keyboard.press('Shift+Home'); // Select line
    
    // Click slur button to remove slurs
    await page.click('#slurButton');
    
    // Check that slur line was removed
    const finalText = await page.locator('#inputTextarea').inputValue();
    expect(finalText).toBe('1 2');
  });
  
  test('should add slur covering entire selection including spaces', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Input test notation with space
    const testInput = '1  2  3';
    await page.fill('#inputTextarea', testInput);
    
    // Select entire line
    const textarea = page.locator('#inputTextarea');
    await textarea.click();
    await textarea.selectText();
    
    // Click slur button
    await page.click('#slurButton');
    
    // Check that continuous slur was added over entire selection
    const finalText = await textarea.inputValue();
    expect(finalText).toBe('_______\n1  2  3');
  });
  
  test('should work with partial selections', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Input test notation
    const testInput = '1 2 3 4';
    await page.fill('#inputTextarea', testInput);
    
    // Select just "2 3" portion
    const textarea = page.locator('#inputTextarea');
    await textarea.click();
    await page.keyboard.press('Home');
    await page.keyboard.press('ArrowRight'); // Move to after "1"
    await page.keyboard.press('ArrowRight'); // Move to space
    await page.keyboard.press('Shift+ArrowRight'); // Select "2"
    await page.keyboard.press('Shift+ArrowRight'); // Select " "
    await page.keyboard.press('Shift+ArrowRight'); // Select "3"
    
    // Click slur button
    await page.click('#slurButton');
    
    // Check that slur was added only over selected portion
    const finalText = await textarea.inputValue();
    expect(finalText).toBe(' ___\n1 2 3 4');
  });
  
  test('should not add slurs to non-musical lines', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Input non-musical text (only one pitch)
    const testInput = 'hello world';
    await page.fill('#inputTextarea', testInput);
    
    // Select entire line
    const textarea = page.locator('#inputTextarea');
    await textarea.click();
    await textarea.selectText();
    
    // Click slur button
    await page.click('#slurButton');
    
    // Check that nothing changed (not musical)
    const finalText = await textarea.inputValue();
    expect(finalText).toBe('hello world');
  });
});