//! `MultiSelect`: the combobox archetype.
//!
//! The chips sit beside the `role="combobox"` element, not in it, so its value
//! reads `Cherry` and not `Cherry Remove Cherry` (todo 70 (b), which rebaselined
//! every `multi_select_*.snap`).

use e2e::archetypes::Combobox;
use e2e::browser::block_on;
use e2e::suite::Step;
use e2e::{
    Fixture, Suite, Viewport,
    passes::{keyboard, pointer},
    wait,
};

pub const TRIGGER: &str = "[role=combobox]";
const LISTBOX: &str = "[role=listbox]";
const OPTION_COUNT: usize = 5;

/// Todo 483: the label focuses the trigger it names by id.
#[test]
fn a_click_on_the_label_focuses_the_trigger() {
    crate::select::label_click_focuses("/multi-select", TRIGGER);
}

/// Todo 631: the selected row is not a tint alone.
#[test]
fn the_selected_row_shows_the_on_state_line() {
    crate::select::selected_row_is_marked("/multi-select");
}

#[test]
fn it_meets_the_baseline() {
    Suite::new("multi_select", "/multi-select")
        .focusable(TRIGGER)
        .targets("[role=option]")
        .state(
            "open",
            &[Step::TabTo(TRIGGER), Step::Press(keyboard::ARROW_DOWN)],
            LISTBOX,
        )
        .run();
}

/// Todo 509: the chip cursor is not moved by a Ctrl/Alt/Meta arrow either.
#[test]
fn chords_on_a_closed_trigger_are_the_browsers() {
    crate::select::closed_chords_are_the_browsers("/multi-select");
}

/// Todo 547: the search box's Home and End edit the query.
#[test]
fn home_and_end_edit_the_search_query() {
    crate::select::search_home_end_edit_the_query("/multi-select/search", TRIGGER, "an");
}

/// Todo 482: a search matching nothing shows "No results" and says it.
#[test]
fn a_search_matching_nothing_is_shown_and_said() {
    crate::select::search_matching_nothing("/multi-select/search", TRIGGER);
}

/// Todo 519: Tab and Alt+ArrowUp only close; a pick toggles, so leaving must
/// not add the highlight. Cherry is held, the list opens on it, then Damson.
#[test]
fn tab_closes_without_a_pick() {
    leaving_keeps_the_value(keyboard::TAB, 0);
}

#[test]
fn alt_arrow_up_closes_without_a_pick() {
    leaving_keeps_the_value(keyboard::ARROW_UP, keyboard::ALT);
}

/// Todo 519: Space and Enter toggle the highlight and keep the list open.
#[test]
fn space_toggles_and_stays_open() {
    picking_keeps_the_list(keyboard::SPACE);
}

#[test]
fn enter_toggles_and_stays_open() {
    picking_keeps_the_list(keyboard::ENTER);
}

async fn open_on_damson(page: &chromiumoxide::Page) {
    keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
    keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
    wait::for_visible(page, "[role=option]").await.unwrap();
    keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
    page.evaluate("new Promise(r => setTimeout(() => r(1), 60))")
        .await
        .unwrap();
}

fn leaving_keeps_the_value(key: keyboard::Key, modifiers: i64) {
    block_on(async {
        let fixture = Fixture::open("/multi-select", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        open_on_damson(page).await;
        keyboard::press_with(page, key, modifiers).await.unwrap();
        wait::for_js_true(
            page,
            &format!("!document.querySelector({LISTBOX:?})"),
            &format!("{} to close the list", key.key),
        )
        .await
        .unwrap();
        page.evaluate("new Promise(r => setTimeout(() => r(1), 100))")
            .await
            .unwrap();
        let text: String = page
            .evaluate("document.body.innerText")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            text.contains("Cherry") && !text.contains("Damson"),
            "{}: the chips should read Cherry alone, the page reads {text:?}",
            key.key
        );
        fixture.console.assert_clean(key.key).unwrap();
        fixture.close().await.unwrap();
    });
}

fn picking_keeps_the_list(key: keyboard::Key) {
    block_on(async {
        let fixture = Fixture::open("/multi-select", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        open_on_damson(page).await;
        keyboard::press(page, key).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "!!document.querySelector({LISTBOX:?}) \
                 && document.querySelectorAll('[role=option][aria-selected=true]').length === 2"
            ),
            &format!("{} to add Damson with the list open", key.key),
        )
        .await
        .unwrap();
        keyboard::press(page, key).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "!!document.querySelector({LISTBOX:?}) \
                 && document.querySelectorAll('[role=option][aria-selected=true]').length === 1"
            ),
            &format!("{} again to drop Damson with the list open", key.key),
        )
        .await
        .unwrap();
        fixture.console.assert_clean(key.key).unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn it_honours_the_combobox_contract() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/multi-select", viewport).await.unwrap();

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

/// Todo 876: a pick writes the selection before `onchange`. A caller that
/// refuses it keeps the old rows, in the posted values and in `aria-selected`;
/// one it takes moves both.
#[test]
fn a_refused_pick_keeps_the_old_selection() {
    const ROUTE: &str = "/multi-select/refused";
    const POSTED: &str = "document.querySelectorAll('input[name=fruit]').length";
    const SELECTED: &str = "[...document.querySelectorAll('[role=option][aria-selected=true]')].map((o) => o.textContent.trim()).join(',')";
    block_on(async {
        let fixture = Fixture::open(ROUTE, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        let read = async |js: &str| -> String {
            page.evaluate(format!("String({js})"))
                .await
                .unwrap()
                .into_value()
                .unwrap()
        };
        let pick = async |label: &str| {
            page.evaluate(format!(
                "document.querySelector('[data-e2e=pick]')?.removeAttribute('data-e2e'); \
                 [...document.querySelectorAll('[role=option]')].find((o) => o.textContent.trim() === {label:?}).setAttribute('data-e2e', 'pick')"
            ))
            .await
            .unwrap();
            pointer::click(page, "[data-e2e=pick]").await.unwrap();
            // Longer than any follow-up pass: a late reset would land by now.
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        };

        pointer::click(page, TRIGGER).await.unwrap();
        wait::for_visible(page, "[role=option]").await.unwrap();

        pick("Apple").await;
        assert_eq!(
            read(SELECTED).await,
            "Cherry",
            "a refused pick keeps aria-selected"
        );
        assert_eq!(
            read(POSTED).await,
            "1",
            "a refused pick posts the old values"
        );

        pick("Banana").await;
        assert_eq!(
            read(SELECTED).await,
            "Banana,Cherry",
            "a taken pick moves aria-selected"
        );
        assert_eq!(read(POSTED).await, "2", "a taken pick posts the new values");

        fixture.console.assert_clean(ROUTE).unwrap();
        fixture.close().await.unwrap();
    });
}
