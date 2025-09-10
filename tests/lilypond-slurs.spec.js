const { test, expect } = require('@playwright/test');

test.describe('LilyPond Slur Rendering', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:3000');
  });

  test('renders slurs in LilyPond source', async ({ page }) => {
    // Test simple slur notation - use overline notation
    // ___
    // S R G
    await page.fill('#input-text', '___\nS R G');
    
    // Click the Generate LilyPond button
    await page.click('#generate-lilypond-btn');
    
    // Wait for LilyPond source to be generated
    await page.waitForTimeout(3000);
    
    // Get the LilyPond source
    const lilypondSource = await page.textContent('#lilypond-source');
    console.log('LilyPond source:', lilypondSource);
    
    // Check that slur markers are present
    expect(lilypondSource).toContain('(');
    expect(lilypondSource).toContain(')');
    
    // Verify the expected pattern with slur - overline creates slur from S to R
    expect(lilypondSource).toMatch(/c\d+\(/); // Opening slur after first note (c4()
    expect(lilypondSource).toMatch(/d\d+\)/); // Closing slur after second note (d4))
  });

  test('renders nested slurs correctly', async ({ page }) => {
    // Test nested slur notation
    // ___
    // __ 
    // S R G
    await page.fill('#input-text', '___\n__ \nS R G');
    await page.click('#generate-lilypond-btn');
    
    await page.waitForTimeout(3000);
    
    const lilypondSource = await page.textContent('#lilypond-source');
    
    // Check for multiple slur markers
    const openParens = (lilypondSource.match(/\(/g) || []).length;
    const closeParens = (lilypondSource.match(/\)/g) || []).length;
    
    // Should have at least 2 opening and 2 closing slur markers
    expect(openParens).toBeGreaterThanOrEqual(2);
    expect(closeParens).toBeGreaterThanOrEqual(2);
  });

  test('renders slurs spanning multiple beats', async ({ page }) => {
    // Test slur across beat boundaries
    // _____
    // S R | G M
    await page.fill('#input-text', '_____\nS R | G M');
    await page.click('#generate-lilypond-btn');
    
    await page.waitForTimeout(3000);
    
    const lilypondSource = await page.textContent('#lilypond-source');
    
    // Check that slur markers span across the barline
    expect(lilypondSource).toContain('(');
    expect(lilypondSource).toContain(')');
    
    // Verify slur starts with S (c) and ends with G (e) - slur spans S R | G
    expect(lilypondSource).toMatch(/c\d+\(/); // Opening slur at S
    expect(lilypondSource).toMatch(/e\d+\)/); // Closing slur at G
  });

  test('renders slurs with tuplets', async ({ page }) => {
    // Test slur with tuplet notation
    // ___
    // 1-2
    await page.fill('#input-text', '___\n1-2');
    await page.click('#generate-lilypond-btn');
    
    await page.waitForTimeout(3000);
    
    const lilypondSource = await page.textContent('#lilypond-source');
    
    // Check for both tuplet and slur notation
    expect(lilypondSource).toContain('\\tuplet');
    expect(lilypondSource).toContain('(');
    expect(lilypondSource).toContain(')');
    
    // Verify slur markers are within or around the tuplet
    expect(lilypondSource).toMatch(/\\tuplet.*\{.*\(.*\)/s);
  });
});