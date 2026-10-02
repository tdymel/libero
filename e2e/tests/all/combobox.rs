//! Bare `Combobox`, its rows drawn by the caller with `ComboboxOption`, and
//! the probes the family's shared web/native scenarios read.

use anyhow::{Result, ensure};
use e2e::archetypes::Combobox;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused};
use e2e::passes::keyboard;
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, wait};

pub const TRIGGER: &str = "[role=combobox]";
pub const LISTBOX: &str = "[role=listbox]";

/// Todo 642: the selected-row bar, until now pinned only through `Select` and
/// `MultiSelect`.
#[test]
fn the_selected_row_shows_the_on_state_line() {
    crate::select::selected_row_is_marked("/combobox");
}

/// Axe, contrast, focus rings, target size and the open state, in both schemes and all viewports.
#[test]
fn it_meets_the_baseline() {
    Suite::new("combobox", "/combobox")
        .focusable(TRIGGER)
        .targets("[role=option]")
        .state(
            "open",
            &[Step::TabTo(TRIGGER), Step::Press(keyboard::ARROW_DOWN)],
            LISTBOX,
        )
        .run();
}

/// Keyboard, focus and the aria contract on the bare `Combobox`.
#[test]
fn it_honours_the_combobox_contract() {
    block_on(async {
        e2e::browser::at_every_viewport(async |viewport| {
            let fixture = Fixture::open("/combobox", viewport).await.unwrap();
            Combobox {
                trigger: TRIGGER,
                option_count: 5,
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

/// The highlight skips a refused row, and so must the trigger's `aria-activedescendant`.
#[test]
fn the_trigger_names_the_row_the_list_highlights() {
    the_trigger_names_the_lit_row("/combobox/refused", "Banana");
}

/// Opens `route`'s combobox by key; the trigger must name the lit row, `expected`, not a refused one.
pub(crate) fn the_trigger_names_the_lit_row(route: &str, expected: &str) {
    block_on(async {
        let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        wait::for_visible(page, LISTBOX).await.unwrap();
        let js = r#"(() => {
            const t = document.querySelector('[role=combobox]');
            const lit = document.querySelector('[role=option][data-state~=active]');
            const named = document.getElementById(t.getAttribute('aria-activedescendant'));
            return [named && named.textContent, named && named.getAttribute('aria-disabled'),
                    lit && lit.textContent];
        })()"#;
        let read: (Option<String>, Option<String>, Option<String>) =
            page.evaluate(js).await.unwrap().into_value().unwrap();
        assert_eq!(
            read.1, None,
            "{route}: the trigger names a refused row: {read:?}"
        );
        assert_eq!(read.0.as_deref(), Some(expected), "{route}: {read:?}");
        assert_eq!(
            read.0, read.2,
            "{route}: the named row is not the lit one: {read:?}"
        );
        fixture.close().await.unwrap();
    });
}

/// Under RTL the box is placed with the width it will have, so it stays on screen.
#[test]
fn an_rtl_dropdown_stays_inside_the_viewport() {
    block_on(async {
        e2e::browser::at_once(["/combobox", "/select"], async |route| {
            let fixture = Fixture::open(route, Viewport::Mobile).await.unwrap();
            let page = &fixture.page;
            page.evaluate("document.documentElement.dir = 'rtl'")
                .await
                .unwrap();
            wait::for_js_true(
                page,
                "getComputedStyle(document.body).direction === 'rtl'",
                "rtl",
            )
            .await
            .unwrap();
            keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
            keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
            wait::for_visible(page, LISTBOX).await.unwrap();
            let edges = "(() => { let p = document.querySelector('[role=listbox]'); \
                 while (p && getComputedStyle(p).position !== 'fixed') p = p.parentElement; \
                 const r = p.getBoundingClientRect(); return [r.left, r.right, innerWidth]; })()";
            // Until placed, in place of a 500 ms sleep: the placement may take a frame or two.
            let inside = format!("(([l, r, w]) => l >= 0 && r <= w + 0.5)({edges})");
            if wait::for_js_true(page, &inside, "the dropdown inside the viewport")
                .await
                .is_err()
            {
                let (left, right, width): (f64, f64, f64) =
                    page.evaluate(edges).await.unwrap().into_value().unwrap();
                panic!("{route}: the dropdown spans {left}..{right} in a {width}px viewport");
            }
            fixture.close().await.unwrap();
        })
        .await;
    });
}

/// Todo 1269: an open list with no options draws and says its empty label, so it is expanded.
#[test]
fn an_empty_open_list_says_its_empty_label() {
    block_on(async {
        let fixture = Fixture::open("/combobox/none", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        wait::for_js_true(
            page,
            "[...document.querySelectorAll('[role=status]')].some(s => s.textContent === 'No results') \
             && document.querySelector('[data-slot=nothing-found]')?.textContent === 'No results'",
            "the empty label drawn and said",
        )
        .await
        .unwrap();
        let expanded: Option<String> = page
            .evaluate("document.querySelector('[role=combobox]').getAttribute('aria-expanded')")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(expanded.as_deref(), Some("true"), "a drawn empty label");
        fixture.close().await.unwrap();
    });
}

pub async fn is_expanded<D: Driver>(d: &mut D) -> Result<bool> {
    Ok(d.attr(TRIGGER, "aria-expanded").await?.as_deref() == Some("true"))
}

pub async fn until_expanded<D: Driver>(d: &mut D, open: bool, after: &str) -> Result<()> {
    eventually(d, &format!("{after}: expanded={open}"), async |d| {
        Ok(is_expanded(d).await? == open)
    })
    .await
}

/// The fixture's `#echo` reads `expected`.
pub async fn echoes<D: Driver>(d: &mut D, expected: &str, after: &str) -> Result<()> {
    eventually(
        d,
        &format!("{after}: #echo to read {expected:?}"),
        async |d| Ok(d.text("#echo").await? == expected),
    )
    .await
}

async fn listbox_shows<D: Driver>(d: &mut D, after: &str) -> Result<()> {
    eventually(d, &format!("{after}: the listbox to show"), async |d| {
        d.exists(LISTBOX).await
    })
    .await
}

// `Autocomplete`, on `/autocomplete/echo`.

async fn autocomplete_typing_and_enter<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(TRIGGER).await?;
    d.type_text("be").await?;
    until_expanded(d, true, "typing \"be\"").await?;
    eventually(d, "\"be\" to leave one option", async |d| {
        Ok(d.exists("[role=option]").await? && !d.exists("[role=option]:nth-child(2)").await?)
    })
    .await?;
    d.press(keyboard::ARROW_DOWN).await?;
    d.press(keyboard::ENTER).await?;
    echoes(d, "Berlin", "Enter on the option").await?;
    until_expanded(d, false, "the pick").await
}

async fn autocomplete_escape<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(TRIGGER).await?;
    d.press(keyboard::ARROW_DOWN).await?;
    listbox_shows(d, "ArrowDown").await?;
    d.press(keyboard::ESCAPE).await?;
    until_expanded(d, false, "Escape").await?;
    eventually_focused(d, TRIGGER, "Escape").await
}

async fn autocomplete_click_picks<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(TRIGGER).await?;
    d.press(keyboard::ARROW_DOWN).await?;
    listbox_shows(d, "ArrowDown").await?;
    d.click("[role=option]:nth-child(3)").await?;
    echoes(d, "Copenhagen", "a click on the third option").await
}

e2e::scenario!(
    typing_filters_an_autocomplete_and_enter_picks_the_option,
    "/autocomplete/echo",
    autocomplete_typing_and_enter
);
e2e::scenario!(
    escape_closes_an_open_autocomplete,
    "/autocomplete/echo",
    autocomplete_escape
);
e2e::scenario!(
    a_click_on_an_autocomplete_option_picks_it,
    "/autocomplete/echo",
    autocomplete_click_picks
);

// `MultiSelect`, on `/multi-select/echo`: Cherry held.

async fn multi_keys_add<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(TRIGGER).await?;
    // Opens on the held Cherry: Enter drops it, the list stays open.
    d.press(keyboard::ARROW_DOWN).await?;
    listbox_shows(d, "ArrowDown").await?;
    d.press(keyboard::ENTER).await?;
    echoes(d, "[]", "Enter on the held Cherry").await?;
    ensure!(is_expanded(d).await?, "Enter closed the multi-select");
    d.press(keyboard::ARROW_UP).await?;
    d.press(keyboard::ENTER).await?;
    echoes(d, "[Banana]", "ArrowUp, Enter").await
}

async fn multi_click_toggles<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(TRIGGER).await?;
    listbox_shows(d, "a click on the trigger").await?;
    d.click("[role=option]:nth-child(2)").await?;
    echoes(d, "[Cherry, Banana]", "a click on Banana").await
}

/// The rows cancel `mousedown` to keep focus on the trigger. Blitz moves focus anyway;
/// libero moves it back and the trigger ignores that blur (472).
async fn multi_click_keeps_open<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(TRIGGER).await?;
    listbox_shows(d, "a click on the trigger").await?;
    d.click("[role=option]:nth-child(2)").await?;
    echoes(d, "[Cherry, Banana]", "a click on Banana").await?;
    ensure!(
        is_expanded(d).await?,
        "a click on an option closed the list"
    );
    eventually_focused(d, TRIGGER, "a click on an option").await
}

async fn multi_backspace_drops<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(TRIGGER).await?;
    d.press(keyboard::BACKSPACE).await?;
    echoes(d, "[]", "Backspace").await
}

e2e::scenario!(
    the_keyboard_adds_a_multi_select_option,
    "/multi-select/echo",
    multi_keys_add
);
e2e::scenario!(
    a_click_on_a_multi_select_option_toggles_it,
    "/multi-select/echo",
    multi_click_toggles
);
e2e::scenario!(
    a_click_on_a_multi_select_option_keeps_the_list_open,
    "/multi-select/echo",
    multi_click_keeps_open
);
e2e::scenario!(
    backspace_in_an_empty_multi_select_drops_the_last_pick,
    "/multi-select/echo",
    multi_backspace_drops
);

// `TagsField`, on `/tags-field/echo`: "rust" held.

async fn tags_typing_adds<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(TRIGGER).await?;
    d.type_text("css").await?;
    d.press(keyboard::ENTER).await?;
    echoes(d, r#"["rust", "css"]"#, "typing css, Enter").await
}

async fn tags_suggestion_adds<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(TRIGGER).await?;
    d.press(keyboard::ARROW_DOWN).await?;
    listbox_shows(d, "ArrowDown").await?;
    d.press(keyboard::ENTER).await?;
    echoes(d, r#"["rust", "dioxus"]"#, "Enter on the first suggestion").await
}

async fn tags_backspace_removes<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(TRIGGER).await?;
    d.press(keyboard::BACKSPACE).await?;
    d.press(keyboard::BACKSPACE).await?;
    echoes(d, "[]", "Backspace twice").await
}

e2e::scenario!(
    typing_and_enter_add_a_tag,
    "/tags-field/echo",
    tags_typing_adds
);
e2e::scenario!(
    a_suggestion_picked_by_the_keys_becomes_a_tag,
    "/tags-field/echo",
    tags_suggestion_adds
);
e2e::scenario!(
    backspace_in_an_empty_tags_field_removes_the_last_tag,
    "/tags-field/echo",
    tags_backspace_removes
);

// `Cascader`, on `/cascader`: Europe, Asia (disabled), Oceania; `#picked`.

async fn picked<D: Driver>(d: &mut D, expected: &str, after: &str) -> Result<()> {
    eventually(
        d,
        &format!("{after}: #picked to read {expected}"),
        async |d| Ok(d.text("#picked").await? == expected),
    )
    .await
}

async fn cascader_keys<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(TRIGGER).await?;
    d.press(keyboard::ARROW_DOWN).await?;
    until_expanded(d, true, "ArrowDown").await?;
    // Europe -> France -> Paris.
    d.press(keyboard::ARROW_RIGHT).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    d.press(keyboard::ENTER).await?;
    picked(d, "paris", "Right, Right, Enter").await
}

/// A click on a branch opens its column. Natively the press blurs the
/// trigger all the same, which must not close the list (todo 472).
async fn cascader_clicks<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(TRIGGER).await?;
    until_expanded(d, true, "a click on the trigger").await?;
    d.click("[id$=-option-0-2]").await?;
    eventually(d, "Oceania's column", async |d| {
        d.exists("[id$=-option-1-0]").await
    })
    .await?;
    ensure!(
        is_expanded(d).await?,
        "the click on Oceania closed the list"
    );
    d.click("[id$=-option-1-0]").await?;
    eventually(d, "Australia's column", async |d| {
        d.exists("[id$=-option-2-0]").await
    })
    .await?;
    d.click("[id$=-option-2-0]").await?;
    picked(d, "sydney", "clicks on Oceania, Australia, Sydney").await
}

e2e::scenario!(
    the_keys_walk_a_cascader_to_a_leaf,
    "/cascader",
    cascader_keys
);
e2e::scenario!(
    clicks_walk_a_cascader_to_a_leaf,
    "/cascader",
    cascader_clicks
);
