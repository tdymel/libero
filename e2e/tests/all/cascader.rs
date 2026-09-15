//! `Cascader`: the keyboard walks levels, and a disabled branch neither takes
//! the cursor nor opens (todo 406).
//!
//! `/cascader` holds three roots: Europe (France: Paris, Lyon; Germany:
//! Berlin), Asia, disabled (Japan: Tokyo), and Oceania (Australia: Sydney).
//! Focus never leaves the trigger; the cursor is its `aria-activedescendant`.
//! `/cascader/any-level` is the same tree with `any_level` on.

use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, wait};

const TRIGGER: &str = "[role=combobox]";
const SEARCH: &str = "input[role=combobox]";

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

/// Todo 484: while the search box is open it is the combobox, so the role-less
/// trigger may not keep `aria-expanded` or `aria-required` (axe
/// `aria-allowed-attr`).
#[test]
fn a_searchable_field_meets_the_baseline() {
    Suite::new("cascader_search", "/cascader/search")
        .focusable(TRIGGER)
        .targets("[role=option]")
        .state(
            "open",
            &[Step::TabTo(TRIGGER), Step::Press(keyboard::ARROW_DOWN)],
            "[role=listbox]",
        )
        .run();
}

/// The open search box carries the field's label, captions and states.
#[test]
fn the_open_search_box_is_announced_as_the_field() {
    block_on(async {
        let fixture = Fixture::open("/cascader/search", Viewport::Desktop)
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
                 const t = document.getElementById('lsx-1'); \
                 return ['aria-labelledby', 'aria-describedby', 'aria-invalid', 'aria-required'] \
                 .map(name => s.getAttribute(name)) \
                 .concat(['aria-expanded', 'aria-required', 'role'].map(name => t.getAttribute(name))); }})()"
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
                None,
                None,
                None,
            ],
            "the search box's labelledby, describedby, invalid, required; \
             the trigger's expanded, required, role"
        );

        fixture.close().await.unwrap();
    });
}

/// Todo 483: the label focuses the trigger it names by id.
#[test]
fn a_click_on_the_label_focuses_the_trigger() {
    crate::select::label_click_focuses("/cascader", TRIGGER);
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

/// Todo 509: Ctrl/Meta chords and Alt+ArrowRight/End leave the walk alone;
/// Alt+ArrowDown opens without moving the cursor, Alt+ArrowUp closes (APG).
#[test]
fn modifier_chords_leave_the_walk_alone() {
    block_on(async {
        let fixture = Fixture::open("/cascader", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        let caret_keys = [
            keyboard::ARROW_DOWN,
            keyboard::ARROW_UP,
            keyboard::ARROW_RIGHT,
            keyboard::ARROW_LEFT,
            keyboard::HOME,
            keyboard::END,
        ];
        let not_alt = [("Ctrl", keyboard::CTRL), ("Meta", keyboard::META)];
        let alt = [("Alt", keyboard::ALT)];

        keyboard::assert_chords_ignored_with(page, &not_alt, &caret_keys, CURSOR)
            .await
            .unwrap_or_else(|e| panic!("closed: {e}"));
        keyboard::assert_chords_ignored_with(page, &alt, &caret_keys[1..], CURSOR)
            .await
            .unwrap_or_else(|e| panic!("closed: {e}"));

        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        expect(page, "France|2|true", "ArrowRight into Europe", "desktop").await;
        keyboard::assert_chords_ignored_with(page, &not_alt, &caret_keys, CURSOR)
            .await
            .unwrap_or_else(|e| panic!("open: {e}"));
        keyboard::assert_chords_ignored_with(page, &alt, &caret_keys[2..], CURSOR)
            .await
            .unwrap_or_else(|e| panic!("open: {e}"));

        keyboard::press_with(page, keyboard::ARROW_DOWN, keyboard::ALT)
            .await
            .unwrap();
        page.evaluate("new Promise(r => setTimeout(() => r(1), 60))")
            .await
            .unwrap();
        expect(
            page,
            "France|2|true",
            "Alt+ArrowDown to keep the cursor",
            "desktop",
        )
        .await;
        keyboard::press_with(page, keyboard::ARROW_UP, keyboard::ALT)
            .await
            .unwrap();
        expect(page, "closed", "Alt+ArrowUp to close", "desktop").await;
        keyboard::press_with(page, keyboard::ARROW_DOWN, keyboard::ALT)
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('[role=combobox]').getAttribute('aria-expanded') === 'true'",
            "Alt+ArrowDown to open",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("cascader chords").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todos 485, 519 (APG select-only): Tab, Alt+ArrowUp and Space on a leaf
/// commit its path, then close.
#[test]
fn tab_commits_the_highlighted_leaf() {
    leaving_commits_lyon(keyboard::TAB, 0);
}

#[test]
fn alt_arrow_up_commits_the_highlighted_leaf() {
    leaving_commits_lyon(keyboard::ARROW_UP, keyboard::ALT);
}

#[test]
fn space_commits_the_highlighted_leaf() {
    leaving_commits_lyon(keyboard::SPACE, 0);
}

fn leaving_commits_lyon(key: keyboard::Key, modifiers: i64) {
    block_on(async {
        let fixture = Fixture::open("/cascader", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        for step in [
            keyboard::ARROW_DOWN,
            keyboard::ARROW_RIGHT,
            keyboard::ARROW_RIGHT,
            keyboard::ARROW_DOWN,
        ] {
            keyboard::press(page, step).await.unwrap();
        }
        expect(page, "Lyon|3|true", "the walk to Lyon", "desktop").await;
        keyboard::press_with(page, key, modifiers).await.unwrap();
        expect(page, "closed", &format!("{} to close", key.key), "desktop").await;
        wait::for_js_true(
            page,
            "document.querySelector('#picked').textContent === 'lyon'",
            &format!("{} to commit Lyon", key.key),
        )
        .await
        .unwrap();
        fixture.console.assert_clean(key.key).unwrap();
        fixture.close().await.unwrap();
    });
}

/// A branch is no pick without `any_level`: Tab on France only closes.
#[test]
fn tab_on_a_branch_commits_nothing() {
    block_on(async {
        let fixture = Fixture::open("/cascader", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        expect(page, "France|2|true", "ArrowRight into Europe", "desktop").await;
        keyboard::press(page, keyboard::TAB).await.unwrap();
        expect(page, "closed", "Tab to close", "desktop").await;
        page.evaluate("new Promise(r => setTimeout(() => r(1), 100))")
            .await
            .unwrap();
        let picked: String = page
            .evaluate("document.querySelector('#picked').textContent")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(picked, "", "Tab on a branch committed it");
        fixture.console.assert_clean("Tab on a branch").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 632: with `any_level` a branch is a pick, so Tab and Alt+ArrowUp on
/// France commit it, then close.
#[test]
fn leaving_commits_the_highlighted_branch_with_any_level() {
    block_on(async {
        for (key, modifiers) in [(keyboard::TAB, 0), (keyboard::ARROW_UP, keyboard::ALT)] {
            let fixture = Fixture::open("/cascader/any-level", Viewport::Desktop)
                .await
                .unwrap();
            let page = &fixture.page;
            keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
            keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
            keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
            expect(page, "France|2|true", "ArrowRight into Europe", "desktop").await;
            keyboard::press_with(page, key, modifiers).await.unwrap();
            expect(page, "closed", &format!("{} to close", key.key), "desktop").await;
            wait::for_js_true(
                page,
                "document.querySelector('#picked').textContent === 'france' \
                 && document.querySelector('[role=combobox]').textContent.includes('Europe / France')",
                &format!("{} to commit France", key.key),
            )
            .await
            .unwrap();
            fixture.console.assert_clean(key.key).unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// `"paths"`: Space commits the highlighted row; the search box takes Tab to
/// commit as well.
#[test]
fn space_and_tab_commit_a_path_row() {
    block_on(async {
        for (route, key) in [
            ("/cascader/paths", keyboard::SPACE),
            ("/cascader/search", keyboard::TAB),
        ] {
            let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
            let page = &fixture.page;
            keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
            keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
            wait::for_visible(page, "[role=option]").await.unwrap();
            if route == "/cascader/search" {
                keyboard::type_text(page, "lyon").await.unwrap();
                wait::for_js_true(
                    page,
                    "document.querySelectorAll('[role=option]').length === 1",
                    "the query to leave Lyon alone",
                )
                .await
                .unwrap();
                keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
            } else {
                keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
            }
            // Lyon's row, whoever holds the active descendant.
            let label = "(() => { const a = document.activeElement.getAttribute('aria-activedescendant'); \
                 const row = a && document.getElementById(a); \
                 const l = row && row.querySelector('[data-slot=label]'); return l ? l.textContent : ''; })()";
            wait::for_js_true(
                page,
                &format!("{label} === 'Europe / France / Lyon'"),
                &format!("{route}: ArrowDown to Lyon"),
            )
            .await
            .unwrap();
            keyboard::press(page, key).await.unwrap();
            wait::for_js_true(
                page,
                "!document.querySelector('[role=listbox]') \
                 && document.getElementById('lsx-1').textContent.includes('Europe / France / Lyon')",
                &format!("{route}: {} to commit Lyon", key.key),
            )
            .await
            .unwrap();
            fixture.console.assert_clean(route).unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// APG select-only: on a closed list Home and End open on the first and last
/// enabled root, Space like Enter on the first.
#[test]
fn home_end_and_space_open_a_closed_list() {
    block_on(async {
        let fixture = Fixture::open("/cascader", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        for (key, want, what) in [
            (
                keyboard::HOME,
                "Europe|1|true",
                "Home to open on the first root",
            ),
            (
                keyboard::END,
                "Oceania|1|true",
                "End to open on the last root",
            ),
            (
                keyboard::SPACE,
                "Europe|1|true",
                "Space to open on the first root",
            ),
        ] {
            keyboard::press(page, key).await.unwrap();
            expect(page, want, what, "desktop").await;
            keyboard::press(page, keyboard::ESCAPE).await.unwrap();
            expect(page, "closed", "Escape to close", "desktop").await;
        }
        fixture
            .console
            .assert_clean("cascader Home/End/Space")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// In `"paths"` the opening keys arm a whole-path row. They armed a root, which
/// is no row there, so the list opened with nothing highlighted.
#[test]
fn the_paths_layout_opens_on_a_row() {
    block_on(async {
        let fixture = Fixture::open("/cascader/paths", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        for (key, want, what) in [
            (
                keyboard::ARROW_DOWN,
                "Europe / France / Paris|1|true",
                "ArrowDown to open on the first path",
            ),
            (
                keyboard::END,
                "Oceania / Australia / Sydney|1|true",
                "End to open on the last enabled path",
            ),
        ] {
            keyboard::press(page, key).await.unwrap();
            expect(page, want, what, "desktop").await;
            keyboard::press(page, keyboard::ESCAPE).await.unwrap();
            expect(page, "closed", "Escape to close", "desktop").await;
        }
        fixture.console.assert_clean("cascader paths").unwrap();
        fixture.close().await.unwrap();
    });
}

/// APG typeahead: a letter opens a closed list on the root it starts, moves
/// within the cursor's column once open, and skips disabled Asia.
#[test]
fn typing_moves_to_the_row_it_starts() {
    block_on(async {
        let fixture = Fixture::open("/cascader", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        for (key, want, what) in [
            (
                letter("o", "KeyO", 79),
                "Oceania|1|true",
                "o to open on Oceania",
            ),
            (letter("e", "KeyE", 69), "Europe|1|true", "e to Europe"),
            (
                letter("a", "KeyA", 65),
                "Europe|1|true",
                "a to pass over disabled Asia",
            ),
        ] {
            keyboard::press(page, key).await.unwrap();
            expect(page, want, what, "desktop").await;
            // Past the typeahead's reset, so the next letter starts afresh.
            page.evaluate("new Promise(r => setTimeout(() => r(1), 600))")
                .await
                .unwrap();
        }
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        expect(page, "France|2|true", "ArrowRight into Europe", "desktop").await;
        keyboard::press(page, letter("g", "KeyG", 71))
            .await
            .unwrap();
        expect(page, "Germany|2|true", "g to Germany", "desktop").await;
        fixture.console.assert_clean("cascader typeahead").unwrap();
        fixture.close().await.unwrap();
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

/// A letter key with its keydown, which typeahead listens to.
const fn letter(key: &'static str, code: &'static str, vk: i64) -> keyboard::Key {
    keyboard::Key {
        key,
        code,
        vk,
        text: Some(key),
    }
}

async fn expect(page: &chromiumoxide::Page, want: &str, what: &str, at: &str) {
    if let Err(e) = wait::for_js_true(page, &format!("{CURSOR} === {want:?}"), what).await {
        let actual: String = page.evaluate(CURSOR).await.unwrap().into_value().unwrap();
        panic!("at {at}: {e}; cursor reads {actual:?}");
    }
}
