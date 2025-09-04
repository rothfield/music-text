const express = require('express');
const cors = require('cors');
const path = require('path');

const app = express();
const port = process.env.PORT || 8000; // Use port 8000 for webapp, 3000 for pest parser

// Middleware
app.use(cors());
app.use(express.json({ limit: '10mb' }));

// Static files
app.use(express.static(path.join(__dirname, 'public')));

// Serve favicon
app.get('/favicon.ico', (req, res) => {
    res.sendFile(path.join(__dirname, 'favicon.ico'));
});

// Proxy health check to pest parser
app.get('/health', async (req, res) => {
    try {
        const response = await fetch('http://127.0.0.1:3000/health');
        const result = await response.json();
        res.status(response.status).json(result);
    } catch (error) {
        res.status(503).json({ 
            status: 'error',
            error: 'Pest parser not available',
            message: error.message 
        });
    }
});

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
            spatial: result.spatial,
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

// Start the server
app.listen(port, () => {
    console.log(`🌐 Webapp server running on http://localhost:${port}`);
    console.log(`📡 Proxying API requests to pest parser on port 3000`);
    console.log(`🛑 Press Ctrl+C to stop`);
});