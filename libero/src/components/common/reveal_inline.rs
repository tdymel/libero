use dioxus::prelude::*;

use crate::{hooks::ElementHandle, platform::ElementApi};

/// Scrolls `strip` sideways just enough to show its descendant `selector`, as
/// `scrollIntoView` `nearest` would, without scrolling the page. Keeps the
/// strip's `scroll-padding-inline` free, as a focus scroll does.
pub(crate) fn reveal_inline(strip: ElementHandle, selector: &str) {
    let Ok(target) = strip.query_selector(selector) else {
        return;
    };
    let reads = (
        strip.client_offset(),
        strip.dimensions(),
        strip.scroll_offset(),
        target.client_offset(),
        target.dimensions(),
        // Physical sides, which the browser resolves from the logical ones.
        strip.computed_px("scroll-padding-left"),
        strip.computed_px("scroll-padding-right"),
    );
    spawn(async move {
        let (strip_at, strip_size, scroll, target_at, target_size, left, right) = reads;
        let (
            Ok((strip_x, _)),
            Ok(strip_size),
            Ok((scroll_x, scroll_y)),
            Ok((target_x, _)),
            Ok(target_size),
        ) = (
            strip_at.await,
            strip_size.await,
            scroll.await,
            target_at.await,
            target_size.await,
        )
        else {
            return;
        };
        // `auto`, or a backend without computed styles: no room.
        let room = |side: Result<Option<f64>, _>| side.ok().flatten().unwrap_or(0.0);
        let (left, right) = (room(left.await), room(right.await));
        let before = target_x - left - strip_x;
        let after = target_x + target_size.width + right - (strip_x + strip_size.width);
        let shift = if before < 0.0 { before } else { after.max(0.0) };
        if shift != 0.0 {
            let _ = strip.scroll_to(scroll_x + shift, scroll_y);
        }
    });
}
