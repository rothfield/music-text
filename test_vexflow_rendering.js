#!/usr/bin/env node
/**
 * Server-side VexFlow rendering tests with real Canvas support
 * Uses node-canvas to provide actual Canvas API for VexFlow
 * Tests both JSON validation AND actual rendering capabilities
 */

const http = require('http');
const { createCanvas } = require('canvas');
const { JSDOM } = require('jsdom');
const fs = require('fs');

// Server configuration
const SERVER_BASE = 'http://127.0.0.1:3000';
const API_ENDPOINT = `${SERVER_BASE}/api/parse`;

class VexFlowRenderingTester {
    constructor() {
        this.testResults = [];
        this.vexFlowLoaded = false;
        this.setupCanvasEnvironment();
    }
    
    setupCanvasEnvironment() {
        // Create DOM environment with Canvas support
        const dom = new JSDOM('<!DOCTYPE html><html><body><div id="vex-container"></div></body></html>', {
            pretendToBeVisual: true,
            resources: "usable"
        });
        
        // Set up globals
        global.window = dom.window;
        global.document = dom.window.document;
        global.HTMLElement = dom.window.HTMLElement;
        global.SVGElement = dom.window.SVGElement;
        
        // Set navigator property with defineProperty to avoid setter errors
        Object.defineProperty(global, 'navigator', {
            value: dom.window.navigator,
            writable: true,
            configurable: true
        });
        
        // Override HTMLCanvasElement with node-canvas
        global.HTMLCanvasElement = function(width = 800, height = 400) {
            const canvas = createCanvas(width, height);
            canvas.style = {}; // Add style property that VexFlow expects
            return canvas;
        };
        
        // Add canvas creation to document
        const originalCreateElement = dom.window.document.createElement;
        dom.window.document.createElement = function(tagName) {
            if (tagName.toLowerCase() === 'canvas') {
                return new global.HTMLCanvasElement();
            }
            return originalCreateElement.call(this, tagName);
        };
        
        console.log('✅ Canvas environment set up with node-canvas support');
    }
    
    async loadVexFlow() {
        if (this.vexFlowLoaded) return true;
        
        try {
            console.log('🎵 Loading VexFlow with Canvas support...');
            
            // Load VexFlow code
            const vexflowPath = './webapp/assets/vexflow4.js';
            if (!fs.existsSync(vexflowPath)) {
                throw new Error('VexFlow file not found at ' + vexflowPath);
            }
            
            const VexFlowCode = fs.readFileSync(vexflowPath, 'utf8');
            
            // Execute VexFlow in our environment
            eval(VexFlowCode);
            
            // Check if VexFlow is available
            if (global.window.Vex && global.window.Vex.Flow) {
                global.Vex = global.window.Vex;
                this.vexFlowLoaded = true;
                console.log('✅ VexFlow loaded successfully with Canvas support');
                console.log(`   Available classes: ${Object.keys(global.Vex.Flow).slice(0, 10).join(', ')}...`);
                return true;
            } else {
                console.error('❌ VexFlow not accessible after loading');
                return false;
            }
        } catch (error) {
            console.error('❌ Failed to load VexFlow:', error.message);
            return false;
        }
    }
    
    async fetchAPIData(input) {
        return new Promise((resolve, reject) => {
            const url = `${API_ENDPOINT}?input=${encodeURIComponent(input)}`;
            
            const req = http.get(url, (res) => {
                let data = '';
                res.on('data', chunk => data += chunk);
                res.on('end', () => {
                    try {
                        const jsonData = JSON.parse(data);
                        resolve({
                            statusCode: res.statusCode,
                            data: jsonData,
                            responseSize: data.length
                        });
                    } catch (error) {
                        reject(new Error(`Failed to parse JSON: ${error.message}`));
                    }
                });
            });
            
            req.on('error', reject);
            req.setTimeout(10000, () => {
                req.destroy();
                reject(new Error('Request timeout'));
            });
        });
    }
    
    async testVexFlowRendering(vexflowData, testName) {
        if (!this.vexFlowLoaded) {
            return {
                success: false,
                error: 'VexFlow not loaded',
                details: {}
            };
        }
        
        try {
            const { Renderer, Stave, StaveNote, Formatter, Voice, Beam, Tuplet } = global.Vex.Flow;
            
            // Create canvas
            const canvas = createCanvas(800, 300);
            const context = canvas.getContext('2d');
            
            // Create VexFlow renderer
            const renderer = new Renderer(canvas, Renderer.Backends.CANVAS);
            renderer.resize(800, 300);
            const vfContext = renderer.getContext();
            
            // Rendering stats
            const stats = {
                notesCreated: 0,
                staveElements: 0,
                tupletsCreated: 0,
                beamsCreated: 0,
                renderingOperations: 0
            };
            
            // Create stave
            const stave = new Stave(10, 50, 750);
            stave.addClef(vexflowData.clef || 'treble');
            stave.addTimeSignature(vexflowData.time_signature || '4/4');
            stave.setContext(vfContext);
            stave.draw();
            stats.renderingOperations++;
            
            // Process staves from VexFlow data
            const allNotes = [];
            const allTuplets = [];
            
            if (vexflowData.staves && vexflowData.staves.length > 0) {
                const firstStave = vexflowData.staves[0];
                
                for (const element of firstStave.notes || []) {
                    stats.staveElements++;
                    
                    if (element.type === 'Note') {
                        try {
                            const noteSpec = {
                                clef: vexflowData.clef || 'treble',
                                keys: element.keys || ['c/4'],
                                duration: element.duration || 'q'
                            };
                            
                            const note = new StaveNote(noteSpec);
                            
                            // Add dots if specified
                            if (element.dots > 0) {
                                for (let i = 0; i < element.dots; i++) {
                                    note.addDotToAll();
                                }
                            }
                            
                            allNotes.push(note);
                            stats.notesCreated++;
                            
                        } catch (error) {
                            console.warn(`Warning: Failed to create note:`, element, error.message);
                        }
                    } else if (element.type === 'Rest') {
                        try {
                            const rest = new StaveNote({
                                clef: vexflowData.clef || 'treble',
                                keys: ['d/5'],
                                duration: (element.duration || 'q') + 'r'
                            });
                            
                            allNotes.push(rest);
                            stats.notesCreated++;
                            
                        } catch (error) {
                            console.warn(`Warning: Failed to create rest:`, element, error.message);
                        }
                    } else if (element.type === 'Tuplet' && element.notes) {
                        // Handle tuplets
                        const tupletNotes = [];
                        
                        for (const tupletNote of element.notes) {
                            if (tupletNote.type === 'Note') {
                                try {
                                    const note = new StaveNote({
                                        clef: vexflowData.clef || 'treble',
                                        keys: tupletNote.keys || ['c/4'],
                                        duration: tupletNote.duration || '8'
                                    });
                                    tupletNotes.push(note);
                                    stats.notesCreated++;
                                } catch (error) {
                                    console.warn(`Warning: Failed to create tuplet note:`, tupletNote, error.message);
                                }
                            }
                        }
                        
                        if (tupletNotes.length > 0) {
                            allNotes.push(...tupletNotes);
                            
                            // Create tuplet if we have ratio information
                            if (element.ratio && element.ratio.length === 2) {
                                try {
                                    const tuplet = new Tuplet(tupletNotes, {
                                        notes_occupied: element.ratio[1],
                                        num_notes: element.ratio[0]
                                    });
                                    tuplet.setContext(vfContext);
                                    allTuplets.push(tuplet);
                                    stats.tupletsCreated++;
                                } catch (error) {
                                    console.warn(`Warning: Failed to create tuplet:`, error.message);
                                }
                            }
                        }
                    }
                }
            }
            
            // Create voice and format if we have notes
            if (allNotes.length > 0) {
                const voice = new Voice({
                    num_beats: 4,
                    beat_value: 4,
                    resolution: global.Vex.Flow.RESOLUTION
                });
                
                voice.setStrict(false);
                voice.addTickables(allNotes);
                
                // Format the notes
                const formatter = new Formatter();
                formatter.joinVoices([voice]).format([voice], 700);
                
                // Draw voice
                voice.draw(vfContext, stave);
                stats.renderingOperations += allNotes.length;
                
                // Draw tuplets
                for (const tuplet of allTuplets) {
                    tuplet.draw();
                    stats.renderingOperations++;
                }
            }
            
            // Generate canvas output
            const imageBuffer = canvas.toBuffer('image/png');
            const base64Image = imageBuffer.toString('base64');
            
            return {
                success: true,
                stats,
                canvasWidth: canvas.width,
                canvasHeight: canvas.height,
                imageSize: imageBuffer.length,
                base64Preview: base64Image.substring(0, 100) + '...',
                details: {
                    vexflowVersion: 'VexFlow 4.2.2',
                    canvasBackend: 'node-canvas',
                    contextType: '2d'
                }
            };
            
        } catch (error) {
            return {
                success: false,
                error: error.message,
                stack: error.stack,
                details: {}
            };
        }
    }
    
    validateVexFlowJSON(vexflowData) {
        const errors = [];
        const warnings = [];
        
        if (!vexflowData) {
            return { valid: false, errors: ['VexFlow data is null'], warnings };
        }
        
        // Validate structure
        const required = ['clef', 'key_signature', 'staves', 'time_signature'];
        for (const field of required) {
            if (!(field in vexflowData)) {
                errors.push(`Missing field: ${field}`);
            }
        }
        
        // Validate staves
        if (!Array.isArray(vexflowData.staves)) {
            errors.push('staves must be an array');
        } else {
            vexflowData.staves.forEach((stave, idx) => {
                if (!stave.notes || !Array.isArray(stave.notes)) {
                    errors.push(`Stave ${idx} missing notes array`);
                } else {
                    let noteCount = 0, tupletCount = 0, restCount = 0, barlineCount = 0;
                    
                    stave.notes.forEach(note => {
                        switch (note.type) {
                            case 'Note': noteCount++; break;
                            case 'Rest': restCount++; break;
                            case 'Tuplet': tupletCount++; break;
                            case 'BarLine': barlineCount++; break;
                        }
                    });
                    
                    if (noteCount === 0 && restCount === 0 && tupletCount === 0) {
                        warnings.push(`Stave ${idx} has no musical content`);
                    }
                }
            });
        }
        
        return {
            valid: errors.length === 0,
            errors,
            warnings
        };
    }
    
    async runComprehensiveRenderingTests() {
        console.log('🧪 Starting comprehensive VexFlow rendering tests...');
        console.log('=' * 60);
        
        // Check server availability
        console.log('🌐 Checking server availability...');
        try {
            await this.fetchAPIData('');
            console.log('✅ Server is available');
        } catch (error) {
            console.error('❌ Server not available:', error.message);
            return { success: false, error: 'Server unavailable' };
        }
        
        // Load VexFlow
        if (!await this.loadVexFlow()) {
            return { success: false, error: 'VexFlow loading failed' };
        }
        
        // Test cases
        const testCases = [
            { name: 'Simple notes', input: '123', expectRendering: true },
            { name: 'Sargam notes', input: 'SRG', expectRendering: true },
            { name: 'With barline', input: '|123', expectRendering: true },
            { name: 'Simple tuplet', input: '|1-2', expectRendering: true },
            { name: 'Complex tuplet', input: '|1-2-3', expectRendering: true },
            { name: 'Extended notes', input: '|1--2', expectRendering: true },
            { name: 'With rests', input: '|-1-', expectRendering: true },
            { name: 'Mixed systems', input: 'SRmG', expectRendering: true },
            { name: 'Long tuplet', input: '|11111', expectRendering: true },
            { name: 'Complex rhythm', input: '|1-2-3-4', expectRendering: true },
            { name: 'Empty input', input: '', expectRendering: false }
        ];
        
        const results = {
            totalTests: testCases.length,
            passed: 0,
            failed: 0,
            details: [],
            renderingStats: {
                totalNotesRendered: 0,
                totalTupletsRendered: 0,
                totalStavesRendered: 0,
                totalCanvasOperations: 0
            }
        };
        
        for (let i = 0; i < testCases.length; i++) {
            const testCase = testCases[i];
            console.log(`\\n📋 Test ${i + 1}/${testCases.length}: ${testCase.name}`);
            console.log(`   Input: "${testCase.input}"`);
            
            try {
                // Fetch API data
                console.log('   🌐 Fetching API data...');
                const apiResponse = await this.fetchAPIData(testCase.input);
                
                let testResult = {
                    testName: testCase.name,
                    input: testCase.input,
                    apiSuccess: apiResponse.data.success,
                    hasVexFlow: !!apiResponse.data.vexflow,
                    validationResult: null,
                    renderingResult: null,
                    success: false
                };
                
                if (testCase.expectRendering) {
                    if (!apiResponse.data.vexflow) {
                        console.log('   ❌ No VexFlow data in response');
                        testResult.error = 'Missing VexFlow data';
                    } else {
                        // Validate JSON structure
                        console.log('   🔍 Validating VexFlow JSON...');
                        testResult.validationResult = this.validateVexFlowJSON(apiResponse.data.vexflow);
                        
                        if (testResult.validationResult.valid) {
                            console.log('   ✅ VexFlow JSON is valid');
                            
                            // Test actual rendering
                            console.log('   🎨 Testing VexFlow rendering...');
                            testResult.renderingResult = await this.testVexFlowRendering(
                                apiResponse.data.vexflow, 
                                testCase.name
                            );
                            
                            if (testResult.renderingResult.success) {
                                const stats = testResult.renderingResult.stats;
                                console.log(`   ✅ RENDERING SUCCESS!`);
                                console.log(`      Notes: ${stats.notesCreated}, Tuplets: ${stats.tupletsCreated}`);
                                console.log(`      Operations: ${stats.renderingOperations}, Image: ${testResult.renderingResult.imageSize} bytes`);
                                
                                // Update global stats
                                results.renderingStats.totalNotesRendered += stats.notesCreated;
                                results.renderingStats.totalTupletsRendered += stats.tupletsCreated;
                                results.renderingStats.totalCanvasOperations += stats.renderingOperations;
                                results.renderingStats.totalStavesRendered++;
                                
                                testResult.success = true;
                                results.passed++;
                            } else {
                                console.log(`   ❌ RENDERING FAILED: ${testResult.renderingResult.error}`);
                                results.failed++;
                            }
                        } else {
                            console.log('   ❌ VexFlow JSON validation failed:');
                            testResult.validationResult.errors.forEach(error => {
                                console.log(`      • ${error}`);
                            });
                            results.failed++;
                        }
                    }
                } else {
                    // Test cases that don't expect rendering
                    console.log('   ✅ Non-rendering test completed');
                    testResult.success = true;
                    results.passed++;
                }
                
                results.details.push(testResult);
                
            } catch (error) {
                console.log(`   ❌ Test failed: ${error.message}`);
                results.failed++;
                results.details.push({
                    testName: testCase.name,
                    input: testCase.input,
                    error: error.message,
                    success: false
                });
            }
        }
        
        return results;
    }
    
    printDetailedReport(results) {
        console.log('\\n' + '=' * 60);
        console.log('📊 VEXFLOW RENDERING TEST RESULTS');
        console.log('=' * 60);
        
        console.log(`Total Tests: ${results.totalTests}`);
        console.log(`Passed: ${results.passed} ✅`);
        console.log(`Failed: ${results.failed} ❌`);
        console.log(`Success Rate: ${((results.passed / results.totalTests) * 100).toFixed(1)}%`);
        
        // Rendering statistics
        const stats = results.renderingStats;
        console.log(`\\n🎵 RENDERING STATISTICS:`);
        console.log(`  Staves Rendered: ${stats.totalStavesRendered}`);
        console.log(`  Notes Rendered: ${stats.totalNotesRendered}`);
        console.log(`  Tuplets Rendered: ${stats.totalTupletsRendered}`);
        console.log(`  Canvas Operations: ${stats.totalCanvasOperations}`);
        
        // Performance data
        const successfulRenders = results.details.filter(test => 
            test.renderingResult && test.renderingResult.success
        );
        
        if (successfulRenders.length > 0) {
            console.log(`\\n⚡ RENDERING PERFORMANCE:`);
            successfulRenders.forEach(test => {
                const result = test.renderingResult;
                console.log(`  • ${test.testName}:`);
                console.log(`    Notes: ${result.stats.notesCreated}, Image: ${result.imageSize} bytes`);
                console.log(`    Canvas: ${result.canvasWidth}x${result.canvasHeight}`);
            });
        }
        
        // Failed tests
        const failedTests = results.details.filter(test => !test.success);
        if (failedTests.length > 0) {
            console.log(`\\n❌ FAILED TESTS (${failedTests.length}):`);
            failedTests.forEach(test => {
                console.log(`  • ${test.testName}: "${test.input}"`);
                if (test.error) {
                    console.log(`    Error: ${test.error}`);
                }
                if (test.renderingResult && test.renderingResult.error) {
                    console.log(`    Rendering: ${test.renderingResult.error}`);
                }
                if (test.validationResult && !test.validationResult.valid) {
                    console.log(`    Validation: ${test.validationResult.errors.join(', ')}`);
                }
            });
        }
        
        console.log(`\\n🎨 Canvas Backend: node-canvas with real 2D rendering`);
        console.log(`🎼 VexFlow Version: 4.2.2 (compatible with old music-text)`);
        
        if (results.passed === results.totalTests) {
            console.log('\\n🎉 All VexFlow rendering tests passed!');
            console.log('✅ Server-side Canvas rendering is fully functional');
        } else {
            console.log(`\\n⚠️  ${results.failed} tests failed`);
        }
    }
}

async function main() {
    const tester = new VexFlowRenderingTester();
    
    try {
        const results = await tester.runComprehensiveRenderingTests();
        tester.printDetailedReport(results);
        
        // Exit with appropriate code  
        process.exit(results.passed === results.totalTests ? 0 : 1);
        
    } catch (error) {
        console.error('❌ Test execution failed:', error.message);
        console.error(error.stack);
        process.exit(1);
    }
}

if (require.main === module) {
    main();
}