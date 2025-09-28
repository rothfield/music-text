# LilyPond Template System Research

This document analyzes the LilyPond template systems from both doremi-script (Clojure) and old.music-text (Rust) to inform the new template implementation.

## Template Evolution History

### Doremi-Script Approach (Clojure-based)
- **Language**: Clojure/ClojureScript
- **Template Engine**: Mustache
- **Templates**: 2 main templates in `doremi-script/grammar/templates/`
- **Philosophy**: Functional, data-driven approach with rich musical intelligence

### Old.Music-Text Approach (Rust-based) 
- **Language**: Rust
- **Template Engine**: Mustache
- **Templates**: 4 specialized templates in `src/renderers/lilypond/templates/`
- **Philosophy**: Type-safe, modular approach with clear separation of concerns

## Template Analysis by Type

### 1. Minimal Templates - Bare Bones Output

#### **old.music-text/minimal.ly.mustache**
```mustache
\score {
  <<
    \new Staff { \relative c' { {{{staves}}} } }
  >>
}
```
**Purpose**: Testing and debug output - just the essential musical content
**Key Feature**: Uses `\relative c'` for automatic octave selection

#### **old.music-text/testing.ly.mustache** 
```mustache
\version "{{version}}"
\language "english"

\paper {
  tagline = ##f
  indent = 0\mm
  ragged-right = ##t
  top-margin = 5\mm
  bottom-margin = 5\mm
  left-margin = 5\mm
  right-margin = 5\mm
}

\fixed c' {
  \clef treble
  {{#time_signature}}{{{time_signature}}}{{/time_signature}}
  {{#key_signature}}{{{key_signature}}}{{/key_signature}}
  {{{staves}}}
}
```
**Purpose**: Clean testing format without score wrapper
**Key Feature**: Uses `\fixed c'` for predictable octave handling

### 2. Standard Templates - Web Optimized

#### **old.music-text/standard.ly.mustache**
```mustache
\version "{{version}}"
\language "english"

{{#source_comment}}
% Original notation source:
{{{source_comment}}}
{{/source_comment}}

\header { 
  {{#title}}title = "{{{title}}}"{{/title}}
  {{#composer}}composer = "{{{composer}}}"{{/composer}}
  tagline = ##f
  print-page-number = ##f
  oddHeaderMarkup = ##f
  evenHeaderMarkup = ##f
  oddFooterMarkup = ##f
  evenFooterMarkup = ##f
}

\paper {
  indent = 0\mm
  top-margin = 0.5\mm
  bottom-margin = 0.5\mm
  left-margin = 1\mm
  right-margin = 1\mm
  ragged-right = ##t
  page-breaking = #ly:one-page-breaking
  system-system-spacing = #'((basic-distance . 1) (minimum-distance . 1) (padding . 0) (stretchability . 0))
  markup-system-spacing = #'((basic-distance . 0) (minimum-distance . 0) (padding . 0) (stretchability . 0))
  score-system-spacing = #'((basic-distance . 0) (minimum-distance . 0) (padding . 0) (stretchability . 0))
  top-system-spacing = #'((basic-distance . 1) (minimum-distance . 1) (padding . 0) (stretchability . 0))
  last-bottom-spacing = #'((basic-distance . 1) (minimum-distance . 1) (padding . 0) (stretchability . 0))
  paper-height = 50\mm
  paper-width = 200\mm
}

\score {
  \new Staff {
    \fixed c' {
      \key c \major
      {{#time_signature}}{{{time_signature}}}{{/time_signature}}{{^time_signature}}\time 4/4{{/time_signature}}
      \autoBeamOff
      \set Score.measureBarType = #""
      \set Score.startRepeatBarType = #""
      \set Score.endRepeatBarType = #""
      {{#key_signature}}{{{key_signature}}}{{/key_signature}}
      {{{staves}}}
    }
  }
  
  {{#lyrics}}
  \addlyrics { 
    \override LyricText.font-size = #-2
    \override LyricText.font-shape = #'italic
    {{{lyrics}}} 
  }
  {{/lyrics}}
  
  \layout {
    \context {
      \Score
      \override SpacingSpanner.base-shortest-duration = #(ly:make-moment 1/32)
      \override SpacingSpanner.shortest-duration-space = #0.8
      \remove "Bar_number_engraver"
    }
  }
}
```

**Purpose**: Professional web UI output with compact dimensions
**Key Features**:
- **`\fixed c'`**: Predictable octave handling (critical improvement over relative)
- **Compact paper**: 200mm × 50mm optimized for web display
- **Professional typography**: Custom spacing, no bar numbers, clean headers
- **Conditional sections**: Title, composer, lyrics only appear if provided
- **Source comments**: Original input preserved for debugging

### 3. Multi-Stave Templates - Complex Arrangements

#### **old.music-text/multi-stave.ly.mustache**
```mustache
\version "{{version}}"
\language "english"

{{#source_comment}}
% Original notation source:
{{{source_comment}}}
{{/source_comment}}

\header { 
  {{#title}}title = "{{{title}}}"{{/title}}
  {{#composer}}composer = "{{{composer}}}"{{/composer}}
  tagline = ##f
  print-page-number = ##f
  oddHeaderMarkup = ##f
  evenHeaderMarkup = ##f
  oddFooterMarkup = ##f
  evenFooterMarkup = ##f
}

\paper {
  indent = 0\mm
  top-margin = 0.5\mm
  bottom-margin = 0.5\mm
  left-margin = 1\mm
  right-margin = 1\mm
  ragged-right = ##t
  page-breaking = #ly:one-page-breaking
  system-system-spacing = #'((basic-distance . 2) (minimum-distance . 2) (padding . 0) (stretchability . 0))
  markup-system-spacing = #'((basic-distance . 0) (minimum-distance . 0) (padding . 0) (stretchability . 0))
  score-system-spacing = #'((basic-distance . 0) (minimum-distance . 0) (padding . 0) (stretchability . 0))
  top-system-spacing = #'((basic-distance . 1) (minimum-distance . 1) (padding . 0) (stretchability . 0))
  last-bottom-spacing = #'((basic-distance . 1) (minimum-distance . 1) (padding . 0) (stretchability . 0))
  paper-height = 100\mm
  paper-width = 200\mm
}

\score {
  <<
    {{{staves}}}
  >>
  
  \layout {
    \context {
      \Score
      \override SpacingSpanner.base-shortest-duration = #(ly:make-moment 1/32)
      \override SpacingSpanner.shortest-duration-space = #0.8
      \remove "Bar_number_engraver"
    }
  }
}
```

**Purpose**: Multiple staves in parallel
**Key Features**:
- **Taller layout**: 100mm height vs 50mm for single stave
- **Parallel staves**: `<<` `>>` wrapper for simultaneous staff display
- **Increased spacing**: `basic-distance . 2` between systems

### 4. Doremi-Script Templates - MIDI-Enhanced

#### **doremi-script/templates/1/lilypond.mustache** - Simple Format
```mustache
#(ly:set-option 'midi-extension "mid")
\version "{{version}}"
\include "english.ly"
\header{ 
title = "{{{title}}}" 
composer = "{{{composer}}}" 
tagline = ""  % remove lilypond footer
}

\include "english.ly"

%{
    {{{doremi-source}}}
%}

{{{staves}}}
\layout {
  \context {
  \Score
  \remove "Bar_number_engraver"
  } 
}
```

**Purpose**: Basic output with MIDI support
**Key Features**:
- **MIDI export**: `#(ly:set-option 'midi-extension "mid")`
- **Source preservation**: Original input in `%{ %} ` comments
- **English notation**: `\include "english.ly"` for sharp/flat names

#### **doremi-script/templates/2/lilypond.mustache** - Advanced Format
```mustache
#(ly:set-option 'midi-extension "mid")
\version "{{version}}"
\include "english.ly"
\header{ 
title = "{{{title}}}" 
composer = "{{{composer}}}" 
tagline = ""  % remove lilypond footer!!!
}

\include "english.ly"

%{
    {{{doremi-source}}}
%}

melody =  {
		%  \clef treble
%		\cadenzaOn
    \accidentalStyle modern-cautionary
		{{{time-signature-snippet}}}
		{{{key-signature-snippet}}}
    \autoBeamOn  
		\override Staff.TimeSignature #'style = #'()
{{{staves}}}
}

\score {
	\new Staff <<
   {{{transpose-snippet}}} \melody
   \addlyrics {  {{{all-lyrics}}}
	 }
>>
\layout { }
		\midi {
				\context {
						\Staff
				}
				\tempo 2 = 72
		}
}
```

**Purpose**: Full-featured output with transposition and MIDI
**Key Features**:
- **Melody wrapper**: Separates musical content from score structure
- **Transposition support**: `{{{transpose-snippet}}} \melody`
- **Advanced notation**: `\accidentalStyle modern-cautionary`
- **MIDI with tempo**: `\tempo 2 = 72`
- **Lyrics integration**: `\addlyrics { {{{all-lyrics}}} }`

## Template Variables Documentation

### Essential Variables (Core Functionality)
- **`{{version}}`**: LilyPond version string (e.g., "2.24.0")
- **`{{{staves}}}`**: Generated musical content (triple braces for no HTML escaping)

### Metadata Variables (Optional Features)  
- **`{{#title}}{{{title}}}{{/title}}`**: Conditional title display
- **`{{#composer}}{{{composer}}}{{/composer}}`**: Conditional composer display  
- **`{{#source_comment}}{{{source_comment}}}{{/source_comment}}`**: Original input preservation

### Musical Variables (Advanced Features)
- **`{{#time_signature}}{{{time_signature}}}{{/time_signature}}`**: Dynamic time signature
- **`{{#key_signature}}{{{key_signature}}}{{/key_signature}}`**: Dynamic key signature
- **`{{{transpose-snippet}}}`**: Transposition commands (doremi-script only)

### Layout Variables (Future Extensions)
- **`{{#lyrics}}{{{lyrics}}}{{/lyrics}}`**: Lyrics section with formatting
- **`{{{all-lyrics}}}`**: Complete lyrics string (doremi-script style)
- **`{{{doremi-source}}}`**: Original source preservation in comments

## ⭐ Interesting Doremi-Script Approaches

### 1. Adaptive Tuplet Logic (Lines 185-267)

**Problem**: Converting fractional durations to LilyPond note values
**Solution**: Smart ratio-to-duration mapping with comprehensive lookup table

```clojure
(def ratio->lilypond-durations
  "ratio->lilypond-durations(3 4) => ['8.']   Ratio is ratio of 1/4 note "
  [my-numerator subdivisions-in-beat]
  
  (let [my-ratio (/ my-numerator subdivisions-in-beat)]
    ;; For subdivision of 3, use 3 1/8 notes.
    ;; For subdivision of 5 use 5 1/16th notes.
    ;; For 6 use 16th notes, etc
    (if (integer? my-ratio)
      ({ 1 ["4"], 2 ["2"], 3 ["2."], 4 ["1"] } my-ratio)
      ;; else - fraction mapping with comprehensive table
      (let [my-table
            { 1 ["4"]           ;; whole beat = quarter note
             (/ 1 2) ["8"]      ;; half beat = eighth
             (/ 1 4) ["16"]     ;; quarter beat = sixteenth
             (/ 3 4) ["8."]     ;; 3/4 beat = dotted eighth
             (/ 5 4) ["4" "16"] ;; 1.25 beat = quarter tied to sixteenth
             ;; ... comprehensive ratio coverage
             }]
        (get my-table new-ratio))))
```

**Innovation**: Pre-computed lookup table handles complex rhythmic relationships that would be difficult to calculate algorithmically.

### 2. Pitch Mapping Dictionaries (Lines 56-121)

**Western Notation Support**:
```clojure
(def normalized-pitch->lilypond-pitch
  {"-" "r", "C" "c", "C#" "cs", "Cb" "cf", "Db" "df", "D" "d", "D#" "ds",
   "Eb" "ef", "E" "e", "E#" "es", "F" "f", "Fb" "ff", "F#" "fs", "Gb" "gf",
   "G" "g", "G#" "gs", "Ab" "af", "A" "a", "A#" "as", "Bb" "bf", "B" "b",
   "B#" "bs"})
```

**Sargam Notation Support**:
```clojure  
(def pitch->lilypond-pitch
  {"-" "r", "S" "c", "S#" "cs", "Sb" "cf", "r" "df", "R" "d", "R#" "ds",
   "g" "ef", "G" "e", "G#" "es", "m" "f", "mb" "ff", "M" "fs", "Pb" "gf",
   "P" "g", "P#" "gs", "d" "af", "D" "a", "D#" "as", "n" "bf", "N" "b",
   "N#" "bs"})
```

**Innovation**: Dual notation system support with clean dictionary mapping approach.

### 3. Smart Octave Calculation (Lines 155-164)

```clojure
(defn octave-number->lilypond-octave[num]
  (let [tick "'"
        comma ","]
    ;; Middle c is c'
    (cond (nil? num) tick
          (>= num 0) (apply str (take (inc num) (repeat tick)))
          true (apply str (take (dec (- num)) (repeat comma))))))
```

**Innovation**: Handles both positive (`'`) and negative (`,`) octave markers with clean mathematical logic.

### 4. Mode-Aware Key Signatures (Lines 77-103)

```clojure
(def valid-lilypond-mode?
  #{:ionian :dorian :phrygian :lydian :mixolydian :aeolian :locrian
    :minor :major})

(defn key-signature-snippet[attributes]
  (let [my-mode (-> (get attributes :mode "major")
                    lower-case keyword)
        my-mode2 (if (not (valid-lilypond-mode? my-mode))
                   :major my-mode)]
    (str "\\key "
         (if (is-abc-composition attributes)
           (str (normalized-pitch->lilypond-pitch (:key attributes "C"))
                " \\" (name my-mode2))
           (str "c \\" (name my-mode2))))))
```

**Innovation**: Full church mode support with fallback to major, handles both ABC and native compositions.

### 5. Comprehensive Barline Handling (Lines 138-153)

```clojure
(defn barline->lilypond-barline[[_ [barline-type]]]
  (let [my-map
        {:reverse-final-barline "\\bar \".|\""
         :final-barline "\\bar \"|.\" "
         :double-barline "\\bar \"||\" " 
         :single-barline "\\bar \"|\"" 
         :left-repeat "\\bar \".|:\"" 
         :right-repeat "\\bar \":|.\""}]
    (str (get my-map barline-type (:single-barline my-map)) " ")))
```

**Innovation**: Complete barline vocabulary with proper LilyPond syntax and fallback behavior.

### 6. Intelligent Tuplet Denominators (Lines 269-288)

```clojure
(defn tuplet-numerator-for-odd-subdivisions[subdivisions-in-beat]
  ;; For \times ???/5 {d'16 e'8 d'16 e'16}
  ;; The ??? should be such that 5/16 * ???/5 = 1/4
  ;; So ??? = 4
  (cond (= 3 subdivisions-in-beat) 2
        (< subdivisions-in-beat 8) 4
        (< subdivisions-in-beat 16) 8
        (< subdivisions-in-beat 32) 16
        (< subdivisions-in-beat 64) 32
        true 32))
```

**Innovation**: Mathematical calculation of tuplet denominators ensures proper rhythmic relationships in complex time divisions.

## Re-Engineering Recommendations

### What to Adopt from Doremi-Script:
1. **MIDI export integration** - `#(ly:set-option 'midi-extension "mid")`
2. **Transposition architecture** - Separate `{{{transpose-snippet}}}` variable
3. **Mode-aware key signatures** - Support beyond major/minor  
4. **Source code preservation** - Original input in comments for debugging
5. **Comprehensive barline support** - Full vocabulary of barline types

### What to Adopt from Old.Music-Text:
1. **`\fixed c'` over `\relative c'`** - Predictable octave behavior
2. **Web-optimized paper settings** - Compact dimensions for UI display
3. **Professional typography** - Custom spacing, clean headers
4. **Conditional template sections** - `{{#variable}}` for optional features
5. **Type-safe template context** - Rust struct-based approach

### What to Avoid (Over-Engineering):
1. **Complex template selection logic** - One good template is better than many mediocre ones
2. **Premature abstraction** - Template engine interfaces before you need multiple engines  
3. **String manipulation complexity** - Let templates handle formatting, not manual string building

### Future Extensibility Considerations:
1. **Format-aware templating**: Web vs Print vs MIDI optimized templates
2. **Progressive variable complexity**: Start simple, add variables as features are implemented
3. **Template composition**: Combine sections (paper + score + midi) rather than monolithic templates

## Implementation Strategy for New System

### Phase 1: Minimal Template Integration
- Use `old.music-text/standard.ly.mustache` as base template
- Support essential variables: `version`, `staves`, `source_comment`
- Replace current hardcoded string template in `lilypond_converter_v2.rs`

### Phase 2: Enhanced Musical Features  
- Add `time_signature`, `key_signature` variables
- Implement doremi-script style mode support
- Add conditional lyrics section

### Phase 3: Advanced Features
- MIDI export support following doremi-script approach  
- Transposition infrastructure with `transpose_snippet`
- Multiple template formats (web, print, MIDI)

### Phase 4: Template Ecosystem
- Template selection based on output format
- User-customizable templates  
- Plugin architecture for custom formats

---

*This document serves as the definitive reference for implementing the LilyPond template system in the new music-text architecture. It captures the lessons learned from both previous systems and provides a roadmap for building a better, more maintainable template engine.*