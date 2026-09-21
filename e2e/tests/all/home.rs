//! The docs home page's booking card: filled in and submitted from the
//! keyboard alone, and the booking lands in its second tab.

use anyhow::Result;
use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually_focused};
use e2e::passes::keyboard;
use e2e::{Fixture, Viewport, ax, wait};

/// Arrow Down enters the picker on the typed day, not on the grid cell the
/// month shown before held.
async fn arrow_down_enters_on_the_typed_day<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("input[role=combobox]").await?;
    d.type_text("October 3, 2026").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    eventually_focused(d, "[data-date='2026-10-03']", "typing a day, ArrowDown").await
}

e2e::scenario!(
    arrow_down_after_typing_enters_on_the_typed_day,
    "/home-booking",
    arrow_down_enters_on_the_typed_day,
    android: skip("958: element identity on the WebView")
);

const TAB_BOOK: &str = "[role=tab][aria-selected=true]";
const NAME: &str = "input[name=name]";
// The day's input is a combobox too.
const GUESTS: &str = "div[role=combobox]";

#[test]
fn the_booking_card_submits_from_the_keyboard() {
    block_on(async {
        let fixture = Fixture::open("/home-booking", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        // An empty submit is refused, and the summary takes focus.
        keyboard::tab_to(page, "button[type=submit]", 12)
            .await
            .unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        expect(
            page,
            "document.activeElement?.closest('[data-slot=summary]') !== null",
            "the error summary to take focus",
        )
        .await
        .unwrap();

        keyboard::tab_to(page, NAME, 12).await.unwrap();
        keyboard::type_text(page, "Linus").await.unwrap();
        // The day's text field is the next stop.
        keyboard::press(page, keyboard::TAB).await.unwrap();
        keyboard::type_text(page, "October 16, 2026").await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();

        keyboard::tab_to(page, GUESTS, 4).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        wait::for_visible(page, "[role=option]").await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        expect(
            page,
            &format!("document.querySelector({GUESTS:?}).textContent.includes('2 guests')"),
            "2 guests picked",
        )
        .await
        .unwrap();

        keyboard::tab_to(page, "input[role=switch]", 2)
            .await
            .unwrap();
        keyboard::press(page, keyboard::SPACE).await.unwrap();
        keyboard::tab_to(page, "button[type=submit]", 2)
            .await
            .unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();

        expect(
            page,
            "document.body.textContent.includes('Booked a table for Linus.')",
            "the booking to be confirmed",
        )
        .await
        .unwrap();
        expect(
            page,
            &format!("document.querySelector({NAME:?}).value === ''"),
            "the form to reset",
        )
        .await
        .unwrap();

        // Back to the tab strip, over to the bookings.
        for _ in 0..8 {
            if focused(page, TAB_BOOK).await.unwrap() {
                break;
            }
            keyboard::press_shift(page, keyboard::TAB).await.unwrap();
        }
        assert!(
            focused(page, TAB_BOOK).await.unwrap(),
            "the tab strip is reachable"
        );
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        expect(
            page,
            "[...document.querySelectorAll('tbody tr')].some(r => r.textContent.includes('Linus') \
             && r.textContent.includes('2026-10-16') && r.textContent.includes('Terrace'))",
            "the new booking in the table",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("booking a table").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The stats section is named by a visually hidden `h2`, not an `aria-label`.
#[test]
fn the_stats_section_is_named_by_a_hidden_h2() {
    block_on(async {
        let fixture = Fixture::open("/home-stats", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_selector(page, "section[aria-labelledby]")
            .await
            .unwrap();

        let facts: Vec<Option<String>> = page
            .evaluate(
                "(() => { const s = document.querySelector('section'); \
                 const h = document.getElementById(s.getAttribute('aria-labelledby')); \
                 const r = h.firstElementChild.getBoundingClientRect(); \
                 return [s.getAttribute('aria-label'), h.tagName + ':' + h.textContent, \
                 String(r.width <= 1 && r.height <= 1)]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            facts,
            [
                None,
                Some("H2:Libero in numbers".into()),
                Some("true".into())
            ]
        );
        fixture.console.assert_clean("the stats section").unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn it_meets_the_baseline() {
    e2e::Suite::new("home_booking", "/home-booking")
        .focusable(NAME)
        // The fields' own target sizes are their fixtures' business.
        .targets("button, [role=tab]")
        .run();
}

/// Todo 1025: each landing code block says what it holds, so its "Copy code" does too.
#[test]
fn the_landing_code_blocks_say_what_they_copy() {
    block_on(async {
        let fixture = Fixture::open("/home-code", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        for (block, label) in [
            ("#install", "Add libero to your project"),
            ("#card-code", "The booking card, Rust code"),
        ] {
            let tree = ax::snapshot(page, block).await.unwrap();
            assert!(tree.starts_with(&format!("group \"{label}\"\n")), "{tree}");
            assert!(tree.contains("button \"Copy code\""), "{tree}");
            let button = format!("{block} button");
            assert_eq!(ax::description(page, &button).await.unwrap(), label);
        }
        fixture.console.assert_clean("the landing code").unwrap();
        fixture.close().await.unwrap();
    });
}

async fn expect(page: &Page, js: &str, what: &str) -> Result<()> {
    wait::for_js_true(page, js, what).await
}

async fn focused(page: &Page, selector: &str) -> Result<bool> {
    Ok(page
        .evaluate(format!(
            "document.activeElement === document.querySelector({selector:?})"
        ))
        .await?
        .into_value()?)
}
