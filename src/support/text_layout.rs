use ggez::graphics::{TextAlign, TextLayout};

// TODO: contribute to ggez
pub trait TextLayoutExtension {
    // top_left
    fn top_middle() -> Self;
    fn top_right() -> Self;

    fn center_left() -> Self;
    // center
    fn center_right() -> Self;

    fn bottom_left() -> Self;
    fn bottom_middle() -> Self;
    fn bottom_right() -> Self;
}

impl TextLayoutExtension for TextLayout {
    fn top_middle() -> Self {
        TextLayout {
            h_align: TextAlign::Middle,
            v_align: TextAlign::Begin,
        }
    }

    fn top_right() -> Self {
        TextLayout {
            h_align: TextAlign::End,
            v_align: TextAlign::Begin,
        }
    }

    fn center_left() -> Self {
        TextLayout {
            h_align: TextAlign::Begin,
            v_align: TextAlign::Middle,
        }
    }

    fn center_right() -> Self {
        TextLayout {
            h_align: TextAlign::End,
            v_align: TextAlign::Middle,
        }
    }

    fn bottom_left() -> Self {
        TextLayout {
            h_align: TextAlign::Begin,
            v_align: TextAlign::End,
        }
    }

    fn bottom_middle() -> Self {
        TextLayout {
            h_align: TextAlign::Middle,
            v_align: TextAlign::End,
        }
    }

    fn bottom_right() -> Self {
        TextLayout {
            h_align: TextAlign::End,
            v_align: TextAlign::End,
        }
    }
}
