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
