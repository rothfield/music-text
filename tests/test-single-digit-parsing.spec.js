const { test, expect } = require('@playwright/test');

test.describe('Single digit parsing', () => {
  test('should parse "1" as content line, not upper line', async ({ page }) => {
    // Navigate to the web interface
    await page.goto('http://localhost:3000');
    
    // Wait for the page to load
    await page.waitForSelector('#input');
    
    // Clear any existing text and type "1"
    await page.fill('#input', '1');
    
    // Trigger input event to ensure parsing happens
    await page.evaluate(() => {
      const input = document.querySelector('#input');
      input.dispatchEvent(new Event('input'));
    });
    
    // Wait for parsing to complete and AST to update
    await page.waitForFunction(
      () => {
        const astElement = document.querySelector('#ast-output');
        return astElement && astElement.textContent !== 'Waiting for input...' && astElement.textContent.includes('staves');
      },
      { timeout: 5000 }
    );
    
    // Get the AST output
    const astOutput = await page.evaluate(() => {
      const astElement = document.querySelector('#ast-output');
      if (!astElement) return null;
      
      // Extract JSON from the formatted output
      const text = astElement.textContent;
      const jsonMatch = text.match(/\{[\s\S]*\}/);
      return jsonMatch ? jsonMatch[0] : null;
    });
    
    console.log('AST Output:', astOutput);
    
    // Parse the AST JSON
    const ast = JSON.parse(astOutput);
    
    // Verify that "1" is parsed as content line, not upper line
    expect(ast.staves).toBeDefined();
    expect(ast.staves.length).toBeGreaterThan(0);
    
    const firstStave = ast.staves[0];
    
    // Check that upper_lines is empty
    expect(firstStave.upper_lines).toEqual([]);
    
    // Check that content_line contains the pitch "1"
    expect(firstStave.content_line).toBeDefined();
    expect(firstStave.content_line.measures).toBeDefined();
    expect(firstStave.content_line.measures.length).toBeGreaterThan(0);
    
    const firstMeasure = firstStave.content_line.measures[0];
    expect(firstMeasure.beats).toBeDefined();
    expect(firstMeasure.beats.length).toBeGreaterThan(0);
    
    const firstBeat = firstMeasure.beats[0];
    expect(firstBeat.elements).toBeDefined();
    expect(firstBeat.elements.length).toBeGreaterThan(0);
    
    const firstElement = firstBeat.elements[0];
    expect(firstElement.Pitch).toBeDefined();
    expect(firstElement.Pitch.value).toBe('1');
    
    console.log('✓ "1" is correctly parsed as content line, not upper line');
  });

  test('should parse multiple single digits as content line', async ({ page }) => {
    await page.goto('http://localhost:3000');
    await page.waitForSelector('#input');
    
    const testCases = ['2', '3', '7', '1 2 3'];
    
    for (const input of testCases) {
      await page.fill('#input', input);
      
      // Trigger input event
      await page.evaluate(() => {
        const inputEl = document.querySelector('#input');
        inputEl.dispatchEvent(new Event('input'));
      });
      
      // Wait for AST to update
      await page.waitForFunction(
        () => {
          const astElement = document.querySelector('#ast-output');
          return astElement && astElement.textContent !== 'Waiting for input...' && astElement.textContent.includes('staves');
        },
        { timeout: 5000 }
      );
      
      const astOutput = await page.evaluate(() => {
        const astElement = document.querySelector('#ast-output');
        if (!astElement) return null;
        
        // Extract JSON from the formatted output
        const text = astElement.textContent;
        const jsonMatch = text.match(/\{[\s\S]*\}/);
        return jsonMatch ? jsonMatch[0] : null;
      });
      
      const ast = JSON.parse(astOutput);
      
      // Verify upper_lines is empty for all test cases
      expect(ast.staves[0].upper_lines).toEqual([]);
      
      // Verify content_line is populated
      expect(ast.staves[0].content_line.measures).toBeDefined();
      expect(ast.staves[0].content_line.measures.length).toBeGreaterThan(0);
      
      console.log(`✓ "${input}" correctly parsed as content line`);
    }
  });
});