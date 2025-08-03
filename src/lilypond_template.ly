\version "2.24.0"
\language "english"

{{header}}

\paper {
  tagline = ##f
  print-page-number = ##f
  oddHeaderMarkup = ##f
  evenHeaderMarkup = ##f
  oddFooterMarkup = ##f
  evenFooterMarkup = ##f
  indent = 0\mm
  top-margin = 1\mm
  bottom-margin = 1\mm
  left-margin = 3\mm
  right-margin = 3\mm
  ragged-right = ##t
  system-system-spacing = #'((basic-distance . 2) (minimum-distance . 2) (padding . 0) (stretchability . 0))
  markup-system-spacing = #'((basic-distance . 0) (minimum-distance . 0) (padding . 0) (stretchability . 0))
  score-system-spacing = #'((basic-distance . 0) (minimum-distance . 0) (padding . 0) (stretchability . 0))
  top-system-spacing = #'((basic-distance . 2) (minimum-distance . 2) (padding . 0) (stretchability . 0))
  last-bottom-spacing = #'((basic-distance . 2) (minimum-distance . 2) (padding . 0) (stretchability . 0))
}
\score {
  \new Staff {
    \fixed c' {
      \clef treble
      \set Score.barCheckSynchronize = ##f
      \set Timing.defaultBarType = #""
      {{notes}}
    }
  }
  \layout {
    \context {
      \Staff
      \remove "Time_signature_engraver"
      \override StaffSymbol.staff-space = #0.7
    }
    \context {
      \Score
      \override SpacingSpanner.base-shortest-duration = #(ly:make-moment 1/32)
      \override SpacingSpanner.shortest-duration-space = #1.0
    }
  }
}

