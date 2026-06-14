use ggez::graphics::{TextAlign, TextLayout};
pub trait TextLayoutExtension {
    fn top_right() -> Self;
}

impl TextLayoutExtension for TextLayout {
    fn top_right() -> Self {
        TextLayout {
            h_align: TextAlign::End,
            v_align: TextAlign::Begin,
        }
    }
}
