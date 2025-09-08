#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

// Generate version based on current timestamp
const version = Date.now();

// Or alternatively, generate based on file content hash
function getFileHash(filePath) {
    if (!fs.existsSync(filePath)) return version;
    const content = fs.readFileSync(filePath, 'utf8');
    return crypto.createHash('md5').update(content).digest('hex').substring(0, 8);
}

// Files to cache-bust
const assets = [
    { file: 'js/app.js', varName: 'appJs' },
    { file: 'css/app.css', varName: 'appCss' }
];

// Generate version map
const versions = {};
assets.forEach(asset => {
    const fullPath = path.join(__dirname, 'public', asset.file);
    versions[asset.varName] = getFileHash(fullPath);
});

// Read the HTML template
const htmlPath = path.join(__dirname, 'public', 'index.html');
let html = fs.readFileSync(htmlPath, 'utf8');

// Update script tags with cache-busting parameters
html = html.replace(/(<script\s+src=")(js\/app\.js)(\?v=[\w]+)?(")/g, 
    `$1$2?v=${versions.appJs}$4`);

// Update link tags for CSS
html = html.replace(/(<link\s+.*href=")(css\/app\.css)(\?v=[\w]+)?(")/g, 
    `$1$2?v=${versions.appCss}$4`);

// If no existing version parameters, add them
if (!html.includes('js/app.js?v=')) {
    html = html.replace('src="js/app.js"', `src="js/app.js?v=${versions.appJs}"`);
}
if (!html.includes('css/app.css?v=')) {
    html = html.replace('href="css/app.css"', `href="css/app.css?v=${versions.appCss}"`);
}

// Write the updated HTML
fs.writeFileSync(htmlPath, html);

console.log('Cache-busting versions updated:');
Object.entries(versions).forEach(([name, ver]) => {
    console.log(`  ${name}: ${ver}`);
});

// Also create a simple version.json for reference
fs.writeFileSync(
    path.join(__dirname, 'public', 'version.json'),
    JSON.stringify({ 
        generated: new Date().toISOString(),
        versions 
    }, null, 2)
);