use skia_safe::{Surface, EncodedImageFormat, Paint, Color, FontMgr};
use crate::models::{Document, DocumentElement, Stave, StaveLine, ContentElement, BeatElement};

pub struct SkiaCanvasRenderer {
    surface: Surface,
    width: i32,
    height: i32,
}

impl SkiaCanvasRenderer {
    pub fn new(width: i32, height: i32) -> Option<Self> {
        let surface = Surface::new_raster_n32_premul((width, height))?;

        Some(Self {
            surface,
            width,
            height,
        })
    }

    pub fn render_document(&mut self, document: &Document) -> Option<Vec<u8>> {
        let canvas = self.surface.canvas();

        // Clear with light gray background
        canvas.clear(Color::from_rgb(249, 249, 249));

        let mut paint = Paint::default();
        paint.set_color(Color::BLACK);
        paint.set_anti_alias(true);

        // Get font for rendering
        let font_mgr = FontMgr::default();
        let typeface = font_mgr.legacy_make_typeface(None, skia_safe::FontStyle::default())?;
        let font = skia_safe::Font::new(typeface, 24.0);

        let mut y_pos = 50.0;

        // Render document elements
        for doc_element in &document.elements {
            match doc_element {
                DocumentElement::Stave(stave) => {
                    self.render_stave(stave, &mut y_pos, &font, &paint);
                },
                DocumentElement::BlankLines(_) => {
                    y_pos += 30.0;
                }
            }
        }

        // Convert to RGBA data for egui texture
        self.to_rgba_data()
    }

    fn render_stave(&mut self, stave: &Stave, y_pos: &mut f64, font: &skia_safe::Font, paint: &Paint) {
        let canvas = self.surface.canvas();

        for line in &stave.lines {
            match line {
                StaveLine::ContentLine(content_line) => {
                    self.render_content_line(content_line, *y_pos, font, paint);
                    *y_pos += 40.0;
                },
                _ => {
                    // Handle other line types or skip them
                    *y_pos += 20.0;
                }
            }
        }
    }

    fn render_content_line(&mut self, content_line: &crate::models::ContentLine, y_pos: f64, font: &skia_safe::Font, paint: &Paint) {
        let canvas = self.surface.canvas();
        let mut x_pos = 50.0;

        for element in &content_line.elements {
            match element {
                ContentElement::Beat(beat) => {
                    for beat_element in &beat.elements {
                        match beat_element {
                            BeatElement::Note(note) => {
                                let note_text = note.value.as_deref().unwrap_or("?");
                                if let Some(text_blob) = TextBlob::new(&note_text, font) {
                                    canvas.draw_text_blob(&text_blob, (x_pos as f32, y_pos as f32), paint);
                                }
                                x_pos += 40.0;
                            },
                            BeatElement::Rest(_) => {
                                if let Some(text_blob) = TextBlob::new("♪", font) {
                                    canvas.draw_text_blob(&text_blob, (x_pos as f32, y_pos as f32), paint);
                                }
                                x_pos += 40.0;
                            },
                            BeatElement::Dash(_) => {
                                if let Some(text_blob) = TextBlob::new("-", font) {
                                    canvas.draw_text_blob(&text_blob, (x_pos as f32, y_pos as f32), paint);
                                }
                                x_pos += 25.0;
                            },
                            _ => {}
                        }
                    }
                },
                ContentElement::Barline(barline) => {
                    let barline_text = "|"; // Simple barline representation
                    if let Some(text_blob) = TextBlob::new(barline_text, font) {
                        canvas.draw_text_blob(&text_blob, (x_pos as f32, y_pos as f32), paint);
                    }
                    x_pos += 30.0;
                },
                _ => {}
            }
        }
    }

    pub fn to_rgba_data(&mut self) -> Option<Vec<u8>> {
        let image = self.surface.image_snapshot();
        let pixmap = image.peek_pixels()?;
        Some(pixmap.bytes()?.to_vec())
    }
}

use skia_safe::TextBlob;