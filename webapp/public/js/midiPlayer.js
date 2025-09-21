/**
 * MIDI Player Module
 * Implements MIDI playback for parsed music-text documents using Tone.js
 */

export class MusicTextPlayer {
    constructor() {
        this.bpm = 120;
        this.playing = false;
        this.currentPosition = 0;
        this.loop = false;
        this.loopStart = 0;
        this.loopEnd = null;

        // Audio components
        this.synth = null;
        this.volume = null;
        this.reverb = null;

        // Playback data
        this.events = [];
        this.scheduledEvents = [];
        this.currentDocument = null;

        // Event listeners
        this.eventListeners = {
            play: [],
            pause: [],
            stop: [],
            note: [],
            beat: [],
            end: []
        };

        this.init();
    }

    async init() {
        try {
            // Don't start Tone.js here - wait for user interaction
            console.log('✅ MIDI Player initialized (audio will start on user interaction)');
        } catch (error) {
            console.error('Failed to initialize MIDI Player:', error);
            throw error;
        }
    }

    // Playback Control Methods
    async play(document = null) {
        try {
            // Ensure Tone.js is started (requires user interaction)
            if (Tone.context.state !== 'running') {
                console.log('🔊 Starting audio context...');
                await Tone.start();
                console.log('🔊 Audio context started, state:', Tone.context.state);
            } else {
                console.log('🔊 Audio context already running');
            }

            // Create audio components if not already created
            if (!this.synth) {
                this.synth = new Tone.PolySynth(Tone.Synth, {
                    oscillator: { type: "triangle" },
                    envelope: {
                        attack: 0.005,
                        decay: 0.1,
                        sustain: 0.3,
                        release: 1
                    }
                }).toDestination();

                console.log('🎹 Synthesizer created');
            }

            // Set tempo
            Tone.Transport.bpm.value = this.bpm;

            if (document) {
                this.currentDocument = document;
                this.events = this.documentToPlaybackEvents(document);
                this.currentPosition = 0;
                console.log(`📝 Processing document with ${document.elements ? document.elements.length : 0} elements`);
            }

            if (!this.events || !this.events.length) {
                console.warn('No events to play. Events:', this.events);
                console.warn('Document structure:', this.currentDocument);
                return;
            }

            // Clear any previous scheduled events
            Tone.Transport.cancel();
            Tone.Transport.position = 0;

            this.playing = true;

            // Play notes directly with absolute timing
            console.log('🎵 Scheduling playback events...');
            const now = Tone.now();

            for (const event of this.events) {
                // Schedule note with absolute time
                const noteTime = now + event.time;
                const frequency = Tone.Frequency(event.pitch, "midi").toFrequency();

                console.log(`🎵 Scheduling: MIDI=${event.pitch} → ${frequency.toFixed(2)}Hz at ${noteTime.toFixed(3)}s for ${event.duration.toFixed(3)}s`);

                this.synth.triggerAttackRelease(
                    frequency,
                    event.duration,
                    noteTime,
                    event.velocity / 127
                );
            }

            console.log(`✅ Scheduled ${this.events.length} notes for playback`);

            this.emit('play');
            console.log(`🎵 Playback started with ${this.events.length} events`);

        } catch (error) {
            console.error('Playback error:', error);
            this.playing = false;
        }
    }

    pause() {
        if (!this.playing) return;

        this.playing = false;
        Tone.Transport.pause();
        this.clearScheduledEvents();

        this.emit('pause');
        console.log('⏸️ Playback paused');
    }

    stop() {
        this.playing = false;
        this.currentPosition = 0;

        // Stop all sounds immediately
        if (this.synth) {
            this.synth.releaseAll();
        }

        Tone.Transport.stop();
        Tone.Transport.cancel();
        Tone.Transport.position = 0;

        this.emit('stop');
        console.log('⏹️ Playback stopped');
    }

    seek(seconds) {
        this.currentPosition = Math.max(0, seconds);
        Tone.Transport.seconds = this.currentPosition;

        if (this.playing) {
            this.clearScheduledEvents();
            this.scheduleEvents();
        }
    }

    // Configuration Methods
    setTempo(bpm) {
        this.bpm = Math.max(40, Math.min(208, bpm));
        Tone.Transport.bpm.value = this.bpm;
        console.log(`🎼 Tempo set to ${this.bpm} BPM`);
    }

    setVolume(dB) {
        const clampedVolume = Math.max(-60, Math.min(0, dB));
        if (this.volume) {
            this.volume.volume.value = clampedVolume;
        }
        console.log(`🔊 Volume set to ${clampedVolume} dB`);
    }

    setInstrument(type) {
        // For now, using basic synth - can be extended with soundfonts
        if (this.synth) {
            this.synth.dispose();
        }

        switch (type) {
            case 'piano':
                this.synth = new Tone.PolySynth(Tone.Synth, {
                    oscillator: { type: 'triangle' },
                    envelope: { attack: 0.02, decay: 0.1, sustain: 0.3, release: 1 }
                });
                break;
            case 'organ':
                this.synth = new Tone.PolySynth(Tone.Synth, {
                    oscillator: { type: 'square' },
                    envelope: { attack: 0.02, decay: 0.1, sustain: 0.9, release: 0.1 }
                });
                break;
            default:
                this.synth = new Tone.PolySynth();
        }

        this.synth.connect(this.volume);
        console.log(`🎹 Instrument set to ${type}`);
    }

    // Core Conversion Methods
    documentToPlaybackEvents(document) {
        const events = [];
        let globalTime = 0;

        if (!document) {
            console.warn('No document provided');
            return events;
        }

        // Log the structure to understand what we're dealing with
        console.log('Document structure:', {
            hasElements: !!document.elements,
            hasStaves: !!document.staves,
            keys: Object.keys(document),
            document: document
        });

        // Try different possible document structures
        const elements = document.elements || document.staves || [];

        if (elements.length === 0) {
            console.warn('Document has no staves/elements');
            return events;
        }

        // Generate events for the piece
        const pieceEvents = [];
        for (const element of elements) {
            // Handle both { Stave: ... } and direct stave objects
            const stave = element.Stave || element;

            if (stave) {
                const staveEvents = this.processStave(stave, globalTime);
                pieceEvents.push(...staveEvents);
                globalTime += this.getStaveDuration(stave);
            }
        }

        // Repeat the piece 4 times total with 4-second pause between repetitions
        const pieceDuration = globalTime;
        const pauseBetweenRepetitions = 4.0; // 4 seconds

        for (let repetition = 0; repetition < 4; repetition++) {
            const repetitionOffset = repetition * (pieceDuration + pauseBetweenRepetitions);
            for (const event of pieceEvents) {
                events.push({
                    ...event,
                    time: event.time + repetitionOffset
                });
            }
        }

        console.log(`📝 Generated ${events.length} playback events from ${elements.length} elements (4 repetitions)`);
        return events;
    }

    processStave(stave, startTime) {
        const events = [];
        let currentTime = startTime;

        if (!stave.lines) {
            console.warn('Stave has no lines:', stave);
            return events;
        }

        for (const line of stave.lines) {
            // Check different possible line structures
            const contentLine = line.ContentLine || line.content_line || line;

            if (contentLine && contentLine.elements) {
                const lineEvents = this.processContentLine(contentLine, currentTime);
                events.push(...lineEvents);
                currentTime += this.getContentLineDuration(contentLine);
            }
        }

        console.log(`Stave processed: ${events.length} events generated`);
        return events;
    }

    processContentLine(contentLine, startTime) {
        const events = [];
        let currentTime = startTime;

        if (!contentLine.elements) {
            console.warn('ContentLine has no elements:', contentLine);
            return events;
        }

        for (const element of contentLine.elements) {
            // Check different possible element structures
            const beat = element.Beat || element.beat || element;

            if (beat && beat.elements) {
                const beatEvents = this.processBeat(beat, currentTime);
                events.push(...beatEvents);
                currentTime += this.getBeatDuration(beat);
            }
        }

        console.log(`ContentLine processed: ${events.length} events`);
        return events;
    }

    processBeat(beat, startTime) {
        const events = [];

        if (!beat.elements || beat.elements.length === 0) {
            console.warn('Beat has no elements:', beat);
            return events;
        }

        const beatDuration = this.getBeatDuration(beat);
        const elementDuration = beatDuration / beat.elements.length;

        let currentTime = startTime;

        for (const element of beat.elements) {
            // Check different possible element structures
            const note = element.Note || element.note;
            const dash = element.Dash || element.dash;
            const breathMark = element.BreathMark || element.breath_mark || element.breath;

            if (note) {
                const pitchCode = note.pitch_code || note.pitchCode;
                const midiNote = this.pitchCodeToMidi(pitchCode);
                const fullDuration = this.calculateNoteDuration(note, elementDuration);
                // Notes last 3/4 of their full duration for natural articulation
                const playbackDuration = fullDuration * 0.75;

                events.push({
                    time: currentTime,
                    pitch: midiNote,
                    duration: playbackDuration,
                    velocity: 80, // Default velocity
                    channel: 0
                });

                console.log(`🎵 Note event: pitch=${pitchCode} → MIDI=${midiNote}, time=${currentTime.toFixed(3)}s, duration=${playbackDuration.toFixed(3)}s (3/4 of ${fullDuration.toFixed(3)}s)`);
                currentTime += elementDuration;
            } else if (dash) {
                // For dashes, we either extend the previous note or create a rest
                // For now, treat as rest by not adding an event
                currentTime += elementDuration;
            } else if (breathMark) {
                // Add a small pause for breath marks
                currentTime += 0.15; // 150ms pause
            }
        }

        console.log(`Beat processed: ${events.length} events`);
        return events;
    }

    // Pitch Mapping (PitchCode to MIDI)
    pitchCodeToMidi(pitchCode) {
        const baseMidiMap = {
            'N1': 60,   // C4
            'N1s': 61,  // C#4
            'N2b': 61,  // Db4
            'N2': 62,   // D4
            'N2s': 63,  // D#4
            'N3b': 63,  // Eb4
            'N3': 64,   // E4
            'N4': 65,   // F4
            'N4s': 66,  // F#4
            'N5': 67,   // G4
            'N5s': 68,  // G#4
            'N6b': 68,  // Ab4
            'N6': 69,   // A4
            'N6s': 70,  // A#4
            'N7b': 70,  // Bb4
            'N7': 71    // B4
        };

        console.log('🎵 Converting pitchCode to MIDI:', pitchCode);

        // Handle different pitch code formats
        if (!pitchCode) {
            console.warn('No pitch code provided');
            return 60; // Default to middle C
        }

        // If it's already a string, use it
        let pitchStr = typeof pitchCode === 'string' ? pitchCode : pitchCode.toString();

        // Handle enum-style pitch codes (e.g., { "N1": null })
        if (typeof pitchCode === 'object' && !Array.isArray(pitchCode)) {
            const keys = Object.keys(pitchCode);
            if (keys.length > 0) {
                pitchStr = keys[0];
            }
        }

        const midi = baseMidiMap[pitchStr];
        if (midi === undefined) {
            console.warn(`Unknown pitch code: ${pitchStr}, defaulting to C4`);
            return 60;
        }

        console.log(`🎵 Mapped ${pitchStr} → MIDI ${midi}`);
        return midi;
    }

    // Duration Calculations
    getBeatDuration(beat) {
        if (beat.total_duration) {
            return this.rationalToSeconds(beat.total_duration);
        }
        return 0.5; // Default quarter note at 120 BPM
    }

    getContentLineDuration(contentLine) {
        let totalDuration = 0;
        for (const element of contentLine.elements) {
            if (element.Beat) {
                totalDuration += this.getBeatDuration(element.Beat);
            }
        }
        return totalDuration;
    }

    getStaveDuration(stave) {
        let totalDuration = 0;
        for (const line of stave.lines) {
            if (line.ContentLine) {
                totalDuration += this.getContentLineDuration(line.ContentLine);
            }
        }
        return totalDuration;
    }

    calculateNoteDuration(note, defaultDuration) {
        if (note.duration) {
            return this.rationalToSeconds(note.duration);
        }
        return defaultDuration;
    }

    rationalToSeconds(rational) {
        // Convert rational duration to seconds based on current BPM
        // Handle both old format and new Rational enum format
        let numerator, denominator;

        if (rational && rational.Rational && Array.isArray(rational.Rational)) {
            // New format: {"Rational": ["Plus", [1, 4]]}
            const [_, fraction] = rational.Rational;
            [numerator, denominator] = fraction;
        } else if (rational && typeof rational === 'object') {
            // Old format with methods
            numerator = rational.numer ? rational.numer() : 1;
            denominator = rational.denom ? rational.denom() : 4;
        } else {
            // Fallback
            numerator = 1;
            denominator = 4;
        }

        const seconds = (numerator / denominator) * (240 / this.bpm);
        console.log(`⏱️ Duration: ${numerator}/${denominator} → ${seconds}s at ${this.bpm} BPM`);
        return seconds;
    }

    // Event Scheduling
    scheduleEvents() {
        this.clearScheduledEvents();

        const now = Tone.now();
        const lookAhead = 2.0; // 2 seconds look-ahead

        for (const event of this.events) {
            if (event.time >= this.currentPosition &&
                event.time <= this.currentPosition + lookAhead) {

                const scheduledId = Tone.Transport.schedule((time) => {
                    this.playNote(event, time);
                }, event.time);

                this.scheduledEvents.push(scheduledId);
            }
        }
    }

    playNote(event, time) {
        if (!this.synth) {
            console.error('No synthesizer available');
            return;
        }

        try {
            const frequency = Tone.Frequency(event.pitch, "midi").toFrequency();
            const velocity = event.velocity / 127;

            console.log(`🎵 Playing note: MIDI=${event.pitch}, Freq=${frequency}Hz, Duration=${event.duration}s, Velocity=${velocity}`);

            // Use the simpler format for triggerAttackRelease
            this.synth.triggerAttackRelease(frequency, event.duration, time, velocity);

            this.emit('note', {
                pitch: event.pitch,
                time: time,
                duration: event.duration,
                frequency: frequency
            });
        } catch (error) {
            console.error('Error playing note:', error, event);
        }
    }

    clearScheduledEvents() {
        for (const id of this.scheduledEvents) {
            Tone.Transport.clear(id);
        }
        this.scheduledEvents = [];
    }

    // State Query Methods
    isPlaying() {
        return this.playing;
    }

    getCurrentTime() {
        return Tone.Transport.seconds;
    }

    getDuration() {
        if (!this.events.length) return 0;
        const lastEvent = this.events[this.events.length - 1];
        return lastEvent.time + lastEvent.duration;
    }

    // Event System
    on(event, callback) {
        if (this.eventListeners[event]) {
            this.eventListeners[event].push(callback);
        }
    }

    off(event, callback) {
        if (this.eventListeners[event]) {
            const index = this.eventListeners[event].indexOf(callback);
            if (index > -1) {
                this.eventListeners[event].splice(index, 1);
            }
        }
    }

    emit(event, data = null) {
        if (this.eventListeners[event]) {
            for (const callback of this.eventListeners[event]) {
                callback(data);
            }
        }
    }

    // Cleanup
    dispose() {
        this.stop();

        if (this.synth) this.synth.dispose();
        if (this.volume) this.volume.dispose();
        if (this.reverb) this.reverb.dispose();

        this.eventListeners = {};
        console.log('🗑️ MIDI Player disposed');
    }
}

// Create global player instance
const midiPlayer = new MusicTextPlayer();
window.MidiPlayer = midiPlayer;

// Export for module use
export { midiPlayer as MidiPlayer };