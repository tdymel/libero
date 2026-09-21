//! `Menu`: the overlay archetype as a non-modal popup. Tab closes every level and moves on
//! from the trigger.

use anyhow::Result;
use e2e::archetypes::Overlay;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused};
use e2e::suite::Step;
use e2e::{
    Fixture, Suite, Viewport,
    passes::{keyboard, pointer},
    wait,
};

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
        .run();
}

#[test]
fn it_honours_the_overlay_contract() {
    block_on(async {
        for viewport in Viewport::ALL {
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
        }
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
    enter_opens_and_escape_closes,
    android: skip("958: element identity on the WebView")
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
    a_click_opens,
    android: skip("958: element identity on the WebView")
);
e2e::scenario!(
    a_click_outside_closes_it_and_leaves_focus_where_it_went,
    "/menu-submenu-reopen",
    a_click_outside_closes,
    android: skip("958: element identity on the WebView")
);

/// The docs demo's row: the trigger and the text beside it share a centre line.
#[test]
fn the_trigger_centres_beside_its_text() {
    block_on(async {
        let fixture = Fixture::open("/menu-row", Viewport::Desktop).await.unwrap();
        let offset: f64 = fixture
            .page
            .evaluate(format!(
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
            pointer::click(page, "[data-menu-group-label]").await?;
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
        page.evaluate(
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
            std::thread::sleep(std::time::Duration::from_millis(300));
            wait::for_js_true(page, &focus_on("Actions"), "the menu left closed").await?;

            keyboard::press(page, keyboard::ARROW_UP).await?;
            wait::for_js_true(page, &focus_on("Share"), "focus on Share").await?;
            keyboard::press(page, keyboard::ARROW_RIGHT).await?;
            wait::for_js_true(page, &focus_on("Email"), "focus in the submenu").await?;
            keyboard::press_with(page, keyboard::ARROW_LEFT, keyboard::ALT).await?;
            std::thread::sleep(std::time::Duration::from_millis(300));
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
                   const labels = [...document.querySelectorAll('[data-menu-label]')]; \
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
                .evaluate(
                    "(() => { \
                       const rows = [...document.querySelectorAll('[role=menu] > [data-menu-index]')]; \
                       const kbd = r => getComputedStyle(r.querySelector('kbd')).color \
                         + ' ' + getComputedStyle(r).opacity; \
                       return [ \
                         rows.map(r => String(r.querySelector('[data-menu-label]').getBoundingClientRect().left)), \
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
            crate::calendar::force_colours(page).await;
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
