//! Inline-start layouts under `dir="rtl"` (todo 747): each check runs in both
//! directions on an existing fixture and expects the RTL layout mirrored.

use e2e::browser::block_on;
use e2e::wait;

use crate::rtl_keys::open_in;

/// Runs `check` (JS, `rtl` bound) on `route` in both directions.
fn mirrors(route: &str, check: &str, what: &str) {
    block_on(async {
        for dir in ["ltr", "rtl"] {
            let fixture = open_in(route, dir).await;
            let condition = format!("(() => {{ const rtl = {}; {check} }})()", dir == "rtl");
            wait::for_js_true(&fixture.page, &condition, &format!("{dir}: {what}"))
                .await
                .unwrap();
            fixture.console.assert_clean(dir).unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// An off switch's thumb sits at the start of its track.
#[test]
fn an_off_switch_starts_at_the_start() {
    mirrors(
        "/switch",
        "const input = document.getElementById('aria'); \
         const thumb = input.parentElement.querySelector('span[aria-hidden=true] > span:only-child:empty'); \
         const track = thumb.parentElement.getBoundingClientRect(), t = thumb.getBoundingClientRect(); \
         const middle = track.left + track.width / 2; \
         return rtl ? t.left > middle : t.right < middle;",
        "the off thumb at the start",
    );
}

/// Each avatar tucks under the one before it, whichever side that is.
#[test]
fn a_group_overlaps_the_previous_avatar() {
    mirrors(
        "/avatar",
        "const [a, b] = [...document.getElementById('group').children].map(c => c.getBoundingClientRect()); \
         return rtl ? b.right > a.left + 1 && b.left < a.left : b.left < a.right - 1 && b.right > a.right;",
        "the second avatar over the first's start edge",
    );
}

/// Collapsed segments share one border: the next one overlaps the previous by
/// a pixel, on its start side.
#[test]
fn collapsed_segments_share_their_border() {
    mirrors(
        "/segmented-control",
        "const [a, b] = [...document.querySelectorAll('[role=radiogroup] > label')].map(l => l.getBoundingClientRect()); \
         const overlap = rtl ? b.right - a.left : a.right - b.left; \
         return Math.abs(overlap - 1) < 0.5;",
        "a one-pixel overlap",
    );
}

/// The required marker keeps its gap from the label text.
#[test]
fn the_required_marker_keeps_its_gap() {
    mirrors(
        "/text-field",
        "const marker = document.querySelector(\"label > [data-slot='required']\"); \
         const style = getComputedStyle(marker); \
         return rtl ? style.marginRight === '2px' && style.marginLeft === '0px' \
                    : style.marginLeft === '2px' && style.marginRight === '0px';",
        "the marker's gap on its start side",
    );
}

/// A `top-end` float sits in the end corner.
#[test]
fn an_end_float_sits_at_the_end() {
    mirrors(
        "/layout",
        "const f = document.getElementById('float').getBoundingClientRect(); \
         const s = document.getElementById('stage').getBoundingClientRect(); \
         return rtl ? f.left - s.left < s.right - f.right : s.right - f.right < f.left - s.left;",
        "the float in the end corner",
    );
}
