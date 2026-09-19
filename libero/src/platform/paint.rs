use super::backend;

/// Whether the renderer blurs behind a `backdrop-filter`. Every anyrender
/// backend Blitz ships drops it, so a `glass` surface stays opaque there.
pub(crate) fn draws_backdrop_filter() -> bool {
    backend::DRAWS_BACKDROP_FILTER
}

/// Whether an SVG `<img>` is drawn as its `object-fit` says. Blitz draws every
/// one `contain`, its letterbox offset outside the element's transform (todo 920).
pub(crate) fn fits_svg_images() -> bool {
    backend::FITS_SVG_IMAGES
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
