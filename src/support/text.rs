//! Text in the game's one font, DejaVu Sans (bundled: macroquad's built-in
//! font is ASCII-only, and key labels need symbols like ⌘ and ⇧).
//!
//! macroquad rasterizes every glyph once per font size into one atlas
//! texture that only ever grows. Text that scales with the window would add
//! a new size each frame of a resize, until the atlas outgrows what the GPU
//! can load and all text disappears. So glyphs are only ever rasterized at a
//! few fixed sizes, and scaled from the nearest one above.

use macroquad::color::Color;
use macroquad::text::{self, load_ttf_font_from_bytes, Font, TextDimensions, TextParams};

thread_local! {
    static FONT: Font = load_ttf_font_from_bytes(include_bytes!("../../assets/fonts/DejaVuSans.ttf"))
        .expect("the bundled font loads");
}

/// The sizes glyphs are rasterized at
const RASTER_SIZES: [u16; 8] = [12, 16, 24, 32, 48, 64, 96, 128];

fn font() -> Font {
    FONT.with(Font::clone)
}

/// The size to rasterize at for text of `font_size`, and the scale from it
fn raster(font_size: f32) -> (u16, f32) {
    let size = RASTER_SIZES
        .into_iter()
        .find(|&size| size as f32 >= font_size)
        .unwrap_or(RASTER_SIZES[RASTER_SIZES.len() - 1]);
    (size, font_size / size as f32)
}

/// Draw `text` with its baseline at `y`.
pub fn draw_text(text: &str, x: f32, y: f32, font_size: f32, color: Color) {
    let font = font();
    let (size, scale) = raster(font_size);
    text::draw_text_ex(text, x, y, TextParams {
        font: Some(&font),
        font_size: size,
        font_scale: scale,
        color,
        ..Default::default()
    });
}

pub fn measure_text(text: &str, font_size: f32) -> TextDimensions {
    let (size, scale) = raster(font_size);
    text::measure_text(text, Some(&font()), size, scale)
}
