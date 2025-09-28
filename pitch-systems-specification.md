# Pitch Systems Specification

## Overview

Music-Text supports multiple pitch notation systems with a unified internal representation. The **Number system** (1, 2, 3, 4, 5, 6, 7) serves as the universal normalized representation, being both universally understood and culturally neutral.

**Core Principles:**
- No enharmonic equivalents (C# ≠ Db in the system)
- All pitch systems must map to the Number system for normalization
- Theoretical completeness includes all chromatic possibilities (even B##, Cb)

## Musicological Concepts

### Tonic
The tonic concept is relevant primarily for Western notation, representing the home pitch or keynote of a scale or key.

### Key Signature
Music-Text treats key signature as separate from tonic. Key signatures indicate which pitches are altered throughout a piece, while tonic refers to the central pitch.

### Scale Degrees
Scale degrees provide a universal concept across all pitch systems:
- **1st degree**: Tonic (Sa, C, do)
- **2nd degree**: Supertonic (Re, D, re)
- **3rd degree**: Mediant (Ga, E, mi)
- **4th degree**: Subdominant (Ma, F, fa)
- **5th degree**: Dominant (Pa, G, sol)
- **6th degree**: Submediant (Dha, A, la)
- **7th degree**: Leading tone (Ni, B, ti)

## Supported Pitch Systems

### Number System (Internal Normalization)
**Examples:** `1`, `2b`, `2`, `2#`, `3`, `4`, `4#`, `5`, `6b`, `6`, `7b`, `7`

The Number system serves as the internal chromatic representation. All other systems map to these numeric pitch codes.

### Sargam System
**Examples:** `S`, `r`, `R`, `g`, `G`, `m`, `M`, `P`, `d`, `D`, `n`, `N`

Indian classical notation using Sanskrit syllables:
- **Case sensitivity**: Lowercase = komal (flat), uppercase = shuddha (natural)
- **Special cases**: `M` = tivra Ma (sharp 4th), `s`/`p` = aliases for `S`/`P`
- **Example mapping**: `g` (komal Ga) = 3b, `m` (shuddha Ma) = 4, `M` (tivra Ma) = 4#

### Western System
**Examples:** `C`, `Db`, `D`, `D#`, `E`, `F`, `F#`, `G`, `Ab`, `A`, `Bb`, `B`

Traditional Western notation using letter names with accidentals (#, b, ##, bb).

### Bhatkhande System
**Examples:** `स`, `रे`, `ग`, `म`, `प`, `ध`, `न` (Devanagari) or `S`, `R`, `G`, `M`, `P`, `D`, `N` (Roman)

Indian notation system supporting both Devanagari script and Roman equivalents.

### Tabla System
**Examples:** `dha`, `dhin`, `ta`, `ka`, `ge`, `na`

Percussion notation using onomatopoeic syllables (bols). All tabla symbols map to the same internal pitch for rhythmic representation.

## Implementation Requirements

Every pitch system must provide:
1. **Bidirectional mapping**: String ↔ PitchCode conversion
2. **Number system mapping**: Direct conversion to normalized representation
3. **Symbol validation**: Determine valid notation symbols

**Current Implementation**: Hardcoded Rust modules for each system
**Preferred Architecture**: JSON-based system definitions (see `json-pitch-systems-architecture.md`)

## Theoretical Completeness

The system includes all chromatic possibilities for completeness, even when musically uncommon:
- **Double sharps**: `B##`, `E##`, `F##`
- **Double flats**: `Cb`, `Fb`, `Dbb`
- **Enharmonic variants**: Treated as distinct (no C# = Db equivalence)

This ensures the system can represent any theoretical pitch relationship without musical judgment.