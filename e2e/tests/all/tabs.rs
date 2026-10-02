//! `Tabs`: the roving-tabindex archetype's first consumer.

use anyhow::Result;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::archetypes::{Orientation, RovingTabindex, reset_tab_position};
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused};
use e2e::passes::pointer;
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, passes::keyboard, wait};

pub const TAB: &str = "[role=tab]";
/// What the "second" state waits on. Not `TAB`: that is visible at rest, so
/// waiting on it returned at once and the snapshot raced the re-render.
const SECOND_SELECTED: &str = "[role=tablist] > [role=tab]:nth-child(2)[aria-selected=true]";

const SELECTED: &str = "[role=tab][aria-selected=true]";

async fn selected_is<D: Driver>(d: &mut D, label: &str, after: &str) -> Result<()> {
    eventually(d, &format!("{after}: {label} selected"), async |d| {
        Ok(d.text(SELECTED).await? == label)
    })
    .await
}

/// Automatic activation: the key moves focus and selection together.
async fn step<D: Driver>(d: &mut D, key: keyboard::Key, label: &str) -> Result<()> {
    d.press(key).await?;
    selected_is(d, label, key.key).await?;
    eventually_focused(d, SELECTED, key.key).await
}

async fn leaves_the_strip<D: Driver>(d: &mut D, after: &str) -> Result<()> {
    eventually(d, &format!("{after} to leave the strip"), async |d| {
        Ok(!d.is_focused(TAB).await?)
    })
    .await
}

async fn arrow_right_moves<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    selected_is(d, "Account", "load").await?;
    d.focus(SELECTED).await?;
    step(d, keyboard::ARROW_RIGHT, "Billing").await?;
    eventually(d, "Billing's panel", async |d| {
        Ok(d.text("[role=tabpanel]").await?.contains("Billing"))
    })
    .await
}

async fn arrows_wrap_and_jump<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(SELECTED).await?;
    step(d, keyboard::ARROW_LEFT, "Admin").await?;
    step(d, keyboard::HOME, "Account").await?;
    step(d, keyboard::END, "Admin").await?;
    step(d, keyboard::ARROW_RIGHT, "Account").await
}

/// Manual mode: the strip is one tab stop wherever focus sits, so Tab leaves from the
/// focused tab for the panel, not for the selected tab.
async fn leaves_from_focused_unselected<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(SELECTED).await?;
    d.press(keyboard::END).await?;
    eventually_focused(d, "[role=tablist] > [role=tab]:nth-child(3)", "End").await?;
    selected_is(d, "Account", "End in manual mode").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, "[role=tabpanel]", "Tab from the focused tab").await?;
    selected_is(d, "Account", "Tab out of the strip").await?;
    d.press_shift(keyboard::TAB).await?;
    eventually_focused(d, SELECTED, "Shift+Tab").await
}

/// A click focuses a disabled tab without selecting it; Shift+Tab then leaves
/// the strip rather than stopping on the selected tab before it.
async fn shift_tab_from_disabled<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("[role=tab][aria-disabled=true]").await?;
    selected_is(d, "Account", "a click on the disabled Billing").await?;
    eventually_focused(
        d,
        "[role=tab][aria-disabled=true]",
        "a click on the disabled Billing",
    )
    .await?;
    d.press_shift(keyboard::TAB).await?;
    leaves_the_strip(d, "Shift+Tab").await
}

async fn tab_out_and_back<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(SELECTED).await?;
    step(d, keyboard::ARROW_RIGHT, "Billing").await?;
    d.press(keyboard::TAB).await?;
    leaves_the_strip(d, "Tab").await?;
    d.press_shift(keyboard::TAB).await?;
    eventually_focused(d, SELECTED, "Shift+Tab").await
}

e2e::scenario!(
    arrow_right_moves_focus_and_selection_to_the_next_tab,
    "/tabs",
    arrow_right_moves,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView")
);
e2e::scenario!(
    arrow_left_wraps_and_home_and_end_jump,
    "/tabs",
    arrows_wrap_and_jump,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView")
);
e2e::scenario!(
    tab_leaves_the_strip_from_a_focused_unselected_tab,
    "/tabs-manual",
    leaves_from_focused_unselected,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView")
);
e2e::scenario!(
    shift_tab_leaves_the_strip_from_a_clicked_disabled_tab,
    "/tabs-disabled",
    shift_tab_from_disabled,
    desktop: skip("958: element identity on the WebView (Shift+Tab stays in the strip)")
);
e2e::scenario!(
    tab_leaves_the_strip_and_shift_tab_comes_back_to_the_selected_tab,
    "/tabs",
    tab_out_and_back,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView")
);

/// Forced colours paint every transparent underline, so each tab looked
/// selected (todo 500). The selected one must keep a line the others lack.
#[test]
fn the_selected_tab_stands_out_in_forced_colours() {
    use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
    block_on(async {
        let fixture = Fixture::open("/tabs", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        page.execute(
            SetEmulatedMediaParams::builder()
                .features(vec![MediaFeature::new("forced-colors", "active")])
                .build(),
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "matchMedia('(forced-colors: active)').matches",
            "forced colours to apply",
        )
        .await
        .unwrap();
        let (selected, others): (String, Vec<String>) = page
            .evaluate(
                "(() => {
                    const line = el => getComputedStyle(el).borderBottomColor;
                    const tabs = [...document.querySelectorAll('[role=tablist] > [role=tab]')];
                    return [
                        line(tabs.find(t => t.getAttribute('aria-selected') === 'true')),
                        tabs.filter(t => t.getAttribute('aria-selected') !== 'true').map(line),
                    ];
                })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(!others.is_empty(), "the fixture has one tab");
        assert!(
            others.iter().all(|other| *other != selected),
            "the selected tab's line {selected} matches an unselected one: {others:?}"
        );
        fixture.close().await.unwrap();
    });
}

#[test]
fn it_meets_the_baseline() {
    Suite::new("tabs", "/tabs")
        .focusable(TAB)
        .targets(TAB)
        // The second tab selected, because an "is the right one marked" bug is
        // invisible in the resting state where the first is selected anyway.
        .state(
            "second",
            &[Step::TabTo(TAB), Step::Press(keyboard::ARROW_RIGHT)],
            SECOND_SELECTED,
        )
        .run();
}

#[test]
fn it_honours_the_roving_tabindex_contract() {
    block_on(async {
        e2e::browser::at_every_viewport(async |viewport| {
            let fixture = Fixture::open("/tabs", viewport).await.unwrap();

            RovingTabindex {
                items: TAB,
                orientation: Orientation::Horizontal,
                // `TabsCore` wraps: it steps through the shared `neighbour`.
                wraps: true,
            }
            .assert_contract(&fixture.page)
            .await
            .unwrap_or_else(|e| panic!("at {}: {e}", viewport.name()));

            fixture
                .console
                .assert_clean(&format!("the tabs contract at {}", viewport.name()))
                .unwrap();
            fixture.close().await.unwrap();
        })
        .await;
    });
}

/// Todo 403: from a selected disabled tab (`Account, Billing (disabled,
/// selected), Admin`) Left and Right used to land on the same tab.
#[test]
fn the_arrows_part_ways_from_a_selected_disabled_tab() {
    block_on(async {
        e2e::browser::at_once(
            [(keyboard::ARROW_LEFT, 0), (keyboard::ARROW_RIGHT, 2)],
            async |(key, expected)| {
                let fixture = Fixture::open("/tabs-disabled-selected", Viewport::Desktop)
                    .await
                    .unwrap();
                let page = &fixture.page;
                reset_tab_position(page).await.unwrap();
                keyboard::press(page, keyboard::TAB).await.unwrap();
                keyboard::press(page, key).await.unwrap();
                let selected = format!(
                    "document.querySelector('[role=tablist] > [role=tab]:nth-child({})')\
                 .getAttribute('aria-selected') === 'true'",
                    expected + 1
                );
                wait::for_js_true(page, &selected, "the arrow to select its neighbour")
                    .await
                    .unwrap_or_else(|e| panic!("{key:?} did not select tab {expected}: {e}"));
                fixture
                    .console
                    .assert_clean("arrowing off a disabled tab")
                    .unwrap();
                fixture.close().await.unwrap();
            },
        )
        .await;
    });
}

async fn settle(fixture: &Fixture) {
    fixture
        .page
        .evaluate("new Promise(r => setTimeout(() => r(1), 100))")
        .await
        .unwrap();
}

/// The focused tab's index and which tab is selected, e.g. `"0:0"`.
async fn focus_and_selection(fixture: &Fixture) -> String {
    fixture
        .page
        .evaluate(
            "(() => { const tabs = [...document.querySelectorAll('[role=tab]')]; \
             return tabs.indexOf(document.activeElement) + ':' + \
             tabs.findIndex(t => t.getAttribute('aria-selected') === 'true'); })()",
        )
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

/// A click focuses a disabled tab without selecting it; the arrows then step
/// from that tab, not from the selected one.
#[test]
fn the_arrows_step_from_the_focused_tab() {
    block_on(async {
        let fixture = Fixture::open("/tabs-disabled", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        pointer::click(page, "[role=tab]:nth-child(2)")
            .await
            .unwrap();
        settle(&fixture).await;
        assert_eq!(
            focus_and_selection(&fixture).await,
            "1:0",
            "after the click"
        );

        keyboard::press(page, keyboard::ARROW_LEFT).await.unwrap();
        settle(&fixture).await;
        assert_eq!(
            focus_and_selection(&fixture).await,
            "0:0",
            "after ArrowLeft"
        );

        fixture
            .console
            .assert_clean("arrowing off a focused disabled tab")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 502: in manual mode the arrows move only the focus, skipping the
/// disabled tab; Enter and Space select the focused tab.
#[test]
fn manual_activation_selects_on_enter_and_space() {
    block_on(async {
        let fixture = Fixture::open("/tabs-manual", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        reset_tab_position(page).await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        for (key, expected) in [
            (keyboard::ARROW_RIGHT, "2:0"),
            (keyboard::ENTER, "2:2"),
            (keyboard::HOME, "0:2"),
            (keyboard::END, "2:2"),
            (keyboard::ARROW_RIGHT, "0:2"),
            (keyboard::SPACE, "0:0"),
        ] {
            keyboard::press(page, key).await.unwrap();
            settle(&fixture).await;
            assert_eq!(
                focus_and_selection(&fixture).await,
                expected,
                "after {key:?}"
            );
        }
        let panel: String = page
            .evaluate("document.querySelector('[role=tabpanel]').textContent")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(panel, "panel for Account");

        fixture
            .console
            .assert_clean("manual tab activation")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Alt+Left is the browser's Back; a chord is not the strip's to take.
#[test]
fn shortcut_chords_pass_through() {
    block_on(async {
        let fixture = Fixture::open("/tabs", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        page.evaluate(
            "window.__prevented = []; \
             window.addEventListener('keydown', e => window.__prevented.push(e.defaultPrevented))",
        )
        .await
        .unwrap();
        reset_tab_position(page).await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        for modifier in [keyboard::ALT, keyboard::CTRL, keyboard::META] {
            keyboard::press_with(page, keyboard::ARROW_RIGHT, modifier)
                .await
                .unwrap();
        }
        settle(&fixture).await;
        assert_eq!(focus_and_selection(&fixture).await, "0:0");
        let prevented: String = page
            .evaluate("window.__prevented.filter(Boolean).length + ''")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(prevented, "0", "a chord's default was prevented");

        fixture.console.assert_clean("chords on a tab").unwrap();
        fixture.close().await.unwrap();
    });
}

/// WCAG 1.4.10: a strip wider than the phone scrolls inside itself, so the
/// page does not, and End brings the last tab into view.
#[test]
fn a_crowded_strip_scrolls_inside_itself() {
    block_on(async {
        let fixture = Fixture::open("/tabs-crowded", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        let (width, _) = Viewport::Mobile.size();
        let page_width: i64 = page
            .evaluate("document.documentElement.scrollWidth")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            page_width <= width,
            "the page is {page_width}px wide at {width}px"
        );

        reset_tab_position(page).await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        keyboard::press(page, keyboard::END).await.unwrap();
        settle(&fixture).await;
        let visible: bool = page
            .evaluate(
                "(() => { const list = document.querySelector('[role=tablist]').getBoundingClientRect(); \
                 const last = document.querySelector('[role=tab]:last-child').getBoundingClientRect(); \
                 return last.left >= list.left - 1 && last.right <= list.right + 1; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(visible, "End left the last tab outside the strip");

        fixture.console.assert_clean("a crowded strip").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Whether `root`'s selected tab lies inside its strip.
fn selected_shown(root: &str) -> String {
    format!(
        "(() => {{ const list = document.querySelector('{root} > [role=tablist]').getBoundingClientRect(); \
         const tab = document.querySelector('{root} {SELECTED}').getBoundingClientRect(); \
         return tab.left >= list.left - 1 && tab.right <= list.right + 1; }})()"
    )
}

/// Todo 1595: at 320px a selected tab past the strip's edge scrolls into view, when
/// selected from the start and when the value arrives later from outside the strip.
#[test]
fn a_selected_tab_scrolls_into_its_strip() {
    block_on(async {
        let fixture = Fixture::open("/tabs-late", Viewport::Mobile).await.unwrap();
        let page = &fixture.page;
        page.execute(SetDeviceMetricsOverrideParams::new(320, 800, 1.0, true))
            .await
            .unwrap();
        // Mounted at 320px: the reveal follows the value, not a later resize.
        page.reload().await.unwrap();
        wait::for_visible(page, "#late-button").await.unwrap();
        wait::for_js_true(
            page,
            &selected_shown("#initial"),
            "the initial last tab in view",
        )
        .await
        .unwrap();
        pointer::click(page, "#late-button").await.unwrap();
        wait::for_js_true(page, &selected_shown("#late"), "the late last tab in view")
            .await
            .unwrap();
        let page_width: f64 = page
            .evaluate("document.documentElement.scrollWidth")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(page_width <= 320.0, "the page is {page_width}px wide");
        fixture.console.assert_clean("late tabs").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 501: the caller's name reaches the tablist, by label and by heading.
#[test]
fn the_tablist_takes_the_callers_name() {
    block_on(async {
        let fixture = Fixture::open("/tabs-named", Viewport::Desktop)
            .await
            .unwrap();
        for (root, name) in [("#labelled", "Settings"), ("#labelledby", "Profile")] {
            let tree = e2e::ax::snapshot(&fixture.page, &format!("{root} > [role=tablist]"))
                .await
                .unwrap();
            assert!(tree.starts_with(&format!("tablist \"{name}\"")), "{tree}");
        }
        fixture.console.assert_clean("named tab strips").unwrap();
        fixture.close().await.unwrap();
    });
}
