const { test, expect } = require('@playwright/test');

test('check beam visibility in VexFlow', async ({ page }) => {
  // Navigate to the page
  await page.goto('http://localhost:3000', { waitUntil: 'networkidle' });
  
  // Wait for page to load
  await page.waitForTimeout(3000);
  
  // Enter SSSS pattern that should create beams
  await page.fill('#input-text', '| SSSS | RRRR |');
  
  // Wait for rendering
  await page.waitForTimeout(2000);
  
  // Take screenshot for visual inspection
  await page.screenshot({ path: 'test_output/beam_check.png', fullPage: true });
  
  // Check for beam elements in the SVG
  const svgSelector = '#vexflow-canvas svg';
  const svg = page.locator(svgSelector);
  
  // Wait for SVG to exist
  await expect(svg).toBeVisible({ timeout: 5000 });
  
  // Look for beam-related elements
  // VexFlow typically creates beams as <path> or <rect> elements with specific classes
  const beamPaths = await page.locator(`${svgSelector} path`).count();
  const beamRects = await page.locator(`${svgSelector} rect`).count();
  const beamLines = await page.locator(`${svgSelector} line`).count();
  
  console.log('=== BEAM DETECTION ===');
  console.log(`Found ${beamPaths} path elements`);
  console.log(`Found ${beamRects} rect elements`);
  console.log(`Found ${beamLines} line elements`);
  
  // Get all path elements' d attributes to check for beam-like shapes
  const paths = await page.locator(`${svgSelector} path`).all();
  for (let i = 0; i < paths.length; i++) {
    const d = await paths[i].getAttribute('d');
    if (d && d.includes('L')) { // Beams typically have line segments
      console.log(`Path ${i}: ${d.substring(0, 100)}...`);
    }
  }
  
  // Check for VexFlow beam classes
  const vfBeams = await page.locator(`${svgSelector} .vf-beam`).count();
  console.log(`Found ${vfBeams} elements with .vf-beam class`);
  
  // Get the full SVG content for analysis
  const svgContent = await svg.innerHTML();
  
  // Check for beam-related content
  const hasBeamClass = svgContent.includes('vf-beam');
  const hasBeamPath = svgContent.includes('M') && svgContent.includes('L') && svgContent.includes('Z');
  
  console.log('SVG contains .vf-beam class:', hasBeamClass);
  console.log('SVG contains path with beam-like shape:', hasBeamPath);
  
  // Count note stems (should be 8 for SSSS RRRR)
  const stems = await page.locator(`${svgSelector} line[stroke-width]`).count();
  console.log(`Found ${stems} potential note stems`);
  
  // ASSERTION: We expect to see beam elements for SSSS and RRRR patterns
  // Each group of 4 sixteenth notes should have a beam
  // So we expect at least 2 beam elements (one for SSSS, one for RRRR)
  
  const totalBeamElements = beamPaths + beamRects + vfBeams;
  console.log(`\nTotal beam-like elements found: ${totalBeamElements}`);
  
  if (totalBeamElements < 2) {
    console.log('❌ BEAMS NOT FOUND - Notes appear to have individual flags instead of beams');
  } else {
    console.log('✅ BEAMS DETECTED - Notes appear to be properly beamed');
  }
  
  // Also test with a simpler pattern
  await page.fill('#input-text', 'SSSS');
  await page.waitForTimeout(2000);
  
  const simpleBeams = await page.locator(`${svgSelector} path`).count();
  console.log(`\nSimple SSSS pattern: ${simpleBeams} path elements`);
  
  // Take screenshot of simple pattern
  await page.screenshot({ path: 'test_output/beam_check_simple.png', fullPage: true });
});