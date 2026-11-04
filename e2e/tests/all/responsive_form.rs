//! Responsive form props (todo 5652): the field's frame and the card look follow the viewport,
//! from the server's copies and from copies inserted after mount.

use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::passes::pointer;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

/// Frame heights of the small, large, responsive and late fields, then the card's border width.
const LOOK: &str = "(() => { const h = id => { const f = document.querySelector(`#${id} [data-slot='frame']`); \
    return f ? getComputedStyle(f).height : '-'; }; \
    const card = getComputedStyle(document.querySelector('#rspf-card > *')).borderTopWidth; \
    return [h('rspf-sm'), h('rspf-lg'), h('rspf-text'), h('rspf-late'), card].join('|'); })()";

async fn look(page: &Page) -> Vec<String> {
    let text: String = page
        .evaluate_bounded(LOOK)
        .await
        .unwrap()
        .into_value()
        .unwrap();
    text.split('|').map(str::to_string).collect()
}

#[test]
fn form_props_follow_the_viewport_width() {
    block_on(async {
        let fixture = Fixture::open("/responsive-form", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#rspf-text").await.unwrap();
        pointer::click(page, "#rspf-add").await.unwrap();
        wait::for_visible(page, "#rspf-late").await.unwrap();
        let wide = look(page).await;
        assert_ne!(wide[0], wide[1], "{wide:?}");
        assert_eq!((&wide[2], &wide[3]), (&wide[1], &wide[1]), "{wide:?}");
        assert_eq!(wide[4], "1px", "{wide:?}");

        page.execute_bounded(SetDeviceMetricsOverrideParams::new(600, 800, 1.0, false))
            .await
            .unwrap();
        let narrow = format!("{0}|{1}|{0}|{0}|0px", wide[0], wide[1]);
        let reached =
            wait::for_js_true(page, &format!("{LOOK} === '{narrow}'"), "the narrow look").await;
        assert!(reached.is_ok(), "{:?}", look(page).await);

        fixture
            .console
            .assert_clean("responsive form props")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// The segment height, the strip's gap and an unselected segment's background.
const SEGMENTS: &str = "(() => { const s = document.querySelectorAll('#rspf-seg label[data-slot=segment]'); \
    const strip = document.querySelector('#rspf-seg [data-slot=control]'); \
    return [getComputedStyle(s[0]).height, getComputedStyle(strip).columnGap, \
    getComputedStyle(s[1]).backgroundColor].join('|'); })()";

#[test]
fn a_segmented_control_follows_the_viewport_width() {
    block_on(async {
        let fixture = Fixture::open("/responsive-form", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#rspf-seg").await.unwrap();
        let read = || async {
            let text: String = page
                .evaluate_bounded(SEGMENTS)
                .await
                .unwrap()
                .into_value()
                .unwrap();
            text.split('|').map(str::to_string).collect::<Vec<_>>()
        };
        let wide = read().await;
        assert_eq!(
            (&wide[1][..], &wide[2][..]),
            ("10px", "rgba(0, 0, 0, 0)"),
            "{wide:?}"
        );

        page.execute_bounded(SetDeviceMetricsOverrideParams::new(600, 800, 1.0, false))
            .await
            .unwrap();
        let narrow = wait::for_js_true(
            page,
            &format!("{SEGMENTS}.split('|')[1] === '2px'"),
            "the narrow gap",
        )
        .await;
        assert!(narrow.is_ok(), "{:?}", read().await);
        let narrow = read().await;
        let px = |v: &str| v.trim_end_matches("px").parse::<f64>().unwrap();
        assert!(px(&narrow[0]) < px(&wide[0]), "{wide:?} {narrow:?}");
        assert_ne!(narrow[2], "rgba(0, 0, 0, 0)", "{narrow:?}");

        fixture
            .console
            .assert_clean("responsive segmented control")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// The first chip's height, the pending loader's width and the first open row's height.
const PARTS: &str = "(() => { const px = el => el ? getComputedStyle(el).height : '-'; \
    const chip = document.querySelector('#rspf-tags [data-slot=tag] > *'); \
    const loader = document.querySelector('#rspf-pending [data-pending] > *'); \
    const row = document.querySelector('[role=option]'); \
    return [px(chip), loader ? getComputedStyle(loader).width : '-', px(row)].join('|'); })()";

#[test]
fn chips_loaders_and_open_rows_follow_the_viewport_width() {
    block_on(async {
        let fixture = Fixture::open("/responsive-form", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#rspf-tags [data-slot=tag]")
            .await
            .unwrap();
        wait::for_visible(page, "#rspf-pending [data-pending]")
            .await
            .unwrap();
        pointer::click(page, "#rspf-select [role=combobox]")
            .await
            .unwrap();
        wait::for_visible(page, "[role=option]").await.unwrap();
        let read = || async {
            let text: String = page
                .evaluate_bounded(PARTS)
                .await
                .unwrap()
                .into_value()
                .unwrap();
            text.split('|').map(str::to_string).collect::<Vec<_>>()
        };
        let px = |v: &str| v.trim_end_matches("px").parse::<f64>().unwrap();
        let wide = read().await;

        page.execute_bounded(SetDeviceMetricsOverrideParams::new(600, 800, 1.0, false))
            .await
            .unwrap();
        let every = format!(
            "(() => {{ const was = '{}'.split('|'); return {PARTS}.split('|').every((v, i) => v !== was[i]); }})()",
            wide.join("|")
        );
        let switched = wait::for_js_true(page, &every, "every part switched").await;
        assert!(switched.is_ok(), "{wide:?} {:?}", read().await);
        let narrow = read().await;
        for (wide, narrow) in wide.iter().zip(&narrow) {
            assert!(px(narrow) < px(wide), "{wide:?} {narrow:?}");
        }

        fixture
            .console
            .assert_clean("responsive field parts")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 5706: the open list's box and its first row round by the field's radius per breakpoint.
const RADII: &str = "(() => { const r = el => el ? getComputedStyle(el).borderTopLeftRadius : '-'; \
    return [r(document.querySelector('[data-slot=dropdown]')), r(document.querySelector('[role=option]'))].join('|'); })()";

#[test]
fn an_open_list_rounds_by_the_responsive_field_radius() {
    block_on(async {
        let fixture = Fixture::open("/responsive-form", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        pointer::click(page, "#rspf-select [role=combobox]")
            .await
            .unwrap();
        wait::for_visible(page, "[role=option]").await.unwrap();
        let read = || async {
            let text: String = page
                .evaluate_bounded(RADII)
                .await
                .unwrap()
                .into_value()
                .unwrap();
            text
        };
        let px = |v: &str| v.trim_end_matches("px").parse::<f64>().unwrap();
        let wide = read().await;

        page.execute_bounded(SetDeviceMetricsOverrideParams::new(600, 800, 1.0, false))
            .await
            .unwrap();
        let every = format!(
            "(() => {{ const was = '{wide}'.split('|'); return {RADII}.split('|').every((v, i) => v !== was[i]); }})()"
        );
        let switched = wait::for_js_true(page, &every, "both radii switched").await;
        assert!(switched.is_ok(), "{wide} {}", read().await);
        let narrow = read().await;
        for (wide, narrow) in wide.split('|').zip(narrow.split('|')) {
            assert!(px(narrow) < px(wide), "{wide:?} {narrow:?}");
        }

        fixture
            .console
            .assert_clean("responsive list radius")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
