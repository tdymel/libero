//! `Menu`: the overlay archetype as a non-modal popup. Tab closes every level and moves on
//! from the trigger.

use anyhow::Result;
use e2e::archetypes::Overlay;
use e2e::browser::block_on;
use e2e::driver::{Driver, Platform, eventually, eventually_focused, eventually_text};
use e2e::passes::pointer::tap;
use e2e::suite::Step;
use e2e::wait::{ElementBounded, PageBounded};
use e2e::{
    Fixture, Suite, Viewport,
    passes::{keyboard, pointer},
    wait,
};

use crate::settle::painted;

/// `a11y_attributes()` gives the trigger a generated id, so it is found by
/// the attribute that makes it a menu button instead.
pub const TRIGGER: &str = "[aria-haspopup=menu]";
const MENU: &str = "[role=menu]";

#[test]
fn it_meets_the_baseline() {
    Suite::new("menu", "/menu")
        .focusable(TRIGGER)
        .targets("[role=menu] [role=menuitem]")
        .state(
            "open",
            &[Step::TabTo(TRIGGER), Step::Press(keyboard::ARROW_DOWN)],
            MENU,
        )
        // Share, the last item, opens its submenu (todo 1773).
        .state(
            "submenu",
            &[
                Step::Press(keyboard::END),
                Step::Press(keyboard::ARROW_RIGHT),
            ],
            "[role=menuitem][aria-haspopup=menu][aria-expanded=true]",
        )
        .run();
}

/// Radio and checkbox items with shortcuts, a long label and a disabled one (todo 1773).
#[test]
fn the_choices_menu_meets_the_baseline() {
    Suite::new("menu-choices", "/menu-choices")
        .focusable(TRIGGER)
        .targets("[role=menu] [role^=menuitem]")
        .state(
            "open",
            &[Step::TabTo(TRIGGER), Step::Press(keyboard::ARROW_DOWN)],
            MENU,
        )
        .run();
}

#[test]
fn it_honours_the_overlay_contract() {
    block_on(async {
        e2e::browser::at_every_viewport(async |viewport| {
            let fixture = Fixture::open("/menu", viewport).await.unwrap();

            Overlay {
                trigger: TRIGGER,
                panel: MENU,
                traps_focus: false,
                tab_budget: 5,
            }
            .assert_contract(&fixture.page)
            .await
            .unwrap_or_else(|e| panic!("at {}: {e}", viewport.name()));

            fixture
                .console
                .assert_clean(&format!("the menu contract at {}", viewport.name()))
                .unwrap();
            fixture.close().await.unwrap();
        })
        .await;
    });
}

async fn expanded<D: Driver>(d: &mut D, open: bool) -> Result<()> {
    let what = if open {
        "the menu to open"
    } else {
        "the menu to close"
    };
    eventually(d, what, async |d| {
        Ok((d.attr(TRIGGER, "aria-expanded").await?.as_deref() == Some("true")) == open)
    })
    .await
}

/// APG's menu button: Enter opens it onto an item, Escape closes it and hands
/// focus back to the trigger.
async fn enter_opens_and_escape_closes<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(TRIGGER).await?;
    d.press(keyboard::ENTER).await?;
    expanded(d, true).await?;
    // The item takes focus once the box is placed, a timer later.
    eventually_focused(d, "[role=menuitem]", "Enter on the trigger").await?;
    d.press(keyboard::ESCAPE).await?;
    expanded(d, false).await?;
    eventually_focused(d, TRIGGER, "Escape").await
}

/// APG typeahead: a typed letter moves focus to the next item it starts (todo 1405).
async fn a_typed_letter_moves_to_its_item<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(TRIGGER).await?;
    d.press(keyboard::ENTER).await?;
    expanded(d, true).await?;
    eventually_focused(
        d,
        "[role=menu] [data-menu-index=\"0\"]",
        "Enter on the trigger",
    )
    .await?;
    d.type_text("s").await?;
    eventually_focused(d, "[role=menu] [data-menu-index=\"1\"]", "s to Save").await
}

e2e::scenario!(
    a_typed_letter_moves_focus_to_the_item_it_starts,
    "/menu-keep-open",
    a_typed_letter_moves_to_its_item
);

/// A click outside closes it and leaves focus where the click put it.
async fn a_click_outside_closes<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(TRIGGER).await?;
    d.press(keyboard::ENTER).await?;
    expanded(d, true).await?;
    d.click("#rename").await?;
    expanded(d, false).await?;
    eventually_focused(d, "#rename", "a click outside").await
}

e2e::scenario!(
    enter_opens_it_and_escape_closes_it_with_focus_back_on_the_trigger,
    "/menu-submenu-reopen",
    enter_opens_and_escape_closes
);
/// `use_dismiss` (todo 46): focus moving from the trigger into the list keeps
/// it open.
async fn a_click_opens<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(TRIGGER).await?;
    expanded(d, true).await?;
    eventually_focused(d, "[role=menuitem]", "a click on the trigger").await?;
    expanded(d, true).await
}

e2e::scenario!(
    a_click_opens_it_and_focus_moving_into_the_list_keeps_it_open,
    "/menu-submenu-reopen",
    a_click_opens
);
e2e::scenario!(
    a_click_outside_closes_it_and_leaves_focus_where_it_went,
    "/menu-submenu-reopen",
    a_click_outside_closes,
    macos: skip("upstream: WebKit does not focus a clicked button"),
    ios: skip("upstream: WebKit does not focus a clicked button")
);

/// APG: Escape and ArrowLeft in a submenu close that level only and focus its
/// parent item (1812; `rtl_keys` covers the mirror).
async fn a_key_closes_only_the_submenu<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const SHARE: &str = "[role=menu] [role=menuitem][aria-haspopup=menu]";
    const SUBMENU_ITEM: &str = "[role=menu][aria-labelledby$='-item-0'] [role=menuitem]";
    for (key, name) in [
        (keyboard::ESCAPE, "Escape"),
        (keyboard::ARROW_LEFT, "ArrowLeft"),
    ] {
        d.focus(TRIGGER).await?;
        d.press(keyboard::ENTER).await?;
        expanded(d, true).await?;
        eventually_focused(d, SHARE, "Enter on the trigger").await?;
        d.press(keyboard::ARROW_RIGHT).await?;
        eventually_focused(d, SUBMENU_ITEM, "ArrowRight on Share").await?;
        d.press(key).await?;
        eventually(d, &format!("{name} to close the submenu"), async |d| {
            Ok(d.attr(SHARE, "aria-expanded").await?.as_deref() != Some("true"))
        })
        .await?;
        eventually_focused(d, SHARE, name).await?;
        expanded(d, true).await?;
        d.press(keyboard::ESCAPE).await?;
        expanded(d, false).await?;
        eventually_focused(d, TRIGGER, "Escape on the root").await?;
    }
    Ok(())
}

e2e::scenario!(
    escape_and_arrow_left_close_only_the_submenu,
    "/menu-submenu-reopen",
    a_key_closes_only_the_submenu
);

/// `level` is open and placed: an unplaced box sits hidden at 0,0.
async fn placed<D: Driver>(d: &mut D, level: &str) -> Result<()> {
    eventually(d, &format!("{level} to open"), async |d| {
        Ok(d.exists(level).await?
            && d.style(level, "visibility").await.unwrap_or_default() == "visible")
    })
    .await
}

/// Todo 1019: a press opens it, and a press outside, on another control or on
/// nothing, closes it. Android's WebView cannot tell where focus went.
async fn a_press_outside_closes<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(TRIGGER).await?;
    expanded(d, true).await?;
    d.click("#rename").await?;
    expanded(d, false).await?;

    // Below the open menu, clear of Android's gesture strip at the very bottom.
    d.click(TRIGGER).await?;
    expanded(d, true).await?;
    let (width, height) = d.viewport().await?;
    d.click_at(width / 2.0, height * 0.75).await?;
    expanded(d, false).await?;

    // A press on the menu itself keeps it open; the trigger toggles it shut.
    d.click(TRIGGER).await?;
    placed(d, MENU).await?;
    d.click(MENU).await?;
    expanded(d, true).await?;
    // Behind the menu's press: had that closed the menu, this one reopens it.
    d.click(TRIGGER).await?;
    expanded(d, false).await?;

    // A press in the portaled submenu is inside too: its item is chosen.
    d.click(TRIGGER).await?;
    placed(d, MENU).await?;
    d.click(MENU).await?;
    let submenu = "[role=menu][aria-labelledby$='-item-0']";
    placed(d, submenu).await?;
    d.click(&format!("{submenu} [role=menuitem]")).await?;
    // `#rename` above renamed the item.
    eventually_text(d, "#picked", "Post", "a press on the submenu's item").await?;
    expanded(d, false).await
}

e2e::scenario!(
    a_press_outside_closes_it,
    "/menu-submenu-reopen",
    a_press_outside_closes
);

/// The docs demo's row: the trigger and the text beside it share a centre line.
#[test]
fn the_trigger_centres_beside_its_text() {
    block_on(async {
        let fixture = Fixture::open("/menu-row", Viewport::Desktop).await.unwrap();
        let offset: f64 = fixture
            .page
            .evaluate_bounded(format!(
                "(() => {{ const mid = e => {{ const r = e.getBoundingClientRect(); return r.top + r.height / 2; }}; \
                 const text = document.querySelector('#menu-row-text'); \
                 const range = document.createRange(); range.selectNodeContents(text); \
                 const line = range.getBoundingClientRect(); \
                 return Math.abs(mid(document.querySelector('{TRIGGER}')) - (line.top + line.height / 2)); }})()"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(offset <= 2.0, "trigger and text centres {offset}px apart");
        fixture.close().await.unwrap();
    });
}

/// Ctrl/Alt/Meta chords on an item are the browser's: no move, no submenu.
#[test]
fn modifier_chords_go_to_the_browser() {
    block_on(async {
        let fixture = Fixture::open("/menu", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement?.getAttribute('role') === 'menuitem'",
            "focus on the first item",
        )
        .await
        .unwrap();
        keyboard::assert_chords_ignored(
            page,
            &[
                keyboard::ARROW_DOWN,
                keyboard::ARROW_UP,
                keyboard::ARROW_RIGHT,
                keyboard::ARROW_LEFT,
                keyboard::HOME,
                keyboard::END,
            ],
            "[document.activeElement.textContent, document.querySelectorAll('[role=menu]').length]",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("menu chords").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 408: the open request is older than the menu, so only its first
/// placement can move focus onto the first item.
#[test]
fn a_menu_opened_before_it_mounts_focuses_its_first_item() {
    block_on(async {
        let fixture = Fixture::open("/menu-open-on-mount", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_visible(&fixture.page, MENU).await.unwrap();
        let outcome = wait::for_js_true(
            &fixture.page,
            "document.activeElement?.textContent.trim() === 'Save'",
            "focus on the first item",
        )
        .await;

        fixture.close().await.unwrap();
        outcome.unwrap();
    });
}

/// Todo 1789: an open menu with no items draws nothing; items arriving show
/// it and focus moves in as on any open.
#[test]
fn items_arriving_in_an_open_menu_show_it_and_take_focus() {
    block_on(async {
        let fixture = Fixture::open("/menu-items-late", Viewport::Desktop)
            .await
            .unwrap();
        let outcome = async {
            fixture
                .page
                .find_element_bounded(TRIGGER)
                .await?
                .click_bounded()
                .await?;
            wait::for_visible(&fixture.page, MENU).await?;
            wait::for_js_true(
                &fixture.page,
                "document.activeElement?.textContent.trim() === 'Save' && \
                 document.querySelector('[aria-haspopup=menu]').getAttribute('aria-expanded') === 'true'",
                "focus on the first item and an expanded trigger",
            )
            .await
        }
        .await;

        fixture.close().await.unwrap();
        outcome.unwrap();
    });
}

/// Todo 3554, 3617: the focused row is past the end of a shrunk list; the menu stays open,
/// focus moves to the last row and it holds the tab stop.
#[test]
fn a_shrunk_list_keeps_a_row_with_tabindex_zero() {
    block_on(async {
        let fixture = Fixture::open("/menu-items-shrink", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let outcome = async {
            pointer::click(page, TRIGGER).await?;
            wait::for_js_true(
                page,
                "document.activeElement?.textContent.trim() === 'Save'",
                "focus on the first item",
            )
            .await?;
            keyboard::press(page, keyboard::END).await?;
            wait::for_js_true(
                page,
                "document.activeElement?.textContent.trim() === 'Shrink'",
                "focus on the last item",
            )
            .await?;
            keyboard::press(page, keyboard::ENTER).await?;
            wait::for_js_true(
                page,
                "document.querySelectorAll('[role=menuitem]').length === 1 && \
                 document.querySelectorAll('[role=menuitem][tabindex=\"0\"]').length === 1 && \
                 document.activeElement === document.querySelector('[role=menuitem]')",
                "the menu open on one item holding the tab stop and focus",
            )
            .await
        }
        .await;

        fixture.close().await.unwrap();
        outcome.unwrap();
    });
}

/// Todo 29: a closed menu unmounts its submenu levels, so a reopen starts
/// them closed, working, and on the items changed while closed.
#[test]
fn a_reopened_menu_starts_its_submenu_closed_on_fresh_items() {
    block_on(async {
        let fixture = Fixture::open("/menu-submenu-reopen", Viewport::Desktop)
            .await
            .unwrap();
        let outcome = reopen_the_submenu(&fixture.page).await;
        let console = fixture.console.assert_clean("the submenu reopen");

        fixture.close().await.unwrap();
        outcome.unwrap();
        console.unwrap();
    });
}

/// Todo 4403: a key or a close within the submenu delay drops the hover's pending open, so
/// it neither pulls focus back to Share nor leaves its submenu open on the next open.
#[test]
fn keys_and_a_close_drop_a_pending_hover_open() {
    const SHARE: &str = "[role=menuitem][aria-haspopup=menu]";
    const SUBMENU_DELAY: u32 = 150;
    block_on(async {
        let fixture = Fixture::open("/menu", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        let focus_on =
            |text: &str| format!("document.activeElement?.textContent.trim() === '{text}'");
        let collapsed =
            format!("document.querySelector('{SHARE}')?.getAttribute('aria-expanded') !== 'true'");
        let fire = format!("window.__heldClock.fireAll({SUBMENU_DELAY})");
        let outcome = async {
            e2e::clock::hold(page, &[SUBMENU_DELAY]).await?;
            pointer::click(page, TRIGGER).await?;
            wait::for_js_true(page, &focus_on("Cut"), "focus on Cut").await?;
            pointer::hover(page, SHARE).await?;
            wait::for_js_true(page, &focus_on("Share"), "the hover to focus Share").await?;
            keyboard::press(page, keyboard::ARROW_UP).await?;
            wait::for_js_true(page, &focus_on("Save"), "ArrowUp to Save").await?;
            page.evaluate_bounded(fire.clone()).await?;
            painted(page).await?;
            anyhow::ensure!(
                page.evaluate_bounded(format!("{} && {collapsed}", focus_on("Save")))
                    .await?
                    .into_value::<bool>()?,
                "the hover's timer pulled focus back to Share or opened its submenu"
            );

            pointer::move_to(page, pointer::Point { x: 2.0, y: 2.0 }).await?;
            pointer::hover(page, SHARE).await?;
            wait::for_js_true(page, &focus_on("Share"), "the hover to focus Share").await?;
            keyboard::press(page, keyboard::ESCAPE).await?;
            wait::for_js_true(page, &focus_on("Actions"), "Escape to close the menu").await?;
            page.evaluate_bounded(fire.clone()).await?;
            pointer::click(page, TRIGGER).await?;
            wait::for_js_true(page, &focus_on("Cut"), "focus on Cut again").await?;
            painted(page).await?;
            anyhow::ensure!(
                page.evaluate_bounded(collapsed.clone())
                    .await?
                    .into_value::<bool>()?,
                "the reopened menu showed Share's submenu open"
            );
            Ok::<_, anyhow::Error>(())
        }
        .await;
        let console = fixture.console.assert_clean("a pending hover open");

        fixture.close().await.unwrap();
        outcome.unwrap();
        console.unwrap();
    });
}

/// Todo 4423: a key in a submenu drops the pending hover of the level above, so the row
/// the pointer rests on neither takes focus nor closes the submenu.
#[test]
fn a_key_in_a_submenu_drops_the_pending_hover_above() {
    const SHARE: &str = "[role=menuitem][aria-haspopup=menu]";
    const SAVE: &str = "[role=menu] [data-menu-index=\"2\"]";
    const SUBMENU_DELAY: u32 = 150;
    block_on(async {
        let fixture = Fixture::open("/menu", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        let focus_on =
            |text: &str| format!("document.activeElement?.textContent.trim() === '{text}'");
        let expanded =
            format!("document.querySelector('{SHARE}')?.getAttribute('aria-expanded') === 'true'");
        let outcome = async {
            e2e::clock::hold(page, &[SUBMENU_DELAY]).await?;
            pointer::click(page, TRIGGER).await?;
            wait::for_js_true(page, &focus_on("Cut"), "focus on Cut").await?;
            pointer::hover(page, SHARE).await?;
            wait::for_js_true(page, &focus_on("Share"), "the hover to focus Share").await?;
            keyboard::press(page, keyboard::ARROW_RIGHT).await?;
            wait::for_js_true(page, &focus_on("Email"), "ArrowRight into the submenu").await?;
            anyhow::ensure!(
                page.evaluate_bounded(format!(
                    "document.querySelector({SAVE:?})?.textContent.trim() === 'Save'"
                ))
                .await?
                .into_value::<bool>()?,
                "row 2 is not Save"
            );
            // Resting on Save schedules focusing it and closing Share's submenu.
            pointer::hover(page, SAVE).await?;
            painted(page).await?;
            keyboard::press(page, keyboard::ARROW_DOWN).await?;
            painted(page).await?;
            page.evaluate_bounded(format!("window.__heldClock.fireAll({SUBMENU_DELAY})"))
                .await?;
            painted(page).await?;
            anyhow::ensure!(
                page.evaluate_bounded(format!("{} && {expanded}", focus_on("Email")))
                    .await?
                    .into_value::<bool>()?,
                "the hover above took focus to Save or closed the submenu"
            );
            Ok::<_, anyhow::Error>(())
        }
        .await;
        let console = fixture.console.assert_clean("a pending hover above");

        fixture.close().await.unwrap();
        outcome.unwrap();
        console.unwrap();
    });
}

/// `parts` reaches the labels on the submenu level too, `sx` its box.
#[test]
fn parts_and_sx_style_every_level() {
    block_on(async {
        let fixture = Fixture::open("/menu-parts", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let outcome = async {
            pointer::click(page, TRIGGER).await?;
            wait::for_js_true(
                page,
                "document.activeElement?.textContent.trim() === 'Share'",
                "focus on Share",
            )
            .await?;
            keyboard::press(page, keyboard::ARROW_RIGHT).await?;
            wait::for_js_true(
                page,
                "document.activeElement?.textContent.trim() === 'Email'",
                "focus on Email",
            )
            .await?;
            let [italic, spacing]: [Vec<String>; 2] = page
                .evaluate_bounded(
                    "(() => { \
                       const labels = [...document.querySelectorAll('[role=menu] [data-slot=label]')]; \
                       const menus = [...document.querySelectorAll('[role=menu]')]; \
                       return [ \
                         labels.map(l => getComputedStyle(l).fontStyle), \
                         menus.map(m => getComputedStyle(m).letterSpacing), \
                       ]; })()",
                )
                .await?
                .into_value()?;
            anyhow::ensure!(
                italic == ["italic", "italic"],
                "a level missed the parts: {italic:?}"
            );
            anyhow::ensure!(
                spacing == ["2px", "2px"],
                "a level missed the sx: {spacing:?}"
            );
            Ok::<_, anyhow::Error>(())
        }
        .await;
        let console = fixture.console.assert_clean("menu parts");

        fixture.close().await.unwrap();
        outcome.unwrap();
        console.unwrap();
    });
}

async fn reopen_the_submenu(page: &chromiumoxide::Page) -> anyhow::Result<()> {
    const SHARE: &str = "[role=menuitem][aria-haspopup=menu]";
    // A closed level may stay in the DOM, hidden.
    let shown = format!(
        "[...document.querySelectorAll('{MENU}')].filter(m => {{ \
         const s = getComputedStyle(m); return s.visibility !== 'hidden' && s.display !== 'none'; }})"
    );
    let menus_read = |text: &str| {
        format!(
            "(() => {{ const m = {shown}; return m.length === 2 && m[1].textContent.includes('{text}'); }})()"
        )
    };
    let focus_on = |text: &str| format!("document.activeElement?.textContent.trim() === '{text}'");

    pointer::click(page, TRIGGER).await?;
    wait::for_js_true(page, &focus_on("Share"), "focus on Share").await?;
    keyboard::press(page, keyboard::ARROW_RIGHT).await?;
    wait::for_js_true(page, &menus_read("Email"), "the submenu open").await?;

    // An outside press closes the root with its submenu open.
    pointer::click(page, "#rename").await?;
    let closed = format!("{shown}.length === 0");
    wait::for_js_true(page, &closed, "every level closed").await?;

    pointer::click(page, TRIGGER).await?;
    wait::for_js_true(page, &focus_on("Share"), "focus on Share again").await?;
    let reopened = format!(
        "{shown}.length === 1 \
         && document.querySelector('{SHARE}').getAttribute('aria-expanded') !== 'true'"
    );
    wait::for_js_true(page, &reopened, "the submenu closed on reopen").await?;

    keyboard::press(page, keyboard::ARROW_RIGHT).await?;
    wait::for_js_true(page, &menus_read("Post"), "the renamed item").await?;
    wait::for_js_true(page, &focus_on("Post"), "focus on Post").await?;
    keyboard::press(page, keyboard::ENTER).await?;
    wait::for_js_true(
        page,
        "document.querySelector('#picked').textContent === 'Post'",
        "the renamed item picked",
    )
    .await
}

const FOCUS_ON_CUT: &str = "document.activeElement?.textContent.trim() === 'Cut'";

/// Todo 449: a click on a group's name focuses the menu box itself, which
/// answered no key, so the arrows were dead until Tab.
#[test]
fn the_arrows_work_after_a_click_on_a_group_name() {
    block_on(async {
        let fixture = Fixture::open("/menu", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        let outcome = async {
            pointer::click(page, TRIGGER).await?;
            wait::for_js_true(page, FOCUS_ON_CUT, "focus on Cut").await?;
            pointer::click(page, "[data-slot=group-label]").await?;
            wait::for_js_true(
                page,
                "document.activeElement?.getAttribute('role') === 'menu'",
                "focus on the menu box",
            )
            .await?;
            keyboard::press(page, keyboard::ARROW_DOWN).await?;
            wait::for_js_true(page, FOCUS_ON_CUT, "ArrowDown onto the first item").await?;
            keyboard::press(page, keyboard::END).await?;
            wait::for_js_true(
                page,
                "document.activeElement?.textContent.trim() === 'Share'",
                "End onto the last item",
            )
            .await
        }
        .await;

        fixture.close().await.unwrap();
        outcome.unwrap();
    });
}

/// Todo 449: Alt+ArrowDown on the trigger opened the menu, default prevented.
/// Alt+ArrowLeft (Back) in a submenu must not close it either.
#[test]
fn chords_on_the_trigger_and_in_a_submenu_go_to_the_browser() {
    block_on(async {
        let fixture = Fixture::open("/menu", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        page.evaluate_bounded(
            "window.__prevented = []; \
             window.addEventListener('keydown', e => { if (e.altKey) window.__prevented.push(e.defaultPrevented) })",
        )
        .await
        .unwrap();
        let focus_on =
            |text: &str| format!("document.activeElement?.textContent.trim() === '{text}'");
        let outcome = async {
            keyboard::tab_to(page, TRIGGER, 10).await?;
            keyboard::press_with(page, keyboard::ARROW_DOWN, keyboard::ALT).await?;
            // Opening would have focused an item by now.
            wait::for_js_true(page, "window.__prevented.length === 1", "the chord").await?;
            painted(page).await?;
            wait::for_js_true(page, &focus_on("Actions"), "the menu left closed").await?;

            keyboard::press(page, keyboard::ARROW_UP).await?;
            wait::for_js_true(page, &focus_on("Share"), "focus on Share").await?;
            keyboard::press(page, keyboard::ARROW_RIGHT).await?;
            wait::for_js_true(page, &focus_on("Email"), "focus in the submenu").await?;
            keyboard::press_with(page, keyboard::ARROW_LEFT, keyboard::ALT).await?;
            wait::for_js_true(page, "window.__prevented.length === 2", "the chord").await?;
            painted(page).await?;
            wait::for_js_true(page, &focus_on("Email"), "the submenu left open").await?;
            wait::for_js_true(
                page,
                "window.__prevented.length === 2 && !window.__prevented.some(Boolean)",
                "no chord's default prevented",
            )
            .await
        }
        .await;

        fixture.close().await.unwrap();
        outcome.unwrap();
    });
}

/// Todo 449, 1.4.10: a label wider than a phone was cut off with an ellipsis.
#[test]
fn a_long_label_wraps_inside_the_viewport() {
    block_on(async {
        let fixture = Fixture::open("/menu-choices", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        let outcome = async {
            pointer::click(page, TRIGGER).await?;
            wait::for_visible(page, MENU).await?;
            wait::for_js_true(
                page,
                "(() => { \
                   const menu = document.querySelector('[role=menu]').getBoundingClientRect(); \
                   const labels = [...document.querySelectorAll('[data-slot=label]')]; \
                   return menu.right <= innerWidth \
                     && document.documentElement.scrollWidth <= innerWidth \
                     && labels.every(l => l.scrollWidth <= l.clientWidth); \
                 })()",
                "every label shown whole inside the viewport",
            )
            .await
        }
        .await;

        fixture.close().await.unwrap();
        outcome.unwrap();
    });
}

/// Todo 3641: a submenu with no room on either side overlapped the viewport edge,
/// squeezed so its rows wrapped and ran past the bottom.
#[test]
fn a_submenu_without_side_room_stays_whole_inside_the_viewport() {
    block_on(async {
        let fixture = Fixture::open("/menu-submenu-low", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        let outcome = async {
            pointer::click(page, TRIGGER).await?;
            wait::for_visible(page, MENU).await?;
            keyboard::press(page, keyboard::END).await?;
            keyboard::press(page, keyboard::ARROW_RIGHT).await?;
            wait::for_js_true(
                page,
                "(() => { \
                   const menus = document.querySelectorAll('[role=menu]'); \
                   if (menus.length !== 2 || getComputedStyle(menus[1]).visibility !== 'visible') return false; \
                   const box = menus[1].getBoundingClientRect(); \
                   const share = document.activeElement.closest('[role=menu]') === menus[1] \
                     && document.querySelector('[role=menuitem][aria-expanded=true]'); \
                   const rows = [...menus[1].querySelectorAll('[role=menuitem]')] \
                     .map(r => r.getBoundingClientRect().height); \
                   return share && box.left >= 0 && box.right <= innerWidth \
                     && box.top >= 0 && box.bottom <= innerHeight \
                     && rows.length === 3 \
                     && rows.every(h => Math.abs(h - share.getBoundingClientRect().height) < 1); \
                 })()",
                "the submenu inside the viewport, its rows as tall as Share",
            )
            .await
        }
        .await;

        fixture.close().await.unwrap();
        outcome.unwrap();
    });
}

/// A checked radio item is where the menu opens, and Space picks another.
#[test]
fn a_menu_of_choices_opens_on_the_checked_one() {
    block_on(async {
        let fixture = Fixture::open("/menu-choices", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let outcome = async {
            pointer::click(page, TRIGGER).await?;
            wait::for_js_true(
                page,
                "document.activeElement?.getAttribute('role') === 'menuitemradio' \
                 && document.activeElement.getAttribute('aria-checked') === 'true' \
                 && document.activeElement.textContent.trim() === 'Name'",
                "focus on the checked Name",
            )
            .await?;
            keyboard::press(page, keyboard::ARROW_DOWN).await?;
            keyboard::press(page, keyboard::SPACE).await?;
            wait::for_js_true(
                page,
                "document.querySelector('#sort').textContent === 'Date'",
                "Date picked",
            )
            .await
        }
        .await;

        fixture.close().await.unwrap();
        outcome.unwrap();
    });
}

/// Todos 510, 511: a toggle is a `menuitemcheckbox`, its shortcut is
/// `aria-keyshortcuts` and stays out of the name, and Space flips it.
#[test]
fn a_toggle_is_a_checkbox_whose_shortcut_stays_out_of_its_name() {
    const GRID: &str = "[role=menuitemcheckbox]";
    block_on(async {
        let fixture = Fixture::open("/menu-choices", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let outcome = async {
            pointer::click(page, TRIGGER).await?;
            wait::for_visible(page, MENU).await?;
            let tree = e2e::ax::snapshot(page, MENU).await?;
            anyhow::ensure!(
                tree.contains("menuitemcheckbox \"Show grid\" ") && !tree.contains("Ctrl"),
                "the toggle's name is its label alone:\n{tree}"
            );
            wait::for_js_true(
                page,
                &format!(
                    "(() => {{ const g = document.querySelector('{GRID}'); \
                     return g.getAttribute('aria-checked') === 'false' \
                       && g.getAttribute('aria-keyshortcuts') === 'Control+G' \
                       && g.textContent.includes('Ctrl+G'); }})()"
                ),
                "an unchecked toggle announcing its shortcut",
            )
            .await?;
            // Name, Date, Size, then the toggle.
            for _ in 0..3 {
                keyboard::press(page, keyboard::ARROW_DOWN).await?;
            }
            wait::for_js_true(
                page,
                &format!("document.activeElement === document.querySelector('{GRID}')"),
                "focus on the toggle",
            )
            .await?;
            keyboard::press(page, keyboard::SPACE).await?;
            wait::for_js_true(
                page,
                "document.querySelector('#grid').textContent === 'true'",
                "the toggle flipped",
            )
            .await?;
            pointer::click(page, TRIGGER).await?;
            wait::for_js_true(
                page,
                &format!(
                    "document.querySelector('{GRID}')?.getAttribute('aria-checked') === 'true'"
                ),
                "the toggle checked on reopen",
            )
            .await
        }
        .await;
        let console = fixture.console.assert_clean("the menu toggle");

        fixture.close().await.unwrap();
        outcome.unwrap();
        console.unwrap();
    });
}

/// Todo 641: a lone toggle's label lines up with plain rows', a disabled row's shortcut
/// dims; 745: the row is `GrayText` in forced colours.
#[test]
fn a_lone_toggle_lines_up_and_a_disabled_shortcut_dims() {
    block_on(async {
        let fixture = Fixture::open("/menu-choices", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let outcome = async {
            pointer::click(page, TRIGGER).await?;
            wait::for_visible(page, MENU).await?;
            // The rows outside the group: Show grid, Export, Print.
            let [lefts, colours]: [Vec<String>; 2] = page
                .evaluate_bounded(
                    "(() => { \
                       const rows = [...document.querySelectorAll('[role=menu] > [data-menu-index]')]; \
                       const kbd = r => getComputedStyle(r.querySelector('kbd')).color \
                         + ' ' + getComputedStyle(r).opacity; \
                       return [ \
                         rows.map(r => String(r.querySelector('[data-slot=label]').getBoundingClientRect().left)), \
                         [kbd(rows[0]), kbd(rows[2])], \
                       ]; })()",
                )
                .await?
                .into_value()?;
            anyhow::ensure!(
                lefts.len() == 3 && lefts.iter().all(|left| *left == lefts[0]),
                "labels start apart: {lefts:?}"
            );
            anyhow::ensure!(
                colours[0] != colours[1],
                "the disabled shortcut keeps the enabled colour: {colours:?}"
            );
            e2e::browser::force_colours(page).await.unwrap();
            crate::button::assert_gray_in_forced_colours(
                page,
                r#"[role=menu] [data-menu-index][aria-disabled="true"]"#,
            )
            .await;
            Ok(())
        }
        .await;

        fixture.close().await.unwrap();
        outcome.unwrap();
    });
}

/// Todo 1080: a keep-open item stays open on click and Enter with focus on it; others close.
#[test]
fn a_keep_open_item_leaves_the_menu_open() {
    const ZOOM: &str = "[role=menu] [data-menu-index=\"0\"]";
    block_on(async {
        let fixture = Fixture::open("/menu-keep-open", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let outcome = async {
            pointer::click(page, TRIGGER).await?;
            wait::for_visible(page, MENU).await?;
            pointer::click(page, ZOOM).await?;
            wait::for_js_true(
                page,
                &format!(
                    "document.querySelector('#count').textContent === '1' \
                     && document.querySelector('{MENU}') !== null \
                     && document.activeElement === document.querySelector('{ZOOM}')"
                ),
                "one click counted, menu open, focus on the item",
            )
            .await?;
            keyboard::press(page, keyboard::ENTER).await?;
            wait::for_js_true(
                page,
                &format!(
                    "document.querySelector('#count').textContent === '2' \
                     && document.activeElement === document.querySelector('{ZOOM}')"
                ),
                "Enter counted, menu still open",
            )
            .await?;
            keyboard::press(page, keyboard::ARROW_DOWN).await?;
            keyboard::press(page, keyboard::ENTER).await?;
            wait::for_js_true(
                page,
                &format!("document.querySelector('{MENU}') === null"),
                "a plain item closes the menu",
            )
            .await
        }
        .await;

        fixture.close().await.unwrap();
        outcome.unwrap();
    });
}

/// 1739: a menu taller than the room scrolls inside the viewport, on either side.
async fn a_tall_menu_fits_the_room<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(TRIGGER).await?;
    let (_, height) = d.viewport().await?;
    eventually(d, "the open menu inside the viewport", async |d| {
        if !d.exists(MENU).await? {
            return Ok(false);
        }
        let menu = d.rect(MENU).await?;
        // A box stays hidden until placed, and only then caps its scroller to the room: End waits for that.
        let fits = match d.platform() {
            Platform::Native => true,
            _ => d
                .evaluate(
                    "(() => { const m = document.querySelector('[role=menu]'); \
                     return getComputedStyle(m).visibility === 'visible' && [...m.querySelectorAll('*')].every((e) => \
                     getComputedStyle(e).overflowY === 'visible' || e.getBoundingClientRect().bottom \
                     <= m.getBoundingClientRect().bottom + 1); })()",
                )
                .await?
                == true,
        };
        Ok(menu.height > 100.0
            && menu.y >= -1.0
            && menu.y + menu.height <= height + 1.0
            && fits)
    })
    .await?;
    d.press(keyboard::END).await?;
    eventually(d, "the last item scrolled into view", async |d| {
        if d.text(":focus").await? != "Item 40" {
            return Ok(false);
        }
        let item = d.rect(":focus").await?;
        Ok(item.y >= 0.0 && item.y + item.height <= height)
    })
    .await
}

/// Todo 2633: a hovered item takes the shared row tint in both schemes; `muted.1` was near
/// invisible on white. Todo 2683: the theme picks the step, `muted.3` on the dark card.
#[test]
fn a_hovered_item_takes_the_row_tint_in_both_schemes() {
    use e2e::Scheme;
    const ITEM: &str = "[role=menu] [data-menu-index=\"1\"]";
    let tint = format!(
        "(() => {{ const p = document.createElement('div'); \
         p.style.background = 'var(--lsx-row-hover)'; \
         document.querySelector({MENU:?}).append(p); \
         const fill = getComputedStyle(p).backgroundColor; p.remove(); return fill; }})()"
    );
    block_on(async {
        for scheme in [Scheme::Light, Scheme::Dark] {
            let fixture = Fixture::open_in("/menu", Viewport::Desktop, scheme)
                .await
                .unwrap();
            let mut d = e2e::driver::Web { fixture };
            d.click(TRIGGER).await.unwrap();
            eventually(&mut d, "the menu", async |d| d.exists(ITEM).await)
                .await
                .unwrap();
            d.hover(ITEM).await.unwrap();
            eventually(&mut d, "the hovered item to take the row tint", async |d| {
                let want = d.evaluate(&tint).await?;
                Ok(d.style(ITEM, "background-color").await? == want.as_str().unwrap_or_default())
            })
            .await
            .unwrap_or_else(|e| panic!("{scheme:?}: {e}"));
            d.fixture.close().await.unwrap();
        }
    });
}

/// 1739: the room caps a menu, it never stretches a short one.
async fn a_short_menu_keeps_its_height<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(TRIGGER).await?;
    eventually(d, "a one-item menu its own height", async |d| {
        Ok(d.exists(MENU).await? && d.rect(MENU).await?.height < 80.0)
    })
    .await
}

e2e::scenario!(
    a_tall_menu_below_fits_the_room,
    "/menu-tall/below",
    a_tall_menu_fits_the_room,
    windows: skip("3421: END scrolls by the listbox height from before the popup shrank")
);
e2e::scenario!(
    a_tall_menu_above_fits_the_room,
    "/menu-tall/above",
    a_tall_menu_fits_the_room,
    windows: skip("3421: END scrolls by the listbox height from before the popup shrank")
);
e2e::scenario!(
    a_short_menu_keeps_its_height,
    "/menu-row",
    a_short_menu_keeps_its_height
);

/// minmax 3893: the wrapper has no box, so a trigger at 100% fills its parent and anchors the menu.
async fn a_trigger_fills_its_box<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let parent = d.rect("#fill-box").await?;
    let trigger = d.rect(TRIGGER).await?;
    anyhow::ensure!(
        (trigger.width - parent.width).abs() < 1.0 && (trigger.height - parent.height).abs() < 1.0,
        "the trigger {trigger:?} fills its box {parent:?}"
    );
    d.click(TRIGGER).await?;
    eventually(d, "the menu below the trigger, at its start", async |d| {
        if !d.exists(MENU).await? {
            return Ok(false);
        }
        let menu = d.rect(MENU).await?;
        let below = menu.y - (trigger.y + trigger.height);
        Ok((0.0..=16.0).contains(&below) && (menu.x - trigger.x).abs() < 1.0)
    })
    .await
}

/// minmax 3892: the placement lives in a class, so the caller's `sx` docks the menu.
async fn sx_docks_the_menu<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let (width, height) = d.viewport().await?;
    let dock = d.rect("#dock").await?;
    d.click(TRIGGER).await?;
    eventually(d, "the menu full width, flush on the row", async |d| {
        if !d.exists(MENU).await? {
            return Ok(false);
        }
        let menu = d.rect(MENU).await?;
        Ok(menu.x.abs() < 1.0
            && (menu.width - width).abs() < 1.0
            && (menu.y + menu.height - dock.y).abs() < 1.0
            && dock.y < height)
    })
    .await
}

e2e::scenario!(
    a_trigger_fills_its_box,
    "/menu-fill",
    a_trigger_fills_its_box
);
e2e::scenario!(sx_docks_the_menu, "/menu-docked", sx_docks_the_menu);

/// Todo 3053: Home on the first item and End on the last stay the page's.
#[test]
fn home_and_end_at_the_ends_leave_the_page_to_scroll() {
    block_on(async {
        let fixture = Fixture::open("/menu", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        wait::for_js_true(page, FOCUS_ON_CUT, "focus on the first item")
            .await
            .unwrap();
        crate::accordion::assert_keys_left_to_page(page, &[keyboard::HOME]).await;
        keyboard::press(page, keyboard::END).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement?.textContent.trim() === 'Share'",
            "focus on the last item",
        )
        .await
        .unwrap();
        crate::accordion::assert_keys_left_to_page(page, &[keyboard::END]).await;
        fixture.console.assert_clean("the ends").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The menu at 360px wide, below `sm` (todo 4110).
async fn phone(route: &str) -> Fixture {
    use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
    let fixture = Fixture::open(route, Viewport::Mobile).await.unwrap();
    fixture
        .page
        .execute_bounded(SetDeviceMetricsOverrideParams::new(360, 780, 1.0, true))
        .await
        .unwrap();
    fixture
}

/// `count` menu levels open, the last one placed edge to edge on the screen's foot.
async fn sheets_shown(page: &chromiumoxide::Page, count: usize) -> Result<()> {
    let js = format!(
        "(() => {{ const all = [...document.querySelectorAll('[role=menu]')]; \
         if (all.length !== {count}) return false; const s = all[all.length - 1]; \
         if (getComputedStyle(s).visibility !== 'visible') return false; \
         const r = s.getBoundingClientRect(); \
         return Math.abs(r.left) < 1 && Math.abs(r.width - innerWidth) < 1 \
           && Math.abs(r.bottom - innerHeight) < 1; }})()"
    );
    wait::for_js_true(
        page,
        &js,
        &format!("{count} sheet(s), the top one at the foot"),
    )
    .await
}

const MENU_GONE: &str = "!document.querySelector('[role=menu]')";

/// Todos 4110, 4112: the same rows as a sheet; Escape, an outside press and a swipe close it,
/// and a red item paints its label and icon alike.
#[test]
fn below_sm_the_menu_is_a_sheet_that_closes_every_way() {
    block_on(async {
        let fixture = phone("/menu-phone").await;
        let page = &fixture.page;
        tap(page, TRIGGER).await.unwrap();
        sheets_shown(page, 1).await.unwrap();
        let colours = "(() => { const row = n => [...document.querySelectorAll('[role=menuitem]')] \
             .find(r => r.textContent.trim() === n); \
             const c = (r, s) => getComputedStyle(r.querySelector(`[data-slot=${s}]`)).color; \
             const del = row('Delete'), ren = row('Rename'); \
             return c(del, 'label') === c(del, 'leading') && c(del, 'label') !== c(ren, 'label') \
               && getComputedStyle(del.querySelector('[data-slot=leading] circle')).fill === c(del, 'label'); })()";
        wait::for_js_true(page, colours, "Delete's label and icon in its own colour")
            .await
            .unwrap();

        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{MENU_GONE} && document.activeElement.matches({TRIGGER:?})"),
            "Escape closed the sheet, focus on the trigger",
        )
        .await
        .unwrap();

        tap(page, TRIGGER).await.unwrap();
        sheets_shown(page, 1).await.unwrap();
        tap(page, "#outside").await.unwrap();
        wait::for_js_true(page, MENU_GONE, "an outside tap closed the sheet")
            .await
            .unwrap();

        tap(page, TRIGGER).await.unwrap();
        sheets_shown(page, 1).await.unwrap();
        let at = pointer::centre_of(page, "[role=menu] > [data-slot=sheet-handle]")
            .await
            .unwrap();
        let to = pointer::Point {
            x: at.x,
            y: at.y + 200.0,
        };
        pointer::swipe(page, at, to, 4).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{MENU_GONE} && document.querySelector('#last').textContent === ''"),
            "a swipe down closed the sheet and chose nothing",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("the phone sheet").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4862: a finger opens the sheet on its box, no row highlighted; ArrowDown takes the first.
#[test]
fn a_tap_opens_the_sheet_on_its_box() {
    block_on(async {
        let fixture = phone("/menu-phone").await;
        let page = &fixture.page;
        tap(page, TRIGGER).await.unwrap();
        sheets_shown(page, 1).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement.matches('[role=menu]') \
             && !document.querySelector('[role=menu] :focus')",
            "the sheet focused, no row",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement === document.querySelector('[role=menu] [role=menuitem]')",
            "ArrowDown on the first row",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("a tapped sheet").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4610: a scrolling sheet keeps its grab strip in view and does not chain into the page.
#[test]
fn a_scrolled_phone_sheet_keeps_its_strip_and_the_page_still() {
    block_on(async {
        let fixture = phone("/menu-phone-long").await;
        let page = &fixture.page;
        tap(page, TRIGGER).await.unwrap();
        sheets_shown(page, 1).await.unwrap();
        let js = "(() => { const s = document.querySelector('[role=menu]'); \
                  s.scrollTop = 300; \
                  const h = s.querySelector('[data-slot=sheet-handle]').getBoundingClientRect(); \
                  return s.scrollTop > 100 && getComputedStyle(s).overscrollBehaviorY === 'contain' \
                    && Math.abs(h.top - s.getBoundingClientRect().top) < 2; })()";
        wait::for_js_true(
            page,
            js,
            "the strip stays at the sheet's top, overscroll contained",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("a scrolled sheet").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4110: Back closes the phone's sheet, not the app.
async fn back_closes_the_sheet<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(TRIGGER).await?;
    eventually(d, "the sheet to open", async |d| d.exists(MENU).await).await?;
    d.press_back().await?;
    eventually(d, "Back to close the sheet", async |d| {
        Ok(!d.exists(MENU).await? && d.exists(TRIGGER).await?)
    })
    .await
}

e2e::scenario!(
    android_back_closes_the_sheet,
    "/menu-phone",
    back_closes_the_sheet,
    android_only("4110: no Back key off Android")
);

/// Todo 4111: a menu item's click is a user gesture, so `args.open` opens the real picker.
async fn a_menu_item_opens_the_picker<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("button[aria-haspopup=menu]:not(#tools)").await?;
    eventually(d, "the menu to open", async |d| d.exists(MENU).await).await?;
    d.choose_files("[role=menuitem]", &[("me.png", "a face")])
        .await?;
    eventually_text(d, "#picked", "me.png", "the picked photo").await
}

e2e::scenario!(
    a_menu_item_opens_the_file_picker,
    "/menu-file",
    a_menu_item_opens_the_picker,
    native: skip("rfd opens the desktop portal's dialog, which no driver answers"),
    desktop: skip("1126: the GTK file chooser is not driven"),
    ios: skip("3194: the simulator's picker opens but is not driven")
);

/// Todo 4110: a submenu stacks a second sheet; Escape closes it alone, focus on its item.
#[test]
fn below_sm_a_submenu_stacks_and_closes_alone() {
    block_on(async {
        let fixture = phone("/menu-phone").await;
        let page = &fixture.page;
        tap(page, TRIGGER).await.unwrap();
        sheets_shown(page, 1).await.unwrap();
        tap(page, "[role=menuitem][aria-haspopup=menu]")
            .await
            .unwrap();
        sheets_shown(page, 2).await.unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelectorAll('[role=menu]').length === 1 \
             && document.activeElement.matches('[role=menuitem][aria-haspopup=menu]')",
            "only the submenu closed, focus on Share",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("a stacked sheet").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4111: a `FileField` trigger keeps the field's id, names the menu by the field's
/// label, and an item reaches `args.open`; a trigger beside its `Menu` keeps its own id too.
#[test]
fn a_file_field_trigger_opens_a_menu_named_by_the_field() {
    block_on(async {
        let fixture = Fixture::open("/menu-file", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let field_trigger = "button[aria-haspopup=menu]:not(#tools)";
        let single_id = format!(
            "(() => {{ const b = document.querySelector({field_trigger:?}); \
             return !!b.id && !b.id.endsWith('-trigger') \
               && document.querySelectorAll('#' + CSS.escape(b.id)).length === 1; }})()"
        );
        wait::for_js_true(page, &single_id, "the field's own id, once")
            .await
            .unwrap();
        page.find_element(field_trigger)
            .await
            .unwrap()
            .click()
            .await
            .unwrap();
        wait::for_visible(page, MENU).await.unwrap();
        let labelled = format!(
            "(() => {{ const b = document.querySelector({field_trigger:?}); \
             return document.querySelector('[role=menu]').getAttribute('aria-labelledby') \
               === (b.getAttribute('aria-labelledby') || b.id); }})()"
        );
        wait::for_js_true(page, &labelled, "the menu labelled as the field's trigger")
            .await
            .unwrap();
        let name = |tree: String| tree.split('"').nth(1).unwrap_or_default().to_string();
        let menu_name = name(e2e::ax::snapshot(page, MENU).await.unwrap());
        let trigger_name = name(e2e::ax::snapshot(page, field_trigger).await.unwrap());
        assert!(menu_name.contains("Photo"), "the menu's name: {menu_name}");
        assert_eq!(menu_name, trigger_name, "the menu named as its trigger");
        page.find_element("[role=menuitem]")
            .await
            .unwrap()
            .click()
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &format!("{MENU_GONE} && document.querySelector('#opened').textContent === '1'"),
            "Upload ran `args.open` and closed the menu",
        )
        .await
        .unwrap();

        page.find_element("#tools")
            .await
            .unwrap()
            .click()
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('[role=menu]')?.getAttribute('aria-labelledby') === 'tools' \
             && document.querySelectorAll('#tools').length === 1",
            "the sibling trigger's own id labels its menu",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("a shared trigger id").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4121: a docked menu is trigger-wide from `sm` up, but a full-width sheet below it.
#[test]
fn a_docked_menu_is_still_a_full_width_sheet_below_sm() {
    block_on(async {
        let fixture = phone("/menu-phone-docked").await;
        let page = &fixture.page;
        tap(page, TRIGGER).await.unwrap();
        sheets_shown(page, 1).await.unwrap();
        fixture
            .console
            .assert_clean("a docked phone sheet")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4122: a press on a trigger beside its `Menu` is the toggle, so a second one closes it
/// instead of the outside press closing it for the click to reopen.
async fn a_second_press_on_a_sibling_trigger_closes<D: Driver>(
    d: &mut D,
    _route: &str,
) -> Result<()> {
    d.click("#tools").await?;
    sibling_expanded(d, true).await?;
    d.click("#tools").await?;
    sibling_expanded(d, false).await
}

/// Todo 4511: Tab in a menu whose trigger sits beside it moves on from that trigger, not from
/// the portal outlet at the app's end.
#[test]
fn tab_out_of_a_sibling_triggers_menu_moves_on_from_the_trigger() {
    block_on(async {
        let fixture = Fixture::open("/menu-file", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, "#tools", 6).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            "!!document.activeElement.closest('[role=menu]')",
            "focus in the open menu",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{MENU_GONE} && document.activeElement.id === 'after-tools'"),
            "the menu closed and focus after the trigger",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("a Tab out").unwrap();
        fixture.close().await.unwrap();
    });
}

async fn sibling_expanded<D: Driver>(d: &mut D, open: bool) -> Result<()> {
    eventually(d, "the sibling trigger's state", async |d| {
        let now = d.attr("#tools", "aria-expanded").await?;
        Ok((now.as_deref() == Some("true")) == open)
    })
    .await
}

e2e::scenario!(
    a_second_press_on_a_sibling_trigger_closes_its_menu,
    "/menu-file",
    a_second_press_on_a_sibling_trigger_closes
);
