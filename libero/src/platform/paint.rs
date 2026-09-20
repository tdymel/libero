use super::backend;

/// Whether the renderer blurs behind a `backdrop-filter`. Every anyrender
/// backend Blitz ships drops it, so a `glass` surface stays opaque there.
pub(crate) fn draws_backdrop_filter() -> bool {
    backend::DRAWS_BACKDROP_FILTER
}

/// Whether `background-clip: text` clips to the glyphs. Blitz fills the whole
/// box, so gradient text there is a solid colour instead (todo 937).
pub(crate) fn clips_background_to_text() -> bool {
    backend::CLIPS_BACKGROUND_TO_TEXT
}

/// Whether an SVG `<img>` is drawn as its `object-fit` says. Blitz draws every
/// one `contain`, its letterbox offset outside the element's transform (todo 920).
pub(crate) fn fits_svg_images() -> bool {
    backend::FITS_SVG_IMAGES
}

/// Whether an `<img>` that fails to fetch or decode fires `error`. Blitz's
/// fetch reports only bytes, and an undecodable picture stays silent (todo 884).
pub(crate) fn fires_image_errors() -> bool {
    backend::FIRES_IMAGE_ERRORS
}

/// Whether an inline element's background shows behind the text of a span
/// inside it. Blitz fills each text run from its innermost element only.
pub(crate) fn paints_outer_inline_backgrounds() -> bool {
    backend::PAINTS_OUTER_INLINE_BACKGROUNDS
}

/// Whether `text-align: start`/`end` follow `direction`. Blitz aligns `start`
/// left and `end` right under `dir=rtl` too; a physical value works.
pub(crate) fn aligns_logical_text() -> bool {
    backend::ALIGNS_LOGICAL_TEXT
}
