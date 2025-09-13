## TODO

- [ ] Unify fraction conversion functions between VexFlow and LilyPond renderers
  - Currently have duplicate logic in `convert_fraction_to_vexflow()` and `fraction_to_lilypond_duration()`
  - Same fraction mappings (3/32 ’ dotted sixteenth, 7/32 ’ double-dotted eighth, etc.)
  - Different output formats: VexFlow returns `(String, u8)`, LilyPond returns `String`
  - Need shared function with different formatters