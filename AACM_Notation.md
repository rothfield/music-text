# Manuscript Notation Grammar

## Purpose

This specification describes the formal grammar of musical notation as found in AACM Sargam manuscripts 

## Notation System Rules

**Single Notation System Per Manuscript**: Each manuscript typically uses one primary notation system throughout.

Notation systems found in manuscripts:
- **Sargam**: S, R, G, M, P, D, N (with variations)
- **Number**: 1, 2, 3, 4, 5, 6, 7 (with modifications)
- **Western**: A, B, C, D, E, F, G (when adapted)
- **Devanagari**: स, र, ग, म, प, ध, न
- **Tabla**: dha, dhin, ta, ka, etc.

## Manuscript Structure

Manuscripts follow consistent spatial organization principles developed through practical use in real-time musical transcription.

## Grammar Rules (EBNF)

### Manuscript Structure

```ebnf
manuscript = header? musical_content+

header = title_section? metadata_section?

title_section = title author?

metadata_section = directive+

directive = key ":" value

musical_content = stave+

stave = content_line annotation_line*

content_line = musical_phrase+

annotation_line = upper_annotation | lower_annotation


key = identifier

value = text_content

text_content = (!newline ANY)*

identifier = letter (letter | digit | "_")*

document_body = stave (blank_lines stave)*

stave = upper_line* content_line (lower_line | lyrics_line)* (blank_lines | (whitespace* newline)* EOI)

blank_lines = newline (whitespace* newline)+
newline = "\n"
whitespace = " "
letter = "A".."Z" | "a".."z"
digit = "0".."9"
```

### Musical Content

```ebnf
musical_phrase = (note | rhythm_extension | breath_mark | barline | space)+

note = sargam_note | number_note | western_note | tabla_note | devanagari_note

rhythm_extension = extension_marking

breath_mark = pause_indication

barline = single_bar | double_bar | repeat_start | repeat_end

space = gap
```



**Examples**:
- `"1 2 3 xxx"` → Three beats ("1", "2", "3") + one unknown token ("xxx")
- `"S R invalid G"` → Three beats ("S", "R", "G") + one unknown token ("invalid")
- `"|1 2 typo 3|"` → Barline + two beats ("1", "2") + unknown token ("typo") + beat ("3") + barline

Pitches are treated as atomic units (e.g., "1", "1#", "1b", "S", "S#") rather than decomposed into base pitch + accidental components.

### Spatial Annotations

```ebnf
upper_annotation = (octave_marker | upper_loop | ornament | chord | mordent | tala)+

lower_annotation = (octave_marker | lower_loop | syllable)+

octave_marker = single_octave | double_octave

upper_loop = musical_phrasing

lower_loop = rhythmic_grouping

ornament = grace_notes | melisma | trill

syllable = lyric_syllable | tabla_bol
```

### Octave Marker Semantics
```ebnf
// Unified octave marker - interpretation determined by spatial assignment rule
octave_marker = one dot, 2 dots, 3 dots

// Spatial assignment rule determines meaning:
// assign_octave_marker(source: octave_marker, destination: note, distance: i8)
//   - distance < 0: upper line (octave increase)
//     - "." → +1 octave
//     - ":" → +2 octave
//   - distance > 0: lower line (octave decrease)
//     - "." → -1 octave
//     - ":" → -2 octave
//
```

### Notation Systems

```ebnf
// Note: Melodic pitch systems support accidentals (#, ##, b, bb) appended to base notes when it makes sense

sargam_note = "S" | "R" | "G" | "M" | "P" | "D" | "N" |
              "s" | "r" | "g" | "m" | "p" | "d" | "n"

number_note = "1" | "2" | "3" | "4" | "5" | "6" | "7"

western_note = "A" | "B" | "C" | "D" | "E" | "F" | "G" |
               "a" | "b" | "c" | "d" | "e" | "f" | "g"

tabla_note = "dha" | "dhin" | "ta" | "ka" | "taka" | "trkt" | "ge" |
             "Dha" | "Dhin" | "Ta" | "Ka" | "Taka" | "Trkt" | "Ge" |
             "DHA" | "DHIN" | "TA" | "KA" | "TAKA" | "TRKT" | "GE"

hindi_note = "स" | "र" | "ग" | "म" | "प" | "ध" | "न"
```

**Examples**: `1#`, `2b`, `S#`, `Cb`, `4##`, `Rbb`



### Barlines and Structure

```ebnf
barline = single_barline | double_barline | final_barline |
          repeat_start | repeat_end | repeat_both


line_number =  IE 1)  to number variations
```


## Spatial Production Rules

### Beat Grouping
```ebnf
spatially-delimited-beat ::= lower loop

### Slur Grouping
```ebnf
spatially-delimited-slur ::= slur

### Octave Assignment
```ebnf
spatially-delimited-octave ::=
    [ octave_marker ]
    [[ content_pitch ]]
    [ octave_marker ]
```

### Other Spatial Relationships

The grammar has not yet been fully updated to formalize all spatial relationships, but the following spatial aspects exist in the current implementation and should be formalized using similar production rules:

- **Upper Octave Markers**: Dots and colons above notes to indicate higher octaves
  ```
  [ .  :     ]  (upper octave markers: +1, +2 octaves)
  [ S  R  G  ]  →  spatially-marked-octaves
  ```

- **Highest Octave Marker**: Colon above notes for maximum octave increase
  ```
  [    :     ]  (highest octave marker: +2 octaves)
  [ S  R  G  ]  →  spatially-marked-octaves
  ```

- **Lower Octave Marker**: Dot below notes to indicate lower octave
  ```
  [ S  R  G  ]  (notes)
  [ .        ]  (lower octave marker: -1 octave)
  ```

- **Lowest Octave Marker**: Colon below notes for maximum octave decrease
  ```
  [ S  R  G  ]  (notes)
  [    :     ]  (lowest octave marker: -2 octaves)
  ```

- **Ornaments**: Mordents, trills, and other decorations above notes
  ```
  [ ~  ~     ]  (ornament markers)
  [ S  R  G  ]  →  spatially-ornamented-notes
  ```

- **Syllables**: Lyrics or tabla bols aligned below notes
  ```
  [ S  R  G  ]  (notes)
  [ ta re ga ]  →  spatially-syllabled-notes
  ```

## Document Metadata

The document header contains optional metadata that appears before the musical content. This section handles titles, composer information, and musical directives.

### Header Structure


```ebnf

// Title followed by directives
Amazing Grace
  raga kafi
  chalan 


## Spatial Relationship Rules



### Lower Line Elements
- **Lower octave markers**: 
- **Beat group indicators**: lower loop
- **Syllables**: Text elements for spatial alignment (lyrics, tabla bols)
- **Spaces**: For alignment with content above

### Syllable Assignment
- **In lower_line**: Syllables can appear for spatial alignment below notes
- **In lyrics_line**: Traditional lyric lines with syllable-to-note assignment
- **Format**: `he-llo world sing-ing` with hyphens and apostrophes supported


