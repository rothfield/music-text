const { test, expect } = require('@playwright/test');

test.describe('Cross-Beat Slurs', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:3000');
  });

  test('renders slurs spanning across beat boundaries', async ({ page }) => {
    // Test slur that starts in one beat and ends in another:
    // ___
    // 1-2 3
    await page.fill('#notation-input', '___\n1-2 3');
    
    await page.click('#generate-lilypond-btn');
    await page.waitForTimeout(3000);
    
    const lilypondSource = await page.textContent('#lilypond-source');
    console.log('Cross-beat slur source:', lilypondSource);
    
    // Should contain slur markers
    expect(lilypondSource).toContain('(');
    expect(lilypondSource).toContain(')');
    
    // Should have slur from first note to last note across the tuplet
    expect(lilypondSource).toMatch(/c\d+\(/); // Opening slur at first note
    expect(lilypondSource).toMatch(/e\d+\)/); // Closing slur at third note
  });

  test('renders slurs spanning across different beats', async ({ page }) => {
    // Test slur across completely separate beats:
    // __
    // 1 2 3
    await page.fill('#notation-input', '__\n1 2 3');
    
    await page.click('#generate-lilypond-btn');
    await page.waitForTimeout(3000);
    
    const lilypondSource = await page.textContent('#lilypond-source');
    console.log('Cross-beat separate source:', lilypondSource);
    
    // Should contain slur markers
    expect(lilypondSource).toContain('(');
    expect(lilypondSource).toContain(')');
    
    // Should have slur from first note to second note only
    expect(lilypondSource).toMatch(/c\d+\(\)/); // Complete slur on first note
  });

  test('renders slurs spanning barlines', async ({ page }) => {
    // Test slur that spans across a barline:
    // ___
    // 1 2 | 3
    await page.fill('#notation-input', '___\n1 2 | 3');
    
    await page.click('#generate-lilypond-btn');
    await page.waitForTimeout(3000);
    
    const lilypondSource = await page.textContent('#lilypond-source');
    console.log('Barline-spanning slur source:', lilypondSource);
    
    // Should contain slur markers and barline
    expect(lilypondSource).toContain('(');
    expect(lilypondSource).toContain(')');
    expect(lilypondSource).toContain('\\bar');
    
    // Should have slur from first note to second note (before barline)
    expect(lilypondSource).toMatch(/c\d+\(/); // Opening slur at first note
    expect(lilypondSource).toMatch(/d\d+\)/); // Closing slur at second note (before barline)
  });

  test('renders complex mixed slurs and beats', async ({ page }) => {
    // Test complex pattern:
    // ___  ___
    // 1-2  3 4-5
    await page.fill('#notation-input', '___  ___\n1-2  3 4-5');
    
    await page.click('#generate-lilypond-btn');
    await page.waitForTimeout(3000);
    
    const lilypondSource = await page.textContent('#lilypond-source');
    console.log('Complex slurs source:', lilypondSource);
    
    // Should contain musical slur markers - count only note-attached slurs
    const noteSlurPattern = /[a-g]\d+\([^)]*\)|[a-g]\d+\(|\)(?=\s|\}|$)/g;
    const slurMatches = lilypondSource.match(noteSlurPattern) || [];
    
    // Should have slur markers for both tuplets
    expect(slurMatches.length).toBeGreaterThanOrEqual(2);
    
    // Verify specific slur patterns exist
    expect(lilypondSource).toMatch(/c\d+\(/); // First tuplet opening slur
    expect(lilypondSource).toMatch(/e\d+\(\)/); // Second tuplet complete slur
  });
});