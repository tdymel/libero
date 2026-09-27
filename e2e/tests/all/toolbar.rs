//! `Toolbar`: the `RovingTabindex` archetype over mixed items, plus vertical and RTL bars.

use anyhow::{Result, ensure};
use e2e::archetypes::{Orientation, RovingTabindex};
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused, linger};
use e2e::passes::keyboard;
use e2e::{Fixture, Suite, Viewport, wait};

const ITEMS: &str = "[role=toolbar] [data-toolbar-item]";
const FONT: &str = "[role=toolbar] [data-toolbar-item][aria-label=Font]";

/// Every item in turn, the disabled `Undo` included, then a wrap and both ends.
async fn the_arrows_rove<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#bold").await?;
    for (key, name, to) in [
        (keyboard::ARROW_RIGHT, "ArrowRight", "#italic"),
        (keyboard::ARROW_RIGHT, "ArrowRight", "#left"),
        (keyboard::ARROW_RIGHT, "ArrowRight", "#right"),
        (keyboard::ARROW_RIGHT, "ArrowRight", "#undo"),
        (keyboard::ARROW_RIGHT, "ArrowRight", FONT),
        (keyboard::ARROW_RIGHT, "ArrowRight", "#bold"),
        (keyboard::ARROW_LEFT, "ArrowLeft", FONT),
        (keyboard::ARROW_LEFT, "ArrowLeft", "#undo"),
        (keyboard::HOME, "Home", "#bold"),
        (keyboard::END, "End", FONT),
    ] {
        d.press(key).await?;
        eventually_focused(d, to, name).await?;
    }
    Ok(())
}

/// `Select` keeps its own Home, which opens its list (APG select-only).
async fn an_item_keeps_the_keys_it_takes<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(FONT).await?;
    d.press(keyboard::HOME).await?;
    eventually_focused(d, FONT, "Home on the select").await?;
    eventually(d, "Home to open the list", async |d| {
        Ok(d.attr(FONT, "aria-expanded").await?.as_deref() == Some("true"))
    })
    .await
}

/// Tabbing back in lands on the item focused last, not the first.
async fn the_tab_stop_follows_focus<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#bold").await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    eventually_focused(d, "#italic", "ArrowRight").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, "#after", "Tab").await?;
    d.press_shift(keyboard::TAB).await?;
    eventually_focused(d, "#italic", "Shift+Tab").await?;

    // A click moves the stop too.
    d.click("#right").await?;
    eventually_focused(d, "#right", "a click").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, "#after", "Tab after a click").await?;
    d.press_shift(keyboard::TAB).await?;
    eventually_focused(d, "#right", "Shift+Tab after a click").await
}

/// A disabled item stays focusable but inert. `#right` is the control: its press lands
/// after the disabled ones, so `undo` still at 0 means swallowed, not merely late.
async fn a_disabled_item_stays_inert<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#undo").await?;
    d.focus("#undo").await?;
    eventually_focused(d, "#undo", "focus on the disabled item").await?;
    d.press(keyboard::ENTER).await?;
    d.press(keyboard::SPACE).await?;
    d.click("#right").await?;
    eventually(d, "the control's click", async |d| {
        Ok(d.attr("#presses", "data-right").await?.as_deref() == Some("1"))
    })
    .await?;
    let undo = d.attr("#presses", "data-undo").await?;
    ensure!(
        undo.as_deref() == Some("0"),
        "the disabled item ran its onclick: {undo:?}"
    );
    Ok(())
}

/// Up and Down move, stopping at the ends (`loop_focus: false`); Left and Right belong to the items.
async fn a_vertical_bar_moves_on_up_and_down<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#pen").await?;
    for (key, name, to) in [
        (keyboard::ARROW_UP, "ArrowUp at the top", "#pen"),
        (keyboard::ARROW_DOWN, "ArrowDown", "#eraser"),
        (keyboard::ARROW_DOWN, "ArrowDown", "#fill"),
        (keyboard::ARROW_DOWN, "ArrowDown at the bottom", "#fill"),
        (keyboard::ARROW_UP, "ArrowUp", "#eraser"),
        (keyboard::ARROW_RIGHT, "ArrowRight", "#eraser"),
    ] {
        d.press(key).await?;
        eventually_focused(d, to, name).await?;
    }
    Ok(())
}

/// Under RTL the next item is on the left.
async fn rtl_swaps_the_arrows<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#one").await?;
    d.press(keyboard::ARROW_LEFT).await?;
    eventually_focused(d, "#two", "ArrowLeft").await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    eventually_focused(d, "#one", "ArrowRight").await
}

/// A script's `focus()` on an item makes it the tab stop, as a click does.
async fn script_focus_moves_the_stop<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#jump").await?;
    eventually_focused(d, "#three", "the script's focus").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, "#jump", "Tab").await?;
    d.press_shift(keyboard::TAB).await?;
    eventually_focused(d, "#three", "Shift+Tab").await
}

/// An item mounted later but placed first is first in the arrow order too.
async fn a_late_item_keeps_its_place<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#show").await?;
    eventually(d, "#zero to mount", async |d| d.exists("#zero").await).await?;
    d.focus("#one").await?;
    for (key, name, to) in [
        (keyboard::ARROW_LEFT, "ArrowLeft", "#zero"),
        (keyboard::ARROW_LEFT, "ArrowLeft wrapping", "#three"),
        (keyboard::HOME, "Home", "#zero"),
        (keyboard::ARROW_RIGHT, "ArrowRight", "#one"),
    ] {
        d.press(key).await?;
        eventually_focused(d, to, name).await?;
    }
    Ok(())
}

const SEGMENT: &str = "[role=radiogroup] input:focus";

/// Fields keep their own arrows until an end: the segments' last, the caret's edge.
async fn fields_pass_arrows_on_at_their_ends<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#first").await?;
    for (key, name, to) in [
        (keyboard::ARROW_RIGHT, "ArrowRight", "#bold"),
        (keyboard::ARROW_RIGHT, "ArrowRight", "#wrap"),
        (
            keyboard::ARROW_RIGHT,
            "ArrowRight into the segments",
            "[aria-label=Serif]",
        ),
        (
            keyboard::ARROW_RIGHT,
            "ArrowRight in the segments",
            "[aria-label=Sans]",
        ),
        (
            keyboard::ARROW_RIGHT,
            "ArrowRight in the segments",
            "[aria-label=Mono]",
        ),
        (
            keyboard::ARROW_RIGHT,
            "ArrowRight past the last segment",
            "#find",
        ),
        (keyboard::END, "End in the text", "#find"),
        (
            keyboard::ARROW_RIGHT,
            "ArrowRight at the caret's end",
            "#size",
        ),
        (keyboard::ARROW_UP, "ArrowUp on the stepper", "#size"),
        (keyboard::HOME, "Home in the number", "#size"),
        (
            keyboard::ARROW_LEFT,
            "ArrowLeft at the caret's start",
            "#find",
        ),
        (keyboard::END, "End in the text", "#find"),
        (keyboard::ARROW_LEFT, "ArrowLeft mid-text", "#find"),
        (keyboard::HOME, "Home in the text", "#find"),
        (
            keyboard::ARROW_LEFT,
            "ArrowLeft at the caret's start",
            SEGMENT,
        ),
    ] {
        d.press(key).await?;
        eventually_focused(d, to, name).await?;
    }
    eventually(d, "ArrowUp to step the number", async |d| {
        Ok(d.attr("#size", "aria-valuenow").await?.as_deref() == Some("13"))
    })
    .await
}

/// Alt+F10 in the editor lands on the bar's tab stop; Escape hands focus back (APG).
async fn alt_f10_reaches_the_bar<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#text").await?;
    d.press_alt(keyboard::F10).await?;
    eventually_focused(d, "#bold", "Alt+F10").await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    eventually_focused(d, "#italic", "ArrowRight").await?;
    d.press(keyboard::ESCAPE).await?;
    eventually_focused(d, "#text", "Escape").await?;
    d.press_alt(keyboard::F10).await?;
    eventually_focused(d, "#italic", "Alt+F10 to the last item").await?;
    d.press(keyboard::ESCAPE).await?;
    eventually_focused(d, "#text", "Escape again").await
}

/// Once focus left the bar, Escape back in it stays the page's (todo 1293).
async fn leaving_the_bar_forgets_the_editor<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#text").await?;
    d.press_alt(keyboard::F10).await?;
    eventually_focused(d, "#bold", "Alt+F10").await?;
    d.click("#text").await?;
    eventually_focused(d, "#text", "a click out of the bar").await?;
    d.click("#italic").await?;
    eventually_focused(d, "#italic", "a click back in").await?;
    d.press(keyboard::ESCAPE).await?;
    linger(d, 5).await;
    eventually_focused(d, "#italic", "Escape after leaving").await
}

/// A dialog the bar opened is not leaving it: Escape closes it, then hands back.
async fn a_popup_of_the_bar_keeps_the_editor<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#text").await?;
    d.press_alt(keyboard::F10).await?;
    eventually_focused(d, "#bold", "Alt+F10").await?;
    d.press(keyboard::END).await?;
    eventually_focused(d, "#help", "End").await?;
    d.press(keyboard::ENTER).await?;
    eventually(d, "the help to open", async |d| {
        d.exists("[role=dialog] dl").await
    })
    .await?;
    d.press(keyboard::ESCAPE).await?;
    eventually_focused(d, "#help", "Escape closing the help").await?;
    d.press(keyboard::ESCAPE).await?;
    eventually_focused(d, "#text", "Escape to the editor").await
}

/// The help lists each chord in the platform's key names.
async fn the_help_lists_the_chords<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#help").await?;
    eventually(d, "the help to open", async |d| {
        d.exists("[role=dialog] dl").await
    })
    .await?;
    let title = d.text("[role=dialog] [data-slot=title]").await?;
    ensure!(title == "Keyboard shortcuts", "title: {title:?}");
    let chords = d.text("[role=dialog] dl").await?;
    for want in ["Alt", "F10", "Go to the toolbar", "Ctrl", "Shift", "K"] {
        ensure!(chords.contains(want), "no {want:?} in {chords:?}");
    }
    d.press(keyboard::ESCAPE).await?;
    eventually_focused(d, "#help", "Escape closing the help").await
}

e2e::scenario!(
    alt_f10_moves_focus_to_the_bar_and_escape_back,
    "/toolbar-editor",
    alt_f10_reaches_the_bar
);
e2e::scenario!(
    escape_stays_put_once_focus_left_the_bar,
    "/toolbar-editor",
    leaving_the_bar_forgets_the_editor
);
e2e::scenario!(
    escape_still_hands_back_after_a_dialog_of_the_bar,
    "/toolbar-editor",
    a_popup_of_the_bar_keeps_the_editor
);
e2e::scenario!(
    shortcut_help_lists_the_chords,
    "/toolbar-editor",
    the_help_lists_the_chords
);
e2e::scenario!(
    fields_join_the_arrow_order,
    "/toolbar-fields",
    fields_pass_arrows_on_at_their_ends
);
e2e::scenario!(
    a_script_focus_moves_the_tab_stop,
    "/toolbar-script-focus",
    script_focus_moves_the_stop
);
e2e::scenario!(
    an_item_shown_later_keeps_its_dom_place,
    "/toolbar-script-focus",
    a_late_item_keeps_its_place
);
e2e::scenario!(
    the_arrows_rove_over_every_kind_of_item,
    "/toolbar",
    the_arrows_rove
);
e2e::scenario!(
    a_select_keeps_its_home_key,
    "/toolbar",
    an_item_keeps_the_keys_it_takes
);
e2e::scenario!(
    tabbing_back_in_returns_to_the_last_item,
    "/toolbar",
    the_tab_stop_follows_focus
);
e2e::scenario!(
    a_disabled_item_ignores_clicks_and_keys,
    "/toolbar",
    a_disabled_item_stays_inert
);
e2e::scenario!(
    a_vertical_toolbar_moves_on_up_and_down,
    "/toolbar-vertical",
    a_vertical_bar_moves_on_up_and_down
);
e2e::scenario!(
    a_right_to_left_toolbar_swaps_the_arrows,
    "/toolbar-rtl",
    rtl_swaps_the_arrows
);

/// A separator drawn as a background fill turns `Canvas` under forced colours: invisible.
#[test]
fn the_separator_shows_in_forced_colours() {
    use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
    block_on(async {
        let fixture = Fixture::open("/toolbar", Viewport::Desktop).await.unwrap();
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
        // The page is `Canvas`; a line of the same colour is no line.
        let [fill, edge, width]: [String; 3] = page
            .evaluate(
                "(() => { const probe = document.createElement('div'); \
                 probe.style.background = 'Canvas'; document.body.append(probe); \
                 const canvas = getComputedStyle(probe).backgroundColor; probe.remove(); \
                 const sep = getComputedStyle(document.querySelector('[role=toolbar] [role=separator]')); \
                 const same = (c) => c === canvas ? 'canvas' : 'drawn'; \
                 return [same(sep.backgroundColor), same(sep.borderInlineStartColor), \
                 sep.borderInlineStartWidth]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let line = fill == "drawn" || (edge == "drawn" && width != "0px");
        assert!(
            line,
            "the separator vanishes: fill {fill}, edge {edge} {width}"
        );
        fixture.close().await.unwrap();
    });
}

#[test]
fn it_meets_the_baseline() {
    Suite::new("toolbar", "/toolbar")
        .focusable("#bold")
        // `Select`'s trigger sits inside its taller frame, which takes the press.
        .targets("[role=toolbar] button[data-toolbar-item]")
        .run();
}

#[test]
fn it_honours_the_roving_tabindex_contract() {
    block_on(async {
        for viewport in Viewport::ALL {
            for (route, orientation, wraps) in [
                ("/toolbar", Orientation::Horizontal, true),
                ("/toolbar-vertical", Orientation::Vertical, false),
            ] {
                let fixture = Fixture::open(route, viewport).await.unwrap();

                RovingTabindex {
                    items: ITEMS,
                    orientation,
                    wraps,
                }
                .assert_contract(&fixture.page)
                .await
                .unwrap_or_else(|e| panic!("{route} at {}: {e}", viewport.name()));

                fixture
                    .console
                    .assert_clean(&format!("{route} contract at {}", viewport.name()))
                    .unwrap();
                fixture.close().await.unwrap();
            }
        }
    });
}
