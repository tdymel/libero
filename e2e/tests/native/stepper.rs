//! `Stepper` in Blitz's layout: no container queries, and the side labels' re-layout.
//! Focus return and header clicks are shared scenarios (`stepper::`).

use dioxus::prelude::*;
use e2e::native::mount;
use libero::components::{Button, Flex, Options, Stepper, Text};

#[derive(Clone, Copy, PartialEq, Options)]
enum Stage {
    Account,
    Shipping,
    Review,
}

const FIRST: &str = "#stepper-step-0";

fn side_at(width: &'static str) -> Element {
    rsx! {
        div { style: "width: {width}",
            Stepper { id: "stepper", value: Some(Stage::Account), label_position: "side",
                panel: |_| rsx! {} }
        }
    }
}

/// Todo 542: side labels stack under the marker at 320px rather than break mid-word.
/// No container queries in Blitz, so only a narrow window does it, not a narrow box.
#[test]
fn side_labels_stack_below_the_marker_in_a_window_under_360px() {
    let mut page = mount(|| side_at("100%"));
    assert_eq!(page.computed(FIRST, "flex-direction"), "row");

    page.resize(320, 640);
    assert_eq!(page.computed(FIRST, "flex-direction"), "column");
    assert_eq!(
        page.wrapped_text("#stepper [data-step-label]"),
        Vec::<String>::new()
    );
}

#[test]
fn a_narrow_box_keeps_side_labels_natively() {
    let page = mount(|| side_at("320px"));
    assert_eq!(page.computed("#stepper", "container-type"), "");
    assert_eq!(page.computed(FIRST, "flex-direction"), "row");
}

/// Two steps: the width at which the text went stale in the harness.
#[derive(Clone, Copy, PartialEq, Options)]
enum Pair {
    Account,
    Shipping,
}

/// The docs demo: the preview beside the control panel, the Stepper in a
/// `flex-start` column, its label position switched by a control.
fn switchable() -> Element {
    let mut below = use_signal(|| false);
    rsx! {
        button { id: "below", onclick: move |_| below.toggle(), "Below" }
        div { style: "display: flex; flex-wrap: wrap; width: 900px;",
            div { style: "flex: 1; min-width: 240px; padding: 24px; display: flex; align-items: center; justify-content: safe center; overflow-x: auto;",
                div { style: "display: flex; flex-direction: column; align-items: flex-start; gap: 16px; width: 100%;",
                    Stepper {
                        value: Some(Pair::Account),
                        label_position: if below() { "below" } else { "side" },
                        panel: |_: Pair| rsx! {
                            Flex { gap: "sm", align: "flex-start",
                                Text { id: "who", "Who is ordering?" }
                                Button { "Continue" }
                            }
                        },
                    }
                }
            }
            div { style: "flex: 1; min-width: 400px; height: 300px;" }
        }
    }
}

/// The re-layout kept the text's min-content layout, broken per word in a
/// one-line box; libero re-lays such text once laid out (todos 887, 888).
#[test]
fn switching_labels_below_keeps_the_content_text_on_one_line() {
    let mut page = mount(switchable);
    page.click("#below");
    page.wait_for(|page| page.wrapped_text("#who").is_empty());
    assert_eq!(page.wrapped_text("#who"), Vec::<String>::new());
}
