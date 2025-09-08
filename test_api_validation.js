#!/usr/bin/env node
/**
 * Comprehensive server-side API validation tests
 * Focuses on JSON structure validation and API functionality testing
 * Does not require VexFlow rendering, just validates the data structures
 */

const http = require('http');

// Server configuration
const SERVER_BASE = 'http://127.0.0.1:3000';
const API_ENDPOINT = `${SERVER_BASE}/api/parse`;
const LILYPOND_SVG_ENDPOINT = `${SERVER_BASE}/api/lilypond-svg`;

class APIValidator {
    constructor() {
        this.testResults = [];
    }
    
    async checkServerAvailability() {
        return new Promise((resolve) => {
            const req = http.get(SERVER_BASE, (res) => {
                resolve(true);
                res.resume(); // Consume response
            });
            
            req.on('error', () => resolve(false));
            req.setTimeout(5000, () => {
                req.destroy();
                resolve(false);
            });
        });
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
    
    async fetchLilyPondSVG(notation) {
        return new Promise((resolve, reject) => {
            const postData = JSON.stringify({ notation });
            
            const options = {
                hostname: '127.0.0.1',
                port: 3000,
                path: '/api/lilypond-svg',
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Content-Length': Buffer.byteLength(postData)
                }
            };
            
            const req = http.request(options, (res) => {
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
            req.setTimeout(15000, () => {
                req.destroy();
                reject(new Error('Request timeout'));
            });
            
            req.write(postData);
            req.end();
        });
    }
    
    validateAPIResponse(response, testCase) {
        const errors = [];
        const warnings = [];
        const data = response.data;
        
        // Check HTTP status
        if (response.statusCode !== 200) {
            errors.push(`HTTP ${response.statusCode} (expected 200)`);
        }
        
        // Check required top-level fields
        const requiredFields = ['success'];
        for (const field of requiredFields) {
            if (!(field in data)) {
                errors.push(`Missing required field: ${field}`);
            }
        }
        
        // Validate success field logic
        if (data.success === true) {
            // When success=true, we should have output data
            if (testCase.expectOutputs && testCase.expectOutputs.length > 0) {
                for (const expectedOutput of testCase.expectOutputs) {
                    if (!(expectedOutput in data) || data[expectedOutput] === null) {
                        warnings.push(`Expected output missing: ${expectedOutput}`);
                    }
                }
            }
            
            // Should not have error field when successful
            if (data.error) {
                warnings.push('Success=true but error field present');
            }
        } else if (data.success === false) {
            // When success=false, should have error message
            if (!data.error) {
                warnings.push('Success=false but no error message provided');
            }
        }
        
        return { errors, warnings };
    }
    
    validateVexFlowStructure(vexflowData) {
        const errors = [];
        const warnings = [];
        
        if (!vexflowData) {
            return { errors: ['VexFlow data is null'], warnings };
        }
        
        // Check top-level structure
        const requiredTopLevel = ['clef', 'key_signature', 'staves', 'time_signature'];
        for (const field of requiredTopLevel) {
            if (!(field in vexflowData)) {
                errors.push(`Missing VexFlow field: ${field}`);
            }
        }
        
        // Validate staves array
        if (!Array.isArray(vexflowData.staves)) {
            errors.push('VexFlow staves must be an array');
        } else if (vexflowData.staves.length === 0) {
            warnings.push('VexFlow staves array is empty');
        } else {
            // Validate each stave
            vexflowData.staves.forEach((stave, staveIdx) => {
                if (!stave.notes || !Array.isArray(stave.notes)) {
                    errors.push(`Stave ${staveIdx} missing or invalid notes array`);
                } else {
                    // Validate each note/element
                    stave.notes.forEach((note, noteIdx) => {
                        this.validateVexFlowElement(note, staveIdx, noteIdx, errors, warnings);
                    });
                }
                
                // Check for key_signature
                if (!('key_signature' in stave)) {
                    warnings.push(`Stave ${staveIdx} missing key_signature`);
                }
            });
        }
        
        return { errors, warnings };
    }
    
    validateVexFlowElement(element, staveIdx, elemIdx, errors, warnings) {
        if (!element.type) {
            errors.push(`Stave ${staveIdx}, Element ${elemIdx}: Missing type field`);
            return;
        }
        
        switch (element.type) {
            case 'Note':
                // Validate note structure
                if (!element.keys || !Array.isArray(element.keys) || element.keys.length === 0) {
                    errors.push(`Stave ${staveIdx}, Note ${elemIdx}: Missing or empty keys array`);
                }
                
                if (!element.duration) {
                    errors.push(`Stave ${staveIdx}, Note ${elemIdx}: Missing duration`);
                } else {
                    // Check duration format
                    const validDurations = ['w', 'h', 'q', '8', '16', '32', '64', '128', 'wr', 'hr', 'qr', '8r', '16r', '32r', '64r'];
                    if (!validDurations.includes(element.duration)) {
                        warnings.push(`Stave ${staveIdx}, Note ${elemIdx}: Unusual duration '${element.duration}'`);
                    }
                }
                
                // Validate key format (e.g., "c/4")
                if (element.keys) {
                    element.keys.forEach((key, keyIdx) => {
                        if (!/^[a-g][#b]?\/[0-9]$/.test(key)) {
                            warnings.push(`Stave ${staveIdx}, Note ${elemIdx}, Key ${keyIdx}: Unusual key format '${key}'`);
                        }
                    });
                }
                
                // Check optional fields
                const noteFields = ['accidentals', 'dots', 'tied', 'beam_start', 'beam_end', 'ornaments'];
                noteFields.forEach(field => {
                    if (field in element && element[field] !== null && element[field] !== undefined) {
                        // Field exists, validate its type
                        switch (field) {
                            case 'accidentals':
                            case 'ornaments':
                                if (!Array.isArray(element[field])) {
                                    warnings.push(`Stave ${staveIdx}, Note ${elemIdx}: ${field} should be an array`);
                                }
                                break;
                            case 'dots':
                                if (typeof element[field] !== 'number' || element[field] < 0) {
                                    warnings.push(`Stave ${staveIdx}, Note ${elemIdx}: dots should be a non-negative number`);
                                }
                                break;
                            case 'tied':
                            case 'beam_start':
                            case 'beam_end':
                                if (typeof element[field] !== 'boolean') {
                                    warnings.push(`Stave ${staveIdx}, Note ${elemIdx}: ${field} should be boolean`);
                                }
                                break;
                        }
                    }
                });
                break;
                
            case 'Rest':
                if (!element.duration) {
                    errors.push(`Stave ${staveIdx}, Rest ${elemIdx}: Missing duration`);
                }
                break;
                
            case 'BarLine':
                if (!element.bar_type) {
                    warnings.push(`Stave ${staveIdx}, BarLine ${elemIdx}: Missing bar_type`);
                }
                break;
                
            case 'Tuplet':
                if (!element.ratio || !Array.isArray(element.ratio) || element.ratio.length !== 2) {
                    errors.push(`Stave ${staveIdx}, Tuplet ${elemIdx}: Invalid or missing ratio array`);
                }
                if (!element.notes || !Array.isArray(element.notes)) {
                    errors.push(`Stave ${staveIdx}, Tuplet ${elemIdx}: Invalid or missing notes array`);
                }
                break;
                
            default:
                warnings.push(`Stave ${staveIdx}, Element ${elemIdx}: Unknown element type '${element.type}'`);
        }
    }
    
    validateLilyPondOutput(lilypondOutput) {
        const errors = [];
        const warnings = [];
        
        if (typeof lilypondOutput !== 'string') {
            errors.push('LilyPond output is not a string');
            return { errors, warnings };
        }
        
        // Check version directive
        if (!lilypondOutput.includes('\\version')) {
            warnings.push('LilyPond output missing version directive');
        }
        
        // Check for basic content
        if (lilypondOutput.length < 10) {
            warnings.push('LilyPond output seems too short');
        }
        
        // Check for common patterns
        const hasNotes = /[a-g][#b]?[0-9]+/.test(lilypondOutput);
        const hasRests = /r[0-9]+/.test(lilypondOutput);
        const hasTuplets = /\\tuplet/.test(lilypondOutput);
        
        if (!hasNotes && !hasRests) {
            warnings.push('LilyPond output contains no recognizable notes or rests');
        }
        
        return { errors, warnings };
    }
    
    async runComprehensiveTests() {
        console.log('🧪 Starting comprehensive server-side API validation tests...');
        console.log('=' * 60);
        
        // Check server availability
        console.log('🌐 Checking server availability...');
        if (!await this.checkServerAvailability()) {
            console.error('❌ Server not available at', SERVER_BASE);
            return { success: false, error: 'Server unavailable' };
        }
        console.log('✅ Server is available');
        
        // Define test cases
        const testCases = [
            {
                name: 'Simple numbers',
                input: '123',
                expectSuccess: true,
                expectOutputs: ['pest_output', 'parsed_document', 'minimal_lilypond', 'vexflow']
            },
            {
                name: 'Simple sargam',
                input: 'SRG',
                expectSuccess: true,
                expectOutputs: ['pest_output', 'parsed_document', 'minimal_lilypond', 'vexflow']
            },
            {
                name: 'With barline',
                input: '|123',
                expectSuccess: true,
                expectOutputs: ['pest_output', 'parsed_document', 'minimal_lilypond', 'vexflow']
            },
            {
                name: 'Simple tuplet',
                input: '|1-2',
                expectSuccess: true,
                expectOutputs: ['pest_output', 'parsed_document', 'minimal_lilypond', 'vexflow']
            },
            {
                name: 'Complex tuplet',
                input: '|1-2-3',
                expectSuccess: true,
                expectOutputs: ['pest_output', 'parsed_document', 'minimal_lilypond', 'vexflow']
            },
            {
                name: 'Extended notes',
                input: '|1--2',
                expectSuccess: true,
                expectOutputs: ['pest_output', 'parsed_document', 'minimal_lilypond', 'vexflow']
            },
            {
                name: 'With rests',
                input: '|-1-',
                expectSuccess: true,
                expectOutputs: ['pest_output', 'parsed_document', 'minimal_lilypond', 'vexflow']
            },
            {
                name: 'Mixed notation',
                input: 'SRmG',
                expectSuccess: true,
                expectOutputs: ['pest_output', 'parsed_document', 'minimal_lilypond', 'vexflow']
            },
            {
                name: 'Blank line before content (fixed)',
                input: '\n123',
                expectSuccess: true,
                expectOutputs: ['pest_output', 'parsed_document', 'minimal_lilypond', 'vexflow']
            },
            {
                name: 'Empty input',
                input: '',
                expectSuccess: true,
                expectOutputs: []
            },
            {
                name: 'Only spaces',
                input: '   ',
                expectSuccess: true,
                expectOutputs: []
            },
            {
                name: 'Long sequence',
                input: '1234567123456712345671234567',
                expectSuccess: true,
                expectOutputs: ['pest_output', 'parsed_document', 'minimal_lilypond', 'vexflow']
            },
            {
                name: 'Very long tuplet',
                input: '|1111111111111111111111111111111',
                expectSuccess: true,
                expectOutputs: ['pest_output', 'parsed_document', 'minimal_lilypond', 'vexflow']
            }
        ];
        
        const results = {
            totalTests: 0,
            passed: 0,
            failed: 0,
            details: []
        };
        
        // Run parse endpoint tests
        console.log('\\n📡 Testing /api/parse endpoint...');
        for (let i = 0; i < testCases.length; i++) {
            const testCase = testCases[i];
            results.totalTests++;
            
            console.log(`\\n📋 Test ${i + 1}/${testCases.length}: ${testCase.name}`);
            console.log(`   Input: "${testCase.input}"`);
            
            try {
                const startTime = Date.now();
                const response = await this.fetchAPIData(testCase.input);
                const responseTime = Date.now() - startTime;
                
                console.log(`   ⏱️  Response time: ${responseTime}ms`);
                
                // Validate basic API response
                const apiValidation = this.validateAPIResponse(response, testCase);
                
                let testResult = {
                    testName: testCase.name,
                    input: testCase.input,
                    responseTime,
                    apiValidation,
                    vexflowValidation: null,
                    lilypondValidation: null,
                    success: apiValidation.errors.length === 0
                };
                
                // Validate VexFlow structure if present
                if (response.data.vexflow) {
                    console.log('   🎵 Validating VexFlow JSON structure...');
                    testResult.vexflowValidation = this.validateVexFlowStructure(response.data.vexflow);
                    
                    if (testResult.vexflowValidation.errors.length > 0) {
                        testResult.success = false;
                    }
                }
                
                // Validate LilyPond output if present
                if (response.data.minimal_lilypond) {
                    console.log('   🎼 Validating LilyPond output...');
                    testResult.lilypondValidation = this.validateLilyPondOutput(response.data.minimal_lilypond);
                    
                    if (testResult.lilypondValidation.errors.length > 0) {
                        testResult.success = false;
                    }
                }
                
                // Print results
                if (testResult.success) {
                    console.log('   ✅ PASS - All validations successful');
                    results.passed++;
                } else {
                    console.log('   ❌ FAIL - Validation errors found:');
                    if (apiValidation.errors.length > 0) {
                        apiValidation.errors.forEach(error => console.log(`      • API: ${error}`));
                    }
                    if (testResult.vexflowValidation && testResult.vexflowValidation.errors.length > 0) {
                        testResult.vexflowValidation.errors.forEach(error => console.log(`      • VexFlow: ${error}`));
                    }
                    if (testResult.lilypondValidation && testResult.lilypondValidation.errors.length > 0) {
                        testResult.lilypondValidation.errors.forEach(error => console.log(`      • LilyPond: ${error}`));
                    }
                    results.failed++;
                }
                
                // Print warnings
                const allWarnings = [
                    ...(apiValidation.warnings || []).map(w => `API: ${w}`),
                    ...(testResult.vexflowValidation?.warnings || []).map(w => `VexFlow: ${w}`),
                    ...(testResult.lilypondValidation?.warnings || []).map(w => `LilyPond: ${w}`)
                ];
                
                if (allWarnings.length > 0) {
                    console.log('   ⚠️  Warnings:');
                    allWarnings.forEach(warning => console.log(`      • ${warning}`));
                }
                
                results.details.push(testResult);
                
            } catch (error) {
                console.log(`   ❌ FAIL - Request error: ${error.message}`);
                results.failed++;
                results.details.push({
                    testName: testCase.name,
                    input: testCase.input,
                    error: error.message,
                    success: false
                });
            }
        }
        
        // Test LilyPond SVG endpoint
        console.log('\\n🎼 Testing /api/lilypond-svg endpoint...');
        const svgTestCases = ['|123', '|SRG', '|1-2-3'];
        
        for (let i = 0; i < svgTestCases.length; i++) {
            const notation = svgTestCases[i];
            results.totalTests++;
            
            console.log(`\\n📋 LilyPond SVG Test ${i + 1}/${svgTestCases.length}: "${notation}"`);
            
            try {
                const startTime = Date.now();
                const response = await this.fetchLilyPondSVG(notation);
                const responseTime = Date.now() - startTime;
                
                console.log(`   ⏱️  Response time: ${responseTime}ms`);
                
                const data = response.data;
                let success = true;
                const errors = [];
                
                if (response.statusCode !== 200) {
                    errors.push(`HTTP ${response.statusCode}`);
                    success = false;
                }
                
                if (!data.success) {
                    errors.push(`SVG generation failed: ${data.error || 'Unknown error'}`);
                    success = false;
                }
                
                if (data.success && !data.svg_content) {
                    errors.push('Success=true but no SVG content');
                    success = false;
                }
                
                if (data.svg_content && data.svg_content.length < 100) {
                    errors.push('SVG content seems too short');
                    success = false;
                }
                
                if (success) {
                    console.log(`   ✅ PASS - SVG generated (${data.svg_content?.length || 0} chars)`);
                    results.passed++;
                } else {
                    console.log('   ❌ FAIL - SVG generation errors:');
                    errors.forEach(error => console.log(`      • ${error}`));
                    results.failed++;
                }
                
                results.details.push({
                    testName: `LilyPond SVG: ${notation}`,
                    input: notation,
                    responseTime,
                    success,
                    errors
                });
                
            } catch (error) {
                console.log(`   ❌ FAIL - Request error: ${error.message}`);
                results.failed++;
                results.details.push({
                    testName: `LilyPond SVG: ${notation}`,
                    input: notation,
                    error: error.message,
                    success: false
                });
            }
        }
        
        return results;
    }
    
    printSummaryReport(results) {
        console.log('\\n' + '=' * 60);
        console.log('📊 COMPREHENSIVE API VALIDATION SUMMARY');
        console.log('=' * 60);
        
        console.log(`Total Tests: ${results.totalTests}`);
        console.log(`Passed: ${results.passed} ✅`);
        console.log(`Failed: ${results.failed} ❌`);
        console.log(`Success Rate: ${((results.passed / results.totalTests) * 100).toFixed(1)}%`);
        
        // Calculate average response time
        const responseTimes = results.details
            .filter(test => test.responseTime)
            .map(test => test.responseTime);
        
        if (responseTimes.length > 0) {
            const avgResponseTime = responseTimes.reduce((a, b) => a + b) / responseTimes.length;
            const minResponseTime = Math.min(...responseTimes);
            const maxResponseTime = Math.max(...responseTimes);
            
            console.log(`\\n⚡ Performance Summary:`);
            console.log(`  Average Response Time: ${avgResponseTime.toFixed(1)}ms`);
            console.log(`  Fastest: ${minResponseTime}ms`);
            console.log(`  Slowest: ${maxResponseTime}ms`);
        }
        
        // Failed tests details
        const failedTests = results.details.filter(test => !test.success);
        if (failedTests.length > 0) {
            console.log(`\\n❌ FAILED TESTS (${failedTests.length}):`);
            failedTests.forEach(test => {
                console.log(`  • ${test.testName}: "${test.input}"`);
                if (test.error) {
                    console.log(`    Error: ${test.error}`);
                }
                if (test.apiValidation?.errors?.length > 0) {
                    console.log(`    API Errors: ${test.apiValidation.errors.join(', ')}`);
                }
                if (test.vexflowValidation?.errors?.length > 0) {
                    console.log(`    VexFlow Errors: ${test.vexflowValidation.errors.join(', ')}`);
                }
                if (test.lilypondValidation?.errors?.length > 0) {
                    console.log(`    LilyPond Errors: ${test.lilypondValidation.errors.join(', ')}`);
                }
            });
        }
        
        // Validation summary
        const vexflowTests = results.details.filter(test => test.vexflowValidation);
        const validVexFlow = vexflowTests.filter(test => test.vexflowValidation.errors.length === 0);
        
        const lilypondTests = results.details.filter(test => test.lilypondValidation);
        const validLilyPond = lilypondTests.filter(test => test.lilypondValidation.errors.length === 0);
        
        console.log(`\\n🎵 VexFlow JSON Validation: ${validVexFlow.length}/${vexflowTests.length} passed`);
        console.log(`🎼 LilyPond Output Validation: ${validLilyPond.length}/${lilypondTests.length} passed`);
        
        if (results.passed === results.totalTests) {
            console.log('\\n🎉 All API validation tests passed!');
        } else {
            console.log(`\\n⚠️  ${results.failed} tests failed`);
        }
    }
}

async function main() {
    const validator = new APIValidator();
    
    try {
        const results = await validator.runComprehensiveTests();
        validator.printSummaryReport(results);
        
        // Exit with appropriate code
        process.exit(results.passed === results.totalTests ? 0 : 1);
        
    } catch (error) {
        console.error('❌ Test execution failed:', error.message);
        process.exit(1);
    }
}

if (require.main === module) {
    main();
}