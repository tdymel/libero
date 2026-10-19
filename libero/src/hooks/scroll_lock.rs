use dioxus::prelude::*;

use crate::{
    hooks::ElementHandle,
    platform::{ElementApi, document, root_padding_right},
};

/// The scroll lock. A classic scrollbar's width is added to `:root`'s own `padding`, so the page
/// does not shift (todo 1305); not `scrollbar-gutter`, which no backdrop covers (todo 1316).
fn scroll_lock_css(gutter: f64, padding: f64) -> String {
    if gutter > 0.0 {
        let padding = padding + gutter;
        format!("html {{ padding-right: {padding}px !important; }} body {{ overflow: hidden; }}")
    } else {
        "body { overflow: hidden; }".to_string()
    }
}

/// The classic scrollbar's width: the viewport less the fixed, full-bleed root.
/// 0 for an overlay scrollbar, or a root not laid out yet.
fn scrollbar_gutter(viewport: Option<f64>, root: Option<f64>) -> f64 {
    match (viewport, root) {
        (Some(viewport), Some(root)) if root > 0.0 && (0.0..=64.0).contains(&(viewport - root)) => {
            viewport - root
        }
        _ => 0.0,
    }
}

/// A measure that resolves after the close is dropped, or the next open reuses its gutter.
fn settled(live: bool, measure: (f64, f64)) -> Option<(f64, f64)> {
    live.then_some(measure)
}

/// Locks the page's scroll while `active`, as `Modal` and a drawn fullscreen do. Render
/// the returned `style` inside `root`, a fixed, full-bleed box, measured before the lock
/// mounts: the scrollbar is gone after. Not `body:has(..)`: `:has()` never matches natively.
pub(crate) fn use_scroll_lock(root: ElementHandle, active: bool) -> Option<Element> {
    let mut lock = use_signal(|| None::<(f64, f64)>);
    let mut live = use_signal(|| false);
    use_effect(use_reactive!(|active| {
        live.set(active);
        if !active {
            if lock.peek().is_some() {
                lock.set(None);
            }
            return;
        }
        if !root.is_mounted() || lock.peek().is_some() {
            return;
        }
        let size = root.dimensions();
        let viewport = document().map(|document| document.viewport());
        let padding = root_padding_right().unwrap_or(0.0);
        spawn(async move {
            let width = size.await.ok().map(|size| size.width);
            let screen = match viewport {
                Some(read) => read.await.ok().map(|viewport| viewport.width),
                None => None,
            };
            if let Some(measure) = settled(live(), (scrollbar_gutter(screen, width), padding)) {
                lock.set(Some(measure));
            }
        });
    }));
    let (gutter, padding) = lock().filter(|_| active)?;
    Some(rsx! {
        style { dangerous_inner_html: scroll_lock_css(gutter, padding) }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_classic_scrollbar_keeps_its_gutter_under_the_lock() {
        assert_eq!(scrollbar_gutter(Some(1280.0), Some(1265.0)), 15.0);
        // On top of the page's own padding, which it must not replace.
        assert!(scroll_lock_css(15.0, 8.0).contains("html { padding-right: 23px !important; }"));
    }

    #[test]
    fn a_measure_after_the_close_is_not_kept() {
        assert_eq!(settled(true, (15.0, 8.0)), Some((15.0, 8.0)));
        assert_eq!(settled(false, (15.0, 8.0)), None);
    }

    #[test]
    fn no_gutter_without_a_measured_classic_scrollbar() {
        // Overlay scrollbars, an unlaid-out root, no document, a nonsense reading.
        for (viewport, root) in [
            (Some(1280.0), Some(1280.0)),
            (Some(1280.0), Some(0.0)),
            (None, Some(1265.0)),
            (Some(1280.0), None),
            (Some(1280.0), Some(900.0)),
        ] {
            assert_eq!(
                scrollbar_gutter(viewport, root),
                0.0,
                "{viewport:?} {root:?}"
            );
        }
        assert_eq!(scroll_lock_css(0.0, 8.0), "body { overflow: hidden; }");
    }
}
