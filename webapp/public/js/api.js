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
