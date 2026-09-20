//! Blitz draws every SVG `<img>` `contain`, whatever its `object-fit` (todo 920):
//! there the picture is drawn as the `<img>`'s background, which Blitz sizes right.

use crate::{
    components::common::{Variables, css_string},
    platform,
    sx::{Sx, sx},
    theme::CssVar,
};

/// The state an `<img>` takes when [`svg_fit`] says so.
pub(crate) const SVG_FIT: &str = "svg-fit";

const SVG_FIT_SRC: CssVar = CssVar::new("--lsx-svg-fit-src");
const SVG_FIT_SIZE: CssVar = CssVar::new("--lsx-svg-fit-size");

/// Whether `src` is an SVG this renderer would misfit, told by a `.svg` path or SVG data URL.
pub(crate) fn svg_fit(src: &str) -> bool {
    !platform::fits_svg_images() && is_svg(src)
}

fn is_svg(src: &str) -> bool {
    let src = src.trim();
    if let Some(data) = src.get(..5).filter(|s| s.eq_ignore_ascii_case("data:")) {
        let mime = &src[data.len()..];
        return mime
            .get(..13)
            .is_some_and(|mime| mime.eq_ignore_ascii_case("image/svg+xml"));
    }
    let path = src.split(['?', '#']).next().unwrap_or_default();
    path.len() >= 4
        && path
            .get(path.len() - 4..)
            .is_some_and(|end| end.eq_ignore_ascii_case(".svg"))
}

/// `variables` plus what [`svg_fit_sx`] reads: the picture, and `size` as its
/// `background-size` (`cover`, `contain`, `100% 100%`, `auto`).
pub(crate) fn svg_fit_variables(variables: Variables, src: &str, size: &str) -> Variables {
    if !svg_fit(src) {
        return variables;
    }
    variables
        .with(SVG_FIT_SRC, format!("url({})", css_string(src)))
        .with(SVG_FIT_SIZE, size.to_string())
}

/// Under [`SVG_FIT`]: the picture as background, its own drawing moved out of view.
/// `!important`: a caller's `background` is a colour behind the picture.
pub(crate) fn svg_fit_sx() -> Sx {
    sx().background_image(format!("{} !important", SVG_FIT_SRC.value()))
        .background_size(format!("{} !important", SVG_FIT_SIZE.value()))
        .background_position("center !important")
        .background_repeat("no-repeat !important")
        .overflow("hidden")
        .object_position("-100000px 0")
}

#[cfg(test)]
mod tests {
    use super::is_svg;

    #[test]
    fn an_svg_is_told_by_its_path_or_its_data_type() {
        for svg in [
            "/assets/1-dxh4f.svg",
            "https://example.com/a.SVG?v=2#top",
            "data:image/svg+xml,<svg/>",
            "DATA:image/SVG+xml;base64,PHN2Zy8+",
        ] {
            assert!(is_svg(svg), "{svg}");
        }
        for raster in [
            "/a.png",
            "/svg/a.png",
            "data:image/png;base64,AA",
            "/a.svgz",
            "",
        ] {
            assert!(!is_svg(raster), "{raster}");
        }
    }
}
