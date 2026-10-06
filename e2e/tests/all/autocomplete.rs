//! `Autocomplete`, the suite's pilot: its listbox is portaled, no descendant of its trigger.
//! The shape to copy: one `Suite` call, then tests only for what the component promises.

use anyhow::{Result, ensure};
use e2e::archetypes::Combobox;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually};
use e2e::passes::{dismissal, keyboard, live_region};
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
        e2e::browser::at_every_viewport(async |viewport| {
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
        })
        .await;
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

/// Todo 1585: a prefiltered list still on its way says and draws the loader, never
/// "No results"; once it lands empty, "No results" comes back.
#[test]
fn a_pending_fetch_says_loading_not_nothing_found() {
    block_on(async {
        let fixture = Fixture::open("/autocomplete/fetch", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        keyboard::type_text(page, "B").await.unwrap();
        wait::for_js_true(
            page,
            "[...document.querySelectorAll('[role=status]')].some(e => e.textContent === 'Loading') \
             && document.querySelector('[aria-busy=true]') !== null",
            "the pending list to say Loading",
        )
        .await
        .unwrap();
        let nothing: bool = page
            .evaluate(
                "document.body.textContent.includes('No results') \
                 || document.querySelector(\"[data-slot='nothing-found']\") !== null",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(!nothing, "a list that has not arrived said No results");

        keyboard::type_text(page, "erx").await.unwrap();
        wait_for_nothing_found(page, "the landed empty list").await;
        fixture.console.assert_clean("a pending fetch").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 2415: a phone keyboard's trailing space still finds the city.
#[test]
fn a_trailing_space_still_matches() {
    block_on(async {
        let fixture = Fixture::open("/autocomplete", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        keyboard::type_text(page, "Berlin ").await.unwrap();
        wait::for_js_true(
            page,
            "(o => o.length === 1 && o[0].textContent.trim() === 'Berlin')\
             ([...document.querySelectorAll('[role=option]')]) \
             && [...document.querySelectorAll('[role=status]')].some(e => e.textContent.includes('1 result'))",
            "Berlin, drawn and counted",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("a trailing space").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 2417: a caller's `empty` over no options is said with the text, and leaves with it.
#[test]
fn a_callers_empty_is_said_and_leaves_an_emptied_field() {
    const SHOWN: &str = "(e => !!e && e.offsetParent !== null)(document.getElementById('no-city'))";
    block_on(async {
        let fixture = Fixture::open("/autocomplete/none", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        keyboard::type_text(page, "Be").await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "{SHOWN} && [...document.querySelectorAll('[role=status]')].some(e => e.textContent.includes('No results'))"
            ),
            "the caller's empty, drawn and said",
        )
        .await
        .unwrap();

        for _ in 0..2 {
            keyboard::press(page, keyboard::BACKSPACE).await.unwrap();
        }
        wait::for_js_true(
            page,
            &format!(
                "!{SHOWN} && document.querySelector({TRIGGER:?}).getAttribute('aria-expanded') === 'false' \
                 && [...document.querySelectorAll('[role=status]')].every(e => e.textContent.trim() === '')"
            ),
            "an emptied field to draw and say nothing",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("a caller's empty").unwrap();
        fixture.close().await.unwrap();
    });
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
    // Todo 1574: what the query left is said too.
    eventually(d, "the result count, said", async |d| {
        Ok(d.text(STATUS).await?.contains(" result"))
    })
    .await?;
    for key in [keyboard::HOME, keyboard::END] {
        d.press(key).await?;
        d.settle().await?;
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

/// Todo 2046: option ids are valid, unique tokens whatever the labels hold, and a filter
/// keeps each kept row's id; two equal labels keep two ids.
#[test]
fn odd_and_equal_labels_get_stable_unique_option_ids() {
    const ROUTE: &str = "/autocomplete/odd";
    const OPTIONS: &str = "[...document.querySelectorAll('[role=option]')].map(o => o.id)";
    const LABELS: &str =
        "[...document.querySelectorAll('[role=option]')].map(o => o.textContent.trim())";
    block_on(async {
        let fixture = Fixture::open(ROUTE, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        let read = async |js: &str| -> Vec<String> {
            page.evaluate(js).await.unwrap().into_value().unwrap()
        };
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        // Every label holds an "r".
        keyboard::type_text(page, "r").await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelectorAll('[role=option]').length === 6",
            "all six rows",
        )
        .await
        .unwrap();
        let before: Vec<(String, String)> = read(LABELS)
            .await
            .into_iter()
            .zip(read(OPTIONS).await)
            .collect();
        for (label, id) in &before {
            assert!(
                !id.is_empty()
                    && id
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
                "{label} has the id {id:?}"
            );
        }
        let unique: std::collections::HashSet<_> = before.iter().map(|(_, id)| id).collect();
        assert_eq!(unique.len(), 6, "duplicate ids: {before:?}");

        // "ri" keeps O'Brien, Zürich and both Parises, each moved up the list.
        keyboard::type_text(page, "i").await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelectorAll('[role=option]').length === 4",
            "the filter",
        )
        .await
        .unwrap();
        let after: Vec<(String, String)> = read(LABELS)
            .await
            .into_iter()
            .zip(read(OPTIONS).await)
            .collect();
        assert_eq!(after, before[1..5], "the kept rows changed ids");

        // The input names a drawn row by the kept id.
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "(id => !!id && !!document.getElementById(id))(document.querySelector({TRIGGER:?}).getAttribute('aria-activedescendant'))"
            ),
            "aria-activedescendant to name a drawn row",
        )
        .await
        .unwrap();
        fixture.console.assert_clean(ROUTE).unwrap();
        fixture.close().await.unwrap();
    });
}
