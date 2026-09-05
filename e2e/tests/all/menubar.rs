//! `Menubar`: `RovingTabindex` along the bar, `Overlay` for each menu.
//!
//! The first component to be two archetypes at once. The bar is a strip - one
//! tab stop, Left and Right, Home and End, wrapping by default - and each menu
//! it opens is a dismissible popup whose Escape returns focus to its trigger.
//! Neither contract knows about the other, which is the composition working.

use e2e::archetypes::{Orientation, Overlay, RovingTabindex};
use e2e::browser::block_on;
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, passes::keyboard};

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
