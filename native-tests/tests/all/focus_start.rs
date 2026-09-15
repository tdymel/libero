//! The sequential focus starting point (todo 622): a click on nothing focusable
//! starts the next Tab or Shift+Tab from the clicked node, as on the web.

use dioxus::prelude::*;
use libero::components::Button;
use native_tests::mount;

fn app() -> Element {
    rsx! {
        Button { id: "first", "First" }
        p { id: "early", "Before the middle" }
        Button { id: "middle", "Middle" }
        p { id: "late", "After the middle" }
        Button { id: "last", "Last" }
    }
}

#[test]
fn tab_after_a_click_starts_from_the_clicked_node() {
    let mut page = mount(app);
    page.click("#late");
    page.tab();
    assert!(
        page.is_focused("#last"),
        "Tab went to {}",
        page.focus_owner()
    );
}

#[test]
fn shift_tab_after_a_click_starts_from_the_clicked_node() {
    let mut page = mount(app);
    page.click("#late");
    page.shift_tab();
    assert!(
        page.is_focused("#middle"),
        "Shift+Tab went to {}",
        page.focus_owner()
    );
}

#[test]
fn a_click_on_prose_moves_the_starting_point_away_from_a_focused_control() {
    let mut page = mount(app);
    page.click("#first");
    page.click("#early");
    page.tab();
    assert!(
        page.is_focused("#middle"),
        "Tab went to {}",
        page.focus_owner()
    );
}

#[test]
fn a_second_tab_carries_on_from_the_first() {
    let mut page = mount(app);
    page.click("#early");
    page.tab();
    page.tab();
    assert!(
        page.is_focused("#last"),
        "Tab went to {}",
        page.focus_owner()
    );
}
