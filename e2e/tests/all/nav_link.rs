//! `NavLink`'s `scroll_into_view` scrolls its sidebar, not the page, just far enough
//! (468 N3). Plus the `NavLink`, `Anchor` and `Burger` baseline (449).

use anyhow::Result;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused};
use e2e::passes::keyboard;
use e2e::{Fixture, Suite, Viewport, wait};

async fn the_toggle_shows_nested<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const TOGGLE: &str = "#docs + button";
    assert_eq!(
        d.attr(TOGGLE, "aria-expanded").await?.as_deref(),
        Some("false")
    );
    d.click(TOGGLE).await?;
    eventually(d, "aria-expanded true", async |d| {
        Ok(d.attr(TOGGLE, "aria-expanded").await?.as_deref() == Some("true"))
    })
    .await?;
    eventually(d, "the nested link to show", async |d| {
        Ok(d.rect("#install").await?.height > 0.0)
    })
    .await
}

e2e::scenario!(
    a_click_on_the_toggle_shows_the_nested_links,
    "/nav-link/states",
    the_toggle_shows_nested
);

/// Tab from the parent link reaches the toggle, Enter opens the panel with focus kept on
/// the toggle, and the next Tab enters the nested links.
async fn the_toggle_works_from_the_keyboard<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const TOGGLE: &str = "#docs + button";
    d.focus("#docs").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, TOGGLE, "Tab from the link").await?;
    d.press(keyboard::ENTER).await?;
    eventually(d, "aria-expanded true", async |d| {
        Ok(d.attr(TOGGLE, "aria-expanded").await?.as_deref() == Some("true"))
    })
    .await?;
    eventually_focused(d, TOGGLE, "Enter").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, "#install", "Tab into the opened panel").await?;
    d.focus(TOGGLE).await?;
    d.press(keyboard::SPACE).await?;
    eventually(d, "aria-expanded false", async |d| {
        Ok(d.attr(TOGGLE, "aria-expanded").await?.as_deref() == Some("false"))
    })
    .await
}

e2e::scenario!(
    the_toggle_works_from_the_keyboard_everywhere,
    "/nav-link/states",
    the_toggle_works_from_the_keyboard
);

/// The link's bottom plus its 8rem margin meets the sidebar's bottom.
const SHOWN_NEAREST: &str = "(() => { \
    const bar = document.querySelector('#sidebar'); \
    const link = document.querySelector('#here'); \
    const margin = parseFloat(getComputedStyle(link).scrollMarginBottom); \
    const gap = bar.getBoundingClientRect().bottom - (link.getBoundingClientRect().bottom + margin); \
    return bar.scrollTop > 0 && Math.abs(gap) < 2 && window.scrollY === 0; \
})()";

/// Todo 574: the active link was a faint tint only.
#[test]
fn the_active_link_shows_the_on_state_bar() {
    use crate::button::{
        assert_gray_in_forced_colours, assert_on_bar, assert_on_in_forced_colours,
    };
    block_on(async {
        let fixture = Fixture::open("/nav-link/states", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        // Todo 715: the start bar, not the ring the other on states draw.
        assert_on_bar(page, "#active", "#idle").await;
        crate::calendar::force_colours(page).await;
        assert_on_in_forced_colours(page, "#active", "#idle").await;
        assert_gray_in_forced_colours(page, "#disabled").await;
        crate::button::assert_text_in_forced_colours(page, "#disabled-active", "HighlightText")
            .await;
        fixture.close().await.unwrap();
    });
}

#[test]
fn an_active_link_scrolls_its_sidebar_just_far_enough() {
    block_on(async {
        let fixture = Fixture::open("/nav-link", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        wait::for_js_true(page, SHOWN_NEAREST, "the link at the sidebar's bottom edge")
            .await
            .unwrap();

        fixture.console.assert_clean("an active nav link").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The stripe's outer edge, left and right, against the sidebar's clip box.
const RING_INSIDE_SIDEBAR: &str = "(() => { \
    const bar = document.querySelector('#sidebar').getBoundingClientRect(); \
    const link = document.querySelector('#here'); \
    const style = getComputedStyle(link); \
    const reach = parseFloat(style.outlineOffset) + parseFloat(style.outlineWidth); \
    const rect = link.getBoundingClientRect(); \
    return document.activeElement === link && style.outlineStyle !== 'none' \
        && rect.left - reach >= bar.left && rect.right + reach <= bar.right; \
})()";

/// A full-width link in a scrolling sidebar: an outset ring would be clipped
/// on both sides by the sidebar's `overflow`.
#[test]
fn the_focus_ring_is_not_clipped_by_the_sidebar() {
    block_on(async {
        let fixture = Fixture::open("/nav-link", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, "#here", 5).await.unwrap();
        let inside: bool = page
            .evaluate(RING_INSIDE_SIDEBAR)
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(inside, "the focus ring reaches past the sidebar's edges");

        fixture.close().await.unwrap();
    });
}

#[test]
fn it_meets_the_baseline() {
    Suite::new("nav_link", "/nav-link/states")
        .focusable("#active")
        .focusable("#idle")
        .focusable("#docs")
        .targets("#docs + button")
        .focusable("#inline")
        .focusable("#external")
        .focusable("#burger")
        .targets("#active")
        .targets("#burger")
        .targets("#burger-xs")
        .targets("#burger-sm")
        .tab_budget(20)
        .no_snapshot()
        .run();
}

fn burger(expression: &str) -> String {
    format!("document.querySelector('#burger').{expression}")
}

/// Enter and Space toggle the disclosure; `aria-expanded` follows under a
/// static name (todo 578), and focus stays on the burger.
#[test]
fn the_burger_toggles_its_panel_from_the_keyboard() {
    block_on(async {
        let fixture = Fixture::open("/nav-link/states", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, "#burger", 20).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "{} === 'true' && {} === 'Toggle navigation' \
                 && !document.querySelector('#burger-panel').hidden \
                 && document.activeElement.id === 'burger'",
                burger("getAttribute('aria-expanded')"),
                burger("getAttribute('aria-label')"),
            ),
            "Enter to open the panel",
        )
        .await
        .unwrap();

        keyboard::press(page, keyboard::SPACE).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "{} === 'false' && {} === 'Toggle navigation' \
                 && document.querySelector('#burger-panel').hidden",
                burger("getAttribute('aria-expanded')"),
                burger("getAttribute('aria-label')"),
            ),
            "Space to close the panel",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("toggling a burger").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1591: forced colours paint the bars' outline, not their fill, so the open
/// X must drop the middle bar's outline too, or it draws an asterisk.
#[test]
fn the_open_burger_drops_its_middle_bar_in_forced_colours() {
    use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
    block_on(async {
        let fixture = Fixture::open("/nav-link/states", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.execute(
            SetEmulatedMediaParams::builder()
                .features(vec![MediaFeature::new("forced-colors", "active")])
                .build(),
        )
        .await
        .unwrap();
        // Each bar's outline: the middle one, then the two that cross.
        let outlines = format!(
            "(() => {{ const glyph = {}; return [null, '::before', '::after'] \
             .map(p => getComputedStyle(glyph, p).outlineStyle).join(' '); }})()",
            burger("querySelector('[data-slot=glyph]')")
        );
        wait::for_js_true(
            page,
            &format!(
                "matchMedia('(forced-colors: active)').matches && {outlines} === 'solid solid solid'"
            ),
            "three bars when closed",
        )
        .await
        .unwrap();

        page.find_element("#burger")
            .await
            .unwrap()
            .click()
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &format!("{outlines} === 'none solid solid'"),
            "only the two crossed bars when open",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// `ActionIcon` writes its own `aria-label`; a caller's spread one must still
/// name the burger.
#[test]
fn a_spread_aria_label_names_the_burger() {
    block_on(async {
        let fixture = Fixture::open("/nav-link/states", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        wait::for_js_true(
            page,
            "document.querySelector('#burger-xs')?.getAttribute('aria-label') === 'Menu xs'",
            "the caller's aria-label on the burger",
        )
        .await
        .unwrap();

        fixture.close().await.unwrap();
    });
}

/// Todo 579: a `_blank` Anchor draws its icon beside the text and says so in
/// its name.
#[test]
fn a_new_tab_anchor_shows_an_icon_and_says_so() {
    block_on(async {
        let fixture = Fixture::open("/nav-link/states", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        let tree = e2e::ax::snapshot(page, "#prose").await.unwrap();
        assert!(
            tree.contains("the external one (opens in a new tab)"),
            "{tree}"
        );
        let drawn: bool = page
            .evaluate(
                "(() => { const svg = document.querySelector('#external svg').getBoundingClientRect(); \
                 const link = document.querySelector('#external').getBoundingClientRect(); \
                 return svg.width > 8 && svg.width < 16 && svg.right <= link.right + 1 \
                     && svg.top >= link.top - 1 && svg.bottom <= link.bottom + 1; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(drawn, "the icon sits inside the link, at text size");

        fixture.close().await.unwrap();
    });
}

/// Todo 580: the toggle beside a parent link is an APG disclosure button,
/// named after the link; Enter shows the nested links and focus stays put.
#[test]
fn a_parent_link_discloses_its_nested_links() {
    const TOGGLE: &str = "document.querySelector('#docs').nextElementSibling";
    block_on(async {
        let fixture = Fixture::open("/nav-link/states", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#docs").await.unwrap();

        let tree = e2e::ax::snapshot(page, "nav").await.unwrap();
        for line in ["link \"Docs\"", "button \"Show links Docs\""] {
            assert!(tree.contains(line), "no {line:?} in\n{tree}");
        }
        assert!(
            !tree.contains("link \"Install\""),
            "a closed panel is read:\n{tree}"
        );
        assert_eq!(
            e2e::ax::description(page, "#docs").await.unwrap(),
            "Guides and API"
        );

        keyboard::tab_to(page, "#docs", 20).await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "{TOGGLE}.getAttribute('aria-expanded') === 'true' \
                 && document.activeElement === {TOGGLE} \
                 && document.getElementById({TOGGLE}.getAttribute('aria-controls')).contains(document.querySelector('#install'))"
            ),
            "Enter to show the nested links",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement.id === 'install'",
            "Tab into the shown links",
        )
        .await
        .unwrap();

        fixture
            .console
            .assert_clean("opening a parent link")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Tab leaves the closed disclosure for the next control, not for a hidden nested link.
#[test]
fn a_closed_panels_links_are_out_of_the_tab_order() {
    block_on(async {
        let fixture = Fixture::open("/nav-link/states", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, "#docs + button", 20).await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        let id: String = page
            .evaluate("document.activeElement.id || document.activeElement.tagName")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            id != "install" && id != "theming",
            "focus entered a closed panel: {id}"
        );
        fixture.close().await.unwrap();
    });
}

/// Without `active`, a link to the route's path is current whatever query or
/// fragment the address carries.
#[test]
fn a_query_on_the_address_keeps_the_route_link_current() {
    block_on(async {
        for route in [
            "/nav-link/states",
            "/nav-link/states?tab=2",
            "/nav-link/states?tab=2#top",
        ] {
            let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
            let current = fixture
                .page
                .evaluate("document.querySelector('#auto').getAttribute('aria-current')")
                .await
                .unwrap()
                .into_value::<Option<String>>()
                .unwrap();
            assert_eq!(current.as_deref(), Some("page"), "{route}");
            fixture.close().await.unwrap();
        }
    });
}

/// The description's contrast against the surface under it, as a string `"4.52"`.
const DESCRIPTION_RATIO: &str = r#"(id => {
  const ctx = document.createElement('canvas').getContext('2d', { willReadFrequently: true });
  const rgba = c => { ctx.clearRect(0, 0, 1, 1); ctx.fillStyle = '#000'; ctx.fillStyle = c; ctx.fillRect(0, 0, 1, 1); const d = ctx.getImageData(0, 0, 1, 1).data; return [d[0], d[1], d[2], d[3] / 255]; };
  const over = (f, b) => [0, 1, 2].map(i => f[i] * f[3] + b[i] * (1 - f[3])).concat([1]);
  const bg = el => { const chain = []; for (let e = el; e; e = e.parentElement) chain.push(rgba(getComputedStyle(e).backgroundColor)); let out = [255, 255, 255, 1]; for (const c of chain.reverse()) out = over(c, out); return out; };
  const lum = c => { const f = v => { v /= 255; return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4); }; return 0.2126 * f(c[0]) + 0.7152 * f(c[1]) + 0.0722 * f(c[2]); };
  const el = document.querySelector(id + ' [data-slot=description]');
  const back = bg(el), front = over(rgba(getComputedStyle(el).color), back);
  const [x, y] = [lum(front), lum(back)];
  return ((Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05)).toFixed(2);
})"#;

/// 1.4.3: the description is 12px text, so 4.5:1 on the active tint and on the
/// hover grey too, in both schemes.
#[test]
fn the_description_reads_on_the_active_tint_and_on_hover() {
    use e2e::Scheme;
    block_on(async {
        for scheme in [Scheme::Light, Scheme::Dark] {
            let fixture = Fixture::open_in("/nav-link/states", Viewport::Desktop, scheme)
                .await
                .unwrap();
            let page = &fixture.page;
            let ratio = |id: &'static str| {
                let expression = format!("({DESCRIPTION_RATIO})('{id}')");
                async move {
                    let text: String = page
                        .evaluate(expression)
                        .await
                        .unwrap()
                        .into_value()
                        .unwrap();
                    text.parse::<f64>().unwrap()
                }
            };
            let active = ratio("#described-active").await;
            e2e::passes::pointer::hover(page, "#docs").await.unwrap();
            let hovered = ratio("#docs").await;
            assert!(
                active >= 4.5 && hovered >= 4.5,
                "{}: description on the active tint {active}:1, on hover {hovered}:1",
                scheme.name()
            );
            fixture.close().await.unwrap();
        }
    });
}

/// Todo 596: a disabled link takes the pointer and shows `not-allowed`, with
/// no hover tint. It has no `href`, so a click has nothing to follow.
#[test]
fn a_disabled_link_shows_not_allowed_and_no_hover() {
    block_on(async {
        let fixture = Fixture::open("/nav-link/states", Viewport::Desktop)
            .await
            .unwrap();
        crate::action_icon::assert_disabled_look(&fixture.page, "#disabled").await;
        crate::action_icon::assert_disabled_look(&fixture.page, "#disabled-active").await;
        fixture.close().await.unwrap();
    });
}
