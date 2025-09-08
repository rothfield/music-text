const express = require('express');
const cors = require('cors');
const fs = require('fs-extra');
const path = require('path');
const { exec, spawn } = require('child_process');
const { promisify } = require('util');
const WebSocket = require('ws');
const chokidar = require('chokidar');

const execAsync = promisify(exec);
const app = express();
const port = process.env.PORT || 8000; // Use port 8000 for webapp, 3000 for pest parser

// Middleware
app.use(cors());
app.use(express.json({ limit: '10mb' }));

// Add no-cache headers for WASM files
app.use('/pkg', (req, res, next) => {
    // Set no-cache headers for all pkg files
    res.set({
        'Cache-Control': 'no-store, no-cache, must-revalidate, proxy-revalidate',
        'Pragma': 'no-cache',
        'Expires': '0',
        'Surrogate-Control': 'no-store'
    });
    next();
}, express.static(path.join(__dirname, 'pkg')));

app.use(express.static(path.join(__dirname, 'public')));

// Serve favicon
app.get('/favicon.ico', (req, res) => {
    res.sendFile(path.join(__dirname, 'favicon.ico'));
});

// Serve temp SVG files
app.get('/temp_*.svg', (req, res) => {
    const filename = req.path.substring(1); // Remove leading /
    const filePath = path.join(__dirname, filename);
    if (fs.existsSync(filePath) && filename.startsWith('temp_') && filename.endsWith('.svg')) {
        res.setHeader('Content-Type', 'image/svg+xml');
        res.sendFile(filePath);
    } else {
        res.status(404).send('File not found');
    }
});

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

// Proxy parse requests to the pest parser server
app.post('/api/parse', async (req, res) => {
    try {
        // Forward the request to the pest parser server on port 3000
        const pestParserUrl = 'http://127.0.0.1:3000/api/parse';
        
        const response = await fetch(pestParserUrl, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(req.body)
        });
        
        const result = await response.json();
        
        if (!response.ok) {
            return res.status(response.status).json(result);
        }
        
        // Return the pest parser result with additional webapp-compatible fields
        res.json({
            success: result.success,
            error: result.error,
            // Keep existing webapp format for compatibility
            colorizedOutput: result.yaml || 'No YAML output available',
            notationType: getNotationType(req.body.notation || ''),
            rawOutput: JSON.stringify(result.ast, null, 2),
            // New pest parser fields
            ast: result.ast,
            yaml: result.yaml,
            vexflow: result.vexflow,
            lilypond: result.lilypond,
            // Legacy fields for backward compatibility
            vexflowFsm: null, // Will be replaced by result.vexflow
            documentOutline: null,
            tokenizedData: null,
            attachedItemsData: null,
            staffNotationUrl: null // TODO: Implement LilyPond PNG generation
        });
        
    } catch (error) {
        console.error('API proxy error:', error);
        res.status(500).json({ 
            success: false, 
            error: `Failed to connect to pest parser: ${error.message}` 
        });
    }
});

// Helper function to detect notation type
function getNotationType(notation) {
    if (notation.match(/[SRGMPDNSrgmpdns]/)) {
        return 'Sargam';
    } else if (notation.match(/[CDEFGAB]/)) {
        return 'Western';  
    } else if (notation.match(/[1-7]/)) {
        return 'Number';
    } else {
        return 'Unknown';
    }
}
        const lilypondFile = path.join('test_output', path.basename(filename).replace('.123', '.ly'));
        let staffNotationUrl = null;
        
        if (await fs.pathExists(lilypondFile)) {
            try {
                const baseFilename = path.basename(filename).replace('.123', '');
                const outputPath = path.join(__dirname, baseFilename);
                const pngFilename = `${outputPath}.png`;
                
                // Generate PNG from LilyPond file
                await execAsync(`lilypond --png --output="${outputPath}" "${lilypondFile}"`);
                
                if (await fs.pathExists(pngFilename)) {
                    // Crop the PNG to remove excess whitespace using netpbm tools (like doremi-script)
                    try {
                        const stemFile = pngFilename.replace('.png', '');
                        // Convert PNG to PNM, crop whitespace, convert back to PNG
                        await execAsync(`pngtopnm "${pngFilename}" > "${stemFile}.pnm"`);
                        await execAsync(`pnmcrop -white "${stemFile}.pnm" > "${stemFile}-cropped.pnm"`);
                        await execAsync(`pnmtopng "${stemFile}-cropped.pnm" > "${pngFilename}"`);
                        // Clean up temp files
                        await execAsync(`rm -f "${stemFile}.pnm" "${stemFile}-cropped.pnm"`);
                        console.log(`Cropped PNG using netpbm: ${pngFilename}`);
                    } catch (cropError) {
                        console.warn('netpbm crop failed, trying ImageMagick fallback:', cropError.message);
                        // Fallback to ImageMagick
                        try {
                            await execAsync(`magick "${pngFilename}" -trim +repage "${pngFilename}"`);
                            console.log(`Cropped PNG with ImageMagick: ${pngFilename}`);
                        } catch (magickError) {
                            console.warn('All cropping methods failed, using uncropped PNG:', magickError.message);
                        }
                    }
                    staffNotationUrl = `/${baseFilename}.png`;
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
            path.join('test_output', path.basename(filename).replace('.123', '.lexer.json')),
            path.join('test_output', path.basename(filename).replace('.123', '.flattener.yaml')),
            path.join('test_output', path.basename(filename).replace('.123', '.ly')),
            path.join('test_output', path.basename(filename).replace('.123', '.outline'))
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

// Get LilyPond source code endpoint
app.post('/api/lilypond-source', async (req, res) => {
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
        
        // Run the notation parser to generate LilyPond file
        console.log(`Processing file for LilyPond source: ${filename}`);
        const { stdout, stderr } = await execAsync(`NOTATION_OUTPUT_DIR="test_output" cargo run --release --bin cli ${filename}`);
        
        if (stderr && stderr.includes('Error:')) {
            throw new Error(stderr);
        }
        
        // Get LilyPond source
        const lilypondFile = path.join('test_output', filename.replace('.123', '.ly'));
        let lilypondSource = null;
        
        if (await fs.pathExists(lilypondFile)) {
            lilypondSource = await fs.readFile(lilypondFile, 'utf8');
        }
        
        if (!lilypondSource) {
            throw new Error('No LilyPond source generated');
        }
        
        res.json({
            success: true,
            lilypondSource: lilypondSource
        });
        
        // Clean up temporary files
        await fs.remove(tempFile);
        if (await fs.pathExists(lilypondFile)) {
            await fs.remove(lilypondFile);
        }
        
    } catch (error) {
        console.error('LilyPond source generation error:', error);
        
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

// In-memory LilyPond to SVG conversion endpoint (no disk writes)
app.post('/api/lilypond-to-svg-stream', (req, res) => {
    const { lilypondCode } = req.body;
    
    if (!lilypondCode || typeof lilypondCode !== 'string') {
        return res.status(400).json({
            success: false,
            error: 'LilyPond code is required'
        });
    }

    console.log('Processing LilyPond code in-memory (no disk writes)...');
    
    // Use a unique temporary filename in memory (Linux /dev/shm is RAM-based)
    const tempId = Date.now() + '_' + Math.random().toString(36).substr(2, 9);
    const useMemFs = process.platform === 'linux' && fs.existsSync('/dev/shm');
    const tempDir = useMemFs ? '/dev/shm' : require('os').tmpdir();
    const tempBase = path.join(tempDir, `lily_${tempId}`);
    const tempLy = `${tempBase}.ly`;
    const tempSvg = `${tempBase}.svg`;
    
    // Write LilyPond code to temp file (in RAM if available)
    fs.writeFileSync(tempLy, lilypondCode);
    
    // Spawn lilypond process
    const lilypond = spawn('lilypond', [
        '--svg',
        '-dno-point-and-click', 
        '-o', tempBase,       // Output base name
        tempLy                // Input file
    ]);
    
    let svgData = '';
    let errorData = '';
    
    lilypond.stdout.on('data', (data) => {
        svgData += data.toString();
    });
    
    lilypond.stderr.on('data', (data) => {
        errorData += data.toString();
    });
    
    lilypond.on('close', async (code) => {
        try {
            if (code === 0 && fs.existsSync(tempSvg)) {
                // Read the SVG file from memory/temp
                const svgContent = fs.readFileSync(tempSvg, 'utf8');
                
                // Clean up temp files immediately
                fs.unlinkSync(tempLy);
                fs.unlinkSync(tempSvg);
                
                console.log(`Generated SVG ${useMemFs ? 'in RAM' : 'via temp'}: ${svgContent.length} bytes`);
                
                // Convert to base64 data URL for easy embedding
                const svgBase64 = Buffer.from(svgContent).toString('base64');
                const dataUrl = `data:image/svg+xml;base64,${svgBase64}`;
                
                res.json({ 
                    success: true, 
                    svg: svgContent,        // Raw SVG string
                    dataUrl: dataUrl,       // Base64 data URL for direct embedding
                    sizeBytes: svgContent.length,
                    usedRam: useMemFs      // Whether we used RAM-based filesystem
                });
            } else {
                // Clean up temp file if it exists
                if (fs.existsSync(tempLy)) fs.unlinkSync(tempLy);
                if (fs.existsSync(tempSvg)) fs.unlinkSync(tempSvg);
                
                console.error('LilyPond generation failed:', errorData);
                res.status(500).json({ 
                    success: false, 
                    error: errorData || 'Failed to generate SVG',
                    exitCode: code
                });
            }
        } catch (err) {
            // Clean up on error
            try {
                if (fs.existsSync(tempLy)) fs.unlinkSync(tempLy);
                if (fs.existsSync(tempSvg)) fs.unlinkSync(tempSvg);
            } catch (cleanupErr) {
                console.error('Cleanup error:', cleanupErr);
            }
            
            res.status(500).json({
                success: false,
                error: 'Error processing SVG: ' + err.message
            });
        }
    });
    
    lilypond.on('error', (err) => {
        // Clean up on error
        try {
            if (fs.existsSync(tempLy)) fs.unlinkSync(tempLy);
        } catch (cleanupErr) {
            console.error('Cleanup error:', cleanupErr);
        }
        
        console.error('Failed to spawn LilyPond:', err);
        res.status(500).json({
            success: false,
            error: 'LilyPond not found or failed to start: ' + err.message
        });
    });
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
    const svgFile = path.join(__dirname, `temp_${timestamp}.svg`);
    
    try {
        // Write LilyPond code to temporary file
        await fs.writeFile(tempFile, lilypondCode, 'utf8');
        
        // Generate SVG using LilyPond (it creates filename based on input file)
        console.log(`Executing: lilypond -dbackend=svg -dno-point-and-click --output="${svgFile.replace('.svg', '')}" "${tempFile}"`);
        console.log('LilyPond input code:', lilypondCode);
        const lilyResult = await execAsync(`lilypond -dbackend=svg -dno-point-and-click --output="${svgFile.replace('.svg', '')}" "${tempFile}"`);
        console.log('LilyPond output:', lilyResult);
        
        // LilyPond creates SVG with the same base name as the .ly file
        const actualSvgFile = tempFile.replace('.ly', '.svg');
        console.log(`Checking for SVG at: ${actualSvgFile}`);
        if (await fs.pathExists(actualSvgFile)) {
            // Return the SVG as URL
            const imageUrl = `/temp_${timestamp}.svg`;
            
            res.json({
                success: true,
                imageUrl: imageUrl
            });
            
            // Clean up LilyPond file (keep SVG for serving)
            await fs.remove(tempFile);
        } else {
            throw new Error('LilyPond failed to generate SVG file');
        }
        
    } catch (error) {
        console.error('LilyPond SVG generation error:', error);
        console.error('LilyPond stderr:', error.stderr);
        console.error('LilyPond stdout:', error.stdout);
        
        // Clean up temporary files
        try {
            if (await fs.pathExists(tempFile)) await fs.remove(tempFile);
            const actualSvgFile = tempFile.replace('.ly', '.svg');
            if (await fs.pathExists(actualSvgFile)) await fs.remove(actualSvgFile);
        } catch (cleanupError) {
            console.error('Cleanup error:', cleanupError);
        }
        
        res.status(500).json({
            success: false,
            error: error.message || 'LilyPond generation failed',
            stderr: error.stderr || 'No stderr available',
            stdout: error.stdout || 'No stdout available'
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
    res.sendFile(path.join(__dirname, 'public', 'index.html'));
});

// Legacy server self-tests removed - using WASM for all parsing now
// Only server-side functionality is LilyPond PNG generation from WASM-generated .ly content

/*
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
            testResults.push('ℹ️ LilyPond not found (staff notation generation disabled)');
        }
        
        // Test 2: Check if WASM files exist (in pkg/ where server actually serves them)
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
                name: 'S--S lilypond self test',
                input: 'S--S',
                expectedNotes: 'c16. c32',  // 3/16 + 1/16 durations
                expectedSystem: 'Sargam'
            },
            {
                name: 'S--S -S tie test',
                input: 'S--S -S',
                expectedNotes: 'c16. c32~ c16 c16',  // Beat 1: 3/16 + 1/16~, Beat 2: tied 1/4 + 1/4
                expectedSystem: 'Sargam'
            },
        ];
        
        let passedTests = 0;
        
        // Create temporary directory for test files
        const tempDir = path.join(__dirname, 'temp_server_tests');
        await fs.ensureDir(tempDir);
        
        for (const testCase of testCases) {
            try {
                const tempFile = path.join(tempDir, `server_test_${testCase.name.replace(/\s+/g, '_').replace(/[()]/g, '')}.123`);
                await fs.writeFile(tempFile, testCase.input);
                
                const { stdout, stderr } = await execAsync(`timeout 30s bash -c 'NOTATION_OUTPUT_DIR="${tempDir}" cargo run --release --bin cli ${tempFile}'`);
                
                // Add small delay to ensure file is fully written
                await new Promise(resolve => setTimeout(resolve, 100));
                
                // Check for lexer JSON output (this is what CLI actually produces)
                const jsonFile = tempFile.replace('.123', '.lexer.json');
                if (await fs.pathExists(jsonFile)) {
                    const jsonContent = await fs.readFile(jsonFile, 'utf8');
                    
                    // Check if JSON contains valid parsing results
                    try {
                        const parsedJson = JSON.parse(jsonContent);
                        if (parsedJson && Array.isArray(parsedJson) && parsedJson.length > 0) {
                            testResults.push(`✅ ${testCase.name}: JSON output generated`);
                            
                            // Also check LilyPond output if we have expected notes
                            if (testCase.expectedNotes) {
                                const lyFile = path.join(tempDir, baseName + '.ly');
                                if (await fs.pathExists(lyFile)) {
                                    const lyContent = await fs.readFile(lyFile, 'utf8');
                                    const lyMatch = lyContent.match(/\\fixed c' \{[^}]*\}/);
                                    if (lyMatch) {
                                        const notesLine = lyMatch[0];
                                        const actualNotes = notesLine.match(/\\clef treble\s+([^}]+)/);
                                        if (actualNotes) {
                                            const actualNotesStr = actualNotes[1].trim();
                                            if (actualNotesStr === testCase.expectedNotes) {
                                                testResults.push(`✅ ${testCase.name}: LilyPond notes match expected`);
                                            } else {
                                                testResults.push(`❌ ${testCase.name}: LilyPond notes mismatch`);
                                                testResults.push(`   Expected: ${testCase.expectedNotes}`);
                                                testResults.push(`   Actual:   ${actualNotesStr}`);
                                                allTestsPassed = false;
                                            }
                                        } else {
                                            testResults.push(`❌ ${testCase.name}: Could not extract notes from LilyPond`);
                                            allTestsPassed = false;
                                        }
                                    } else {
                                        testResults.push(`❌ ${testCase.name}: Invalid LilyPond format`);
                                        allTestsPassed = false;
                                    }
                                } else {
                                    testResults.push(`❌ ${testCase.name}: No LilyPond file generated`);
                                    allTestsPassed = false;
                                }
                            }
                            
                            passedTests++;
                        } else {
                            testResults.push(`❌ ${testCase.name}: JSON output has no valid tokens`);
                            allTestsPassed = false;
                        }
                    } catch (e) {
                        testResults.push(`❌ ${testCase.name}: Invalid JSON output generated`);
                        allTestsPassed = false;
                    }
                    await fs.remove(jsonFile);
                } else {
                    testResults.push(`❌ ${testCase.name}: No JSON output generated`);
                    allTestsPassed = false;
                }
                
                // Clean up individual test files as we go
                await fs.remove(tempFile);
                const baseName = path.basename(tempFile, '.123');
                const extensions = ['.ly', '.outline', '.flattener.yaml', '.lexer.json'];
                for (const ext of extensions) {
                    const outputFile = path.join(tempDir, baseName + ext);
                    if (await fs.pathExists(outputFile)) {
                        await fs.remove(outputFile);
                    }
                }
                
            } catch (error) {
                testResults.push(`❌ ${testCase.name}: CLI parsing failed - ${error.message}`);
                allTestsPassed = false;
            }
        }
        
        testResults.push(`📊 Complex CLI tests: ${passedTests}/${testCases.length} passed`);
        
        
        // Test 5: Check if index.html exists
        if (await fs.pathExists(path.join(__dirname, 'public', 'index.html'))) {
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
        
        // Clean up temporary directory
        try {
            if (await fs.pathExists(tempDir)) {
                await fs.remove(tempDir);
            }
        } catch (cleanupError) {
            console.warn('⚠️  Failed to clean up temp directory:', cleanupError.message);
        }
        
    } catch (error) {
        console.error('\n❌ Server self-test encountered an error:', error.message);
        console.log('Partial test results:');
        testResults.forEach(result => console.log(`  ${result}`));
        console.log('\n⚠️  Server starting despite test failures...\n');
        
        // Clean up temporary directory even on error
        try {
            const tempDir = path.join(__dirname, 'temp_server_tests');
            if (await fs.pathExists(tempDir)) {
                await fs.remove(tempDir);
            }
        } catch (cleanupError) {
            console.warn('⚠️  Failed to clean up temp directory after error:', cleanupError.message);
        }
    }
}
*/

// Start the server
const server = app.listen(port, () => {
    console.log(`🎵 Notation Parser Web Server running at http://localhost:${port}`);
    console.log(`📁 Serving files from: ${__dirname}`);
    console.log(`🎼 Ready to parse Sargam, Western, and Number notation!`);
    console.log('✨ All notation parsing now handled by WASM module');
    console.log('📄 Server only handles LilyPond PNG generation');
});

// WebSocket server for WASM auto-reload during development
// Watches /pkg directory and notifies connected clients when WASM files change
const wss = new WebSocket.Server({ server });

const clients = new Set();

wss.on('connection', (ws) => {
    console.log('🔌 WebSocket client connected for auto-reload');
    clients.add(ws);
    
    ws.on('close', () => {
        console.log('🔌 WebSocket client disconnected');
        clients.delete(ws);
    });
    
    ws.on('error', (error) => {
        console.error('WebSocket error:', error);
        clients.delete(ws);
    });
});

// File watcher for WASM files
const wasmPath = path.join(__dirname, 'pkg');
console.log(`🔍 Watching WASM files in: ${wasmPath}`);

const watcher = chokidar.watch(wasmPath, {
    ignored: /node_modules/,
    persistent: true,
    ignoreInitial: true
});

watcher.on('change', (filePath) => {
    const fileName = path.basename(filePath);
    if (fileName.endsWith('.wasm') || fileName.endsWith('.js')) {
        console.log(`🔄 WASM file changed: ${fileName}`);
        
        // Notify all connected clients to reload WASM
        clients.forEach((client) => {
            if (client.readyState === WebSocket.OPEN) {
                client.send(JSON.stringify({
                    type: 'wasm-reload',
                    file: fileName,
                    timestamp: Date.now()
                }));
            }
        });
        
        console.log(`📡 Notified ${clients.size} clients to reload WASM`);
    }
});

watcher.on('error', (error) => {
    console.error('❌ File watcher error:', error);
});

console.log('🔄 WASM auto-reload system enabled');

// Graceful shutdown
process.on('SIGTERM', () => {
    console.log('🛑 Received SIGTERM, shutting down gracefully');
    process.exit(0);
});

process.on('SIGINT', () => {
    console.log('🛑 Received SIGINT, shutting down gracefully');
    process.exit(0);
});