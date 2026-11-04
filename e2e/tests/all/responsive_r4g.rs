//! Responsive leftovers (bundle r4g): what Rust reads per breakpoint follows the window.

use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

/// Each event's first slot after the bullet, as `body|opposite|..`.
const TIMELINE_ORDER: &str = "[...document.querySelectorAll('#r4g-timeline > [data-slot=item]')] \
    .map((item) => item.children[1].dataset.slot).join('|')";

async fn js_is(page: &Page, js: &str, want: &str) {
    let reached = wait::for_js_true(page, &format!("({js}) === '{want}'"), want).await;
    let now: String = page
        .evaluate_bounded(js)
        .await
        .unwrap()
        .into_value()
        .unwrap();
    assert!(reached.is_ok(), "{now}, want {want}");
}

async fn resize(page: &Page, width: i64) {
    page.execute_bounded(SetDeviceMetricsOverrideParams::new(width, 800, 1.0, false))
        .await
        .unwrap();
}

#[test]
fn a_responsive_timeline_align_reorders_its_events_with_the_window() {
    block_on(async {
        let fixture = Fixture::open("/responsive-r4g", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#r4g-timeline").await.unwrap();
        // Alternate from md: the second event's body sits first, on the start half.
        js_is(page, TIMELINE_ORDER, "opposite|body|opposite").await;
        resize(page, 600).await;
        js_is(page, TIMELINE_ORDER, "opposite|opposite|opposite").await;
        fixture
            .console
            .assert_clean("responsive r4g timeline")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Each avatar's width, the `+N` chip last.
const AVATAR_WIDTHS: &str = "[...document.querySelectorAll('#r4g-avatars [role=img]')] \
    .map((avatar) => Math.round(avatar.getBoundingClientRect().width)).join('|')";

#[test]
fn a_responsive_avatar_group_sizes_its_members_and_chip_with_the_window() {
    block_on(async {
        let fixture = Fixture::open("/responsive-r4g", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#r4g-avatars").await.unwrap();
        js_is(page, AVATAR_WIDTHS, "56|56|56").await;
        resize(page, 600).await;
        js_is(page, AVATAR_WIDTHS, "20|20|20").await;
        fixture
            .console
            .assert_clean("responsive r4g avatars")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// A body cell's inline padding.
const CELL_PADDING: &str =
    "getComputedStyle(document.querySelector('#r4g-table tbody td')).paddingLeft";

#[test]
fn a_responsive_table_size_pads_its_cells_with_the_window() {
    block_on(async {
        let fixture = Fixture::open("/responsive-r4g", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#r4g-table").await.unwrap();
        js_is(page, CELL_PADDING, "12px").await;
        resize(page, 600).await;
        js_is(page, CELL_PADDING, "8px").await;
        fixture
            .console
            .assert_clean("responsive r4g table")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// The second button's height, and whether its seam side takes its own rim colour.
const SEAM: &str = "(() => { const css = getComputedStyle(document.getElementById('r4g-italic')); \
    const own = css.borderInlineStartColor === css.borderTopColor ? 'rim' : 'divider'; \
    return Math.round(parseFloat(css.height)) + '|' + own; })()";

#[test]
fn a_responsive_button_group_sizes_and_seams_its_buttons_with_the_window() {
    block_on(async {
        let fixture = Fixture::open("/responsive-r4g", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#r4g-italic").await.unwrap();
        // Outlined from md: no divider over the rim.
        wait::for_js_true(page, &format!("{SEAM}.endsWith('|rim')"), "rim")
            .await
            .unwrap();
        let wide: String = page
            .evaluate_bounded(SEAM)
            .await
            .unwrap()
            .into_value()
            .unwrap();
        resize(page, 600).await;
        // Filled below md: the divider, and the xs height.
        wait::for_js_true(page, &format!("{SEAM}.endsWith('|divider')"), "divider")
            .await
            .unwrap();
        let narrow: String = page
            .evaluate_bounded(SEAM)
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let height = |seam: &str| seam.split('|').next().unwrap().parse::<u32>().unwrap();
        assert!(height(&narrow) < height(&wide), "{narrow} vs {wide}");
        fixture
            .console
            .assert_clean("responsive r4g group")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// The text's wrapping and its line clamp.
const CUT: &str = "(() => { const css = getComputedStyle(document.getElementById('r4g-text')); \
    return css.whiteSpace + '|' + css.webkitLineClamp; })()";

#[test]
fn a_zero_clamp_range_truncates_one_line_and_clamps_from_md() {
    block_on(async {
        let fixture = Fixture::open("/responsive-r4g", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#r4g-text").await.unwrap();
        js_is(page, CUT, "normal|3").await;
        resize(page, 600).await;
        js_is(page, CUT, "nowrap|none").await;
        fixture.console.assert_clean("responsive r4g text").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The first stone's gap margin and whether the second sits exactly one gap below it.
const STONES: &str = "(() => { const a = document.getElementById('r4g-stone-a'); \
    const b = document.getElementById('r4g-stone-b').getBoundingClientRect(); \
    const margin = parseFloat(getComputedStyle(a).marginBottom); \
    const space = b.top - a.getBoundingClientRect().bottom; \
    return Math.round(margin) + '|' + (Math.abs(space - margin) < 1); })()";

#[test]
fn a_masonry_zone_packs_by_the_gap_of_each_breakpoint() {
    block_on(async {
        let fixture = Fixture::open("/responsive-r4g", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#r4g-stone-b").await.unwrap();
        js_is(page, STONES, "16|true").await;
        resize(page, 600).await;
        js_is(page, STONES, "4|true").await;
        fixture
            .console
            .assert_clean("responsive r4g masonry")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
