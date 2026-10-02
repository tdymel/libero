//! `Spotlight`: a modal palette trapping focus in its search box. `contrast_covers` guards
//! todo 327 (the scroll lock hid the rows from axe); keys and Ctrl/Cmd+K are real (406).

use anyhow::Result;
use e2e::archetypes::Overlay;
use e2e::browser::block_on;
use e2e::driver::{Driver, Platform, eventually, eventually_focused};
use e2e::passes::{focus, keyboard::Key};
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, js, passes::keyboard, wait};

pub const TRIGGER: &str = "#open-spotlight";
const DIALOG: &str = "[role=dialog]";
const SEARCH: &str = "[role=dialog] input[role=combobox]";
const OPTIONS: &str = "[role=dialog] [role=option]";
/// The fixture writes the label of the action that ran into its `data-ran`.
const RAN: &str = "#ran";
/// A chord's key carries no text: a held Ctrl or Meta types nothing.
const K: Key = Key {
    key: "k",
    code: "KeyK",
    vk: 75,
    text: None,
};

async fn ctrl_k_opens<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(TRIGGER).await?;
    d.press_ctrl(K).await?;
    eventually(d, "Ctrl+K to open the palette", async |d| {
        d.exists(DIALOG).await
    })
    .await
}

e2e::scenario!(ctrl_k_opens_the_palette, "/spotlight", ctrl_k_opens);

/// Todo 1014: a tap on the trigger lands focus in the search box, and on a
/// phone raises the keyboard.
async fn a_tap_focuses_the_search<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(TRIGGER).await?;
    eventually_focused(d, SEARCH, "a tap on the trigger").await?;
    if d.platform() == Platform::Android {
        eventually(d, "the soft keyboard", async |d| {
            d.soft_keyboard_shown().await
        })
        .await?;
    }
    Ok(())
}

e2e::scenario!(
    a_tap_on_the_trigger_focuses_the_search_box,
    "/spotlight",
    a_tap_focuses_the_search
);

#[test]
fn it_meets_the_baseline() {
    Suite::new("spotlight", "/spotlight")
        .focusable(TRIGGER)
        .targets("[role=dialog] [role=option]")
        .contrast_covers(DIALOG)
        .state(
            "open",
            &[Step::TabTo(TRIGGER), Step::Press(keyboard::ENTER)],
            DIALOG,
        )
        .run();
}

#[test]
fn it_honours_the_overlay_contract() {
    block_on(async {
        e2e::browser::at_every_viewport(async |viewport| {
            let fixture = Fixture::open("/spotlight", viewport).await.unwrap();

            Overlay {
                trigger: TRIGGER,
                panel: DIALOG,
                traps_focus: true,
                tab_budget: 5,
            }
            .assert_contract(&fixture.page)
            .await
            .unwrap_or_else(|e| panic!("at {}: {e}", viewport.name()));

            fixture
                .console
                .assert_clean(&format!("the spotlight contract at {}", viewport.name()))
                .unwrap();
            fixture.close().await.unwrap();
        })
        .await;
    });
}

/// `None` read as `""`: a JS `null` does not deserialise into an `Option`.
async fn highlight(page: &chromiumoxide::Page) -> String {
    js(
        page,
        &format!("document.querySelector({SEARCH:?}).getAttribute('aria-activedescendant') ?? ''"),
    )
    .await
}

/// The search box names `expected`, that row alone is drawn active, and focus
/// never left the search box.
async fn expect_active(page: &chromiumoxide::Page, expected: &str, during: &str) {
    let check = format!(
        "(() => {{ const input = document.querySelector({SEARCH:?}); \
         const active = [...document.querySelectorAll({OPTIONS:?})] \
           .filter(o => o.hasAttribute('data-active')).map(o => o.id); \
         return input.getAttribute('aria-activedescendant') === {expected:?} \
           && active.length === 1 && active[0] === {expected:?}; }})()"
    );
    if wait::for_js_true(page, &check, during).await.is_err() {
        let actual = highlight(page).await;
        panic!("after {during}, the highlight is on {actual:?}, expected {expected:?}");
    }
    focus::assert_focused(page, SEARCH, during).await.unwrap();
}

async fn open_by_keyboard(page: &chromiumoxide::Page) {
    keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
    keyboard::press(page, keyboard::ENTER).await.unwrap();
    wait::for_visible(page, DIALOG).await.unwrap();
    focus::wait_for_focus(page, SEARCH, "opening the palette")
        .await
        .unwrap();
}

async fn ran(page: &chromiumoxide::Page, label: &str, what: &str) {
    wait::for_js_true(
        page,
        &format!("document.querySelector({RAN:?})?.dataset.ran === {label:?}"),
        what,
    )
    .await
    .unwrap();
}

/// The arrows wrap through the rows while focus stays in the search box, and
/// Enter runs the highlighted row, closes, and hands focus back.
#[test]
fn the_arrows_move_the_highlight_and_enter_runs_it() {
    block_on(async {
        let fixture = Fixture::open("/spotlight", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        open_by_keyboard(page).await;

        let ids: Vec<String> = js(
            page,
            &format!("[...document.querySelectorAll({OPTIONS:?})].map(o => o.id)"),
        )
        .await;
        assert_eq!(ids.len(), 3, "the fixture offers three actions: {ids:?}");
        assert_eq!(highlight(page).await, "", "a highlight on opening");

        for (index, id) in ids.iter().enumerate() {
            keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
            expect_active(page, id, &format!("ArrowDown to row {index}")).await;
        }
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        expect_active(page, &ids[0], "ArrowDown past the last row").await;
        keyboard::press(page, keyboard::ARROW_UP).await.unwrap();
        expect_active(page, &ids[2], "ArrowUp past the first row").await;
        keyboard::press(page, keyboard::ARROW_UP).await.unwrap();
        expect_active(page, &ids[1], "ArrowUp to row 1").await;

        keyboard::press(page, keyboard::ENTER).await.unwrap();
        ran(page, "Changelog", "Enter to run the highlighted row").await;
        wait::for_hidden(page, DIALOG).await.unwrap();
        focus::wait_for_focus(page, TRIGGER, "running an action")
            .await
            .unwrap();

        // A typed query highlights its first hit, so Enter runs it at once.
        open_by_keyboard(page).await;
        keyboard::type_text(page, "new").await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.querySelectorAll({OPTIONS:?}).length === 1"),
            "the query to narrow the rows to one",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        ran(page, "New file", "Enter to run the query's first hit").await;
        wait::for_hidden(page, DIALOG).await.unwrap();

        fixture
            .console
            .assert_clean("driving the palette's keys")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Some status region says `text`.
fn status_says(text: &str) -> String {
    format!(
        "[...document.querySelectorAll('[role=dialog] [role=status]')].some(e => e.textContent.includes({text:?}))"
    )
}

/// WCAG 4.1.3 (todo 1810): a query matching nothing is shown and said, focus stays
/// in the search box, and Enter runs nothing.
#[test]
fn a_query_matching_nothing_is_shown_and_said() {
    block_on(async {
        let fixture = Fixture::open("/spotlight", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        open_by_keyboard(page).await;
        keyboard::type_text(page, "zzz").await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "{} && document.querySelectorAll({OPTIONS:?}).length === 0 \
                 && document.querySelector('[role=dialog] [role=status]').offsetHeight > 0",
                status_says("Nothing found")
            ),
            "zzz to say Nothing found",
        )
        .await
        .unwrap();
        focus::assert_focused(page, SEARCH, "a query matching nothing")
            .await
            .unwrap();

        keyboard::press(page, keyboard::ENTER).await.unwrap();
        // Keys run in order: once the query below lands, the Enter above is handled.
        for _ in 0..3 {
            keyboard::press(page, keyboard::BACKSPACE).await.unwrap();
        }
        keyboard::type_text(page, "home").await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.querySelectorAll({OPTIONS:?}).length === 1"),
            "the query after Enter to narrow to Home",
        )
        .await
        .unwrap();
        let ran_nothing: bool = js(
            page,
            &format!(
                "document.querySelector({RAN:?}).dataset.ran === '' && document.querySelector({DIALOG:?}) !== null"
            ),
        )
        .await;
        assert!(
            ran_nothing,
            "Enter on nothing found ran or closed something"
        );
        fixture
            .console
            .assert_clean("a query matching nothing")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// WCAG 4.1.3 (todo 1810): a search still on its way says Searching and marks the list
/// busy, never Nothing found; the landed empty answer says Nothing found.
#[test]
fn a_pending_search_says_searching_not_nothing_found() {
    block_on(async {
        let fixture = Fixture::open("/spotlight-fetch", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        open_by_keyboard(page).await;
        keyboard::type_text(page, "z").await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "{} && document.querySelector('[role=dialog] [role=listbox][aria-busy=true]') !== null",
                status_says("Searching")
            ),
            "the pending search to say Searching",
        )
        .await
        .unwrap();
        let nothing: bool = js(page, &status_says("Nothing found")).await;
        assert!(!nothing, "a search that has not landed said Nothing found");
        focus::assert_focused(page, SEARCH, "a pending search")
            .await
            .unwrap();

        keyboard::type_text(page, "zz").await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "{} && !{} && document.querySelector('[aria-busy=true]') === null",
                status_says("Nothing found"),
                status_says("Searching")
            ),
            "the landed empty search to say Nothing found",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("a pending search").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 627: a narrowing query remounts the rows. Focus stays in the search
/// box, and the highlight and `aria-activedescendant` land on the new row.
#[test]
fn a_narrowing_query_keeps_the_highlight_on_a_live_row() {
    block_on(async {
        let fixture = Fixture::open("/spotlight", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        open_by_keyboard(page).await;
        let count = |n: usize| format!("document.querySelectorAll({OPTIONS:?}).length === {n}");

        keyboard::type_text(page, "n").await.unwrap();
        wait::for_js_true(page, &count(2), "\"n\" to narrow to two rows")
            .await
            .unwrap();
        // Todo 1574: the count is said, as "nothing found" is.
        let says = |text: &str| {
            format!(
                "[...document.querySelectorAll('[role=status]')].some(e => e.textContent.includes({text:?}))"
            )
        };
        wait::for_js_true(page, &says("2 results"), "the status to say 2 results")
            .await
            .unwrap();
        let ids: Vec<String> = js(
            page,
            &format!("[...document.querySelectorAll({OPTIONS:?})].map(o => o.id)"),
        )
        .await;
        expect_active(page, &ids[0], "a narrowing query").await;
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        expect_active(page, &ids[1], "ArrowDown after the rekey").await;

        let _: bool = js(
            page,
            &format!("!!(window.__row = document.getElementById({:?}))", ids[0]),
        )
        .await;
        keyboard::type_text(page, "e").await.unwrap();
        wait::for_js_true(page, &count(1), "\"ne\" to narrow to one row")
            .await
            .unwrap();
        wait::for_js_true(page, &says("1 result"), "the status to say 1 result")
            .await
            .unwrap();
        expect_active(page, &ids[0], "narrowing again").await;
        let remounted: bool = js(page, "!window.__row.isConnected").await;
        assert!(remounted, "the narrowed list kept its old row");
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        ran(page, "New file", "Enter on the remounted row").await;

        fixture.console.assert_clean("a narrowing query").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 509: Ctrl/Alt/Meta with an arrow is the search box's caret or the
/// browser's, never a highlight move.
#[test]
fn modifier_chords_leave_the_highlight() {
    block_on(async {
        let fixture = Fixture::open("/spotlight", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        open_by_keyboard(page).await;
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        wait::for_js_true(
            page,
            &format!("!!document.querySelector({SEARCH:?}).getAttribute('aria-activedescendant')"),
            "ArrowDown to highlight a row",
        )
        .await
        .unwrap();
        keyboard::assert_chords_ignored(
            page,
            &[
                keyboard::ARROW_DOWN,
                keyboard::ARROW_UP,
                keyboard::HOME,
                keyboard::END,
            ],
            &format!(
                "[document.querySelector({SEARCH:?}).getAttribute('aria-activedescendant'), \
                 document.activeElement === document.querySelector({SEARCH:?})]"
            ),
        )
        .await
        .unwrap();

        fixture.console.assert_clean("spotlight chords").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Ctrl+K and Cmd+K each open the palette with focus in its search box, and
/// the same chord from inside it closes it again.
#[test]
fn ctrl_or_cmd_k_toggles_the_palette() {
    block_on(async {
        let fixture = Fixture::open("/spotlight", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_selector(page, TRIGGER).await.unwrap();

        for (name, modifier) in [("Ctrl", keyboard::CTRL), ("Cmd", keyboard::META)] {
            keyboard::press_with(page, K, modifier).await.unwrap();
            wait::for_visible(page, DIALOG)
                .await
                .unwrap_or_else(|e| panic!("{name}+K to open the palette: {e}"));
            focus::wait_for_focus(page, SEARCH, &format!("{name}+K opening the palette"))
                .await
                .unwrap();

            keyboard::press_with(page, K, modifier).await.unwrap();
            wait::for_hidden(page, DIALOG)
                .await
                .unwrap_or_else(|e| panic!("{name}+K inside to close the palette: {e}"));
        }

        fixture
            .console
            .assert_clean("the palette's hotkey")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Each option is named by its label alone; the description and the shortcut
/// hint describe it rather than run on into its name.
#[test]
fn an_option_is_named_by_its_label() {
    block_on(async {
        let fixture = Fixture::open("/spotlight", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        open_by_keyboard(page).await;
        let tree = e2e::ax::snapshot(page, DIALOG).await.unwrap();
        for name in ["Home", "Changelog", "New file"] {
            assert!(
                tree.contains(&format!("option \"{name}\"")),
                "no option named {name:?} alone:\n{tree}"
            );
        }
        let described: Vec<String> = js(
            page,
            &format!(
                "[...document.querySelectorAll({OPTIONS:?})].map(o => (o.getAttribute('aria-describedby') ?? '') \
                 .split(' ').filter(Boolean).map(id => document.getElementById(id).textContent).join(' | '))"
            ),
        )
        .await;
        assert_eq!(described, ["The start page", "", "Ctrl + N"]);
        fixture
            .console
            .assert_clean("spotlight option names")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// A label with no break opportunity wraps inside its row at 320px (1.4.10).
#[test]
fn a_long_label_wraps_in_its_row() {
    use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
    block_on(async {
        let fixture = Fixture::open("/spotlight-long", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        page.execute(SetDeviceMetricsOverrideParams::new(320, 640, 1.0, true))
            .await
            .unwrap();
        open_by_keyboard(page).await;
        crate::floating_window::assert_fits_at_320(
            page,
            DIALOG,
            &format!(
                "[document.documentElement, document.querySelector({DIALOG:?}), ...document.querySelectorAll({OPTIONS:?})]"
            ),
        )
        .await;
        fixture
            .console
            .assert_clean("spotlight long label")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1307: a long result list leaves the search box and the last row reachable.
#[test]
fn a_tall_spotlight_stays_reachable() {
    block_on(async {
        e2e::browser::at_every_viewport(async |viewport| {
            let fixture = Fixture::open("/spotlight-tall", viewport).await.unwrap();
            let page = &fixture.page;
            keyboard::tab_to(page, TRIGGER, 3).await.unwrap();
            keyboard::press(page, keyboard::ENTER).await.unwrap();
            wait::for_visible(page, DIALOG).await.unwrap();
            let name = viewport.name();
            crate::modal::assert_reachable(
                page,
                "[...document.querySelectorAll('[role=dialog] [role=option]')].at(-1)",
                &format!("the last row at {name}"),
            )
            .await;
            crate::modal::assert_reachable(
                page,
                "document.querySelector('[role=dialog] input')",
                &format!("the search box at {name}"),
            )
            .await;
            fixture
                .console
                .assert_clean(&format!("/spotlight-tall at {name}"))
                .unwrap();
            fixture.close().await.unwrap();
        })
        .await;
    });
}
