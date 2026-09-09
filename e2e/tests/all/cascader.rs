//! `Cascader`: the keyboard walks levels, and a disabled branch neither takes
//! the cursor nor opens (todo 406).
//!
//! `/cascader` holds three roots: Europe (France: Paris, Lyon; Germany:
//! Berlin), Asia, disabled (Japan: Tokyo), and Oceania (Australia: Sydney).
//! Focus never leaves the trigger; the cursor is its `aria-activedescendant`.

use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, wait};

const TRIGGER: &str = "[role=combobox]";

/// `<cursor row's label>|<open columns>|<cursor row's aria-selected>`, or
/// `closed`.
const CURSOR: &str = "(() => { const t = document.querySelector('[role=combobox]'); \
     if (t.getAttribute('aria-expanded') !== 'true') return 'closed'; \
     const row = document.getElementById(t.getAttribute('aria-activedescendant')); \
     const label = row && row.querySelector('[data-slot=label]'); \
     return `${label && label.textContent}|${document.querySelectorAll('[role=listbox]').length}|${row && row.getAttribute('aria-selected')}`; })()";

#[test]
fn it_meets_the_baseline() {
    Suite::new("cascader", "/cascader")
        .focusable(TRIGGER)
        .targets("[role=option]")
        .state(
            "open",
            &[Step::TabTo(TRIGGER), Step::Press(keyboard::ARROW_DOWN)],
            "[role=listbox]",
        )
        .state(
            "drilled",
            &[Step::Press(keyboard::ARROW_RIGHT)],
            "[data-slot=column]:nth-child(2) [role=listbox]",
        )
        .run();
}

/// Down and Up skip the disabled root, Right opens a level, Left closes one,
/// Enter on a leaf commits the whole path and closes the list.
#[test]
fn the_keyboard_walks_the_levels() {
    block_on(async {
        for viewport in Viewport::ALL {
            let at = viewport.name();
            let fixture = Fixture::open("/cascader", viewport).await.unwrap();
            let page = &fixture.page;
            keyboard::tab_to(page, TRIGGER, 10).await.unwrap();

            let steps = [
                (
                    keyboard::ARROW_DOWN,
                    "Europe|1|true",
                    "ArrowDown to open on the first root",
                ),
                (
                    keyboard::ARROW_DOWN,
                    "Oceania|1|true",
                    "ArrowDown to skip disabled Asia",
                ),
                (
                    keyboard::ARROW_UP,
                    "Europe|1|true",
                    "ArrowUp to skip disabled Asia",
                ),
                (
                    keyboard::END,
                    "Oceania|1|true",
                    "End to the last enabled root",
                ),
                (keyboard::HOME, "Europe|1|true", "Home to the first root"),
                (
                    keyboard::ARROW_RIGHT,
                    "France|2|true",
                    "ArrowRight into Europe",
                ),
                (
                    keyboard::ARROW_DOWN,
                    "Germany|2|true",
                    "ArrowDown within the second level",
                ),
                (
                    keyboard::ARROW_UP,
                    "France|2|true",
                    "ArrowUp within the second level",
                ),
                (
                    keyboard::ARROW_RIGHT,
                    "Paris|3|true",
                    "ArrowRight into France",
                ),
                (keyboard::ARROW_DOWN, "Lyon|3|true", "ArrowDown to Lyon"),
                (
                    keyboard::ARROW_LEFT,
                    "France|2|true",
                    "ArrowLeft back out of France",
                ),
                (
                    keyboard::ENTER,
                    "Paris|3|true",
                    "Enter on a branch to open it",
                ),
                (
                    keyboard::ARROW_DOWN,
                    "Lyon|3|true",
                    "ArrowDown to Lyon again",
                ),
                (
                    keyboard::ENTER,
                    "closed",
                    "Enter on a leaf to commit and close",
                ),
            ];
            for (key, want, what) in steps {
                keyboard::press(page, key).await.unwrap();
                expect(page, want, what, at).await;
            }

            wait::for_js_true(
                page,
                "document.querySelector('#picked').textContent === 'lyon' \
                 && document.querySelector('[role=combobox]').textContent.includes('Europe / France / Lyon') \
                 && document.activeElement === document.querySelector('[role=combobox]')",
                "Lyon committed with its path shown, focus on the trigger",
            )
            .await
            .unwrap_or_else(|e| panic!("at {at}: {e}"));

            fixture
                .console
                .assert_clean(&format!("the cascader keys at {at}"))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// A click on disabled Asia leaves the cursor on Europe: the ArrowDown after
/// it lands on Oceania. Had the click opened Asia, the cursor would sit on
/// Japan, alone in its column, and the key would leave it there. A click on
/// Oceania is the positive control that a click on a root does open it.
#[test]
fn a_disabled_branch_does_not_open() {
    block_on(async {
        let fixture = Fixture::open("/cascader", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        expect(page, "Europe|1|true", "ArrowDown to open", "desktop").await;

        const ASIA: &str = "[role=option][id$='-option-0-1']";
        wait::for_visible(page, ASIA).await.unwrap();
        let disabled: Option<String> = page
            .evaluate(format!(
                "document.querySelector({ASIA:?}).getAttribute('aria-disabled')"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            disabled.as_deref(),
            Some("true"),
            "Asia's row is aria-disabled"
        );

        pointer::click(page, ASIA).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        expect(
            page,
            "Oceania|1|true",
            "ArrowDown from Europe after the click on Asia",
            "desktop",
        )
        .await;

        pointer::click(page, "[role=option][id$='-option-0-2']")
            .await
            .unwrap();
        expect(
            page,
            "Australia|2|true",
            "a click on Oceania to open it",
            "desktop",
        )
        .await;

        fixture
            .console
            .assert_clean("clicking a disabled branch")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

async fn expect(page: &chromiumoxide::Page, want: &str, what: &str, at: &str) {
    if let Err(e) = wait::for_js_true(page, &format!("{CURSOR} === {want:?}"), what).await {
        let actual: String = page.evaluate(CURSOR).await.unwrap().into_value().unwrap();
        panic!("at {at}: {e}; cursor reads {actual:?}");
    }
}
