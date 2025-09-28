# JSON Pitch Systems Architecture Specification

## Overview

This specification defines a JSON-based architecture for defining pitch systems dynamically, replacing hardcoded Rust modules with data-driven configuration. This approach enables runtime extensibility and user-defined notation systems.

## JSON Format Specification

### Schema

```json
{
  "name": "string",
  "description": "string (optional)",
  "case_sensitive": "boolean (optional, default: false)",
  "mappings": {
    "symbol": "pitch_code",
    ...
  }
}
```

### Required Fields

- **`name`**: Unique identifier for the pitch system
- **`mappings`**: Object mapping notation symbols to PitchCode values

### Optional Fields

- **`description`**: Human-readable description
- **`case_sensitive`**: Whether symbol matching is case-sensitive

### PitchCode Values

Valid PitchCode values follow the pattern: `N{degree}{accidental}`

**Degrees**: `1`, `2`, `3`, `4`, `5`, `6`, `7`
**Accidentals**: `bb` (double flat), `b` (flat), `` (natural), `s` (sharp), `ss` (double sharp)

**Examples**: `N1`, `N2b`, `N3`, `N4s`, `N5bb`, `N6ss`, `N7`

## Example JSON Definitions

### Sargam System

```json
{
  "name": "sargam",
  "description": "Indian classical notation with komal/tivra variants",
  "case_sensitive": true,
  "mappings": {
    "S": "N1", "s": "N1",
    "r": "N2b", "R": "N2",
    "g": "N3b", "G": "N3",
    "m": "N4", "M": "N4s",
    "P": "N5", "p": "N5",
    "d": "N6b", "D": "N6",
    "n": "N7b", "N": "N7",
    "S#": "N1s", "R#": "N2s", "G#": "N3s",
    "P#": "N5s", "D#": "N6s", "N#": "N7s",
    "Sb": "N1b", "mb": "N4b", "Pb": "N5b"
  }
}
```

### Western System

```json
{
  "name": "western",
  "description": "Traditional Western notation with letter names",
  "case_sensitive": false,
  "mappings": {
    "C": "N1", "D": "N2", "E": "N3", "F": "N4",
    "G": "N5", "A": "N6", "B": "N7",
    "C#": "N1s", "D#": "N2s", "E#": "N3s", "F#": "N4s",
    "G#": "N5s", "A#": "N6s", "B#": "N7s",
    "Cb": "N1b", "Db": "N2b", "Eb": "N3b", "Fb": "N4b",
    "Gb": "N5b", "Ab": "N6b", "Bb": "N7b",
    "C##": "N1ss", "D##": "N2ss", "E##": "N3ss", "F##": "N4ss",
    "G##": "N5ss", "A##": "N6ss", "B##": "N7ss",
    "Cbb": "N1bb", "Dbb": "N2bb", "Ebb": "N3bb", "Fbb": "N4bb",
    "Gbb": "N5bb", "Abb": "N6bb", "Bbb": "N7bb"
  }
}
```

### Number System

```json
{
  "name": "number",
  "description": "Numeric notation with accidentals",
  "case_sensitive": false,
  "mappings": {
    "1": "N1", "2": "N2", "3": "N3", "4": "N4",
    "5": "N5", "6": "N6", "7": "N7",
    "1#": "N1s", "2#": "N2s", "3#": "N3s", "4#": "N4s",
    "5#": "N5s", "6#": "N6s", "7#": "N7s",
    "1b": "N1b", "2b": "N2b", "3b": "N3b", "4b": "N4b",
    "5b": "N5b", "6b": "N6b", "7b": "N7b",
    "1##": "N1ss", "2##": "N2ss", "3##": "N3ss", "4##": "N4ss",
    "5##": "N5ss", "6##": "N6ss", "7##": "N7ss",
    "1bb": "N1bb", "2bb": "N2bb", "3bb": "N3bb", "4bb": "N4bb",
    "5bb": "N5bb", "6bb": "N6bb", "7bb": "N7bb"
  }
}
```

## Implementation Architecture

### Loading Strategy

**Option 1: Compile-time inclusion**
```rust
const SARGAM_JSON: &str = include_str!("systems/sargam.json");
const WESTERN_JSON: &str = include_str!("systems/western.json");

let sargam = PitchSystem::from_json(SARGAM_JSON)?;
let western = PitchSystem::from_json(WESTERN_JSON)?;
```

**Option 2: Runtime loading**
```rust
let sargam = PitchSystem::from_file("systems/sargam.json")?;
let western = PitchSystem::from_file("systems/western.json")?;
```

### Auto-generation

The JSON loader automatically generates:

1. **Symbol list**: Extract `Object.keys(mappings)`, sorted by length
2. **Regex pattern**: Generate `(?i)(symbol1|symbol2|...)` with case sensitivity
3. **Reverse mapping**: Create `PitchCode → String` lookup table
4. **Validation**: Verify all PitchCode values are valid

### Core Implementation

```rust
pub struct PitchSystem {
    pub name: String,
    pub description: Option<String>,
    pub case_sensitive: bool,
    pub forward_map: HashMap<String, PitchCode>,  // String → PitchCode
    pub reverse_map: HashMap<PitchCode, String>,  // PitchCode → String
    pub symbols: Vec<String>,                     // For regex generation
    pub regex: Regex,                            // Compiled pattern
}

impl PitchSystem {
    pub fn from_json(json: &str) -> Result<Self, PitchSystemError> {
        // Parse JSON, validate mappings, generate reverse map and regex
    }

    pub fn lookup(&self, symbol: &str) -> Option<PitchCode> {
        // Forward lookup: String → PitchCode
    }

    pub fn pitchcode_to_string(&self, code: PitchCode) -> Option<String> {
        // Reverse lookup: PitchCode → String
    }
}
```

## File Organization

```
systems/
├── sargam.json
├── western.json
├── number.json
├── bhatkhande.json
├── tabla.json
└── custom/
    ├── do-re-mi.json
    ├── user-defined.json
    └── ...
```

## Migration Plan

### Phase 1: JSON Loader Implementation
1. Create `PitchSystem` struct with JSON parsing
2. Implement auto-generation of symbols, regex, reverse mapping
3. Add validation and error handling

### Phase 2: Convert Existing Systems
1. Extract mappings from hardcoded Rust modules
2. Create JSON files for each system
3. Update pitch_systems.rs to use JSON loader
4. Verify functionality with existing tests

### Phase 3: Deprecate Hardcoded Modules
1. Remove individual pitch system modules (sargam.rs, western.rs, etc.)
2. Update documentation and examples
3. Add support for user-defined systems directory

## Validation Rules

1. **PitchCode format**: Must match `N[1-7](bb|b|s|ss)?` pattern
2. **Unique mappings**: No duplicate PitchCode values in mappings
3. **Valid symbols**: No empty or whitespace-only symbols
4. **Name uniqueness**: System names must be unique across all loaded systems

## Extensibility

### User-Defined Systems

Users can create custom pitch systems by:
1. Creating JSON file following the schema
2. Placing in `systems/custom/` directory
3. System automatically loaded at startup

### Runtime Registration

```rust
let custom_system = PitchSystem::from_json(user_json)?;
pitch_system_registry.register(custom_system);
```

## Benefits

- **Maintainability**: Edit mappings without recompilation
- **Extensibility**: Users can define custom notation systems
- **Consistency**: All systems use identical JSON format
- **Performance**: Compile-time inclusion with runtime flexibility
- **Simplicity**: No boilerplate Rust code for new systems