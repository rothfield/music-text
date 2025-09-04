// Version check for api.js
if (typeof window !== 'undefined' && confirm) {
    if (confirm('api.js version: STRUCTURE_PRESERVING_FSM_v2.0 - Continue?')) {
        console.log('Loading api.js STRUCTURE_PRESERVING_FSM_v2.0');
    }
}

export async function parseNotationApi(notation, system = 'auto') {
    const response = await fetch('/api/parse', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ 
            input: notation,
            notation: system,
            output: ['ast', 'yaml', 'vexflow', 'lilypond']
        })
    });
    
    if (!response.ok) {
        throw new Error(`API request failed: ${response.status}`);
    }
    
    return await response.json();
}

export async function generateLilypondSvgApi(notation, system = 'auto') {
    const response = await fetch('/api/lilypond/svg', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({
            input: notation,
            notation: system
        })
    });

    if (!response.ok) {
        throw new Error(`Server error: ${response.status}`);
    }

    return await response.json();
}

// Legacy function for backward compatibility
export async function generateLilypondPngApi(lilypondCode) {
    // This is deprecated - use generateLilypondSvgApi instead
    throw new Error('PNG generation deprecated - use SVG generation instead');
}

export async function checkServerHealth() {
    try {
        const response = await fetch('/health', {
            method: 'GET',
            headers: {
                'Content-Type': 'application/json',
            },
            // Use a shorter timeout for health checks
            signal: AbortSignal.timeout(5000)
        });

        if (response.ok) {
            const data = await response.json();
            return { 
                online: true, 
                status: 'Server online',
                details: data.version ? `v${data.version}` : 'Ready'
            };
        } else {
            return { 
                online: false, 
                status: 'Server error',
                details: `HTTP ${response.status}`
            };
        }
    } catch (error) {
        if (error.name === 'TimeoutError') {
            return { 
                online: false, 
                status: 'Server timeout',
                details: 'No response in 5s'
            };
        } else if (error.name === 'TypeError' && error.message.includes('fetch')) {
            return { 
                online: false, 
                status: 'Server offline',
                details: 'Connection refused'
            };
        } else {
            return { 
                online: false, 
                status: 'Connection error',
                details: error.message.slice(0, 30) + '...'
            };
        }
    }
}
