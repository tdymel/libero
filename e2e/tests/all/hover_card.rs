//! `HoverCard`: a non-modal dialog on hover delays or at once on focus. The card is
//! portaled, so Tab is carried into it and back out past the trigger (406).

use anyhow::Result;
use e2e::browser::block_on;
use e2e::clock::HELD_CLOCK;
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

const TRIGGER: &str = "#trigger";
const CARD: &str = "[role=dialog]";
const CARD_FIRST: &str = "#card-first";
const CARD_LAST: &str = "#card-last";
const BEFORE: &str = "#before";
const AFTER: &str = "#after";
/// The `/hover-card-color-field` card and its read-only field (todo 446).
const COLOR_CARD: &str = "#card";
const FIELD: &str = "#card input";
/// The fixture's delays: values nothing else on the page schedules, so the
/// held clock takes these two timers and no other.
const OPEN_MS: u32 = 707;
const CLOSE_MS: u32 = 808;
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

async fn js<T: serde::de::DeserializeOwned>(page: &chromiumoxide::Page, expression: &str) -> T {
    page.evaluate(expression)
        .await
        .unwrap_or_else(|e| panic!("evaluate {expression}: {e}"))
        .into_value()
        .unwrap_or_else(|e| panic!("read {expression}: {e}"))
}

async fn armed(page: &chromiumoxide::Page, ms: u32, count: usize, what: &str) {
    wait::for_js_true(
        page,
        &format!("window.__heldClock.armed({ms}) === {count}"),
        what,
    )
    .await
    .unwrap();
}

async fn fire(page: &chromiumoxide::Page, ms: u32) {
    let fired: usize = js(page, &format!("window.__heldClock.fire({ms})")).await;
    assert_eq!(fired, 1, "fired {fired} timers of {ms}ms");
}

/// Waits for focus to land, then names where it is if it did not.
async fn assert_focused(page: &chromiumoxide::Page, selector: &str, during: &str) {
    let _ = wait::for_js_true(
        page,
        &format!(
            "document.activeElement === document.querySelector({})",
            serde_json::to_string(selector).unwrap()
        ),
        during,
    )
    .await;
    focus::assert_focused(page, selector, during).await.unwrap();
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
        let _: bool = js(
            page,
            &format!("(({HELD_CLOCK})([{OPEN_MS}, {CLOSE_MS}]), true)"),
        )
        .await;
        pointer::move_to(page, AWAY).await.unwrap();

        pointer::hover(page, TRIGGER).await.unwrap();
        armed(page, OPEN_MS, 1, "hovering to arm the open delay").await;
        // The control on the clock: were the delay not held, the real one
        // would open the card while this sleeps.
        tokio::time::sleep(std::time::Duration::from_millis(u64::from(OPEN_MS) + 300)).await;
        assert!(
            !wait::is_visible(page, CARD).await.unwrap(),
            "the card opened before its held open delay fired"
        );
        fire(page, OPEN_MS).await;
        wait::for_visible(page, CARD).await.unwrap();

        // Into the card: the trigger's leave arms the close, the card's enter
        // drops it.
        pointer::hover(page, CARD).await.unwrap();
        armed(page, CLOSE_MS, 0, "entering the card to cancel the close").await;
        assert!(wait::is_visible(page, CARD).await.unwrap());

        pointer::move_to(page, AWAY).await.unwrap();
        armed(page, CLOSE_MS, 1, "leaving the card to arm the close delay").await;
        assert!(
            wait::is_visible(page, CARD).await.unwrap(),
            "the card closed before its close delay fired"
        );
        fire(page, CLOSE_MS).await;
        wait::for_hidden(page, CARD).await.unwrap();

        fixture
            .console
            .assert_clean("hovering a hover card")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Tab from the trigger enters the card and leaves it for what follows the trigger;
/// Shift+Tab from its first control goes back to the trigger.
#[test]
fn tab_crosses_into_the_portaled_card_and_back() {
    block_on(async {
        for viewport in Viewport::ALL {
            let at = viewport.name();
            let fixture = Fixture::open("/hover-card", viewport).await.unwrap();
            let page = &fixture.page;

            keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
            wait::for_visible(page, CARD)
                .await
                .unwrap_or_else(|e| panic!("at {at}, focus to open the card: {e}"));

            keyboard::press(page, keyboard::TAB).await.unwrap();
            assert_focused(page, CARD_FIRST, "Tab from the trigger into the card").await;
            keyboard::press(page, keyboard::TAB).await.unwrap();
            assert_focused(page, CARD_LAST, "Tab within the card").await;
            keyboard::press(page, keyboard::TAB).await.unwrap();
            assert_focused(
                page,
                AFTER,
                "Tab out of the card to what follows the trigger",
            )
            .await;
            wait::for_hidden(page, CARD).await.unwrap();

            // Backwards: the browser's own Shift+Tab reaches the trigger,
            // which opens the card again; Tab in, then Shift+Tab back out.
            keyboard::press_shift(page, keyboard::TAB).await.unwrap();
            assert_focused(page, TRIGGER, "Shift+Tab back to the trigger").await;
            wait::for_visible(page, CARD).await.unwrap();
            keyboard::press(page, keyboard::TAB).await.unwrap();
            assert_focused(page, CARD_FIRST, "Tab into the card again").await;
            keyboard::press_shift(page, keyboard::TAB).await.unwrap();
            assert_focused(page, TRIGGER, "Shift+Tab from the card's first control").await;
            assert!(wait::is_visible(page, CARD).await.unwrap(), "at {at}");

            keyboard::press_shift(page, keyboard::TAB).await.unwrap();
            assert_focused(page, BEFORE, "Shift+Tab past the trigger").await;
            wait::for_hidden(page, CARD).await.unwrap();

            fixture
                .console
                .assert_clean(&format!("tabbing through a hover card at {at}"))
                .unwrap();
            fixture.close().await.unwrap();
        }
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
        assert_focused(page, CARD_FIRST, "Tab into the card").await;

        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_hidden(page, CARD).await.unwrap();
        assert_focused(page, TRIGGER, "Escape inside the hover card").await;
        // A reopen would come from the trigger's `focusin` a render later.
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        assert!(
            !wait::is_visible(page, CARD).await.unwrap(),
            "focus returning to the trigger opened the card again"
        );

        // Focus leaves and comes back: the card opens again on the trigger,
        // and Escape there closes it with focus left in place.
        keyboard::press(page, keyboard::TAB).await.unwrap();
        assert_focused(page, AFTER, "Tab past the closed card").await;
        keyboard::press_shift(page, keyboard::TAB).await.unwrap();
        assert_focused(page, TRIGGER, "Shift+Tab back to the trigger").await;
        wait::for_visible(page, CARD).await.unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_hidden(page, CARD).await.unwrap();
        assert_focused(page, TRIGGER, "Escape on the trigger").await;

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
        assert_focused(page, AFTER, "Tab past the closed card").await;
        keyboard::press_shift(page, keyboard::TAB).await.unwrap();
        assert_focused(page, TRIGGER, "Shift+Tab back to the trigger").await;
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

/// Todo 523: a plain-text trigger takes no focus, so no keyboard opens the
/// card. A debug build says so; a focusable trigger stays quiet.
#[test]
fn a_trigger_with_nothing_focusable_warns() {
    const WARNING: &str = "holds nothing focusable";
    block_on(async {
        let fixture = Fixture::open("/hover-card-text", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_js_true(
            &fixture.page,
            "new Promise((r) => setTimeout(() => r(true), 300))",
            "the mount effect to run",
        )
        .await
        .unwrap();
        let messages = fixture.console.drain();
        assert!(
            messages.iter().any(|message| message.contains(WARNING)),
            "no warning: {messages:?}"
        );
        fixture.close().await.unwrap();

        let fixture = Fixture::open("/hover-card", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_js_true(
            &fixture.page,
            "new Promise((r) => setTimeout(() => r(true), 300))",
            "the mount effect to run",
        )
        .await
        .unwrap();
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
