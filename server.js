const express = require('express');
const cors = require('cors');
const fs = require('fs-extra');
const path = require('path');
const { exec } = require('child_process');
const { promisify } = require('util');

const execAsync = promisify(exec);
const app = express();
const port = process.env.PORT || 3000;

// Middleware
app.use(cors());
app.use(express.json({ limit: '10mb' }));
app.use(express.static('web'));
app.use('/pkg', express.static('pkg'));

// Convert ANSI colors to HTML
function ansiToHtml(ansiText) {
    return ansiText
        // Remove ANSI escape sequences and replace with HTML spans
        .replace(/\x1b\[1m(.*?)\x1b\[0m/g, '<strong>$1</strong>')  // Bold
        .replace(/\x1b\[4m(.*?)\x1b\[0m/g, '<u>$1</u>')  // Underline
        .replace(/\x1b\[1;4m(.*?)\x1b\[1;4m(.*?)\x1b\[0m/g, '<strong><u>$1$2</u></strong>')  // Bold + Underline
        .replace(/\x1b\[1;4;37m(.*?)\x1b\[1;4m(.*?)\x1b\[0m/g, '<span class="token-title"><strong><u>$1$2</u></strong></span>')  // Title
        .replace(/\x1b\[32m(.*?)\x1b\[0m/g, '<span class="token-barline">$1</span>')  // Green (barlines)
        .replace(/\x1b\[33m(.*?)\x1b\[0m/g, '<span class="token-pitch">$1</span>')  // Yellow (pitches)
        .replace(/\x1b\[4;33m(.*?)\x1b\[0m/g, '<span class="token-pitch"><u>$1</u></span>')  // Yellow underlined (pitches)
        .replace(/\x1b\[31m(.*?)\x1b\[0m/g, '<span class="token-symbols">$1</span>')  // Red (symbols)
        .replace(/\x1b\[38;2;165;142;142m(.*?)\x1b\[0m/g, '<span class="token-word">$1</span>')  // Brown (words) - updated RGB values
        .replace(/\x1b\[35m(.*?)\x1b\[0m/g, '<span class="token-unknown">$1</span>')  // Magenta (unknown)
        .replace(/\x1b\[35m(.*?)\x1b\[0m/g, '<span class="token-octave-marker">$1</span>')  // Magenta (octave markers)
        .replace(/\x1b\[34m(.*?)\x1b\[0m/g, '<span class="token-metadata">$1</span>')  // Blue (metadata)
        .replace(/\x1b\[48;2;50;50;50;37m(.*?)\x1b\[0m/g, '<span class="token-unassigned">$1</span>')  // Unassigned (reverse)
        .replace(/\x1b\[48;2;50;50;50;37m(.*?)\x1b\[0m/g, '<span class="token-whitespace">$1</span>')  // Whitespace (reverse)
        .replace(/\x1b\[48;2;50;50;50;37m(.*?)\x1b\[0m/g, '<span class="token-newline">$1</span>')  // Newline (reverse)
        .replace(/\x1b\[37m(.*?)\x1b\[0m/g, '<span>$1</span>')  // White/default
        // Clean up any remaining ANSI codes
        .replace(/\x1b\[[0-9;]*m/g, '');
}

// API endpoint to parse notation
app.post('/api/parse', async (req, res) => {
    const { notation, filename } = req.body;
    
    if (!notation || !filename) {
        return res.status(400).json({ 
            success: false, 
            error: 'Missing notation or filename' 
        });
    }

    const tempFile = path.join(__dirname, filename);
    
    try {
        // Write the notation to a temporary file
        await fs.writeFile(tempFile, notation);
        
        // Build the Rust project only if needed (skip for frequent requests)
        // Note: The binary should already be built from previous setup
        
        // Run the notation parser
        console.log(`Processing file: ${filename}`);
        const { stdout, stderr } = await execAsync(`cargo run --release --bin cli ${filename}`);
        
        if (stderr && stderr.includes('Error:')) {
            throw new Error(stderr);
        }
        
        // Get VexFlow FSM output
        let vexflowFsmOutput = null;
        try {
            console.log('Getting VexFlow FSM output...');
            const { stdout: vexflowStdout } = await execAsync(`cargo run --release --bin get_vexflow_fsm ${filename}`);
            // Parse only the first line as JSON (ignore any debug output that follows)
            const firstLine = vexflowStdout.split('\n')[0].trim();
            vexflowFsmOutput = JSON.parse(firstLine);
            console.log(`VexFlow FSM output: ${firstLine.length} chars`);
        } catch (vexflowError) {
            console.warn('Failed to get VexFlow FSM output:', vexflowError.message);
        }
        
        // Get document outline for debugging
        let documentOutline = null;
        try {
            const outlineFile = filename.replace('.123', '.outline');
            if (await fs.pathExists(outlineFile)) {
                documentOutline = await fs.readFile(outlineFile, 'utf8');
            }
        } catch (outlineError) {
            console.warn('Failed to get document outline:', outlineError.message);
        }
        
        // Get tokenized data for debugging
        let tokenizedData = null;
        try {
            const tokenFile = filename.replace('.123', '.lexer.json');
            if (await fs.pathExists(tokenFile)) {
                const tokenJson = await fs.readFile(tokenFile, 'utf8');
                tokenizedData = JSON.parse(tokenJson);
            }
        } catch (tokenError) {
            console.warn('Failed to get tokenized data:', tokenError.message);
        }
        
        // Get attached items data for debugging
        let attachedItemsData = null;
        try {
            const attachedFile = filename.replace('.123', '.flattener.yaml');
            if (await fs.pathExists(attachedFile)) {
                attachedItemsData = await fs.readFile(attachedFile, 'utf8');
            }
        } catch (attachedError) {
            console.warn('Failed to get attached items data:', attachedError.message);
        }
        
        // Read the colorized output file
        const colorizedFile = filename.replace('.123', '.flattener.clr');
        let colorizedOutput = '';
        
        if (await fs.pathExists(colorizedFile)) {
            const rawOutput = await fs.readFile(colorizedFile, 'utf8');
            colorizedOutput = ansiToHtml(rawOutput);
        } else {
            // Fallback to stdout if file doesn't exist
            colorizedOutput = ansiToHtml(stdout);
        }
        
        // Generate staff notation using LilyPond
        const lilypondFile = filename.replace('.123', '.ly');
        let staffNotationUrl = null;
        
        if (await fs.pathExists(lilypondFile)) {
            try {
                const baseFilename = filename.replace('.123', '');
                const pngFilename = `${baseFilename}.png`;
                
                // Generate PNG from LilyPond file
                await execAsync(`lilypond --png --output=${baseFilename} ${lilypondFile}`);
                
                if (await fs.pathExists(pngFilename)) {
                    staffNotationUrl = `/${pngFilename}`;
                }
            } catch (lilyError) {
                console.warn('LilyPond generation failed:', lilyError.message);
            }
        }
        
        // Try to detect the notation type from the output or parsing logic
        let notationType = 'Unknown';
        if (notation.match(/[SRGMPDNSrgmpdns]/)) {
            notationType = 'Sargam';
        } else if (notation.match(/[CDEFGAB]/)) {
            notationType = 'Western';
        } else if (notation.match(/[1-7]/)) {
            notationType = 'Number';
        }
        
        // Schedule PNG cleanup after 5 minutes
        if (staffNotationUrl) {
            const pngFile = staffNotationUrl.substring(1); // Remove leading /
            setTimeout(async () => {
                try {
                    if (await fs.pathExists(pngFile)) {
                        await fs.remove(pngFile);
                        console.log(`Cleaned up PNG: ${pngFile}`);
                    }
                } catch (err) {
                    console.warn(`Failed to cleanup PNG ${pngFile}:`, err.message);
                }
            }, 5 * 60 * 1000); // 5 minutes
        }
        
        res.json({
            success: true,
            colorizedOutput,
            staffNotationUrl,
            notationType,
            rawOutput: stdout,
            vexflowFsm: vexflowFsmOutput,
            documentOutline: documentOutline,
            tokenizedData: tokenizedData,
            attachedItemsData: attachedItemsData
        });
        
        // Clean up temporary files (keep PNG for serving) - moved after response
        await fs.remove(tempFile);
        const outputFiles = [
            filename.replace('.123', '.lexer.json'),
            filename.replace('.123', '.flattener.yaml'),
            filename.replace('.123', '.flattener.clr'),
            filename.replace('.123', '.tokenizer.clr'),
            filename.replace('.123', '.ly'),
            filename.replace('.123', '.outline')
        ];
        
        for (const file of outputFiles) {
            if (await fs.pathExists(file)) {
                await fs.remove(file);
            }
        }
        
    } catch (error) {
        console.error('Parser error:', error);
        
        // Clean up temp file on error
        if (await fs.pathExists(tempFile)) {
            await fs.remove(tempFile);
        }
        
        res.status(500).json({
            success: false,
            error: error.message || 'Unknown error occurred'
        });
    }
});

// LilyPond to PNG conversion endpoint (for WASM integration)
app.post('/api/lilypond-to-png', async (req, res) => {
    const { lilypondCode } = req.body;
    
    if (!lilypondCode || typeof lilypondCode !== 'string') {
        return res.status(400).json({
            success: false,
            error: 'LilyPond code is required'
        });
    }

    const timestamp = Date.now();
    const tempFile = path.join(__dirname, `temp_${timestamp}.ly`);
    const pngFile = path.join(__dirname, `temp_${timestamp}.png`);
    
    try {
        // Write LilyPond code to temporary file
        await fs.writeFile(tempFile, lilypondCode, 'utf8');
        
        // Generate PNG using LilyPond (it creates filename based on input file)
        console.log(`Executing: lilypond --png --output="${pngFile.replace('.png', '')}" "${tempFile}"`);
        const lilyResult = await execAsync(`lilypond --png --output="${pngFile.replace('.png', '')}" "${tempFile}"`);
        console.log('LilyPond output:', lilyResult);
        
        // LilyPond creates PNG with the same base name as the .ly file
        const actualPngFile = tempFile.replace('.ly', '.png');
        console.log(`Checking for PNG at: ${actualPngFile}`);
        if (await fs.pathExists(actualPngFile)) {
            // Return the PNG as URL
            const imageUrl = `/temp_${timestamp}.png`;
            
            res.json({
                success: true,
                imageUrl: imageUrl
            });
            
            // Clean up LilyPond file (keep PNG for serving)
            await fs.remove(tempFile);
        } else {
            throw new Error('LilyPond failed to generate PNG file');
        }
        
    } catch (error) {
        console.error('LilyPond PNG generation error:', error);
        
        // Clean up temporary files
        try {
            if (await fs.pathExists(tempFile)) await fs.remove(tempFile);
            const actualPngFile = tempFile.replace('.ly', '.png');
            if (await fs.pathExists(actualPngFile)) await fs.remove(actualPngFile);
        } catch (cleanupError) {
            console.error('Cleanup error:', cleanupError);
        }
        
        res.status(500).json({
            success: false,
            error: error.message || 'LilyPond generation failed'
        });
    }
});

// Health check endpoint
app.get('/api/health', (req, res) => {
    res.json({ 
        status: 'ok', 
        timestamp: new Date().toISOString(),
        version: '1.0.0'
    });
});

// Serve the main page
app.get('/', (req, res) => {
    res.sendFile(path.join(__dirname, 'index.html'));
});

// Server startup self-test
async function runServerSelfTest() {
    console.log('\n🧪 Running server startup self-test...\n');
    
    const testResults = [];
    let allTestsPassed = true;
    
    try {
        // Test 1: Check if required binaries are available
        try {
            await execAsync('cargo --version');
            testResults.push('✅ Cargo available');
        } catch (error) {
            testResults.push('❌ Cargo not found');
            allTestsPassed = false;
        }
        
        try {
            await execAsync('lilypond --version');
            testResults.push('✅ LilyPond available');
        } catch (error) {
            testResults.push('❌ LilyPond not found (staff notation generation will fail)');
            allTestsPassed = false;
        }
        
        // Test 2: Check if WASM files exist
        if (await fs.pathExists(path.join(__dirname, 'pkg/notation_parser.js'))) {
            testResults.push('✅ WASM JS module exists');
        } else {
            testResults.push('❌ WASM JS module missing');
            allTestsPassed = false;
        }
        
        if (await fs.pathExists(path.join(__dirname, 'pkg/notation_parser_bg.wasm'))) {
            testResults.push('✅ WASM binary exists');
        } else {
            testResults.push('❌ WASM binary missing');
            allTestsPassed = false;
        }
        
        // Test 3: Complex CLI parsing tests
        const testCases = [
            {
                name: 'Simple Sargam',
                input: '| S R G M |',
                expectedNotes: 'c4 d4 e4 fs4',
                expectedSystem: 'Sargam'
            },
            {
                name: 'Western notation',
                input: '| C D E F |',
                expectedNotes: 'c4 d4 e4 f4',
                expectedSystem: 'Western'
            },
            {
                name: 'Number notation',
                input: '| 1 2 3 4 |',
                expectedNotes: 'c4 d4 e4 f4',
                expectedSystem: 'Number'
            },
            {
                name: 'Multi-line with metadata',
                input: `Title: Test Song
Author: Test Author

| S R G M |
| P D N S |`,
                expectedNotes: 'c4 d4 e4 fs4',
                expectedSystem: 'Sargam'
            },
            {
                name: 'Chromatic with sharps and flats',
                input: '| C C# D Eb E F |',
                expectedNotes: 'c4 cs4 d4 ef4 e4 f4',
                expectedSystem: 'Western'
            },
            {
                name: 'Octave markers',
                input: `  .   :
| S R G P |
      .   :`,
                expectedNotes: 'c\'4 d4 e4 g4',
                expectedSystem: 'Sargam'
            },
            {
                name: 'Empty input with rests',
                input: 'Test Title\n\n|     |',
                expectedNotes: 'r4',
                expectedSystem: '???'
            }
        ];
        
        let passedTests = 0;
        for (const testCase of testCases) {
            try {
                const tempFile = path.join(__dirname, `server_test_${testCase.name.replace(/\s+/g, '_')}.123`);
                await fs.writeFile(tempFile, testCase.input);
                
                const { stdout, stderr } = await execAsync(`cargo run --release --bin cli ${tempFile}`);
                
                const lilyFile = tempFile.replace('.123', '.ly');
                if (await fs.pathExists(lilyFile)) {
                    const lilyContent = await fs.readFile(lilyFile, 'utf8');
                    
                    // Check for expected notes (or at least some musical content)
                    if (lilyContent.includes(testCase.expectedNotes) || 
                        (testCase.name.includes('Empty') && lilyContent.includes('r4'))) {
                        testResults.push(`✅ ${testCase.name}: CLI parsing works`);
                        passedTests++;
                    } else {
                        testResults.push(`❌ ${testCase.name}: Expected '${testCase.expectedNotes}' but got different output`);
                        allTestsPassed = false;
                    }
                    await fs.remove(lilyFile);
                } else {
                    testResults.push(`❌ ${testCase.name}: No LilyPond output generated`);
                    allTestsPassed = false;
                }
                
                // Clean up test files
                await fs.remove(tempFile);
                const outlineFile = tempFile.replace('.123', '.outline');
                if (await fs.pathExists(outlineFile)) await fs.remove(outlineFile);
                
            } catch (error) {
                testResults.push(`❌ ${testCase.name}: CLI parsing failed - ${error.message}`);
                allTestsPassed = false;
            }
        }
        
        testResults.push(`📊 Complex CLI tests: ${passedTests}/${testCases.length} passed`);
        
        // Test 4: Test LilyPond PNG generation
        try {
            const testLilyCode = `\\version "2.24.0"
\\language "english"
\\paper { tagline = ##f }
\\score {
  \\new Staff {
    \\fixed c' {
      \\clef treble
      c4 d4 e4 fs4
    }
  }
}`;
            
            const timestamp = Date.now();
            const tempFile = path.join(__dirname, `server_test_${timestamp}.ly`);
            await fs.writeFile(tempFile, testLilyCode);
            
            await execAsync(`lilypond --png --output="${tempFile.replace('.ly', '')}" "${tempFile}"`);
            
            const pngFile = tempFile.replace('.ly', '.png');
            if (await fs.pathExists(pngFile)) {
                testResults.push('✅ LilyPond PNG generation works');
                await fs.remove(pngFile);
            } else {
                testResults.push('❌ LilyPond PNG generation failed');
                allTestsPassed = false;
            }
            
            await fs.remove(tempFile);
            
        } catch (error) {
            testResults.push('❌ LilyPond PNG generation failed: ' + error.message);
            allTestsPassed = false;
        }
        
        // Test 5: Check if index.html exists
        if (await fs.pathExists(path.join(__dirname, 'index.html'))) {
            testResults.push('✅ Web interface available');
        } else {
            testResults.push('❌ Web interface missing');
            allTestsPassed = false;
        }
        
        // Display results
        console.log('Server Self-Test Results:');
        testResults.forEach(result => console.log(`  ${result}`));
        
        if (allTestsPassed) {
            console.log('\n🎉 All server tests passed! System ready to serve users.\n');
        } else {
            console.log('\n⚠️  Some server tests failed. System may have limited functionality.\n');
        }
        
    } catch (error) {
        console.error('\n❌ Server self-test encountered an error:', error.message);
        console.log('Partial test results:');
        testResults.forEach(result => console.log(`  ${result}`));
        console.log('\n⚠️  Server starting despite test failures...\n');
    }
}

// Start the server
app.listen(port, async () => {
    console.log(`🎵 Notation Parser Web Server running at http://localhost:${port}`);
    console.log(`📁 Serving files from: ${__dirname}`);
    console.log(`🎼 Ready to parse Sargam, Western, and Number notation!`);
    
    // Run self-test after server starts
    await runServerSelfTest();
});

// Graceful shutdown
process.on('SIGTERM', () => {
    console.log('🛑 Received SIGTERM, shutting down gracefully');
    process.exit(0);
});

process.on('SIGINT', () => {
    console.log('🛑 Received SIGINT, shutting down gracefully');
    process.exit(0);
});