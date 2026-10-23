use dioxus::prelude::*;

use crate::{hooks::ElementHandle, platform::ElementApi};

/// Scrolls `strip` sideways just enough to show its descendant `selector`, as
/// `scrollIntoView` `nearest` would, without scrolling the page. Keeps the
/// strip's `scroll-padding-inline` free, as a focus scroll does.
pub(crate) fn reveal_inline(strip: ElementHandle, selector: &str) {
    reveal(strip, selector, false);
}

/// [`reveal_inline`] down the block axis, for a vertical strip (3976).
pub(crate) fn reveal_block(strip: ElementHandle, selector: &str) {
    reveal(strip, selector, true);
}

fn reveal(strip: ElementHandle, selector: &str, block: bool) {
    let Ok(target) = strip.query_selector(selector) else {
        return;
    };
    // Physical sides, which the browser resolves from the logical ones.
    let (start, end) = match block {
        true => ("scroll-padding-top", "scroll-padding-bottom"),
        false => ("scroll-padding-left", "scroll-padding-right"),
    };
    let reads = (
        strip.client_offset(),
        strip.dimensions(),
        strip.scroll_offset(),
        target.client_offset(),
        target.dimensions(),
        strip.computed_px(start),
        strip.computed_px(end),
    );
    spawn(async move {
        let (strip_at, strip_size, scroll, target_at, target_size, start, end) = reads;
        let (
            Ok((strip_x, strip_y)),
            Ok(strip_size),
            Ok((scroll_x, scroll_y)),
            Ok((target_x, target_y)),
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
        let (start, end) = (room(start.await), room(end.await));
        let shift = nearest_shift(
            match block {
                true => (strip_y, strip_size.height, target_y, target_size.height),
                false => (strip_x, strip_size.width, target_x, target_size.width),
            },
            start,
            end,
        );
        if shift != 0.0 {
            let _ = match block {
                true => strip.scroll_to(scroll_x, scroll_y + shift),
                false => strip.scroll_to(scroll_x + shift, scroll_y),
            };
        }
    });
}

/// How far to scroll one axis so the target, at `(strip_at, strip_len, target_at, target_len)`,
/// shows inside the strip's padded edges; negative scrolls back.
fn nearest_shift(
    (strip_at, strip_len, target_at, target_len): (f64, f64, f64, f64),
    start: f64,
    end: f64,
) -> f64 {
    let before = target_at - start - strip_at;
    let after = target_at + target_len + end - (strip_at + strip_len);
    if before < 0.0 { before } else { after.max(0.0) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shift_shows_the_target_at_the_nearest_edge() {
        assert_eq!(nearest_shift((0.0, 100.0, 120.0, 30.0), 0.0, 10.0), 60.0);
        assert_eq!(nearest_shift((0.0, 100.0, -20.0, 30.0), 5.0, 0.0), -25.0);
        assert_eq!(nearest_shift((0.0, 100.0, 40.0, 30.0), 0.0, 0.0), 0.0);
    }
}
