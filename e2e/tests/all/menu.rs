//! `Menu`: the overlay archetype, as a non-modal popup.
//!
//! Not a focus trap: Tab closes every level and moves on from the trigger.

use e2e::archetypes::Overlay;
use e2e::browser::block_on;
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
