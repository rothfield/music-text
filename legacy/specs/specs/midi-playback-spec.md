# MIDI Playback Specification for Music-Text

## Implementation Status
- **Phase 1**: ✅ Completed (Basic Playback MVP)
- **Phase 2**: 🔄 In Progress (Enhanced Features)
- **Phase 3**: 📋 Planned (Advanced Features)

**Last Updated**: 2025-01-19 - **MIDI PLAYBACK NOW FULLY FUNCTIONAL**

## Overview
This specification defines how parsed music-text documents are converted to MIDI playback using JavaScript libraries in the web interface.

## Quick Reference

### What's Working Now
- ✅ Basic playback of parsed documents
- ✅ Play/Pause/Stop controls
- ✅ Tempo adjustment (40-208 BPM)
- ✅ All notation systems (Sargam, Number, Western)
- ✅ Tuplets and complex rhythms
- ✅ Breath marks (150ms pause)

### What's Coming Next
- 🔄 Octave adjustments from spatial lines
- 🔄 Volume control slider
- 🔄 Loop functionality
- 📋 Visual feedback (note highlighting)
- 📋 Multi-voice support
- 📋 MIDI file export

### How to Use
1. Enter music notation in the text area
2. Click **Parse** to analyze the notation
3. Click **▶️ Play** to start MIDI playback
4. Adjust tempo with the slider
5. Use **⏸️ Pause** and **⏹️ Stop** as needed

## 1. Technology Stack

### Primary Library: Tone.js
- **Version**: Latest stable (>=14.7.0)
- **Rationale**:
  - Active development and large community
  - Built-in timing and scheduling engine
  - Works well with existing VexFlow renderer
  - Supports complex rhythms and tuplets
  - Web Audio API abstraction

### Supporting Libraries
- **Soundfont**: For realistic instrument sounds (optional enhancement)
- **Web MIDI API**: For hardware MIDI device output (future enhancement)

## 2. Architecture

### 2.1 Component Structure
```
MusicTextPlayer
├── Core Engine (Tone.js Transport)
├── Pitch Mapper (PitchCode → MIDI)
├── Rhythm Converter (Rational → Time)
├── Playback Controller (Play/Pause/Stop/Tempo)
└── Instrument Manager (Synth selection)
```

### 2.2 Data Flow
```
Parsed Document → Playback Queue → Tone.js Scheduler → Audio Output
                      ↑
                 User Controls
```

## 3. Core Mappings

### 3.1 Pitch Code to MIDI Mapping (✅ Implemented)

| PitchCode | MIDI Note | Frequency | Sargam | Western | Number |
|-----------|-----------|-----------|---------|---------|---------|
| N1        | 60        | 261.63 Hz | Sa      | C4      | 1       |
| N1s       | 61        | 277.18 Hz | S#      | C#4     | 1#      |
| N2b       | 61        | 277.18 Hz | komal Re| Db4     | 2b      |
| N2        | 62        | 293.66 Hz | Re      | D4      | 2       |
| N2s       | 63        | 311.13 Hz | R#      | D#4     | 2#      |
| N3b       | 63        | 311.13 Hz | komal Ga| Eb4     | 3b      |
| N3        | 64        | 329.63 Hz | Ga      | E4      | 3       |
| N4        | 65        | 349.23 Hz | ma      | F4      | 4       |
| N4s       | 66        | 369.99 Hz | tivra Ma| F#4     | 4#      |
| N5        | 67        | 392.00 Hz | Pa      | G4      | 5       |
| N5s       | 68        | 415.30 Hz | P#      | G#4     | 5#      |
| N6b       | 68        | 415.30 Hz | komal Dha| Ab4    | 6b      |
| N6        | 69        | 440.00 Hz | Dha     | A4      | 6       |
| N6s       | 70        | 466.16 Hz | D#      | A#4     | 6#      |
| N7b       | 70        | 466.16 Hz | komal Ni| Bb4     | 7b      |
| N7        | 71        | 493.88 Hz | Ni      | B4      | 7       |

**Implementation Note**: The current implementation uses a simplified mapping in `midiPlayer.js`:
```javascript
const baseMidiMap = {
    'N1': 60, 'N1s': 61, 'N2b': 61, 'N2': 62, 'N2s': 63,
    'N3b': 63, 'N3': 64, 'N4': 65, 'N4s': 66, 'N5': 67,
    'N5s': 68, 'N6b': 68, 'N6': 69, 'N6s': 70,
    'N7b': 70, 'N7': 71
};
```

### 3.2 Octave Adjustments (🔄 Planned)
- Base octave: 4 (middle octave)
- Octave modifiers from spatial analysis:
  - Upper markers (`.`, `:`, `*`): +1, +2, +3 octaves
  - Lower markers (`.`, `:`): -1, -2 octaves
- Formula: `final_midi = base_midi + (octave * 12)`

**Current Status**: Not yet implemented. Octave markers in spatial lines need to be parsed and applied.

### 3.3 Duration Mapping (✅ Implemented)

#### Standard Durations
| Rational  | Musical Value | Tone.js | Seconds at 120 BPM |
|-----------|---------------|---------|---------------------|
| 1/1       | Whole         | "1n"    | 2.0                 |
| 1/2       | Half          | "2n"    | 1.0                 |
| 1/4       | Quarter       | "4n"    | 0.5                 |
| 1/8       | Eighth        | "8n"    | 0.25                |
| 1/16      | Sixteenth     | "16n"   | 0.125               |
| 1/32      | 32nd          | "32n"   | 0.0625              |

#### Note Duration Implementation (✅ Required)
- **Note Duration**: Notes should last 3/4 of their full duration
- **Gap Between Notes**: 1/4 of duration for natural articulation
- Formula: `playback_duration = calculated_duration * 0.75`

#### Piece Repetition (✅ Required)
- **Repeat Count**: Entire piece should repeat 4 times total
- **Implementation**: Loop the complete parsed document 4 times
- **Timing**: Use consistent tempo across all repetitions
- **Pause Between Repetitions**: 4 seconds of silence between each repetition

#### Tuplet Durations (✅ Implemented)
- Formula: `tuplet_duration = beat_duration / divisions`
- Example: 5 notes in 1 beat = each note gets 1/5 of beat duration

#### Duration Calculation (✅ Implemented)
```javascript
rationalToSeconds(rational) {
    const numerator = rational.numer ? rational.numer() : 1;
    const denominator = rational.denom ? rational.denom() : 4;
    return (numerator / denominator) * (240 / this.bpm);
}
```

## 4. Playback Engine Implementation

### 4.1 Class Structure

```javascript
class MusicTextPlayer {
  // Configuration
  bpm: number = 120
  swing: number = 0
  humanize: number = 0

  // State
  playing: boolean = false
  currentPosition: number = 0
  loop: boolean = false
  loopStart: number = 0
  loopEnd: number = null

  // Audio
  synth: Tone.PolySynth
  volume: Tone.Volume
  reverb: Tone.Reverb (optional)

  // Methods
  play(document: ParsedDocument): void
  pause(): void
  stop(): void
  seek(position: number): void
  setTempo(bpm: number): void
  setInstrument(type: string): void
  exportMIDI(): Blob
}
```

### 4.2 Playback Algorithm

```typescript
interface PlaybackEvent {
  time: number;        // Start time in seconds
  pitch: number;       // MIDI note number
  duration: number;    // Duration in seconds
  velocity: number;    // 0-127
  channel: number;     // MIDI channel (0-15)
}

function documentToPlaybackEvents(document: ParsedDocument): PlaybackEvent[] {
  const events: PlaybackEvent[] = [];
  let globalTime = 0;

  for (const element of document.elements) {
    if (element.Stave) {
      for (const line of element.Stave.lines) {
        if (line.ContentLine) {
          const lineEvents = processContentLine(line.ContentLine, globalTime);
          events.push(...lineEvents);
          globalTime += getLineDuration(line.ContentLine);
        }
      }
    }
  }

  return events;
}
```

## 5. User Interface Controls

### 5.1 Transport Controls (✅ Implemented)
- **Play/Pause Button**: Toggle playback ✅
- **Stop Button**: Stop and reset to beginning ✅
- **Tempo Slider**: 40-208 BPM ✅
- **Tempo Display**: Current BPM setting ✅
- **Loop Toggle**: Enable/disable looping 🔄 Planned
- **Volume Slider**: -60dB to 0dB 🔄 Planned

### 5.2 Visual Feedback (🔄 Planned)
- **Playhead**: Moving cursor on VexFlow notation 📋
- **Note Highlighting**: Current playing note(s) highlighted 📋
- **Beat Counter**: Current beat/measure display 📋
- **Time Display**: Elapsed/Total time 📋

### 5.3 Keyboard Shortcuts (🔄 Planned)
- `Space`: Play/Pause 📋
- `Enter`: Stop 📋
- `←/→`: Seek backward/forward 📋
- `↑/↓`: Tempo up/down 📋
- `L`: Toggle loop 📋

## 6. Special Elements Handling

### 6.1 Breath Marks (✅ Implemented)
- Add pause of 150ms ✅
- Configurable breath duration 🔄 Planned
- Visual indication during playback 📋 Planned

### 6.2 Dashes (Extended Notes) (✅ Partial)
- Create rest if no previous pitch ✅
- Continue previous pitch 🔄 Planned (needs tie logic)
- Duration based on beat division ✅

### 6.3 Barlines (✅ Basic)
- Single barline: No effect on playback ✅
- Repeat barlines: Implement repeat logic 📋 Planned
- End barline: Stop playback 📋 Planned

### 6.4 Ornaments (Future)
- Grace notes: Quick notes before main note
- Mordents: Rapid alternation
- Slides: Pitch bend between notes

## 7. Advanced Features

### 7.1 Multi-Voice Support
- Separate tracks for each stave
- Independent volume/pan per voice
- Synchronization between voices

### 7.2 Dynamics (Future)
- Parse dynamic markings (p, f, mf, etc.)
- Map to MIDI velocity values
- Smooth transitions (crescendo/diminuendo)

### 7.3 Articulations (Future)
- Staccato: Shorten note to 50% duration
- Legato: Full duration, no gap
- Accent: Increase velocity by 20%

## 8. Performance Optimizations

### 8.1 Scheduling
- Pre-calculate all events before playback
- Use Tone.Transport for accurate timing
- Buffer events in chunks of 1-2 seconds

### 8.2 Memory Management
- Dispose of completed events
- Limit polyphony to 32 voices
- Implement note stealing for overflow

### 8.3 Latency Compensation
- Look-ahead time: 100ms
- Schedule events early
- Adjust for system audio latency

## 9. Export Capabilities

### 9.1 MIDI File Export
- Standard MIDI File (SMF) Type 1
- Preserve tempo changes
- Include program changes
- Metadata (title, author)

### 9.2 Audio Export (Future)
- WAV/MP3 export via Web Audio API
- Offline rendering for quality
- Configurable sample rate/bit depth

## 10. Error Handling

### 10.1 Graceful Degradation
- Missing durations: Default to quarter note
- Invalid pitch codes: Skip note with warning
- Unsupported elements: Log and continue

### 10.2 User Notifications
- Audio context blocked: Show enable button
- Browser compatibility: Feature detection
- Performance issues: Suggest reduced polyphony

## 11. Testing Requirements

### 11.1 Unit Tests
- Pitch mapping accuracy
- Duration calculations
- Tuplet timing
- Event scheduling

### 11.2 Integration Tests
- Full document playback
- Loop functionality
- Tempo changes
- Multi-stave synchronization

### 11.3 Browser Compatibility
- Chrome 80+
- Firefox 75+
- Safari 13.1+
- Edge 80+

## 12. Implementation Phases

### Phase 1: Basic Playback (MVP) ✅ COMPLETED
- Single voice playback ✅
- Basic pitch/duration mapping ✅
- Play/pause/stop controls ✅
- Tempo adjustment ✅
- Document structure parsing ✅
- AudioContext initialization ✅
- Tone.js integration ✅

### Phase 2: Enhanced Features 🔄 IN PROGRESS
- Multi-voice support 📋
- Visual feedback on notation 📋
- Loop functionality 📋
- Breath marks and ties ✅ (partial)
- Volume control 📋
- Octave adjustments 📋

### Phase 3: Advanced Features 📋 PLANNED
- MIDI export 📋
- Instrument selection 📋
- Dynamics and articulations 📋
- Audio export 📋
- Soundfont support 📋

## 13. Configuration

### 13.1 Default Settings
```javascript
const DEFAULT_CONFIG = {
  bpm: 120,
  instrument: 'synth',
  volume: -6, // dB
  reverb: 0.2,
  swing: 0,
  humanize: 0,
  breathDuration: 150, // ms
  lookAhead: 100, // ms
  bufferSize: 2048
};
```

### 13.2 User Preferences
- Store in localStorage
- Per-document overrides
- Export/import settings

## 14. API Reference

### 14.1 Public Methods
```typescript
interface MusicTextPlayerAPI {
  // Playback control
  play(): Promise<void>;
  pause(): void;
  stop(): void;
  seek(seconds: number): void;

  // Configuration
  setTempo(bpm: number): void;
  setVolume(dB: number): void;
  setInstrument(name: string): void;

  // State queries
  isPlaying(): boolean;
  getCurrentTime(): number;
  getDuration(): number;

  // Events
  on(event: string, callback: Function): void;
  off(event: string, callback: Function): void;
}
```

### 14.2 Events
- `play`: Playback started
- `pause`: Playback paused
- `stop`: Playback stopped
- `note`: Note triggered {pitch, time, duration}
- `beat`: Beat boundary crossed
- `end`: Playback completed

## 15. Security Considerations

- Sanitize all user inputs
- Prevent audio feedback loops
- Limit resource consumption
- Validate MIDI data before export
- Use Content Security Policy for external soundfonts

## 16. Current Implementation Details

### 16.1 File Structure
```
webapp/public/
├── js/
│   ├── midiPlayer.js    # Core MIDI player module
│   └── app.js           # Integration with main app
├── css/
│   └── style.css        # MIDI control styling
└── index.html           # UI controls
```

### 16.2 Key Functions
```javascript
// Core playback methods
play(document)           // Start playback with parsed document
pause()                  // Pause current playback
stop()                   // Stop and reset position
setTempo(bpm)           // Adjust tempo (40-208)

// Conversion methods
documentToPlaybackEvents(document)    // Parse to MIDI events
pitchCodeToMidi(pitchCode)           // PitchCode to MIDI number
rationalToSeconds(rational)          // Duration conversion
```

### 16.3 Integration Points
- **Parser Output**: Uses `parsed_document` from API response
- **UI Controls**: HTML buttons with onclick handlers
- **Event System**: Custom events for play/pause/stop states
- **Tone.js**: Web Audio synthesis and scheduling

### 16.4 Known Limitations
1. **No octave support**: Upper/lower line markers not processed ⚠️
2. **Single voice only**: Multi-stave playback not implemented ⚠️
3. **No visual feedback**: Notes don't highlight during playback ⚠️
4. **Basic synthesis**: Default Tone.js synth (no soundfonts) ⚠️
5. **No ties/slurs**: Dashes create rests instead of extending notes ⚠️

**Note**: Core MIDI playback functionality is now working - all basic features are operational.

### 16.5 Browser Requirements
- Web Audio API support required
- ES6 modules support
- Tone.js v14.7.77 or compatible
- User interaction required to start audio context

### 16.6 Testing Checklist
- [ ] Parse document before playing
- [ ] Play/pause/stop controls work
- [ ] Tempo slider updates BPM
- [ ] All notation systems play correctly
- [ ] Breath marks add pauses
- [ ] Button states update properly
- [ ] Error handling for invalid input