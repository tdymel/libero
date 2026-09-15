//! `Select`: the combobox archetype.

use e2e::archetypes::Combobox;
use e2e::browser::block_on;
use e2e::suite::Step;
use e2e::{
    Fixture, Suite, Viewport, ax,
    passes::{keyboard, pointer},
    wait,
};

pub const TRIGGER: &str = "[role=combobox]";
const LISTBOX: &str = "[role=listbox]";
const SEARCH: &str = "input[role=combobox]";
const OPTION_COUNT: usize = 5;

#[test]
fn it_meets_the_baseline() {
    Suite::new("select", "/select")
        .focusable(TRIGGER)
        .targets("[role=option]")
        .state(
            "open",
            &[Step::TabTo(TRIGGER), Step::Press(keyboard::ARROW_DOWN)],
            LISTBOX,
        )
        .run();
}

/// The docs page's switches all on: captions, an error, the clear button and
/// the search box, which owns the combobox role while the list is open.
#[test]
fn a_searchable_field_meets_the_baseline() {
    Suite::new("select_field", "/select/field")
        .focusable(TRIGGER)
        .targets("[role=option], button")
        .state(
            "open",
            &[Step::TabTo(TRIGGER), Step::Press(keyboard::ARROW_DOWN)],
            LISTBOX,
        )
        .run();
}

/// `[active row is the selected one, selected row's box-shadow, its colour,
/// an unselected idle row's box-shadow]`.
const ROWS: &str = "(() => { const t = document.querySelector('[aria-activedescendant]'); \
     const act = t && document.getElementById(t.getAttribute('aria-activedescendant')); \
     const rows = [...document.querySelectorAll('[role=option]')]; \
     const sel = rows.find(r => r.getAttribute('aria-selected') === 'true'); \
     const off = rows.find(r => r !== act && r !== sel); \
     const s = e => getComputedStyle(e); \
     return [String(act === sel), s(sel).boxShadow, s(sel).color, s(off).boxShadow]; })()";

/// Todo 631: the selected row carries the house line at its start edge, beside
/// the tint, and keeps it inside the active row's ring. `Combobox`'s rows are
/// the same `ComboboxOption`.
pub fn selected_row_is_marked(route: &str) {
    block_on(async {
        let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        wait::for_visible(page, "[role=option]").await.unwrap();
        let rows =
            async || -> [String; 4] { page.evaluate(ROWS).await.unwrap().into_value().unwrap() };

        let [on_it, shadow, color, _] = rows().await;
        assert_eq!(on_it, "true", "{route}: the list opens on the selected row");
        assert!(
            shadow.contains(&format!("{color} 4px 0px 0px 0px inset"))
                && shadow.matches("inset").count() > 4,
            "{route}: the active selected row keeps its bar inside the ring: {shadow}"
        );

        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        let [on_it, shadow, color, off] = rows().await;
        assert_eq!(
            on_it, "false",
            "{route}: ArrowDown moves off the selected row"
        );
        assert_eq!(
            shadow,
            format!("{color} 2px 0px 0px 0px inset"),
            "{route}: the selected row's start bar"
        );
        assert!(
            !off.contains("inset"),
            "{route}: an idle row is marked: {off}"
        );

        crate::calendar::force_colours(page).await;
        crate::button::assert_on_in_forced_colours(
            page,
            "[role=option][aria-selected=true]",
            "[role=option]:not([aria-selected=true])",
        )
        .await;
        fixture.console.assert_clean(route).unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn the_selected_row_shows_the_on_state_line() {
    selected_row_is_marked("/select");
}

#[test]
fn it_honours_the_combobox_contract() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/select", viewport).await.unwrap();

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

/// While open, the search box is the combobox a screen reader announces, so
/// it is named by the field's label and carries its captions and states.
#[test]
fn the_open_search_box_is_announced_as_the_field() {
    block_on(async {
        let fixture = Fixture::open("/select/field", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.activeElement === document.querySelector({SEARCH:?})"),
            "the search box to take focus",
        )
        .await
        .unwrap();

        let wiring: Vec<Option<String>> = page
            .evaluate(format!(
                "(() => {{ const s = document.querySelector({SEARCH:?}); \
                 return ['aria-labelledby', 'aria-describedby', 'aria-invalid', 'aria-required'] \
                 .map(name => s.getAttribute(name)); }})()"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            wiring,
            [
                Some("lsx-1-label".to_string()),
                Some("lsx-1-description lsx-1-helper lsx-1-status".to_string()),
                Some("true".to_string()),
                Some("true".to_string()),
            ],
            "the search box's labelledby, describedby, invalid, required"
        );

        let tree = ax::snapshot(page, "body").await.unwrap();
        assert!(
            tree.contains(r#"combobox "Fruit""#),
            "the open search box is not named by the label:\n{tree}"
        );

        fixture.close().await.unwrap();
    });
}

/// Todo 509: on a closed trigger no Ctrl/Alt/Meta chord opens the list or moves
/// the chip cursor, bar Alt+ArrowDown (APG, the archetype checks it).
#[test]
fn chords_on_a_closed_select_are_the_browsers() {
    closed_chords_are_the_browsers("/select");
}

pub fn closed_chords_are_the_browsers(route: &str) {
    block_on(async {
        let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        let probe = format!(
            "(t => [t.getAttribute('aria-expanded'), t.getAttribute('aria-activedescendant')])(document.querySelector({TRIGGER:?}))"
        );
        keyboard::assert_chords_ignored(
            page,
            &[
                keyboard::ARROW_UP,
                keyboard::HOME,
                keyboard::END,
                keyboard::ARROW_LEFT,
                keyboard::ARROW_RIGHT,
            ],
            &probe,
        )
        .await
        .unwrap_or_else(|e| panic!("{route}: {e}"));
        keyboard::assert_chords_ignored_with(
            page,
            &[("Ctrl", keyboard::CTRL), ("Meta", keyboard::META)],
            &[keyboard::ARROW_DOWN],
            &probe,
        )
        .await
        .unwrap_or_else(|e| panic!("{route}: {e}"));
        fixture.console.assert_clean(route).unwrap();
        fixture.close().await.unwrap();
    });
}

/// APG select-only combobox: a closed trigger opens on ArrowUp (on the
/// selection, like ArrowDown), Home (first row) and End (last row).
#[test]
fn arrow_up_home_and_end_open_a_closed_select() {
    block_on(async {
        let fixture = Fixture::open("/select", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();

        // The fixture selects Banana, the second of five rows.
        for (name, key, row) in [
            ("ArrowUp", keyboard::ARROW_UP, 1),
            ("Home", keyboard::HOME, 0),
            ("End", keyboard::END, 4),
        ] {
            keyboard::press(page, key).await.unwrap();
            let check = format!(
                "(() => {{ const t = document.querySelector({TRIGGER:?}); \
                 return t.getAttribute('aria-expanded') === 'true' \
                 && (t.getAttribute('aria-activedescendant') || '').endsWith('-option-{row}'); }})()"
            );
            wait::for_js_true(page, &check, &format!("{name} to open on row {row}"))
                .await
                .unwrap_or_else(|e| panic!("{name} on a closed select: {e}"));

            keyboard::press(page, keyboard::ESCAPE).await.unwrap();
            wait::for_js_true(
                page,
                &format!(
                    "document.querySelector({TRIGGER:?}).getAttribute('aria-expanded') === 'false'"
                ),
                "Escape to close",
            )
            .await
            .unwrap();
        }

        fixture.close().await.unwrap();
    });
}

/// Todos 485, 519 (APG select-only): Tab, Alt+ArrowUp and Space commit the
/// highlight, then close. The list opens on Banana; ArrowDown moves to Cherry.
#[test]
fn tab_commits_the_highlight() {
    leaving_commits("/select", keyboard::TAB, 0, "Cherry");
}

#[test]
fn alt_arrow_up_commits_the_highlight() {
    leaving_commits("/select", keyboard::ARROW_UP, keyboard::ALT, "Cherry");
}

#[test]
fn space_commits_the_highlight() {
    leaving_commits("/select", keyboard::SPACE, 0, "Cherry");
}

/// Searchable: the search box has the focus, and Tab still commits (Banana,
/// then Damson past the disabled Cherry).
#[test]
fn tab_commits_from_the_search_box() {
    leaving_commits("/select/field", keyboard::TAB, 0, "Damson");
}

/// Opens `route`'s list, moves one row down, presses `key` with `modifiers`,
/// and expects a closed list showing `expected`.
pub fn leaving_commits(route: &str, key: keyboard::Key, modifiers: i64, expected: &str) {
    block_on(async {
        let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        wait::for_visible(page, "[role=option]").await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        page.evaluate("new Promise(r => setTimeout(() => r(1), 60))")
            .await
            .unwrap();
        keyboard::press_with(page, key, modifiers).await.unwrap();
        let check = format!(
            "(t => t.getAttribute('aria-expanded') === 'false' && t.textContent.includes({expected:?}))(document.querySelector({TRIGGER:?}))"
        );
        wait::for_js_true(page, &check, &format!("{} to commit {expected}", key.key))
            .await
            .unwrap_or_else(|e| panic!("{route}, {}: {e}", key.key));
        fixture.console.assert_clean(route).unwrap();
        fixture.close().await.unwrap();
    });
}

/// Space in the search box is typing, and in typeahead a space mid-query is
/// part of the query: neither commits.
#[test]
fn space_while_typing_commits_nothing() {
    block_on(async {
        for (route, searchable) in [("/select/field", true), ("/select", false)] {
            let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
            let page = &fixture.page;
            keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
            // Open first: typeahead on a closed single select picks in place.
            keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
            wait::for_visible(page, "[role=option]").await.unwrap();
            if searchable {
                wait::for_js_true(
                    page,
                    &format!("document.activeElement === document.querySelector({SEARCH:?})"),
                    "the search box to take focus",
                )
                .await
                .unwrap();
            }
            // A real keydown: typeahead listens to it, `type_text` sends none.
            keyboard::press(page, KEY_D).await.unwrap();
            keyboard::press(page, keyboard::SPACE).await.unwrap();
            page.evaluate("new Promise(r => setTimeout(() => r(1), 100))")
                .await
                .unwrap();
            // A search box that took the space is still open, whatever it
            // matches. The trigger keeps the field's id, role or not.
            let (open, value): (bool, String) = page
                .evaluate(format!(
                    "(s => [s ? s.value === 'd ' : !!document.querySelector({LISTBOX:?}), \
                     document.getElementById('lsx-1').textContent])(document.querySelector({SEARCH:?}))"
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert!(
                value.contains("Banana"),
                "{route}: Space while typing changed the value to {value:?}"
            );
            assert!(
                open,
                "{route}: Space while typing closed the list or was not typed"
            );
            fixture.console.assert_clean(route).unwrap();
            fixture.close().await.unwrap();
        }
    });
}

const KEY_D: keyboard::Key = keyboard::Key {
    key: "d",
    code: "KeyD",
    vk: 68,
    text: Some("d"),
};

/// Todo 487: with no `label`, the caller's `aria-label` names the trigger
/// while closed and moves to the search box while open.
#[test]
fn a_caller_aria_label_follows_the_combobox_role() {
    block_on(async {
        let fixture = Fixture::open("/select/unlabelled", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let names = format!(
            "(() => {{ const t = document.getElementById('lsx-1'); \
             const s = document.querySelector({SEARCH:?}); \
             return [t.getAttribute('aria-label'), s && s.getAttribute('aria-label')]; }})()"
        );
        let read = async || -> Vec<Option<String>> {
            page.evaluate(names.as_str())
                .await
                .unwrap()
                .into_value()
                .unwrap()
        };

        assert_eq!(
            read().await,
            [Some("Fruit".to_string()), None],
            "closed: the trigger's and the search box's aria-label"
        );

        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.activeElement === document.querySelector({SEARCH:?})"),
            "the search box to take focus",
        )
        .await
        .unwrap();
        assert_eq!(
            read().await,
            [None, Some("Fruit".to_string())],
            "open: the trigger's and the search box's aria-label"
        );
        let tree = ax::snapshot(page, "body").await.unwrap();
        assert!(
            tree.contains(r#"combobox "Fruit""#),
            "the open search box is not named by the caller's aria-label:\n{tree}"
        );

        fixture.close().await.unwrap();
    });
}

/// A click on the label focuses the trigger, as `<label for>` does on a
/// native `<select>` (todo 483).
#[test]
fn a_click_on_the_label_focuses_the_trigger() {
    label_click_focuses("/select", TRIGGER);
}

/// Clicks the page's first `<label>`, then waits for `control` to hold focus.
/// Every field named by `aria-labelledby` shares this check.
pub fn label_click_focuses(path: &str, control: &str) {
    block_on(async {
        let fixture = Fixture::open(path, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        pointer::click(page, "label").await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.activeElement === document.querySelector({control:?})"),
            &format!("a click on the label to focus {control} on {path}"),
        )
        .await
        .unwrap();
        fixture
            .console
            .assert_clean(&format!("the label click on {path}"))
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 547: the search box's Home and End edit the query, though typing arms
/// the top row; the arrows still move the rows.
#[test]
fn home_and_end_edit_the_search_query() {
    search_home_end_edit_the_query("/select/field", TRIGGER, "an");
}

/// Opens the list from `trigger`, types `query` (it must match a row) into the
/// search box, and checks Home/End move the caret and leave the highlight.
pub fn search_home_end_edit_the_query(route: &str, trigger: &str, query: &str) {
    block_on(async {
        let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, trigger, 10).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.activeElement === document.querySelector({SEARCH:?})"),
            "the search box to take focus",
        )
        .await
        .unwrap_or_else(|e| panic!("{route}: {e}"));
        keyboard::type_text(page, query).await.unwrap();

        let probe = format!(
            "(s => [s.selectionStart, s.getAttribute('aria-activedescendant')])(document.querySelector({SEARCH:?}))"
        );
        let read = async || -> (usize, Option<String>) {
            page.evaluate(probe.as_str())
                .await
                .unwrap()
                .into_value()
                .unwrap()
        };
        let (_, armed) = read().await;
        assert!(armed.is_some(), "{route}: typing arms a row");
        keyboard::press(page, keyboard::HOME).await.unwrap();
        assert_eq!(
            read().await,
            (0, armed.clone()),
            "{route}: Home in the search box"
        );
        keyboard::press(page, keyboard::END).await.unwrap();
        assert_eq!(
            read().await,
            (query.chars().count(), armed),
            "{route}: End in the search box"
        );

        fixture.console.assert_clean(route).unwrap();
        fixture.close().await.unwrap();
    });
}
