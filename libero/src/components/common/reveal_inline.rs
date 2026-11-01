use dioxus::prelude::*;

#[cfg(feature = "table")]
use crate::platform::{element, mounted_by_selector};
use crate::{
    hooks::ElementHandle,
    platform::{ElementApi, PlatformError},
};

/// Scrolls `strip` sideways just enough to show its descendant `selector`, as
/// `scrollIntoView` `nearest` would, without scrolling the page. Keeps the
/// strip's `scroll-padding-inline` free, as a focus scroll does.
pub(crate) fn reveal_inline(strip: ElementHandle, selector: &str) {
    reveal(strip, [found(strip.query_selector(selector)), None]);
}

/// [`reveal_inline`] down the block axis, for a vertical strip (3976).
pub(crate) fn reveal_block(strip: ElementHandle, selector: &str) {
    reveal(strip, [None, found(strip.query_selector(selector))]);
}

/// Shows `inline`'s match sideways and `block`'s down `strip` in one scroll, so neither undoes
/// the other. Both are unique in the page: a WebView, whose handles cannot query, looks there (5232).
#[cfg(feature = "table")]
pub(crate) fn reveal_unique(strip: ElementHandle, inline: &str, block: Option<&str>) {
    let find = |selector: &str| match strip.query_selector(selector) {
        Err(PlatformError::Unsupported) => mounted_by_selector(selector).map(|page| Target {
            element: element(&page),
            page: true,
        }),
        queried => found(queried),
    };
    reveal(strip, [find(inline), block.and_then(find)]);
}

struct Target {
    element: Box<dyn ElementApi>,
    /// Looked up page-side, which measures no match 0x0.
    page: bool,
}

fn found(queried: Result<Box<dyn ElementApi>, PlatformError>) -> Option<Target> {
    queried.ok().map(|element| Target {
        element,
        page: false,
    })
}

/// `targets` holds the inline axis's target, then the block axis's.
fn reveal(strip: ElementHandle, targets: [Option<Target>; 2]) {
    if targets.iter().all(Option::is_none) {
        return;
    }
    let reads = (
        strip.client_offset(),
        strip.dimensions(),
        strip.scroll_offset(),
    );
    // Physical sides, which the browser resolves from the logical ones.
    let padding = [
        ("scroll-padding-left", "scroll-padding-right"),
        ("scroll-padding-top", "scroll-padding-bottom"),
    ]
    .map(|(start, end)| (strip.computed_px(start), strip.computed_px(end)));
    let targets = targets.map(|target| {
        target.map(|Target { element, page }| (element.client_offset(), element.dimensions(), page))
    });
    spawn(async move {
        let (Ok((strip_x, strip_y)), Ok(strip_size), Ok((scroll_x, scroll_y))) =
            (reads.0.await, reads.1.await, reads.2.await)
        else {
            return;
        };
        let mut shift = [0.0; 2];
        for (axis, (target, (start, end))) in targets.into_iter().zip(padding).enumerate() {
            let Some((at, size, page)) = target else {
                continue;
            };
            let (Ok((target_x, target_y)), Ok(size)) = (at.await, size.await) else {
                continue;
            };
            if page && size.width == 0.0 && size.height == 0.0 {
                continue;
            }
            // `auto`, or a backend without computed styles: no room.
            let room =
                |side: Result<Option<f64>, PlatformError>| side.ok().flatten().unwrap_or(0.0);
            let (start, end) = (room(start.await), room(end.await));
            shift[axis] = nearest_shift(
                match axis {
                    0 => (strip_x, strip_size.width, target_x, size.width),
                    _ => (strip_y, strip_size.height, target_y, size.height),
                },
                start,
                end,
            );
        }
        if shift != [0.0; 2] {
            let _ = strip.scroll_to(scroll_x + shift[0], scroll_y + shift[1]);
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
