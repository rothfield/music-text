import React, { useState, useEffect, useRef } from 'react';
import '../css/musical-notation.css';

/**
 * MusicTextRenderer - React component for rendering music notation in doremi-script style
 * 
 * This component takes parsed music notation data and renders it as HTML/CSS
 * following the doremi-script visual style (text-based music notation).
 */
const MusicTextRenderer = ({ 
  notationData, 
  notationSystem = 'number',
  showLyrics = true,
  interactive = true 
}) => {
  const [highlightedBeat, setHighlightedBeat] = useState(null);
  const [selectedNotes, setSelectedNotes] = useState([]);
  const containerRef = useRef(null);

  // Parse the notation data if it's a string (HTML from converter)
  const renderNotation = () => {
    if (!notationData) {
      return <div className="placeholder">No notation to display</div>;
    }

    // If notationData is already HTML string from the converter
    if (typeof notationData === 'string') {
      return (
        <div 
          className={`musical-notation ${notationSystem}-notation`}
          dangerouslySetInnerHTML={{ __html: notationData }}
        />
      );
    }

    // If notationData is structured data, render it as React components
    if (typeof notationData === 'object') {
      return renderStructuredNotation(notationData);
    }

    return <div className="error">Invalid notation data format</div>;
  };

  // Render structured notation data as React components
  const renderStructuredNotation = (data) => {
    const { staves, metadata } = data;

    return (
      <div className={`musical-notation ${notationSystem}-notation`}>
        {metadata && renderMetadata(metadata)}
        {staves && staves.map((stave, index) => (
          <div key={index} className="stave">
            {renderStaveContent(stave)}
            {showLyrics && stave.lyrics && renderLyrics(stave.lyrics)}
          </div>
        ))}
      </div>
    );
  };

  const renderMetadata = (metadata) => {
    return (
      <div className="metadata">
        {metadata.title && <h2 className="title">{metadata.title}</h2>}
        {metadata.key && <div className="key-signature">Key: {metadata.key}</div>}
        {metadata.time && <div className="time-signature">{metadata.time}</div>}
      </div>
    );
  };

  const renderStaveContent = (stave) => {
    return (
      <div className="stave-content">
        {stave.beats && stave.beats.map((beat, beatIndex) => (
          <span
            key={beatIndex}
            className={`beat ${highlightedBeat === beatIndex ? 'highlighted' : ''}`}
            onMouseEnter={() => interactive && setHighlightedBeat(beatIndex)}
            onMouseLeave={() => interactive && setHighlightedBeat(null)}
            onClick={() => interactive && handleBeatClick(beatIndex)}
          >
            {renderBeat(beat, beatIndex)}
          </span>
        ))}
      </div>
    );
  };

  const renderBeat = (beat, beatIndex) => {
    if (!beat.notes) return null;

    return beat.notes.map((note, noteIndex) => {
      const noteKey = `${beatIndex}-${noteIndex}`;
      const isSelected = selectedNotes.includes(noteKey);

      return (
        <span
          key={noteIndex}
          className={`note-wrapper ${isSelected ? 'selected' : ''}`}
          onClick={(e) => {
            e.stopPropagation();
            interactive && handleNoteClick(noteKey);
          }}
        >
          {renderNote(note)}
        </span>
      );
    });
  };

  const renderNote = (note) => {
    const elements = [];

    // Add accidental if present
    if (note.accidental) {
      elements.push(
        <span key="accidental" className={`accidental ${note.accidental}`}>
          {note.accidental === 'sharp' ? '♯' : note.accidental === 'flat' ? '♭' : ''}
        </span>
      );
    }

    // Add the note itself
    elements.push(
      <span key="note" className={`note ${note.duration ? `duration-${note.duration}` : ''}`}>
        {note.value}
      </span>
    );

    // Add octave indicators
    if (note.octave > 0) {
      elements.push(
        <span key="octave" className={`octave-upper-${Math.min(note.octave, 2)}`}>
          {'·'.repeat(note.octave)}
        </span>
      );
    } else if (note.octave < 0) {
      elements.push(
        <span key="octave" className={`octave-lower-${Math.min(Math.abs(note.octave), 2)}`}>
          {'·'.repeat(Math.abs(note.octave))}
        </span>
      );
    }

    // Add ornaments
    if (note.ornaments) {
      note.ornaments.forEach((ornament, index) => {
        elements.push(
          <span key={`ornament-${index}`} className={`ornament ${ornament.type}`}>
            {ornament.symbol || ''}
          </span>
        );
      });
    }

    // Add slur indicators
    if (note.slur) {
      elements.push(
        <span key="slur" className={`slur slur-${note.slur}`} />
      );
    }

    // Add tie indicators
    if (note.tied) {
      elements.push(
        <span key="tie" className="tie" />
      );
    }

    return elements;
  };

  const renderLyrics = (lyrics) => {
    return (
      <div className="lyrics-line">
        {lyrics.map((lyric, index) => (
          <span key={index} className="lyric">
            {lyric.text || '\u00A0'}
          </span>
        ))}
      </div>
    );
  };

  const handleBeatClick = (beatIndex) => {
    console.log('Beat clicked:', beatIndex);
    // Could emit an event or callback here
  };

  const handleNoteClick = (noteKey) => {
    setSelectedNotes(prev => {
      if (prev.includes(noteKey)) {
        return prev.filter(k => k !== noteKey);
      } else {
        return [...prev, noteKey];
      }
    });
  };

  // Auto-scroll to keep notation visible
  useEffect(() => {
    if (containerRef.current && notationData) {
      const container = containerRef.current;
      const scrollWidth = container.scrollWidth;
      const clientWidth = container.clientWidth;
      
      if (scrollWidth > clientWidth) {
        // Scroll to show the beginning by default
        container.scrollLeft = 0;
      }
    }
  }, [notationData]);

  return (
    <div ref={containerRef} className="music-text-renderer">
      {renderNotation()}
    </div>
  );
};

export default MusicTextRenderer;