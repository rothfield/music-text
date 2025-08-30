import { test, expect } from '@playwright/test';

test.describe('Octave Transposition', () => {
  test('correctly transposes scale degree 7 to proper octave in D major', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Wait for the app to load
    await page.waitForSelector('#input');
    
    // Input notation with key D and scale degrees including 7
    const notation = `key: D
time: 4/4

1 2 3 4 5 6 7 .1
.7 .6 .5 .4 .3 .2 .1 7`;
    
    await page.fill('#input', notation);
    
    // Wait for VexFlow to render
    await page.waitForTimeout(500);
    
    // Check that VexFlow rendering completed without errors
    const vexflowSvg = await page.$('#vexflow-output svg');
    expect(vexflowSvg).toBeTruthy();
    
    // Get the VexFlow debug output
    const vexflowDebug = await page.$eval('#vexflow-debug', el => el.textContent);
    
    // Parse the JSON to check note values
    const vexflowData = JSON.parse(vexflowDebug);
    const notes = vexflowData[0].notes;
    
    // Check that scale degree 7 (index 6) is C#5 (cs/5), not cs/4
    expect(notes[6].Note.keys[0]).toBe('cs/5');
    
    // Check that octave up 7 (.7 at index 8) is also C#5
    expect(notes[8].Note.keys[0]).toBe('cs/5');
    
    // Check that the last note (7 at index 15) is also C#5
    expect(notes[15].Note.keys[0]).toBe('cs/5');
    
    // Verify the scale in D major
    expect(notes[0].Note.keys[0]).toBe('d/4'); // 1 -> D
    expect(notes[1].Note.keys[0]).toBe('e/4'); // 2 -> E
    expect(notes[2].Note.keys[0]).toBe('fs/4'); // 3 -> F#
    expect(notes[3].Note.keys[0]).toBe('g/4'); // 4 -> G
    expect(notes[4].Note.keys[0]).toBe('a/4'); // 5 -> A
    expect(notes[5].Note.keys[0]).toBe('b/4'); // 6 -> B
    expect(notes[6].Note.keys[0]).toBe('cs/5'); // 7 -> C# (octave above!)
    expect(notes[7].Note.keys[0]).toBe('d/5'); // .1 -> D (octave up)
  });

  test('correctly transposes in G major', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    await page.waitForSelector('#input');
    
    const notation = `key: G
time: 4/4

1 2 3 4 5 6 7`;
    
    await page.fill('#input', notation);
    await page.waitForTimeout(500);
    
    const vexflowDebug = await page.$eval('#vexflow-debug', el => el.textContent);
    const vexflowData = JSON.parse(vexflowDebug);
    const notes = vexflowData[0].notes;
    
    // Verify the scale in G major
    expect(notes[0].Note.keys[0]).toBe('g/4'); // 1 -> G
    expect(notes[1].Note.keys[0]).toBe('a/4'); // 2 -> A
    expect(notes[2].Note.keys[0]).toBe('b/4'); // 3 -> B
    expect(notes[3].Note.keys[0]).toBe('c/5'); // 4 -> C (octave above!)
    expect(notes[4].Note.keys[0]).toBe('d/5'); // 5 -> D (octave above!)
    expect(notes[5].Note.keys[0]).toBe('e/5'); // 6 -> E (octave above!)
    expect(notes[6].Note.keys[0]).toBe('fs/5'); // 7 -> F# (octave above!)
  });
});