use crate::{
    platform::clips_background_to_text,
    sx::{Sx, sx},
    theme::{GRADIENT_FROM, gradient_image},
};

/// The gradient through the glyphs. Blitz paints the whole box, so there it is the first stop.
pub(crate) fn gradient_text_sx() -> Sx {
    if !clips_background_to_text() {
        return sx().color(GRADIENT_FROM.value());
    }
    sx().color("transparent")
        .background_image(gradient_image(None))
        .background_clip("text")
        .with("-webkit-background-clip", "text")
        // Print drops backgrounds by default, so the text would print blank; `exact` left a
        // hairline round the clip in Chromium's PDF, hence solid in the first stop.
        .media(
            "print",
            sx().color(GRADIENT_FROM.value()).background_image("none"),
        )
}
