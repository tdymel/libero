//! Blitz draws every SVG `<img>` `contain`, whatever its `object-fit` (todo 920):
//! there the picture is drawn as the `<img>`'s background, which Blitz sizes right.

use std::{cell::Cell, rc::Rc};

use dioxus::prelude::{MountedData, spawn, use_effect, use_hook, use_reactive};

use crate::{
    components::common::{Variables, css_string},
    hooks::{SubscriptionSlot, use_subscription_slot},
    platform::{self, ContentSubscription},
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

/// The slot [`on_failed_after_mount`] keeps an SVG probe in: emptied on a new `src`, whose
/// failure the live `onerror` reports, and on unmount.
pub(crate) fn use_svg_probe(src: &str) -> SubscriptionSlot<dyn ContentSubscription> {
    let probe = use_subscription_slot::<dyn ContentSubscription>();
    let seen = use_hook(|| Rc::new(Cell::new(false)));
    let stale = probe.clone();
    let src = src.to_string();
    use_effect(use_reactive!(|src| {
        let _ = &src;
        if seen.replace(true) {
            stale.clear();
        }
    }));
    probe
}

/// Whether [`on_failed_after_mount`] can find anything: off the web an image takes no
/// `onmounted`, as a WebView pays a blocking round trip for each (todo 2749).
pub(crate) const PROBES_AFTER_MOUNT: bool = cfg!(target_arch = "wasm32");

/// Calls `on_failed` if the `<img>` showing `src` has failed by the task after it mounted: a
/// server-rendered one failed before hydration attached its `onerror` (todo 2529). An SVG with
/// no size reads as empty, so a fresh image loads it instead, held in `probe` until the
/// component unmounts (todo 2671).
pub(crate) fn on_failed_after_mount(
    src: &str,
    mounted: Rc<MountedData>,
    probe: &SubscriptionSlot<dyn ContentSubscription>,
    on_failed: impl FnOnce() + 'static,
) {
    if is_svg(src) {
        let on_failed = Cell::new(Some(on_failed));
        probe.set(platform::on_image_error(
            src,
            Box::new(move || {
                if let Some(on_failed) = on_failed.take() {
                    on_failed();
                }
            }),
        ));
        return;
    }
    spawn(async move {
        platform::next_task().await;
        if platform::load_failed(&mounted) {
            on_failed();
        }
    });
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
