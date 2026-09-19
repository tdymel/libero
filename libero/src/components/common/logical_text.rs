use crate::{
    platform,
    sx::{Sx, sx},
};

/// `text-align: start`/`end` that hold under `rtl` natively too, where Blitz
/// reads `start` as left and `end` as right.
pub(crate) trait LogicalTextAlign {
    fn text_align_start(self) -> Self;
    fn text_align_end(self) -> Self;
}

impl LogicalTextAlign for Sx {
    fn text_align_start(self) -> Self {
        logical(self, "start", "right")
    }

    fn text_align_end(self) -> Self {
        logical(self, "end", "left")
    }
}

fn logical(style: Sx, value: &str, rtl_side: &str) -> Sx {
    let base = style.text_align(value);
    match platform::aligns_logical_text() {
        true => base,
        false => base.rtl(sx().text_align(rtl_side)),
    }
}
