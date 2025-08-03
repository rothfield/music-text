whole parser
  lexer
     splits input into lines, does initial tokenization (coarse)
  parser
    attach
       -- attach octaves,lyrics, etc from above and below musical line. consumes tokens. but some may remain.
    musical_grouping
       -- creates needed structures like beats, ornaments, does counting of divisions, additional parsing of input stream


The output is our document model, a parse tree

document
   metadata
   nodes
      line
      muiscal-line
