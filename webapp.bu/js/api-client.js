// API Client Module
// Centralize all backend communication with proper error handling

import { API_ENDPOINTS } from './config.js';
import { convertUnicodeToStandard } from './unicode-processor.js';

export class ApiClient {
    // Generate cache busting parameter
    static getCacheBuster() {
        return Date.now();
    }
    
    static async parseInput(input) {
        console.log('🚀 parseInput() called:', {
            inputLength: input.length,
            isEmpty: !input.trim(),
            firstLine: input.split('\n')[0],
            totalLines: input.split('\n').length,
            timestamp: new Date().toISOString()
        });
        
        if (!input.trim()) {
            return { success: true, isEmpty: true };
        }

        try {
            // Convert Unicode characters back to standard characters for backend
            const standardInput = convertUnicodeToStandard(input);
            console.log('🔄 Converting Unicode to standard for backend:', {
                original: input.slice(0, 50) + (input.length > 50 ? '...' : ''),
                converted: standardInput.slice(0, 50) + (standardInput.length > 50 ? '...' : ''),
                hasUnicode: input !== standardInput
            });
            
            // Fetch all outputs from unified endpoint with cache busting
            const cacheBuster = ApiClient.getCacheBuster();
            const apiUrl = `${API_ENDPOINTS.PARSE}?input=${encodeURIComponent(standardInput)}&_cb=${cacheBuster}`;
            console.log('🔄 Making API request:', { 
                input: standardInput.slice(0, 100) + (standardInput.length > 100 ? '...' : ''),
                url: apiUrl,
                cacheBuster: cacheBuster,
                timestamp: new Date().toISOString()
            });
            
            const response = await fetch(apiUrl);
            console.log('📡 API Response received:', {
                status: response.status,
                ok: response.ok,
                headers: Object.fromEntries(response.headers.entries())
            });
            
            const data = await response.json();
            console.log('📋 Parsed API data:', {
                success: data.success,
                hasError: !!data.error,
                error: data.error?.slice(0, 200),
                detectedSystems: data.detected_notation_systems,
                outputsGenerated: {
                    document: !!data.parsed_document,
                    lily: !!data.minimal_lilypond,
                    vexflow: !!data.vexflow
                }
            });
            
            return data;
            
        } catch (error) {
            console.error('🚨 Network/JavaScript error caught:', {
                message: error.message,
                name: error.name,
                stack: error.stack,
                timestamp: new Date().toISOString()
            });
            
            return {
                success: false,
                error: error.message,
                networkError: true
            };
        }
    }

    static async generateSvgFromLilypond(notation) {
        console.log("🎵 generateSvgFromLilypond() called");
        
        if (!notation || !notation.trim()) {
            return {
                success: false,
                error: "Please enter music notation first."
            };
        }
        
        // Convert Unicode characters back to standard characters for backend
        const standardNotation = convertUnicodeToStandard(notation);
        console.log('🔄 Converting Unicode to standard for SVG generation:', {
            original: notation.slice(0, 50) + (notation.length > 50 ? '...' : ''),
            converted: standardNotation.slice(0, 50) + (standardNotation.length > 50 ? '...' : ''),
            hasUnicode: notation !== standardNotation
        });
        
        try {
            const cacheBuster = ApiClient.getCacheBuster();
            const response = await fetch(`${API_ENDPOINTS.LILYPOND_SVG}?_cb=${cacheBuster}`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    notation: standardNotation
                })
            });
            
            const result = await response.json();
            
            if (result.success && result.svg_content) {
                console.log("✅ SVG generated successfully");
                return result;
            } else {
                console.error("❌ SVG generation failed:", result.error);
                return {
                    success: false,
                    error: result.error || "Unknown error"
                };
            }
        } catch (error) {
            console.error("🚨 Network error during SVG generation:", error);
            return {
                success: false,
                error: error.message,
                networkError: true
            };
        }
    }

    static async loadValidPitches() {
        console.log('🔄 Loading valid pitch patterns from server...');
        try {
            const cacheBuster = ApiClient.getCacheBuster();
            const response = await fetch(`${API_ENDPOINTS.VALID_PITCHES}?_cb=${cacheBuster}`);
            if (!response.ok) {
                throw new Error(`HTTP ${response.status}: ${response.statusText}`);
            }
            const data = await response.json();
            console.log('✅ Loaded valid pitch patterns:', { 
                flats: data.flat_patterns?.length || 0, 
                sharps: data.sharp_patterns?.length || 0,
                flatSample: data.flat_patterns?.slice(0, 3) || [],
                sharpSample: data.sharp_patterns?.slice(0, 3) || []
            });
            return data;
        } catch (error) {
            console.error('❌ Failed to load valid pitch patterns, falling back to regex:', error);
            return {
                flat_patterns: [],
                sharp_patterns: []
            };
        }
    }
}