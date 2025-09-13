#!/usr/bin/env node

// Node.js script to test font character widths
// Requires: npm install canvas (for server-side canvas rendering)

const { createCanvas } = require('canvas');

const fonts = [
    'JuliaMono',
    'Kurinto Mono', 
    'Cascadia Code',
    'DejaVu Sans Mono',
    'JetBrains Mono',
    'Source Code Pro',
    'Roboto Mono',
    'Courier New'
];

const testChars = [
    '-',  // ASCII dash
    '▬',  // Unicode dash replacement
    '.',  // ASCII dot
    '•',  // Unicode dot replacement  
    '|',  // ASCII pipe
    '┃',  // Unicode pipe replacement
    '1',  // Regular character for comparison
    'A'   // Another regular character
];

function measureCharWidth(char, fontFamily) {
    const canvas = createCanvas(100, 100);
    const ctx = canvas.getContext('2d');
    ctx.font = `bold 24px "${fontFamily}", monospace`;
    return ctx.measureText(char).width;
}

function testFont(fontFamily) {
    const widths = testChars.map(char => ({
        char,
        width: measureCharWidth(char, fontFamily)
    }));
    
    // Check if all widths are equal (within 0.1px tolerance)
    const baseWidth = widths[0].width;
    const tolerance = 0.1;
    const isMonospace = widths.every(({width}) => 
        Math.abs(width - baseWidth) <= tolerance
    );
    
    return {
        font: fontFamily,
        widths,
        isMonospace,
        baseWidth
    };
}

function runTests() {
    console.log('MONOSPACE UNICODE FONT TEST RESULTS');
    console.log('====================================\\n');
    
    const results = fonts.map(testFont);
    
    results.forEach(result => {
        const status = result.isMonospace ? '✅ PASS' : '❌ FAIL';
        console.log(`${result.font}: ${status}`);
        
        result.widths.forEach(({char, width}) => {
            console.log(`  ${char}: ${width.toFixed(2)}px`);
        });
        console.log(`  Base width: ${result.baseWidth.toFixed(2)}px`);
        console.log(`  Monospace: ${result.isMonospace}\\n`);
    });
    
    // Summary
    const passCount = results.filter(r => r.isMonospace).length;
    console.log(`SUMMARY: ${passCount}/${results.length} fonts maintain monospace alignment`);
    
    // Recommendations
    const bestFonts = results.filter(r => r.isMonospace).map(r => r.font);
    if (bestFonts.length > 0) {
        console.log(`\\nRECOMMENDED: ${bestFonts.join(', ')}`);
    }
    
    return results;
}

// Check if canvas module is available
try {
    runTests();
} catch (error) {
    console.log('❌ Canvas module not available. Run: npm install canvas');
    console.log('\\n🌐 Alternative: Open font-test.html in a web browser for visual testing');
}