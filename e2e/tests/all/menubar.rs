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

/// Todo 449: Alt+ArrowRight (Forward) moved along the bar with the browser's
/// default prevented. No chord may move along the bar or open a menu.
#[test]
fn shortcut_chords_pass_through() {
    block_on(async {
        let fixture = Fixture::open("/menubar", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        let outcome = async {
            keyboard::tab_to(page, FIRST, 10).await?;
            keyboard::assert_chords_ignored(
                page,
                &[
                    keyboard::ARROW_RIGHT,
                    keyboard::ARROW_LEFT,
                    keyboard::ARROW_DOWN,
                    keyboard::ARROW_UP,
                    keyboard::HOME,
                    keyboard::END,
                ],
                &format!(
                    "[document.activeElement?.getAttribute('data-menubar-index'), \
                     !!document.querySelector('{MENU}')]"
                ),
            )
            .await
        }
        .await;

        fixture.close().await.unwrap();
        outcome.unwrap();
    });
}

fn trigger(index: usize) -> String {
    format!("[role=menubar] [data-menubar-index=\"{index}\"]")
}

/// Which triggers read `aria-expanded="true"`, as "0 1 2 3" flags.
const EXPANDED: &str = "[...document.querySelectorAll('[data-menubar-index]')]\
     .map(t => t.getAttribute('aria-expanded') === 'true' ? 1 : 0).join('')";

/// A touch tap: touch emulation must be on. CDP rejects a `touchEnd` without
/// a point, though its docs ask for none.
async fn tap(page: &chromiumoxide::Page, selector: &str) -> anyhow::Result<()> {
    use chromiumoxide::cdp::browser_protocol::input::{
        DispatchTouchEventParams, DispatchTouchEventType, TouchPoint,
    };
    let at = pointer::centre_of(page, selector).await?;
    for kind in [
        DispatchTouchEventType::TouchStart,
        DispatchTouchEventType::TouchEnd,
    ] {
        let point = TouchPoint::builder()
            .x(at.x)
            .y(at.y)
            .build()
            .map_err(anyhow::Error::msg)?;
        let event = DispatchTouchEventParams::builder()
            .r#type(kind)
            .touch_point(point)
            .build()
            .map_err(anyhow::Error::msg)?;
        page.execute(event).await?;
    }
    Ok(())
}

/// Todo 449: a tap on another trigger while a menu was open sent the
/// compatibility `mouseenter` first, which switched menus, and then the click,
/// which closed the new one. The tap opened nothing.
#[test]
fn a_tap_on_another_trigger_opens_its_menu() {
    use chromiumoxide::cdp::browser_protocol::emulation::SetTouchEmulationEnabledParams;
    block_on(async {
        let fixture = Fixture::open("/menubar-docs", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        let outcome = async {
            page.execute(SetTouchEmulationEnabledParams::new(true))
                .await?;
            tap(page, &trigger(0)).await?;
            wait::for_js_true(page, &format!("{EXPANDED} === '1000'"), "File open").await?;
            tap(page, &trigger(1)).await?;
            wait::for_js_true(page, &format!("{EXPANDED} === '0100'"), "Edit open").await?;
            wait::for_js_true(
                page,
                "document.activeElement?.textContent.trim() === 'Undo'",
                "focus on Undo",
            )
            .await
        }
        .await;

        fixture.close().await.unwrap();
        outcome.unwrap();
    });
}

/// APG menubar: an open menu travels along the bar with Left and Right, from
/// a submenu too; a disabled trigger takes focus and opens nothing, but the
/// next enabled one opens its menu (todo 568).
#[test]
fn an_open_menu_travels_along_the_bar() {
    block_on(async {
        let fixture = Fixture::open("/menubar-docs", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let focus_on =
            |text: &str| format!("document.activeElement?.textContent.trim() === '{text}'");
        let outcome = async {
            use keyboard::{ARROW_DOWN, ARROW_LEFT, ARROW_RIGHT, ARROW_UP, ENTER, ESCAPE, press};
            keyboard::tab_to(page, FIRST, 5).await?;
            press(page, ENTER).await?;
            wait::for_js_true(page, &focus_on("New"), "File open on New").await?;
            press(page, ARROW_UP).await?;
            press(page, ARROW_UP).await?;
            wait::for_js_true(page, &focus_on("Open recent"), "Open recent").await?;
            press(page, ARROW_RIGHT).await?;
            wait::for_js_true(page, &focus_on("notes.md"), "the submenu on notes.md").await?;
            press(page, ARROW_RIGHT).await?;
            wait::for_js_true(page, &focus_on("Undo"), "Edit open on Undo").await?;
            wait::for_js_true(page, &format!("{EXPANDED} === '0100'"), "only Edit open").await?;
            press(page, ARROW_RIGHT).await?;
            wait::for_js_true(page, &focus_on("View"), "focus on the disabled View").await?;
            wait::for_js_true(page, &format!("{EXPANDED} === '0000'"), "no menu open").await?;
            // Todo 568: the bar stays in open mode across the disabled trigger.
            press(page, ARROW_RIGHT).await?;
            wait::for_js_true(page, &focus_on("Documentation"), "Help open past View").await?;
            wait::for_js_true(page, &format!("{EXPANDED} === '0001'"), "only Help open").await?;
            press(page, ARROW_LEFT).await?;
            wait::for_js_true(page, &focus_on("View"), "back on the disabled View").await?;
            press(page, ARROW_LEFT).await?;
            wait::for_js_true(page, &focus_on("Undo"), "Edit open again").await?;
            press(page, ARROW_RIGHT).await?;
            wait::for_js_true(page, &focus_on("View"), "View once more").await?;
            // Escape on the disabled trigger ends open mode.
            press(page, ESCAPE).await?;
            press(page, ARROW_LEFT).await?;
            wait::for_js_true(page, &focus_on("Edit"), "Edit, closed").await?;
            wait::for_js_true(page, &format!("{EXPANDED} === '0000'"), "no menu open").await?;
            press(page, ARROW_DOWN).await?;
            wait::for_js_true(page, &focus_on("Undo"), "Edit open by ArrowDown").await?;
            press(page, ESCAPE).await?;
            wait::for_js_true(page, &focus_on("Edit"), "Escape back on Edit").await?;
            wait::for_js_true(page, &format!("{EXPANDED} === '0000'"), "all closed").await?;
            // Tabbing away from the disabled trigger ends open mode too.
            press(page, ARROW_DOWN).await?;
            wait::for_js_true(page, &focus_on("Undo"), "Edit open").await?;
            press(page, ARROW_RIGHT).await?;
            wait::for_js_true(page, &focus_on("View"), "on View again").await?;
            press(page, keyboard::TAB).await?;
            wait::for_js_true(page, &focus_on("after"), "focus past the bar").await?;
            keyboard::press_with(page, keyboard::TAB, keyboard::SHIFT).await?;
            wait::for_js_true(page, &focus_on("View"), "back on View").await?;
            press(page, ARROW_RIGHT).await?;
            wait::for_js_true(page, &focus_on("Help"), "Help, closed").await?;
            wait::for_js_true(page, &format!("{EXPANDED} === '0000'"), "no menu open").await
        }
        .await;

        fixture.close().await.unwrap();
        outcome.unwrap();
    });
}

/// A menu dropped from `menus` while open comes back closed (todo 569).
#[test]
fn a_menu_dropped_while_open_comes_back_closed() {
    block_on(async {
        let fixture = Fixture::open("/menubar-shrink", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let view = "[role=menubar] [data-menubar-index=\"2\"]";
        let toggle =
            "document.getElementById('drop').dispatchEvent(new Event('input', { bubbles: true }))";

        page.evaluate(format!("document.querySelector({view:?}).focus()"))
            .await
            .unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        wait::for_js_true(
            page,
            &format!("!!document.querySelector({MENU:?})"),
            "View open",
        )
        .await
        .unwrap();

        page.evaluate(toggle).await.unwrap();
        wait::for_js_true(
            page,
            &format!("!document.querySelector({view:?})"),
            "View dropped",
        )
        .await
        .unwrap();
        page.evaluate(toggle).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "document.querySelector({view:?})?.getAttribute('aria-expanded') === 'false' \
                 && !document.querySelector({MENU:?})"
            ),
            "View back, closed",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("a dropped menu").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 745: a disabled trigger is `GrayText` in forced colours, not only dimmed.
#[test]
fn a_disabled_trigger_grays_out_in_forced_colours() {
    block_on(async {
        let fixture = Fixture::open("/menubar-docs", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        crate::calendar::force_colours(page).await;
        crate::button::assert_gray_in_forced_colours(
            page,
            r#"[role=menubar] [data-menubar-index][aria-disabled="true"]"#,
        )
        .await;
        fixture.close().await.unwrap();
    });
}

/// Todo 711: a menu starts at its trigger's start edge, the left under LTR and
/// the right under RTL. `rtl_keys` covers the submenu opening towards the end.
#[test]
fn a_menu_starts_at_its_trigger_s_start_edge() {
    block_on(async {
        for dir in ["ltr", "rtl"] {
            let fixture = crate::rtl_keys::open_in("/menubar", dir).await;
            let page = &fixture.page;

            keyboard::tab_to(page, FIRST, 10).await.unwrap();
            keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
            wait::for_visible(page, MENU).await.unwrap();
            let edge = match dir {
                "rtl" => "right",
                _ => "left",
            };
            wait::for_js_true(
                page,
                &format!(
                    "Math.abs(document.querySelector({MENU:?}).getBoundingClientRect().{edge} \
                     - document.querySelector({FIRST:?}).getBoundingClientRect().{edge}) < 1.5"
                ),
                &format!("{dir}: the menu to line up on the {edge}"),
            )
            .await
            .unwrap();

            fixture.console.assert_clean(dir).unwrap();
            fixture.close().await.unwrap();
        }
    });
}
