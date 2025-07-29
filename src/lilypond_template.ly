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
}
\score {
  <<
    \new Staff {
      \fixed c' {
        \clef treble
        \set Score.barCheckSynchronize = ##f
        \set Timing.defaultBarType = #""
        {{notes}}
      }
    }
    \addlyrics {
      {{lyrics}}
    }
  >>
}

