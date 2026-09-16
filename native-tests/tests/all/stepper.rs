//! `Stepper`: a "Continue" inside the current step's content moves the step
//! on, and focus returns to the step the user is now on (todo 406). A step
//! header's click moves there.

use std::time::Duration;

use dioxus::prelude::*;
use libero::components::{Button, Options, Stepper};
use native_tests::{Key, Page, mount};

#[derive(Clone, Copy, PartialEq, Options)]
enum Stage {
    Account,
    Shipping,
    Review,
}

fn stepper(vertical: bool) -> Element {
    let mut stage = use_signal(|| Some(Stage::Account));
    rsx! {
        Stepper {
            id: "stepper",
            value: stage(),
            orientation: if vertical { "vertical" } else { "horizontal" },
            label_position: "below",
            onstepclick: move |s| stage.set(Some(s)),
            panel: move |s: Stage| match s {
                Stage::Account => rsx! {
                    Button { id: "next-0", onclick: move |_| stage.set(Some(Stage::Shipping)), "Continue to shipping" }
                },
                Stage::Shipping => rsx! {
                    Button { id: "next-1", onclick: move |_| stage.set(Some(Stage::Review)), "Continue to review" }
                },
                Stage::Review => rsx! {
                    Button { id: "finish", onclick: move |_| stage.set(None), "Finish" }
                },
            },
        }
    }
}

fn horizontal() -> Element {
    stepper(false)
}

fn vertical() -> Element {
    stepper(true)
}

fn header(index: usize) -> String {
    format!("#stepper-step-{index}")
}

fn continue_from(page: &mut Page, button: &str, landing: usize) {
    page.focus(button);
    page.press(Key::Enter);
    page.advance(1.0);
    page.wait(Duration::from_millis(50));
    assert!(
        page.is_focused(&header(landing)),
        "{button}: focus is on {}\n{}",
        page.focus_owner(),
        page.tree()
    );
}

fn moving_on_returns_focus(app: fn() -> Element) {
    let mut page = mount(app);
    continue_from(&mut page, "#next-0", 1);
    assert_eq!(
        page.attr(&header(1), "aria-current").as_deref(),
        Some("step")
    );
    continue_from(&mut page, "#next-1", 2);
    continue_from(&mut page, "#finish", 2);
}

#[test]
fn moving_on_returns_focus_to_the_current_step_horizontally() {
    moving_on_returns_focus(horizontal);
}

#[test]
fn moving_on_returns_focus_to_the_current_step_vertically() {
    moving_on_returns_focus(vertical);
}

#[test]
fn a_clicked_continue_returns_focus_too() {
    for app in [horizontal as fn() -> Element, vertical] {
        let mut page = mount(app);
        page.click("#next-0");
        page.advance(1.0);
        assert!(
            page.is_focused(&header(1)),
            "focus is on {}",
            page.focus_owner()
        );
    }
}

fn side_at(width: &'static str) -> Element {
    rsx! {
        div { style: "width: {width}",
            Stepper { id: "stepper", value: Some(Stage::Account), label_position: "side",
                panel: |_| rsx! {} }
        }
    }
}

/// Side labels at 320px broke words mid-word; they stack under the marker
/// there (todo 542). Blitz has no container queries, so only a narrow window
/// does it natively, not a narrow box.
#[test]
fn side_labels_stack_below_the_marker_in_a_window_under_360px() {
    let mut page = mount(|| side_at("100%"));
    assert_eq!(page.computed(&header(0), "flex-direction"), "row");

    page.resize(320, 640);
    assert_eq!(page.computed(&header(0), "flex-direction"), "column");
    assert_eq!(
        page.wrapped_text("#stepper [data-step-label]"),
        Vec::<String>::new()
    );
}

#[test]
fn a_narrow_box_keeps_side_labels_natively() {
    let page = mount(|| side_at("320px"));
    assert_eq!(page.computed("#stepper", "container-type"), "");
    assert_eq!(page.computed(&header(0), "flex-direction"), "row");
}

#[test]
fn a_click_on_a_done_step_header_moves_back_there() {
    let mut page = mount(horizontal);
    page.click("#next-0");
    assert!(page.exists("#next-1"), "{}", page.tree());
    page.click(&header(0));
    assert_eq!(
        page.attr(&header(0), "aria-current").as_deref(),
        Some("step")
    );
    assert!(page.exists("#next-0"), "{}", page.tree());
}
