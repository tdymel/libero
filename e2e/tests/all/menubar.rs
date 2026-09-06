//! `Menubar`: `RovingTabindex` along the bar, `Overlay` for each menu.
//!
//! The first component to be two archetypes at once. The bar is a strip - one
//! tab stop, Left and Right, Home and End, wrapping by default - and each menu
//! it opens is a dismissible popup whose Escape returns focus to its trigger.
//! Neither contract knows about the other, which is the composition working.

use e2e::archetypes::{Orientation, Overlay, RovingTabindex};
use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, wait};

pub const TRIGGERS: &str = "[role=menubar] [data-menubar-index]";
const FIRST: &str = "[role=menubar] [data-menubar-index=\"0\"]";
const MENU: &str = "[role=menu]";

#[test]
fn it_meets_the_baseline() {
    Suite::new("menubar", "/menubar")
        .focusable(FIRST)
        .targets(TRIGGERS)
        .targets("[role=menu] [role=menuitem]")
        .state(
            "open",
            &[Step::TabTo(FIRST), Step::Press(keyboard::ARROW_DOWN)],
            MENU,
        )
        .run();
}

#[test]
fn its_bar_honours_the_roving_tabindex_contract() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/menubar", viewport).await.unwrap();

            RovingTabindex {
                items: TRIGGERS,
                orientation: Orientation::Horizontal,
                // `loop_focus` defaults to true.
                wraps: true,
            }
            .assert_contract(&fixture.page)
            .await
            .unwrap_or_else(|e| panic!("at {}: {e}", viewport.name()));

            fixture
                .console
                .assert_clean(&format!("the menubar contract at {}", viewport.name()))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}

#[test]
fn its_menu_honours_the_overlay_contract() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/menubar", viewport).await.unwrap();

            Overlay {
                trigger: FIRST,
                panel: MENU,
                // Tab closes a menu and leaves the bar; it is not a trap.
                traps_focus: false,
                tab_budget: 5,
            }
            .assert_contract(&fixture.page)
            .await
            .unwrap_or_else(|e| panic!("at {}: {e}", viewport.name()));

            fixture
                .console
                .assert_clean(&format!("the menubar menu at {}", viewport.name()))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// Todo 283: moving the pointer between triggers while a menu is open logged
/// a dioxus scope warning, twice per switch, and nothing in the DOM showed
/// it. Edit holds the submenu level that the warning came from.
#[test]
fn switching_menus_by_pointer_logs_nothing() {
    block_on(async {
        let fixture = Fixture::open("/menubar", Viewport::Desktop).await.unwrap();
        let trigger = |index: usize| format!("[role=menubar] [data-menubar-index=\"{index}\"]");

        pointer::click(&fixture.page, &trigger(0)).await.unwrap();
        wait::for_visible(&fixture.page, MENU).await.unwrap();
        for index in [1, 0, 1, 2, 1] {
            pointer::hover(&fixture.page, &trigger(index))
                .await
                .unwrap();
            let expanded = format!(
                "document.querySelector({}).getAttribute('aria-expanded') === 'true'",
                serde_json::to_string(&trigger(index)).unwrap()
            );
            wait::for_js_true(&fixture.page, &expanded, "the hovered menu to open")
                .await
                .unwrap();
        }

        let outcome = fixture.console.assert_clean("switching menus by pointer");
        fixture.close().await.unwrap();
        outcome.unwrap();
    });
}
