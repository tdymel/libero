//! `FloatingActionButton` in Chromium: it stays at its corner while the page scrolls,
//! flips under `dir="rtl"`, clears a fixed `NavigationBar`, keeps its focus ring
//! whole and gives a finger a 44px target at every size. A hidden one leaves the tab
//! order, and as a `Menu`'s trigger it opens the menu and takes focus back, in every form,
//! opens nothing while busy or disabled, and draws no tooltip over the menu.

use anyhow::{Result, ensure};
use e2e::browser::block_on;
use e2e::driver::{Driver, Platform, eventually, eventually_focused, eventually_text};
use e2e::passes::{focus, keyboard, pointer};
use e2e::{Fixture, Suite, Viewport, js, wait};
use serde_json::Value;

const FAB: &str = "#fab";
const SMALL: &str = "#small";
const TRIGGER: &str = "[aria-haspopup='menu']";
const MENU: &str = "[role='menu']";
const STATES: &str = "/floating-action-button/menu-states";

/// Todo 5297: the portal sits outside a local `dir`, as the Localization page says.
#[test]
fn a_menu_under_a_local_rtl_wrapper_takes_the_document_s_direction() {
    block_on(async {
        let fixture = Fixture::open("/floating-action-button/menu-local-rtl", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        pointer::click(page, TRIGGER).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "getComputedStyle(document.querySelector(\"{TRIGGER}\")).direction === 'rtl' \
                 && getComputedStyle(document.querySelector(\"{MENU}\")).direction === 'ltr'"
            ),
            "an rtl trigger and an ltr menu",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 5297: what the Localization page offers instead of a local `dir`.
#[test]
fn a_menu_under_a_provider_s_rtl_direction_is_rtl() {
    block_on(async {
        let fixture = Fixture::open("/floating-action-button/menu-states-rtl", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        pointer::click(page, TRIGGER).await.unwrap();
        wait::for_js_true(
            page,
            &format!("getComputedStyle(document.querySelector(\"{MENU}\")).direction === 'rtl'"),
            "an rtl menu",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn it_meets_the_baseline() {
    Suite::new("floating_action_button", "/floating-action-button")
        .focusable(FAB)
        .focusable(SMALL)
        .targets(FAB)
        .targets(SMALL)
        .run();
}

/// Its box, the viewport and the spacing it should keep from the edges.
async fn placed(page: &chromiumoxide::Page) -> Value {
    js::<Value>(
        page,
        "(() => { const r = document.querySelector('#fab').getBoundingClientRect(); \
            const gap = parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--lsx-spacing-md')); \
            return { left: r.left, right: r.right, top: r.top, bottom: r.bottom, width: r.width, \
                vw: document.documentElement.clientWidth, vh: innerHeight, gap }; })()",
    )
    .await
}

fn near(a: &Value, b: f64) -> bool {
    (a.as_f64().unwrap() - b).abs() < 1.0
}

#[test]
fn it_holds_its_corner_while_the_page_scrolls() {
    block_on(async {
        let fixture = Fixture::open("/floating-action-button", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, FAB).await.unwrap();
        let _: bool = js(page, "(() => { scrollTo(0, 1500); return true; })()").await;
        wait::for_js_true(page, "scrollY >= 1500", "the scroll")
            .await
            .unwrap();

        let at = placed(page).await;
        let gap = at["gap"].as_f64().unwrap();
        assert!(near(&at["right"], at["vw"].as_f64().unwrap() - gap), "{at}");
        assert!(
            near(&at["bottom"], at["vh"].as_f64().unwrap() - gap),
            "{at}"
        );
        assert!(near(&at["width"], 60.0), "{at}");
        fixture.console.assert_clean("scrolling").unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn end_is_the_left_corner_under_rtl() {
    block_on(async {
        let fixture = Fixture::open("/floating-action-button/rtl", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, FAB).await.unwrap();
        let at = placed(page).await;
        assert!(near(&at["left"], at["gap"].as_f64().unwrap()), "{at}");
        fixture.console.assert_clean("rtl").unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn it_sits_above_a_fixed_navigation_bar() {
    block_on(async {
        let fixture = Fixture::open("/floating-action-button/bar", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, FAB).await.unwrap();
        wait::for_js_true(
            page,
            "getComputedStyle(document.documentElement).getPropertyValue('--lsx-navigation-bar-height').trim().endsWith('px')",
            "the measured bar height",
        )
        .await
        .unwrap();
        let report = js::<Value>(
            page,
            "(() => { const fab = document.querySelector('#fab').getBoundingClientRect(); \
                const bar = document.querySelector('#bar').getBoundingClientRect(); \
                const gap = parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--lsx-spacing-md')); \
                const icon = document.querySelector('#fab svg').getBoundingClientRect(); \
                return { fab: fab.bottom, bar: bar.top, gap, icon: icon.width, name: document.querySelector('#fab').textContent.trim() }; })()",
        )
        .await;
        let bar = report["bar"].as_f64().unwrap();
        assert!(
            near(&report["fab"], bar - report["gap"].as_f64().unwrap()),
            "{report}"
        );
        assert_eq!(report["name"], "Compose", "{report}");
        // `Button` leaves an unsized svg at 0; the FAB sizes it.
        assert!(report["icon"].as_f64().unwrap() >= 16.0, "{report}");
        fixture.console.assert_clean("over the bar").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The shadow gives way on keyboard focus: the ring's own layers come first in `box-shadow`.
#[test]
fn keyboard_focus_draws_the_ring_over_the_shadow() {
    block_on(async {
        let fixture = Fixture::open("/floating-action-button", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, FAB).await.unwrap();
        let resting: String = js(
            page,
            "getComputedStyle(document.querySelector('#fab')).boxShadow",
        )
        .await;
        e2e::passes::keyboard::tab_to(page, FAB, 6).await.unwrap();
        let focused = js::<Value>(
            page,
            "(() => { const s = getComputedStyle(document.querySelector('#fab')); \
                return { shadow: s.boxShadow, outline: s.outlineStyle }; })()",
        )
        .await;
        assert_ne!(resting, "none", "a resting shadow");
        let shadow = focused["shadow"].as_str().unwrap();
        assert!(
            shadow.len() > resting.len(),
            "ring layers join the shadow: {focused}"
        );
        assert_eq!(focused["outline"], "solid", "{focused}");
        fixture.console.assert_clean("focusing").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Actions go through a `Menu`: a press opens it above the button at its end edge, an
/// item runs and hands focus back, Escape does too, and a press outside closes it.
#[test]
fn it_opens_a_menu_as_its_trigger() {
    block_on(async {
        let fixture = Fixture::open("/floating-action-button/menu", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, TRIGGER).await.unwrap();
        pointer::click(page, TRIGGER).await.unwrap();
        wait::for_visible(page, MENU).await.unwrap();
        let at = js::<Value>(
            page,
            "(() => { const t = document.querySelector(\"[aria-haspopup='menu']\"); \
                const b = t.getBoundingClientRect(); \
                const m = document.querySelector(\"[role='menu']\").getBoundingClientRect(); \
                return { fab_top: b.top, fab_right: b.right, bottom: m.bottom, right: m.right, \
                    expanded: t.getAttribute('aria-expanded') }; })()",
        )
        .await;
        assert_eq!(at["expanded"], "true", "{at}");
        assert!(
            at["bottom"].as_f64().unwrap() <= at["fab_top"].as_f64().unwrap(),
            "{at}"
        );
        assert!(
            near(&at["right"], at["fab_right"].as_f64().unwrap()),
            "{at}"
        );

        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_hidden(page, MENU).await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#sent').textContent === '1'",
            "the item",
        )
        .await
        .unwrap();
        focus::wait_for_focus(page, TRIGGER, "an item's press")
            .await
            .unwrap();

        pointer::click(page, TRIGGER).await.unwrap();
        wait::for_visible(page, MENU).await.unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_hidden(page, MENU).await.unwrap();
        focus::wait_for_focus(page, TRIGGER, "Escape")
            .await
            .unwrap();

        pointer::click(page, TRIGGER).await.unwrap();
        wait::for_visible(page, MENU).await.unwrap();
        pointer::click(page, "#sent").await.unwrap();
        wait::for_hidden(page, MENU).await.unwrap();
        fixture.console.assert_clean("the menu").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A press on the switch's state, then on the trigger, opens nothing: `tick` flushes the press.
async fn press_opens_nothing<D: Driver>(d: &mut D, switch: &str, tick: &str) -> Result<()> {
    d.click(switch).await?;
    eventually(d, "the switch's state", async |d| {
        Ok(d.attr(TRIGGER, "aria-disabled").await?.as_deref() == Some("true"))
    })
    .await?;
    d.click(TRIGGER).await?;
    d.click("#tick").await?;
    eventually_text(d, "#tick", tick, "the flush").await?;
    let opened = d.text("#opened").await?;
    ensure!(opened == "0", "{switch} opened the menu {opened} times");
    d.click(switch).await?;
    eventually(d, "the switch back", async |d| {
        Ok(d.attr(TRIGGER, "aria-disabled").await?.is_none())
    })
    .await
}

/// `aria-disabled` is no native `disabled`, so the press used to bubble to `Menu`'s wrapper.
async fn busy_opens_no_menu<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    press_opens_nothing(d, "#loading", "1").await?;
    press_opens_nothing(d, "#disabled", "2").await?;
    // Extended, it is a `Button`, not an `ActionIcon`.
    d.click("#extended").await?;
    eventually(d, "the label", async |d| {
        Ok(d.text(TRIGGER).await?.contains("Create"))
    })
    .await?;
    press_opens_nothing(d, "#loading", "3").await?;
    press_opens_nothing(d, "#disabled", "4").await?;
    d.click(TRIGGER).await?;
    eventually_text(d, "#opened", "1", "a press once it is free").await
}

e2e::scenario!(
    a_loading_or_disabled_trigger_opens_no_menu,
    "/floating-action-button/menu-states",
    busy_opens_no_menu
);

/// A press opens it and Escape hands focus back: icon-only, with a tooltip and extended.
async fn every_form_opens<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    // A finger opens the sheet on its box, no row highlighted (4862).
    let first = match d.platform() {
        Platform::Android | Platform::Ios => MENU,
        _ => "[role=menuitem]",
    };
    for (switch, opened) in [
        (None, "1"),
        (Some("#tooltip"), "2"),
        (Some("#extended"), "3"),
    ] {
        if let Some(switch) = switch {
            d.click(switch).await?;
        }
        d.click(TRIGGER).await?;
        eventually_text(d, "#opened", opened, "the press").await?;
        eventually_focused(d, first, "the press").await?;
        ensure!(
            !d.exists("[role=tooltip]").await?,
            "a tooltip over the menu"
        );
        d.press(keyboard::ESCAPE).await?;
        eventually_focused(d, TRIGGER, "Escape").await?;
    }
    Ok(())
}

e2e::scenario!(
    every_form_opens_the_menu_and_takes_focus_back,
    "/floating-action-button/menu-states",
    every_form_opens
);

/// Todo 5272, on a phone's WebView: the sheet at the screen's foot in the page's direction, a
/// hidden button opening nothing, and a button hidden under an open sheet leaving it working.
async fn phone_menu<D: Driver>(d: &mut D, route: &str) -> Result<()> {
    let rtl = route.ends_with("-rtl");
    let (width, height) = d.viewport().await?;
    let fab = d.rect(TRIGGER).await?;
    ensure!((fab.x < width / 2.0) == rtl, "at the wrong corner: {fab:?}");

    d.click(TRIGGER).await?;
    eventually_text(d, "#opened", "1", "the tap").await?;
    eventually(d, "the sheet at the screen's foot", async |d| {
        if !d.exists(MENU).await? {
            return Ok(false);
        }
        let sheet = d.rect(MENU).await?;
        Ok((sheet.width - width).abs() < 1.0 && (sheet.y + sheet.height - height).abs() < 1.0)
    })
    .await?;
    let direction = d.style(MENU, "direction").await?;
    let expected = if rtl { "rtl" } else { "ltr" };
    ensure!(direction == expected, "a {direction} sheet");
    d.click("[role=menuitem]").await?;
    eventually_text(d, "#sent", "1", "the row's tap").await?;
    eventually(d, "the sheet to close", async |d| {
        Ok(!d.exists(MENU).await?)
    })
    .await?;

    d.click("#hide").await?;
    eventually(d, "the button out of the tree", async |d| {
        let probe = format!("document.querySelector(\"{TRIGGER}\").closest('[inert]') !== null");
        Ok(d.evaluate(&probe).await?.as_bool() == Some(true))
    })
    .await?;
    d.click_at(fab.x + fab.width / 2.0, fab.y + fab.height / 2.0)
        .await?;
    d.click("#tick").await?;
    eventually_text(d, "#tick", "1", "the flush").await?;
    let opened = d.text("#opened").await?;
    ensure!(opened == "1", "a hidden button opened the menu: {opened}");
    d.click("#hide").await?;
    eventually(d, "the button back", async |d| {
        let probe = format!("document.querySelector(\"{TRIGGER}\").closest('[inert]') === null");
        Ok(d.evaluate(&probe).await?.as_bool() == Some(true))
    })
    .await?;

    d.click(TRIGGER).await?;
    eventually_text(d, "#opened", "2", "a tap once it is back").await?;
    eventually(d, "the sheet", async |d| d.exists(MENU).await).await?;
    d.script_click("#hide").await?;
    eventually(d, "the button out of the tree", async |d| {
        let probe = format!("document.querySelector(\"{TRIGGER}\").closest('[inert]') !== null");
        Ok(d.evaluate(&probe).await?.as_bool() == Some(true))
    })
    .await?;
    ensure!(d.exists(MENU).await?, "hiding the button closed the sheet");
    d.click("[role=menuitem]").await?;
    eventually_text(d, "#sent", "2", "the row's tap under a hidden button").await
}

e2e::scenario!(
    a_phone_opens_the_menu_as_a_sheet_and_a_hidden_button_opens_nothing,
    "/floating-action-button/menu-states",
    phone_menu,
    web: skip("5272: a phone's WebView; `menu::below_sm_the_menu_is_a_sheet_that_closes_every_way` has Chromium"),
    native: skip("5272: a phone's WebView"),
    desktop: skip("5272: a phone's WebView"),
);

e2e::scenario!(
    under_rtl_a_phone_opens_the_menu_as_a_sheet_and_a_hidden_button_opens_nothing,
    "/floating-action-button/menu-states-rtl",
    phone_menu,
    web: skip("5272: a phone's WebView; `menu::below_sm_the_menu_is_a_sheet_that_closes_every_way` has Chromium"),
    native: skip("5272: a phone's WebView"),
    desktop: skip("5272: a phone's WebView"),
);

/// The bubble sits where the menu opens: a showing one leaves when the menu comes.
#[test]
fn an_open_menu_hides_a_showing_tooltip() {
    block_on(async {
        let fixture = Fixture::open(STATES, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        pointer::click(page, "#tooltip").await.unwrap();
        wait::for_visible(page, TRIGGER).await.unwrap();
        pointer::hover(page, TRIGGER).await.unwrap();
        wait::for_visible(page, "[role='tooltip']").await.unwrap();
        pointer::click(page, TRIGGER).await.unwrap();
        wait::for_visible(page, MENU).await.unwrap();
        wait::for_hidden(page, "[role='tooltip']").await.unwrap();
        fixture.console.assert_clean("the tooltip").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Hidden, it leaves the tab order: Tab from the switch skips to the control after it.
#[test]
fn hidden_it_drops_out_of_the_tab_order() {
    block_on(async {
        let fixture = Fixture::open("/floating-action-button/hide", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, FAB).await.unwrap();
        pointer::click(page, "#hide").await.unwrap();
        wait::for_hidden(page, FAB).await.unwrap();
        let inert: bool = js(
            page,
            "document.querySelector('#fab').closest('[inert]') !== null",
        )
        .await;
        assert!(inert, "out of the accessibility tree at once");
        focus::wait_for_focus(page, "#hide", "the switch")
            .await
            .unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        focus::wait_for_focus(page, "#after", "Tab past the hidden button")
            .await
            .unwrap();
        fixture.console.assert_clean("hiding").unwrap();
        fixture.close().await.unwrap();
    });
}

/// `xl` is 60px anyway; an `xs` one grows a 44px press area round its 24px box.
#[test]
fn a_finger_gets_44px_at_every_size() {
    block_on(async {
        let fixture = Fixture::open("/floating-action-button", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        crate::hit_area::emulate_coarse_pointer(page).await;
        wait::for_visible(page, SMALL).await.unwrap();
        let report = js::<Value>(
            page,
            "(() => { const b = document.querySelector('#small'); const r = b.getBoundingClientRect(); \
                const hit = document.elementFromPoint(r.left + r.width / 2 + 20, r.top + r.height / 2); \
                const fab = document.querySelector('#fab').getBoundingClientRect(); \
                return { hit: b.contains(hit), small: r.width, fab: Math.min(fab.width, fab.height) }; })()",
        )
        .await;
        assert_eq!(report["hit"], true, "{report}");
        assert!(report["fab"].as_f64().unwrap() >= 43.5, "{report}");
        fixture.console.assert_clean("measuring").unwrap();
        fixture.close().await.unwrap();
    });
}
