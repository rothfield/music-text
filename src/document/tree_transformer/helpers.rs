use crate::document::model::{Position, Source};

// Helper function to extract line and column from PEST span
pub(super) fn position_from_span(span: pest::Span) -> Position {
    let (line, column) = span.start_pos().line_col();
    Position { line, column }
}

// Helper function to create Source from PEST span
pub(super) fn source_from_span(span: pest::Span) -> Source {
    Source {
        value: span.as_str().to_string(),
        position: position_from_span(span),
    }
}