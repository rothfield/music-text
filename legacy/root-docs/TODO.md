## TODO

- [ ] Unify fraction conversion functions between VexFlow and LilyPond renderers
  - Currently have duplicate logic in `convert_fraction_to_vexflow()` and `fraction_to_lilypond_duration()`
  - Same fraction mappings (3/32 � dotted sixteenth, 7/32 � double-dotted eighth, etc.)
  - Different output formats: VexFlow returns `(String, u8)`, LilyPond returns `String`
  - Need shared function with different formatters

- [ ] Resurrect original pest grammar from git history for parser reference  
  - The handwritten recursive descent parser was originally based on a pest grammar
  - Need authoritative grammar reference for EOI rules (e.g., upper_lines can't end with EOI)
  - Cross-reference current parser against original grammar specifications
  - Document grammar constraints and ensure compliance in handwritten parser

- [ ] Review rhythm analyzer line 490: why does position() method include Newline/EndOfInput tokens?
  - These are grammar structure tokens, not musical content tokens
  - Should rhythm analyzer filter out grammar tokens before processing?
  - Or should position() method handle all ParsedElement variants for utility purposes?
  - Consider separating grammar tokens from musical content tokens in the type system