export async function parseNotationApi(notation) {
    const response = await fetch('/api/parse', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ 
            notation: notation, 
            filename: `web_live_${Date.now()}.123` 
        })
    });
    
    if (!response.ok) {
        throw new Error(`API request failed: ${response.status}`);
    }
    
    return await response.json();
}

export async function generateLilypondPngApi(lilypondCode) {
    const response = await fetch('/api/lilypond-to-png', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({
            lilypondCode: lilypondCode
        })
    });

    if (!response.ok) {
        throw new Error(`Server error: ${response.status}`);
    }

    return await response.json();
}

export async function checkServerHealth() {
    try {
        const response = await fetch('/api/health', {
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
