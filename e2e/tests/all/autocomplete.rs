//! `Autocomplete`, the suite's pilot: its listbox is portaled, no descendant of its trigger.
//! The shape to copy: one `Suite` call, then tests only for what the component promises.

use anyhow::{Result, ensure};
use e2e::archetypes::Combobox;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, linger};
use e2e::passes::{contrast, dismissal, keyboard, live_region};
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, wait};

const TRIGGER: &str = "[role=combobox]";
const LISTBOX: &str = "[role=listbox]";
const STATUS: &str = "[role=status]";
const OPTION_COUNT: usize = 6;

/// Contrast, focus rings, the accessibility tree and a clean console, at both viewports.
#[test]
fn it_meets_the_baseline() {
    Suite::new("autocomplete", "/autocomplete")
        .focusable(TRIGGER)
        .waive(contrast::TODO_297)
        // The open state is the interesting one: a tree that is right closed
        // and wrong open is the normal shape of these bugs.
        .state(
            "open",
            &[Step::TabTo(TRIGGER), Step::Press(keyboard::ARROW_DOWN)],
            LISTBOX,
        )
        .targets("[role=option]")
        .run();
}

/// Keyboard, focus management and the aria contract, at both viewports.
#[test]
fn it_honours_the_combobox_contract() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/autocomplete", viewport).await.unwrap();

            // The archetype tabs to the trigger, so keyboard reachability is part of the contract.
            Combobox {
                trigger: TRIGGER,
                option_count: OPTION_COUNT,
                tab_budget: 10,
            }
            .assert_contract(&fixture.page)
            .await
            .unwrap_or_else(|e| panic!("at {}: {e}", viewport.name()));

            fixture
                .console
                .assert_clean(&format!("the combobox contract at {}", viewport.name()))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// A dismissed listbox must not stay readable. Trivial while nothing animates a popover;
/// the first animated dropdown inherits it.
#[test]
fn a_dismissed_list_leaves_the_accessibility_tree() {
    block_on(async {
        let fixture = Fixture::open("/autocomplete", Viewport::Desktop)
            .await
            .unwrap();

        keyboard::tab_to(&fixture.page, TRIGGER, 10).await.unwrap();
        keyboard::press(&fixture.page, keyboard::ARROW_DOWN)
            .await
            .unwrap();
        wait::for_visible(&fixture.page, LISTBOX).await.unwrap();

        keyboard::press(&fixture.page, keyboard::ESCAPE)
            .await
            .unwrap();
        // The close signal, not the list's disappearance: waiting for it to be
        // hidden waited for what the check asserts (review 7, E6).
        wait::for_js_true(
            &fixture.page,
            &format!(
                "document.querySelector({:?}).getAttribute('aria-expanded') === 'false'",
                TRIGGER
            ),
            "Escape to collapse the combobox",
        )
        .await
        .unwrap();

        dismissal::assert_gone_from_at(&fixture.page, LISTBOX)
            .await
            .expect("the dismissed listbox");

        fixture.close().await.unwrap();
    });
}

/// The status region is polite and mounted before it speaks: some screen readers skip a
/// region that mounts with its text (`codebase/components/combobox`).
#[test]
fn its_status_region_is_mounted_and_silent_at_rest() {
    block_on(async {
        let fixture = Fixture::open("/autocomplete", Viewport::Desktop)
            .await
            .unwrap();

        live_region::assert_politeness(&fixture.page, STATUS, "polite")
            .await
            .expect("the status region");

        let text = live_region::text_of(&fixture.page, STATUS).await.unwrap();
        assert!(
            text.trim().is_empty(),
            "the status region should be silent at rest, but it says {text:?}"
        );

        fixture.close().await.unwrap();
    });
}

/// While typing, Home and End move the caret (APG editable combobox). A query matching
/// nothing shows and says "No results" (482), so the popup stays expanded.
#[test]
fn typing_keeps_home_end_and_expanded_honest() {
    editable_combobox_typing("/autocomplete", "e");
}

/// Waits until a status region says "No results" and the dropdown shows it.
pub async fn wait_for_nothing_found(page: &chromiumoxide::Page, what: &str) {
    wait::for_js_true(
        page,
        "[...document.querySelectorAll('[role=status]')].some(e => e.textContent.includes('No results')) \
         && [...document.querySelectorAll(\"[data-slot='nothing-found']\")].some(e => e.offsetParent !== null)",
        what,
    )
    .await
    .unwrap_or_else(|e| panic!("{e}"));
}

/// Types `query` (it must match a row), checks Home/End edit the text, then
/// types a query matching nothing and checks it is shown and announced.
pub fn editable_combobox_typing(route: &str, query: &str) {
    block_on(async {
        let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        keyboard::type_text(page, query).await.unwrap();
        wait::for_visible(page, LISTBOX).await.unwrap();

        let probe = format!(
            "(t => [t.selectionStart, t.getAttribute('aria-activedescendant')])(document.querySelector({TRIGGER:?}))"
        );
        let read = async || -> (usize, Option<String>) {
            page.evaluate(probe.as_str())
                .await
                .unwrap()
                .into_value()
                .unwrap()
        };
        keyboard::press(page, keyboard::HOME).await.unwrap();
        assert_eq!(read().await, (0, None), "{route}: Home while typing");
        keyboard::press(page, keyboard::END).await.unwrap();
        assert_eq!(
            read().await,
            (query.chars().count(), None),
            "{route}: End while typing"
        );

        keyboard::type_text(page, "zzz").await.unwrap();
        wait_for_nothing_found(page, &format!("{route}: a query matching nothing")).await;
        // The popup draws the text, and no listbox is left to point at.
        let wiring: (Option<String>, Option<String>) = page
            .evaluate(format!(
                "(t => [t.getAttribute('aria-expanded'), t.getAttribute('aria-controls')])(document.querySelector({TRIGGER:?}))"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            wiring,
            (Some("true".into()), None),
            "{route}: nothing found"
        );

        fixture.console.assert_clean(route).unwrap();
        fixture.close().await.unwrap();
    });
}

/// Home and End while typing leave the options alone, and "No results" is drawn and said,
/// on every platform. The caret is read on the web only, above.
async fn typing_keeps_the_list_honest<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(TRIGGER).await?;
    d.type_text("e").await?;
    eventually(d, "the list to open", async |d| d.exists(LISTBOX).await).await?;
    for key in [keyboard::HOME, keyboard::END] {
        d.press(key).await?;
        linger(d, 2).await;
        let active = d.attr(TRIGGER, "aria-activedescendant").await?;
        ensure!(
            active.is_none(),
            "{:?}: a key highlighted {active:?}",
            d.platform()
        );
    }
    d.type_text("zzz").await?;
    eventually(d, "No results, shown and said", async |d| {
        Ok(d.exists("[data-slot='nothing-found']").await?
            && d.text(STATUS).await?.contains("No results"))
    })
    .await?;
    let expanded = d.attr(TRIGGER, "aria-expanded").await?;
    let controls = d.attr(TRIGGER, "aria-controls").await?;
    ensure!(
        expanded.as_deref() == Some("true") && controls.is_none(),
        "{:?}: nothing found wired as {expanded:?} {controls:?}",
        d.platform()
    );
    Ok(())
}

e2e::scenario!(
    typing_keeps_the_list_honest_on_every_platform,
    "/autocomplete",
    typing_keeps_the_list_honest
);
