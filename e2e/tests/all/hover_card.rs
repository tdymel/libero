//! `HoverCard`: a non-modal dialog on hover delays or at once on focus. The card is
//! portaled, so Tab is carried into it and back out past the trigger (406).

use anyhow::{Result, bail, ensure};
use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::clock;
use e2e::driver::{Driver, eventually, eventually_focused};
use e2e::passes::{focus, keyboard, pointer};
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, wait};

async fn shown<D: Driver>(d: &mut D, selector: &str, open: bool, after: &str) -> Result<()> {
    let what = format!(
        "{selector} {} after {after}",
        if open { "open" } else { "closed" }
    );
    // Open means placed: a card still hidden takes no focus (1006, the WebView
    // places it a round trip later).
    eventually(d, &what, async |d| {
        let exists = d.exists(selector).await?;
        Ok(match open {
            true => exists && d.style(selector, "visibility").await? == "visible",
            false => !exists,
        })
    })
    .await
}

async fn escape_on_the_trigger<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#trigger").await?;
    d.hover("#trigger").await?;
    shown(d, "[role=dialog]", true, "focus and hover").await?;
    d.press(keyboard::ESCAPE).await?;
    shown(d, "[role=dialog]", false, "Escape").await
}

async fn escape_with_focus_elsewhere<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#before").await?;
    d.hover("#trigger").await?;
    shown(d, "[role=dialog]", true, "hover").await?;
    d.press(keyboard::ESCAPE).await?;
    shown(d, "[role=dialog]", false, "Escape").await?;
    eventually_focused(d, "#before", "Escape").await
}

async fn escape_closes_the_list_first<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.hover("#trigger").await?;
    shown(d, "[role=dialog]", true, "hover").await?;
    d.focus("[role=dialog] [role=combobox]").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    shown(d, "[role=listbox]", true, "ArrowDown").await?;
    d.press(keyboard::ESCAPE).await?;
    shown(d, "[role=listbox]", false, "Escape").await?;
    assert!(
        d.exists("[role=dialog]").await?,
        "Escape closed the card with the list"
    );
    d.press(keyboard::ESCAPE).await?;
    shown(d, "[role=dialog]", false, "the second Escape").await
}

e2e::scenario!(
    escape_on_the_trigger_closes_a_pointer_opened_card,
    "/hover-card",
    escape_on_the_trigger
);
e2e::scenario!(
    escape_closes_a_pointer_opened_card_with_focus_elsewhere,
    "/hover-card",
    escape_with_focus_elsewhere
);
e2e::scenario!(
    escape_closes_a_select_list_in_the_card_before_the_card,
    "/hover-card-select",
    escape_closes_the_list_first
);

/// Tab crosses into the portaled card and out past its last control; Shift+Tab walks back.
async fn tab_crosses<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(BEFORE).await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, TRIGGER, "Tab from Before").await?;
    shown(d, CARD, true, "Tab onto the trigger").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, CARD_FIRST, "Tab from the trigger into the card").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, CARD_LAST, "Tab within the card").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, AFTER, "Tab out of the card").await?;
    shown(d, CARD, false, "Tab out of the card").await?;
    d.press_shift(keyboard::TAB).await?;
    eventually_focused(d, TRIGGER, "Shift+Tab back to the trigger").await?;
    shown(d, CARD, true, "Shift+Tab back to the trigger").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, CARD_FIRST, "Tab into the card again").await?;
    d.press_shift(keyboard::TAB).await?;
    eventually_focused(d, TRIGGER, "Shift+Tab from the card's first control").await?;
    ensure!(d.exists(CARD).await?, "the card closed on Shift+Tab");
    Ok(())
}

/// A two-button trigger: Tab walks both before the card, and leaves past it (1614).
async fn tab_crosses_a_pair<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(BEFORE).await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, TRIGGER, "Tab from Before").await?;
    shown(d, CARD, true, "Tab onto the trigger").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, TRIGGER_SECOND, "Tab from the trigger's first button").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, CARD_FIRST, "Tab from the trigger's last button").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, CARD_LAST, "Tab within the card").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, AFTER, "Tab out of the card").await?;
    d.press_shift(keyboard::TAB).await?;
    eventually_focused(d, TRIGGER_SECOND, "Shift+Tab back to the trigger").await?;
    shown(d, CARD, true, "Shift+Tab back to the trigger").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, CARD_FIRST, "Tab into the card again").await?;
    d.press_shift(keyboard::TAB).await?;
    eventually_focused(d, TRIGGER_SECOND, "Shift+Tab from the card's first control").await
}

/// A hidden last control on both sides ignores `focus()`: Tab still enters and leaves the card (2688).
async fn tab_crosses_past_hidden_last_controls<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(BEFORE).await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, TRIGGER, "Tab from Before").await?;
    shown(d, CARD, true, "Tab onto the trigger").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(
        d,
        CARD_FIRST,
        "Tab past the trigger's hidden control into the card",
    )
    .await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, CARD_LAST, "Tab within the card").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, AFTER, "Tab out past the card's hidden control").await?;
    d.press_shift(keyboard::TAB).await?;
    eventually_focused(d, TRIGGER, "Shift+Tab back to the trigger").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, CARD_FIRST, "Tab into the card again").await?;
    d.press_shift(keyboard::TAB).await?;
    eventually_focused(d, TRIGGER, "Shift+Tab from the card's first control").await
}

/// A positive `tabindex` leads the card's Tab order, whatever its place in the DOM (2696).
async fn tab_follows_positive_tabindex<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(BEFORE).await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, TRIGGER, "Tab from Before").await?;
    shown(d, CARD, true, "Tab onto the trigger").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, CARD_FIRST, "Tab into the card, tabindex 1").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, CARD_MIDDLE, "Tab to tabindex 2").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, CARD_LAST, "Tab to the control without a tabindex").await?;
    d.press_shift(keyboard::TAB).await?;
    eventually_focused(d, CARD_MIDDLE, "Shift+Tab back to tabindex 2").await?;
    d.press_shift(keyboard::TAB).await?;
    eventually_focused(d, CARD_FIRST, "Shift+Tab back to tabindex 1").await?;
    d.press_shift(keyboard::TAB).await?;
    eventually_focused(d, TRIGGER, "Shift+Tab from the first in Tab order").await
}

/// The pointer rests to open, may cross into the card, and leaving closes it.
async fn pointer_crosses<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.hover(TRIGGER).await?;
    shown(d, CARD, true, "hovering the trigger").await?;
    d.hover(CARD_FIRST).await?;
    // The trigger's leave armed the close, the card's enter must have cancelled it.
    eventually(d, "the pending close to clear", async |d| {
        Ok(d.attr(CARD, "data-closing").await?.is_none())
    })
    .await?;
    ensure!(
        d.exists(CARD).await?,
        "the card closed with the pointer on it"
    );
    d.hover(BEFORE).await?;
    // The mark the wait above relies on, seen while the close counts down.
    eventually(d, "the pointer leaving to arm the close", async |d| {
        Ok(d.attr(CARD, "data-closing").await?.as_deref() == Some("true"))
    })
    .await?;
    shown(d, CARD, false, "the pointer leaving").await
}

/// As `Tooltip`: a long press opens the card, the release lets it close; a tap is no
/// hover, so its compatibility `mouseenter` arms no open (2448).
async fn long_press_opens<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    ensure!(!d.exists(CARD).await?, "open at rest");
    d.long_press(TRIGGER, 800).await?;
    shown(d, CARD, true, "a long press").await?;
    shown(d, CARD, false, "the release").await?;
    // Held only now: the long press above needs the real clock (1662).
    if !d.hold_timers(&[LONG_PRESS_MS, OPEN_MS]).await? {
        return Ok(());
    }
    d.touch_down(TRIGGER).await?;
    eventually(d, "a held touch to arm the open", async |d| {
        Ok(d.armed(LONG_PRESS_MS).await? == 1)
    })
    .await?;
    d.touch_up(TRIGGER).await?;
    eventually(d, "the release to cancel the open", async |d| {
        Ok(d.armed(LONG_PRESS_MS).await? == 0)
    })
    .await?;
    // Past the tap's compatibility mouse events.
    d.settle().await?;
    ensure!(
        d.armed(OPEN_MS).await? == 0,
        "a tap armed the hover open delay"
    );
    ensure!(!d.exists(CARD).await?, "a tap opened the card");
    Ok(())
}

e2e::scenario!(
    a_long_press_opens_it_on_touch_and_a_tap_does_not,
    "/hover-card",
    long_press_opens,
    native: skip("996: Blitz has no touch input"),
    desktop: skip("1126: no touch input under Xvfb")
);
e2e::scenario!(
    tab_crosses_into_the_portaled_card_and_back_on_every_backend,
    "/hover-card",
    tab_crosses,
    android: skip("958: the Tab bridge reads the card's focusables inside a handler"),
    desktop: skip("958: the Tab bridge reads the card's focusables inside a handler")
);
e2e::scenario!(
    tab_enters_the_card_only_from_the_triggers_last_focusable,
    "/hover-card-pair",
    tab_crosses_a_pair,
    android: skip("958: the Tab bridge reads the card's focusables inside a handler"),
    desktop: skip("958: the Tab bridge reads the card's focusables inside a handler")
);
e2e::scenario!(
    tab_crosses_past_a_hidden_last_control_on_the_trigger_and_the_card,
    "/hover-card-hidden",
    tab_crosses_past_hidden_last_controls,
    android: skip("958: the Tab bridge reads the card's focusables inside a handler"),
    desktop: skip("958: the Tab bridge reads the card's focusables inside a handler")
);
e2e::scenario!(
    tab_walks_the_card_in_positive_tabindex_order,
    "/hover-card-tabindex",
    tab_follows_positive_tabindex,
    android: skip("958: the Tab bridge reads the card's focusables inside a handler"),
    desktop: skip("958: the Tab bridge reads the card's focusables inside a handler")
);
e2e::scenario!(
    the_pointer_crosses_into_the_card_and_leaving_closes_it,
    "/hover-card",
    pointer_crosses
);

const TRIGGER: &str = "#trigger";
/// `/hover-card-pair`'s second trigger button.
const TRIGGER_SECOND: &str = "#trigger-second";
const CARD: &str = "[role=dialog]";
const CARD_FIRST: &str = "#card-first";
const CARD_LAST: &str = "#card-last";
/// `/hover-card-tabindex`'s `tabindex` 2 button, DOM-first and Tab-second.
const CARD_MIDDLE: &str = "#card-middle";
const BEFORE: &str = "#before";
const AFTER: &str = "#after";
/// The `/hover-card-color-field` card and its read-only field (todo 446).
const COLOR_CARD: &str = "#card";
const FIELD: &str = "#card input";
/// The fixture's delays: values nothing else on the page schedules, so the
/// held clock takes these two timers and no other.
const OPEN_MS: u32 = 707;
const CLOSE_MS: u32 = 808;
/// `Tooltip`'s long press, which `HoverCard` shares.
const LONG_PRESS_MS: u32 = 500;
/// Well outside the card and the trigger, inside the fixture's padding.
const AWAY: pointer::Point = pointer::Point { x: 2.0, y: 2.0 };

/// No `focusable(TRIGGER)`: the ring pass runs at rest and leaves focus on the
/// trigger, which opens the card before the "open" state is reached.
#[test]
fn it_meets_the_baseline() {
    Suite::new("hover_card", "/hover-card")
        .state("open", &[Step::TabTo(TRIGGER)], CARD)
        .run();
}

/// Hover waits `open_delay` and leaving waits `close_delay`, each on a timer
/// the test holds, and crossing into the card cancels the pending close.
#[test]
fn the_pointer_opens_and_closes_it_on_its_delays() {
    block_on(async {
        let fixture = Fixture::open("/hover-card", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        clock::hold(page, &[OPEN_MS, CLOSE_MS]).await.unwrap();
        pointer::move_to(page, AWAY).await.unwrap();

        pointer::hover(page, TRIGGER).await.unwrap();
        clock::until_armed(page, OPEN_MS, 1, "hovering to arm the open delay")
            .await
            .unwrap();
        // The open waits on the held timer, not on the hover's own render (1634).
        e2e::clock::settle(page).await.unwrap();
        assert!(
            !wait::exists(page, CARD).await.unwrap(),
            "the card opened before its held open delay fired"
        );
        clock::fire(page, OPEN_MS).await.unwrap();
        wait::for_visible(page, CARD).await.unwrap();

        // Into the card: the trigger's leave arms the close, the card's enter
        // drops it.
        pointer::hover(page, CARD).await.unwrap();
        clock::until_armed(page, CLOSE_MS, 0, "entering the card to cancel the close")
            .await
            .unwrap();
        assert!(wait::is_visible(page, CARD).await.unwrap());

        pointer::move_to(page, AWAY).await.unwrap();
        clock::until_armed(page, CLOSE_MS, 1, "leaving the card to arm the close delay")
            .await
            .unwrap();
        assert!(
            wait::is_visible(page, CARD).await.unwrap(),
            "the card closed before its close delay fired"
        );
        clock::fire(page, CLOSE_MS).await.unwrap();
        wait::for_hidden(page, CARD).await.unwrap();

        fixture
            .console
            .assert_clean("hovering a hover card")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

const TEXT_STOP: &str = "[role=dialog] > [tabindex='0']";

/// Opens the scrolling card by keyboard and Tabs into its text stop.
async fn tab_into_the_text_stop<D: Driver>(d: &mut D) -> Result<()> {
    d.focus(BEFORE).await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, TRIGGER, "Tab from Before").await?;
    shown(d, CARD, true, "Tab onto the trigger").await?;
    eventually(d, "the overflowing card to become a tab stop", async |d| {
        d.exists(TEXT_STOP).await
    })
    .await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, TEXT_STOP, "Tab from the trigger into the card").await
}

async fn a_scrolling_card_is_a_tab_stop<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    tab_into_the_text_stop(d).await?;
    // Todo 2643: the stop is the scroller, so its ring is closed at top and bottom.
    let (stop, card) = (d.rect(TEXT_STOP).await?, d.rect(CARD).await?);
    if stop.height > card.height {
        bail!("the stop is taller than the card: {stop:?} in {card:?}");
    }
    // Todo 2644: Blitz scrolls no focused element by key, so the card does it.
    const FIRST_LINE: &str = "[role=dialog] > [tabindex='0'] > :first-child";
    let top = d.rect(FIRST_LINE).await?.y;
    d.press(keyboard::ARROW_DOWN).await?;
    eventually(d, "ArrowDown to scroll the text", async |d| {
        Ok(d.rect(FIRST_LINE).await?.y < top)
    })
    .await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, AFTER, "Tab out of the card").await
}

// Todo 2445: a text card that scrolls is a tab stop the arrows scroll.
e2e::scenario!(
    a_scrolling_text_card_takes_a_tab_stop_that_scrolls_it,
    "/hover-card-scroll",
    a_scrolling_card_is_a_tab_stop,
    android: skip("958: the Tab bridge reads the card's focusables inside a handler"),
    desktop: skip("958: the Tab bridge reads the card's focusables inside a handler")
);

/// Hovers the trigger of the scrolling card and waits for its text stop.
async fn open_scrolling_card(page: &Page) {
    pointer::hover(page, TRIGGER).await.unwrap();
    wait::for_js_true(
        page,
        &format!("!!document.querySelector(\"{TEXT_STOP}\")"),
        "the overflowing card to become a tab stop",
    )
    .await
    .unwrap();
}

/// Todo 2640: the text tab stop is a region named as the card.
#[test]
fn the_text_stop_is_a_region_named_as_the_card() {
    block_on(async {
        let fixture = Fixture::open("/hover-card-scroll", Viewport::Desktop)
            .await
            .unwrap();
        open_scrolling_card(&fixture.page).await;
        let stop = format!("document.querySelector(\"{TEXT_STOP}\")");
        wait::for_js_true(
            &fixture.page,
            &format!(
                "{stop}.getAttribute('role') === 'region' \
                 && {stop}.getAttribute('aria-label') === 'Ada Lovelace'"
            ),
            "the text stop to be a region named Ada Lovelace",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 2639: a press on the text stop is the pointer's, so the card closes once it leaves.
#[test]
fn a_pointer_press_on_the_text_stop_does_not_hold_the_card_open() {
    block_on(async {
        let fixture = Fixture::open("/hover-card-scroll", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        open_scrolling_card(page).await;
        pointer::click(page, CARD).await.unwrap();
        focus::wait_for_focus(page, TEXT_STOP, "the press to focus the text stop")
            .await
            .unwrap();
        pointer::move_to(page, AWAY).await.unwrap();
        wait::for_hidden(page, CARD)
            .await
            .unwrap_or_else(|e| panic!("a pressed text stop held the card open: {e}"));
        fixture.close().await.unwrap();
    });
}

#[derive(serde::Deserialize)]
struct StopRing {
    padding: f64,
    inside_card: bool,
    clipped_x: bool,
    outline_alpha: f64,
    stripe: String,
    card_fill: String,
}

/// Todo 2631: the text stop's ring stays inside the card, clear of the text, at 200% text,
/// in the dark scheme and in forced colours.
#[test]
fn the_text_stop_ring_shows_at_200_percent_text_dark_and_in_forced_colours() {
    const PROBE: &str = "(() => { const stop = document.querySelector(\"[role=dialog] > [tabindex='0']\"); \
        const card = stop.parentElement, style = getComputedStyle(stop); \
        const a = stop.getBoundingClientRect(), b = card.getBoundingClientRect(); \
        const colour = (style.boxShadow.match(/rgba?\\([^)]*\\)/g) || [])[4] || ''; \
        const outline = (style.outlineColor.match(/[\\d.]+/g) || []).map(Number); \
        return { padding: parseFloat(style.paddingLeft), \
          inside_card: a.left >= b.left && a.right <= b.right && b.bottom <= innerHeight, \
          clipped_x: stop.scrollWidth > stop.clientWidth + 1, \
          outline_alpha: outline.length > 3 ? outline[3] : 1, stripe: colour, \
          card_fill: getComputedStyle(card).backgroundColor }; })()";
    block_on(async {
        for (dark, forced) in [(false, false), (true, false), (false, true)] {
            let what = format!("dark {dark}, forced {forced}");
            let fixture = Fixture::open("/hover-card-scroll", Viewport::Desktop)
                .await
                .unwrap();
            let page = &fixture.page;
            page.evaluate("document.documentElement.style.fontSize = '200%'")
                .await
                .unwrap();
            if dark {
                e2e::browser::emulate_media(page, e2e::browser::Scheme::Dark, None)
                    .await
                    .unwrap();
            }
            if forced {
                e2e::browser::force_colours(page).await.unwrap();
            }
            keyboard::tab_to(page, "#trigger", 5).await.unwrap();
            wait::for_visible(page, CARD).await.unwrap();
            wait::for_js_true(
                page,
                &format!("!!document.querySelector(\"{TEXT_STOP}\")"),
                "the overflowing text card to become a tab stop",
            )
            .await
            .unwrap();
            keyboard::press(page, keyboard::TAB).await.unwrap();
            focus::wait_for_focus(page, TEXT_STOP, "Tab into the text stop")
                .await
                .unwrap();

            let ring: StopRing = page.evaluate(PROBE).await.unwrap().into_value().unwrap();
            assert!(ring.padding >= 2.0, "{what}: text under the ring band");
            assert!(
                ring.inside_card,
                "{what}: the stop left the card or viewport"
            );
            assert!(!ring.clipped_x, "{what}: the text overflows sideways");
            if forced {
                assert!(ring.outline_alpha > 0.0, "{what}: the outline vanished");
            } else {
                assert!(
                    !ring.stripe.is_empty() && ring.stripe != ring.card_fill,
                    "{what}: the stripe {} is the card's fill {}",
                    ring.stripe,
                    ring.card_fill
                );
            }
            fixture.console.assert_clean(&what).unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// Todo 2445: a text card that fits takes no tab stop.
#[test]
fn a_text_card_that_fits_takes_no_tab_stop() {
    block_on(async {
        let fixture = Fixture::open("/hover-card-text", Viewport::Desktop)
            .await
            .unwrap();
        pointer::hover(&fixture.page, "#trigger").await.unwrap();
        wait::for_visible(&fixture.page, "[role=dialog]")
            .await
            .unwrap();
        crate::settle::painted(&fixture.page).await.unwrap();
        assert!(
            !wait::exists(&fixture.page, TEXT_STOP).await.unwrap(),
            "a card that fits took a tab stop"
        );
        // A plain-text trigger warns, by design.
        fixture.console.drain();
        fixture.close().await.unwrap();
    });
}

/// Todo 2447: disabling keeps the trigger's node, so a focused trigger keeps its focus.
#[test]
fn disabling_a_hover_card_keeps_a_focused_trigger() {
    block_on(async {
        let fixture = Fixture::open("/hover-card-toggle", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, "#trigger", 5).await.unwrap();
        page.evaluate("window.__trigger = document.getElementById('trigger')")
            .await
            .unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            "!document.querySelector('[role=dialog]')",
            "the disabled card to leave",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "document.activeElement === window.__trigger && document.getElementById('trigger') === window.__trigger",
            "the trigger to keep its node and focus",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Tab from the trigger enters the card and leaves it for what follows the trigger;
/// Shift+Tab from its first control goes back to the trigger.
#[test]
fn tab_crosses_into_the_portaled_card_and_back() {
    block_on(async {
        e2e::browser::at_every_viewport(async |viewport| {
            let at = viewport.name();
            let fixture = Fixture::open("/hover-card", viewport).await.unwrap();
            let page = &fixture.page;

            keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
            wait::for_visible(page, CARD)
                .await
                .unwrap_or_else(|e| panic!("at {at}, focus to open the card: {e}"));

            keyboard::press(page, keyboard::TAB).await.unwrap();
            focus::wait_for_focus(page, CARD_FIRST, "Tab from the trigger into the card")
                .await
                .unwrap();
            keyboard::press(page, keyboard::TAB).await.unwrap();
            focus::wait_for_focus(page, CARD_LAST, "Tab within the card")
                .await
                .unwrap();
            keyboard::press(page, keyboard::TAB).await.unwrap();
            focus::wait_for_focus(
                page,
                AFTER,
                "Tab out of the card to what follows the trigger",
            )
            .await
            .unwrap();
            wait::for_hidden(page, CARD).await.unwrap();

            // Backwards: the browser's own Shift+Tab reaches the trigger,
            // which opens the card again; Tab in, then Shift+Tab back out.
            keyboard::press_shift(page, keyboard::TAB).await.unwrap();
            focus::wait_for_focus(page, TRIGGER, "Shift+Tab back to the trigger")
                .await
                .unwrap();
            wait::for_visible(page, CARD).await.unwrap();
            keyboard::press(page, keyboard::TAB).await.unwrap();
            focus::wait_for_focus(page, CARD_FIRST, "Tab into the card again")
                .await
                .unwrap();
            keyboard::press_shift(page, keyboard::TAB).await.unwrap();
            focus::wait_for_focus(page, TRIGGER, "Shift+Tab from the card's first control")
                .await
                .unwrap();
            assert!(wait::is_visible(page, CARD).await.unwrap(), "at {at}");

            keyboard::press_shift(page, keyboard::TAB).await.unwrap();
            focus::wait_for_focus(page, BEFORE, "Shift+Tab past the trigger")
                .await
                .unwrap();
            wait::for_hidden(page, CARD).await.unwrap();

            fixture
                .console
                .assert_clean(&format!("tabbing through a hover card at {at}"))
                .unwrap();
            fixture.close().await.unwrap();
        })
        .await;
    });
}

/// Escape in the card or on the trigger closes it; the returning focus does not reopen it.
#[test]
fn escape_closes_it_and_returns_focus_to_the_trigger() {
    block_on(async {
        let fixture = Fixture::open("/hover-card", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
        wait::for_visible(page, CARD).await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        focus::wait_for_focus(page, CARD_FIRST, "Tab into the card")
            .await
            .unwrap();

        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_hidden(page, CARD).await.unwrap();
        focus::wait_for_focus(page, TRIGGER, "Escape inside the hover card")
            .await
            .unwrap();
        // A reopen would come from the trigger's `focusin` a render later.
        e2e::clock::settle(page).await.unwrap();
        assert!(
            !wait::exists(page, CARD).await.unwrap(),
            "focus returning to the trigger opened the card again"
        );

        // Focus leaves and comes back: the card opens again on the trigger,
        // and Escape there closes it with focus left in place.
        keyboard::press(page, keyboard::TAB).await.unwrap();
        focus::wait_for_focus(page, AFTER, "Tab past the closed card")
            .await
            .unwrap();
        keyboard::press_shift(page, keyboard::TAB).await.unwrap();
        focus::wait_for_focus(page, TRIGGER, "Shift+Tab back to the trigger")
            .await
            .unwrap();
        wait::for_visible(page, CARD).await.unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_hidden(page, CARD).await.unwrap();
        focus::wait_for_focus(page, TRIGGER, "Escape on the trigger")
            .await
            .unwrap();

        fixture
            .console
            .assert_clean("escaping a hover card")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// A read-only `ColorField` never shows its dropdown, so it must not sit on
/// the Escape stack: one Escape in it closes the card.
#[test]
fn escape_in_a_read_only_color_field_closes_the_card() {
    block_on(async {
        let fixture = Fixture::open("/hover-card-color-field", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
        wait::for_visible(page, COLOR_CARD).await.unwrap();
        keyboard::tab_to(page, FIELD, 3).await.unwrap();

        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_hidden(page, COLOR_CARD).await.unwrap();

        fixture
            .console
            .assert_clean("Escape in a card's read-only field")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// A press on a trigger that already holds focus fires no `focusin`, so it
/// must not swallow the next keyboard focus: Tab away and back opens the card.
#[test]
fn a_second_click_does_not_swallow_the_next_keyboard_focus() {
    block_on(async {
        let fixture = Fixture::open("/hover-card", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        pointer::click(page, TRIGGER).await.unwrap();
        pointer::click(page, TRIGGER).await.unwrap();
        pointer::move_to(page, AWAY).await.unwrap();
        wait::for_hidden(page, CARD).await.unwrap();

        keyboard::press(page, keyboard::TAB).await.unwrap();
        focus::wait_for_focus(page, AFTER, "Tab past the closed card")
            .await
            .unwrap();
        keyboard::press_shift(page, keyboard::TAB).await.unwrap();
        focus::wait_for_focus(page, TRIGGER, "Shift+Tab back to the trigger")
            .await
            .unwrap();
        wait::for_visible(page, CARD)
            .await
            .unwrap_or_else(|e| panic!("keyboard focus did not open the card: {e}"));
        fixture.close().await.unwrap();
    });
}

/// Todo 449: `disabled` returned before the card's portal slot was told, so an
/// open card stayed on the page.
#[test]
fn disabling_an_open_card_removes_it() {
    block_on(async {
        let fixture = Fixture::open("/hover-card-disable", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, COLOR_CARD).await.unwrap();
        pointer::click(page, "#disable").await.unwrap();
        wait::for_js_true(
            page,
            "!document.querySelector('#card')",
            "the disabled card left the page",
        )
        .await
        .unwrap();
        fixture
            .console
            .assert_clean("disabling an open hover card")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 2446: disabling unmounted the wrapper before the pointer left, so re-enabling with
/// the pointer elsewhere reopened the card.
#[test]
fn re_enabling_with_the_pointer_elsewhere_keeps_the_card_closed() {
    block_on(async {
        let fixture = Fixture::open("/hover-card-toggle", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        pointer::hover(page, TRIGGER).await.unwrap();
        wait::for_visible(page, CARD).await.unwrap();
        pointer::click(page, TRIGGER).await.unwrap();
        wait::for_js_true(
            page,
            "!document.querySelector('[role=dialog]')",
            "the disabled card left the page",
        )
        .await
        .unwrap();
        pointer::click(page, "#enable").await.unwrap();
        // The trigger is back in the wrapper span, then the renders a reopen would take.
        wait::for_js_true(
            page,
            "document.querySelector('#trigger').parentElement.tagName === 'SPAN'",
            "the card enabled again",
        )
        .await
        .unwrap();
        crate::settle::painted(page).await.unwrap();
        assert!(
            !wait::exists(page, CARD).await.unwrap(),
            "the card reopened with no pointer on its trigger"
        );
        fixture
            .console
            .assert_clean("re-enabling a hover card")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 523: a plain-text trigger takes no focus, so no keyboard opens the
/// card. A debug build says so; a focusable trigger stays quiet.
#[test]
fn a_trigger_with_nothing_focusable_warns() {
    const WARNING: &str = "holds nothing focusable";
    block_on(async {
        let fixture = Fixture::open("/hover-card-text", Viewport::Desktop)
            .await
            .unwrap();
        let warned = wait::until("the mount effect's warning", || async {
            Ok(fixture
                .console
                .peek()
                .iter()
                .any(|message| message.contains(WARNING)))
        })
        .await;
        let messages = fixture.console.drain();
        assert!(warned.is_ok(), "no warning: {messages:?}");
        fixture.close().await.unwrap();

        // Nothing to wait for on a quiet page; `close` checks the console again, later.
        let fixture = Fixture::open("/hover-card", Viewport::Desktop)
            .await
            .unwrap();
        e2e::clock::settle(&fixture.page).await.unwrap();
        fixture.console.settle().await.unwrap();
        fixture.console.assert_clean("a button trigger").unwrap();
        fixture.close().await.unwrap();
    });
}

#[derive(serde::Deserialize)]
struct Sides {
    on_start: bool,
    start_aligned: bool,
}

/// Todo 711: `Side::Start` puts the card left of its trigger under LTR, right under RTL;
/// a start-aligned card below lines up the start edges.
#[test]
fn the_start_side_and_align_follow_the_direction() {
    block_on(async {
        for dir in ["ltr", "rtl"] {
            let fixture = crate::rtl_keys::open_in("/hover-card-sides", dir).await;
            let page = &fixture.page;
            pointer::click(page, "#open").await.unwrap();
            wait::for_visible(page, "#start-card").await.unwrap();
            wait::for_visible(page, "#below-card").await.unwrap();

            let expression = format!(
                "(() => {{ const rtl = {rtl}; \
                 const rect = id => document.getElementById(id).getBoundingClientRect(); \
                 const t = rect('start-trigger'), c = rect('start-card'); \
                 const bt = rect('below-trigger'), bc = rect('below-card'); \
                 return {{ on_start: rtl ? c.left >= t.right : c.right <= t.left, \
                   start_aligned: Math.abs(rtl ? bc.right - bt.right : bc.left - bt.left) < 1.5 }}; }})()",
                rtl = dir == "rtl"
            );
            wait::until(&format!("{dir}: the cards placed"), || async {
                let sides: Sides = page.evaluate(expression.as_str()).await?.into_value()?;
                Ok(sides.on_start && sides.start_aligned)
            })
            .await
            .unwrap();

            fixture.console.assert_clean(dir).unwrap();
            fixture.close().await.unwrap();
        }
    });
}
