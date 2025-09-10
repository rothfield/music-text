// VexFlow Integration Module
// Handle VexFlow-specific rendering and integration

import { syntaxHighlight } from './ui-utils.js';

export class VexFlowIntegration {
    constructor() {
        this.isRendererLoaded = false;
    }

    // Load VexFlow renderer dynamically
    async loadVexFlowRenderer() {
        if (window.VexFlowRenderer) {
            this.isRendererLoaded = true;
            return;
        }
        
        try {
            const script = document.createElement('script');
            script.src = 'vexflow-renderer.js';
            script.async = true;
            
            return new Promise((resolve, reject) => {
                script.onload = () => {
                    console.log('✅ VexFlow renderer loaded');
                    this.isRendererLoaded = true;
                    resolve();
                };
                script.onerror = () => {
                    console.error('❌ Failed to load VexFlow renderer');
                    reject(new Error('Failed to load VexFlow renderer'));
                };
                document.head.appendChild(script);
            });
        } catch (error) {
            console.error('🚨 Error loading VexFlow renderer:', error);
        }
    }

    // Draw placeholder content on VexFlow canvas
    drawVexFlowPlaceholder(canvas, input) {
        const ctx = canvas.getContext('2d');
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        
        // Background
        ctx.fillStyle = '#fafafa';
        ctx.fillRect(0, 0, canvas.width, canvas.height);
        
        // Staff lines
        ctx.strokeStyle = '#333';
        ctx.lineWidth = 1;
        const staffY = 100;
        const staffWidth = 500;
        const staffX = 50;
        
        for (let i = 0; i < 5; i++) {
            ctx.beginPath();
            ctx.moveTo(staffX, staffY + i * 10);
            ctx.lineTo(staffX + staffWidth, staffY + i * 10);
            ctx.stroke();
        }
        
        // Title
        ctx.fillStyle = '#333';
        ctx.font = 'bold 16px serif';
        ctx.fillText('VexFlow Notation (Demo)', 50, 30);
        
        // Input display
        ctx.fillStyle = '#666';
        ctx.font = '11px monospace';
        ctx.fillText('Input: ' + input.substring(0, 50), 50, 50);
        
        // Treble clef (simplified)
        ctx.fillStyle = '#333';
        ctx.font = '30px serif';
        ctx.fillText('𝄞', staffX + 10, staffY + 25);
        
        // Time signature
        ctx.font = '16px serif';
        ctx.fillText('4', staffX + 50, staffY + 10);
        ctx.fillText('4', staffX + 50, staffY + 25);
        
        // Notes based on input
        let noteX = staffX + 80;
        const notes = input.match(/[1-7A-G]/g) || ['1', '2', '3'];
        
        for (let i = 0; i < Math.min(notes.length, 8); i++) {
            const note = notes[i];
            let noteY = staffY + 20; // Default middle line (B)
            
            // Map notes to staff positions
            if (['1', 'C'].includes(note)) noteY = staffY + 50; // C below staff
            else if (['2', 'D'].includes(note)) noteY = staffY + 45; // D
            else if (['3', 'E'].includes(note)) noteY = staffY + 40; // E
            else if (['4', 'F'].includes(note)) noteY = staffY + 35; // F
            else if (['5', 'G'].includes(note)) noteY = staffY + 30; // G
            else if (['6', 'A'].includes(note)) noteY = staffY + 25; // A
            else if (['7', 'B'].includes(note)) noteY = staffY + 20; // B
            
            // Draw note head
            ctx.fillStyle = '#333';
            ctx.beginPath();
            ctx.ellipse(noteX, noteY, 6, 4, 0, 0, 2 * Math.PI);
            ctx.fill();
            
            // Draw stem
            ctx.beginPath();
            ctx.moveTo(noteX + 6, noteY);
            ctx.lineTo(noteX + 6, noteY - 25);
            ctx.lineWidth = 2;
            ctx.stroke();
            ctx.lineWidth = 1;
            
            // Ledger lines if needed
            if (noteY >= staffY + 45) {
                ctx.beginPath();
                ctx.moveTo(noteX - 8, staffY + 50);
                ctx.lineTo(noteX + 14, staffY + 50);
                ctx.stroke();
            }
            
            noteX += 50;
        }
        
        // Bar line
        if (input.includes('|')) {
            ctx.lineWidth = 2;
            ctx.beginPath();
            ctx.moveTo(noteX - 10, staffY);
            ctx.lineTo(noteX - 10, staffY + 40);
            ctx.stroke();
            ctx.lineWidth = 1;
        }
        
        // Footer
        ctx.fillStyle = '#999';
        ctx.font = '10px Arial';
        ctx.fillText('VexFlow-style rendering placeholder', 50, canvas.height - 20);
    }

    // Handle VexFlow output rendering and display
    handleVexFlowOutput(vexflowData) {
        const vexflowOutput = document.getElementById('vexflow-output');
        
        if (!vexflowData) {
            console.log('⚠️ No VexFlow data available');
            vexflowOutput.innerHTML = '<div class="text-muted p-3">No VexFlow data available - check parser output</div>';
            return;
        }

        console.log('🎼 Processing VexFlow output:', {
            hasVexflowData: !!vexflowData,
            staves: vexflowData.staves?.length,
            hasAdvancedFeatures: vexflowData.staves?.some(s => s.notes?.some(n => n.type === 'Tuplet' || n.type === 'SlurStart'))
        });
        
        // Create container for VexFlow rendering
        vexflowOutput.innerHTML = `
            <div class="vexflow-professional">
                <div class="text-muted mb-2">Professional VexFlow Rendering with Advanced Features</div>
                <div id="vexflow-notation" style="width: 100%; min-height: 200px; border: 1px solid #ddd; background: #fafafa;"></div>
                <div class="mt-2">
                    <button id="toggle-vexflow-data" class="btn btn-sm btn-outline-secondary">Show JSON Data</button>
                </div>
                <div id="vexflow-data" class="json-output mt-2" style="display: none; max-height: 300px; overflow-y: auto;"></div>
            </div>
        `;
        
        // Render with enhanced VexFlow renderer
        this.renderVexFlowNotation(vexflowData, 'vexflow-notation');
        
        // Setup JSON data display
        this.setupVexFlowDataToggle(vexflowData);
    }

    // Render VexFlow notation using the renderer
    async renderVexFlowNotation(vexflowData, containerId) {
        if (!this.isRendererLoaded) {
            console.warn('⚠️ VexFlowRenderer not available, loading...');
            try {
                await this.loadVexFlowRenderer();
            } catch (error) {
                console.error('🚨 Failed to load VexFlow renderer:', error);
                return;
            }
        }

        if (window.VexFlowRenderer) {
            try {
                const success = await window.VexFlowRenderer.renderVexFlowNotation(vexflowData, containerId);
                if (success) {
                    console.log('✅ Enhanced VexFlow rendering completed');
                } else {
                    console.warn('⚠️ VexFlow rendering had issues');
                }
            } catch (error) {
                console.error('🚨 VexFlow rendering failed:', error);
                // Could display error message in the container
            }
        }
    }

    // Setup the toggle for VexFlow JSON data display
    setupVexFlowDataToggle(vexflowData) {
        const vexJsonString = JSON.stringify(vexflowData, null, 2);
        const vexflowDataDiv = document.getElementById('vexflow-data');
        
        if (vexflowDataDiv) {
            vexflowDataDiv.innerHTML = '<div class="syntax-highlight">' + syntaxHighlight(vexJsonString) + '</div>';
        }
        
        // Toggle VexFlow data visibility
        const toggleButton = document.getElementById('toggle-vexflow-data');
        if (toggleButton) {
            toggleButton.addEventListener('click', function() {
                const dataDiv = document.getElementById('vexflow-data');
                const button = this;
                
                if (dataDiv.style.display === 'none') {
                    dataDiv.style.display = 'block';
                    button.textContent = 'Hide JSON Data';
                } else {
                    dataDiv.style.display = 'none';
                    button.textContent = 'Show JSON Data';
                }
            });
        }
    }

    // Initialize VexFlow integration
    async init() {
        try {
            await this.loadVexFlowRenderer();
            console.log('✅ VexFlow integration initialized');
        } catch (error) {
            console.warn('⚠️ VexFlow integration initialization failed:', error);
        }
    }
}