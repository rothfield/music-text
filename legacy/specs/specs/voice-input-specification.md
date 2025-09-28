# Voice Input Transcription Specification

## Overview
This specification defines how voice input can be used to create music notation in Music-Text format through speech recognition and natural language processing.

## User Experience

### Basic Voice Commands
User speaks naturally, and the system converts to music notation:

**Examples:**
- User says: "one two three four"
  - Outputs: `| 1 2 3 4 |`

- User says: "sa re ga ma"
  - Outputs: `| S R G M |`

- User says: "C D E F"
  - Outputs: `| C D E F |`

### Advanced Voice Commands

#### Rhythm Specification
- "quarter note C, eighth note D E"
  - Outputs: `| C D(2) E(2) |`

- "triplet sa re ga"
  - Outputs: `| [S R G] |`

#### Octave Specification
- "high sa, low re"
  - Outputs: `| S. R, |`

- "C up D down"
  - Outputs: `| C. D, |`

#### Bar Lines and Structure
- "bar C D bar E F bar"
  - Outputs: `| C D | E F |`

- "repeat C D end repeat"
  - Outputs: `|: C D :|`

## Technical Implementation

### Web Speech API Integration

```javascript
class VoiceInput {
    constructor() {
        this.recognition = new webkitSpeechRecognition() || new SpeechRecognition();
        this.setupRecognition();
    }

    setupRecognition() {
        this.recognition.continuous = true;
        this.recognition.interimResults = true;
        this.recognition.lang = 'en-US';

        this.recognition.onresult = (event) => {
            const transcript = event.results[event.results.length - 1][0].transcript;
            this.processVoiceInput(transcript);
        };
    }

    processVoiceInput(transcript) {
        const notation = this.transcriptToNotation(transcript);
        this.appendToEditor(notation);
    }

    transcriptToNotation(transcript) {
        // Convert spoken words to music notation
        const words = transcript.toLowerCase().split(' ');
        return this.parseMusicalWords(words);
    }
}
```

### Speech Recognition Patterns

#### Note Recognition
```javascript
const NOTE_MAPPINGS = {
    // Number system
    'one': '1', 'two': '2', 'three': '3', 'four': '4',
    'five': '5', 'six': '6', 'seven': '7',

    // Sargam system
    'sa': 'S', 'saa': 'S', 'shah': 'S',
    're': 'R', 'ray': 'R', 'ri': 'R',
    'ga': 'G', 'gaa': 'G', 'gah': 'G',
    'ma': 'M', 'maa': 'M', 'mah': 'M',
    'pa': 'P', 'paa': 'P', 'pah': 'P',
    'dha': 'D', 'dhaa': 'D', 'da': 'D',
    'ni': 'N', 'nee': 'N', 'knee': 'N',

    // Western system
    'c': 'C', 'see': 'C', 'sea': 'C',
    'd': 'D', 'dee': 'D',
    'e': 'E', 'ee': 'E',
    'f': 'F', 'eff': 'F',
    'g': 'G', 'gee': 'G', 'jee': 'G',
    'a': 'A', 'ay': 'A',
    'b': 'B', 'bee': 'B', 'be': 'B',

    // Accidentals
    'sharp': '#', 'flat': 'b', 'natural': '♮',
    'komal': 'b', 'tivra': '#'
};
```

#### Command Recognition
```javascript
const COMMAND_MAPPINGS = {
    // Structure
    'bar': '|', 'barline': '|',
    'repeat': '|:', 'end repeat': ':|',
    'double bar': '||',

    // Rhythm
    'whole': '(1)', 'half': '(2)',
    'quarter': '', 'eighth': '(8)',
    'sixteenth': '(16)',
    'triplet': '[', 'end triplet': ']',

    // Octave
    'up': '.', 'higher': '.', 'high': '.',
    'down': ',', 'lower': ',', 'low': ',',
    'upper': '^', 'lower': '_',

    // Control
    'new line': '\n', 'next line': '\n',
    'space': ' ', 'rest': '-', 'dash': '-',
    'breath': ',', 'comma': ','
};
```

### Natural Language Processing

#### Phrase Parsing
```javascript
class PhraseParser {
    parsePhrase(words) {
        const result = [];
        let i = 0;

        while (i < words.length) {
            // Check for compound commands
            if (words[i] === 'c' && words[i + 1] === 'sharp') {
                result.push('C#');
                i += 2;
            }
            // Check for rhythm specifications
            else if (words[i] === 'eighth' && words[i + 1] === 'note') {
                const note = this.parseNote(words[i + 2]);
                result.push(note + '(8)');
                i += 3;
            }
            // Check for octave modifications
            else if (words[i] === 'high' && this.isNote(words[i + 1])) {
                const note = this.parseNote(words[i + 1]);
                result.push(note + '.');
                i += 2;
            }
            // Default note parsing
            else if (this.isNote(words[i])) {
                result.push(this.parseNote(words[i]));
                i++;
            }
            // Skip unrecognized words
            else {
                i++;
            }
        }

        return '| ' + result.join(' ') + ' |';
    }
}
```

## UI Integration

### Voice Input Button
```html
<button id="voice-input-btn" onclick="toggleVoiceInput()">
    🎤 Voice Input
</button>

<div id="voice-status" class="voice-status">
    <span class="recording-indicator">●</span>
    <span id="transcript-preview"></span>
</div>
```

### Visual Feedback
```css
.voice-status {
    display: none;
    padding: 10px;
    background: #f0f0f0;
    border-radius: 5px;
}

.voice-status.active {
    display: flex;
    align-items: center;
}

.recording-indicator {
    color: red;
    animation: pulse 1s infinite;
    margin-right: 10px;
}

@keyframes pulse {
    0% { opacity: 1; }
    50% { opacity: 0.5; }
    100% { opacity: 1; }
}
```

## Voice Training Mode

### Guided Input
System can guide users through voice input:

1. **Tutorial Mode**: Teaches voice commands
2. **Practice Mode**: Shows expected vs. actual input
3. **Correction Mode**: Allows voice corrections

### Example Tutorial
```
System: "Say 'sa re ga ma' to input those notes"
User: "sa re ga ma"
System: "Great! Now say 'bar' to add a bar line"
User: "bar"
Result: | S R G M |
```

## Language Support

### Multi-language Recognition
```javascript
const LANGUAGE_CONFIGS = {
    'en-US': {
        notes: ['c', 'd', 'e', 'f', 'g', 'a', 'b'],
        commands: ['bar', 'repeat', 'sharp', 'flat']
    },
    'hi-IN': {
        notes: ['सा', 'रे', 'ग', 'म', 'प', 'ध', 'नि'],
        commands: ['बार', 'दोहराना', 'कोमल', 'तीव्र']
    },
    'fr-FR': {
        notes: ['do', 'ré', 'mi', 'fa', 'sol', 'la', 'si'],
        commands: ['barre', 'répéter', 'dièse', 'bémol']
    }
};
```

## Error Handling

### Recognition Errors
```javascript
class VoiceErrorHandler {
    handleError(error) {
        switch(error.error) {
            case 'no-speech':
                this.showMessage('No speech detected. Please try again.');
                break;
            case 'audio-capture':
                this.showMessage('No microphone found. Please check your audio input.');
                break;
            case 'not-allowed':
                this.showMessage('Microphone permission denied.');
                break;
            default:
                this.showMessage('Voice input error. Please try again.');
        }
    }
}
```

### Ambiguity Resolution
When the system isn't sure:
```
User says: "see sharp"
System shows options:
1. C# (C sharp)
2. "see sharp" (literal text)
3. Skip

User can click or say the number to choose.
```

## Privacy and Security

### Microphone Permissions
- Request permission only when voice button clicked
- Clear visual indicator when recording
- Stop recording on button click or timeout

### Data Handling
- Process locally when possible (Web Speech API)
- No audio stored without user consent
- Clear indication if using cloud services

## Performance Considerations

### Optimization Strategies
1. **Chunked Processing**: Process phrases in chunks
2. **Confidence Threshold**: Only accept high-confidence results
3. **Timeout Handling**: Auto-stop after 30 seconds of silence
4. **Resource Management**: Release microphone when not in use

## Testing Approach

### Test Phrases
```javascript
const TEST_PHRASES = [
    { input: "one two three four", expected: "| 1 2 3 4 |" },
    { input: "sa re ga ma", expected: "| S R G M |" },
    { input: "c sharp d flat", expected: "| C# Db |" },
    { input: "high c low d", expected: "| C. D, |" },
    { input: "triplet one two three", expected: "| [1 2 3] |" }
];
```

### Accent Testing
Test with various accents and pronunciations:
- American English
- British English
- Indian English
- Non-native speakers

## Future Enhancements

### Advanced Features
1. **Humming Input**: Convert hummed melodies to notation
2. **Rhythm Tapping**: Tap rhythm while speaking notes
3. **Chord Recognition**: "C major chord" → multiple notes
4. **Lyrics Integration**: Speak lyrics with melody

### AI Integration
1. **Context Understanding**: "Make that note higher"
2. **Pattern Recognition**: "Repeat that phrase"
3. **Style Suggestions**: "Make it more jazzy"
4. **Error Correction**: "Change the last note to D"

## Implementation Priority

### Phase 1: Basic Voice Input
- Simple note recognition
- Basic commands (bar, repeat)
- English only

### Phase 2: Enhanced Recognition
- Rhythm specifications
- Octave controls
- Multiple languages

### Phase 3: Advanced Features
- Natural language commands
- Context-aware corrections
- Humming/singing input