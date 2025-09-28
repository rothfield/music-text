/**
 * API Client Module
 * Handles all API communication with the music text parser backend
 */

export const API = {
    // Base API endpoint
    baseURL: '/api',

    // Parse music notation
    async parse(input, generateSVG = false, notationType = null) {
        if (!input || !input.trim()) {
            throw new Error('Input text is required');
        }

        const url = new URL(`${this.baseURL}/parse`, window.location.origin);
        url.searchParams.set('input', input);
        if (generateSVG) {
            url.searchParams.set('generate_svg', 'true');
        }
        // notation_type removed from API

        const response = await fetch(url);
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        const result = await response.json();
        return result;
    },

    // Parse for real-time updates (no status messages)
    async parseForPreview(input) {
        try {
            return await this.parse(input, false);
        } catch (error) {
            console.warn('Parse error during preview:', error.message);
            return {
                success: false,
                error: error.message,
                vexflow: null,
                lilypond: null
            };
        }
    },

    // Parse with SVG generation
    async parseWithSVG(input) {
        return await this.parse(input, true);
    },

    // Validate input before sending
    validateInput(input) {
        if (!input || typeof input !== 'string') {
            return { valid: false, error: 'Input must be a non-empty string' };
        }

        if (!input.trim()) {
            return { valid: false, error: 'Input cannot be empty or whitespace only' };
        }

        if (input.length > 10000) { // Reasonable limit
            return { valid: false, error: 'Input too long (max 10000 characters)' };
        }

        return { valid: true };
    },

    // Check if result indicates success
    isSuccessfulResult(result) {
        return result && result.success === true;
    },

    // Extract error message from result
    getErrorMessage(result) {
        if (!result) return 'Unknown error occurred';
        return result.error || 'Unknown error occurred';
    },

    // Check if result has VexFlow data
    hasVexFlowData(result) {
        return this.isSuccessfulResult(result) && result.vexflow;
    },

    // Check if result has LilyPond data
    hasLilyPondData(result) {
        return this.isSuccessfulResult(result) && result.lilypond;
    },

    // Check if result has SVG data
    hasSVGData(result) {
        return this.isSuccessfulResult(result) && result.lilypond_svg;
    }
};
