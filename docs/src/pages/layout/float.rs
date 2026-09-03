use crate::components::{
    Control, Demo, DemoValues, DocPage, DocSection, Wrap, indent, or_unset, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Box, Button, Checkbox, Code, CodeBlock, Flex, Float, Paper, States, Text},
    hooks::use_element,
    platform::ElementApi,
    sx::{Sx, sx},
    use_theme,
};

const CHILD: &str = r#"Box { sx: sx().padding("4px 8px").background("primary"), "Badge" }"#;

/// A float positions against the nearest `position: relative` ancestor, so
/// the preview has to be one - and the code block has to say so.
fn wrap_anchor(_: &DemoValues, code: &str) -> String {
    format!(
        "Box {{\n    sx: sx().position(\"relative\").width(\"160px\").height(\"120px\").background(\"primary.1\"),\n{}}}",
        indent(code)
    )
}

/// Printed verbatim under the live example below - keep the two in step.
const ACTION_BAR: &str = r##"const INVOICES: [&str; 3] = ["Invoice 1042", "Invoice 1043", "Invoice 1044"];
const REDUCED_MOTION: &str = "(prefers-reduced-motion: reduce)";

/// CSS drives the show and hide, keyed on the `open`/`closed` state: a fade and
/// a short slide, with `visibility` flipping to `hidden` only once the fade has
/// ended. That flip is what takes a hidden bar out of the tab order and the
/// accessibility tree. Each transition's reduced-motion guard sits in the same
/// block as the transition, or the block's higher specificity wins.
fn bar_sx() -> Sx {
    let motion = "opacity 200ms ease, translate 200ms ease";
    sx().when(
        "open",
        sx().opacity("1")
            .with("translate", "0 0")
            .visibility("visible")
            .transition(format!("{motion}, visibility 0s"))
            .media(REDUCED_MOTION, sx().transition("none")),
    )
    .when(
        "closed",
        sx().opacity("0")
            .with("translate", "0 8px")
            .visibility("hidden")
            .transition(format!("{motion}, visibility 0s linear 200ms"))
            .media(REDUCED_MOTION, sx().transition("none")),
    )
}

#[component]
fn Invoices() -> Element {
    let mut selected = use_signal(Vec::<usize>::new);
    let list = use_element();
    let count = selected.read().len();
    let open = count > 0;

    rsx! {
        div { onmounted: list.mount(),
            for (index, invoice) in INVOICES.iter().enumerate() {
                Checkbox {
                    id: "invoice-{index}",
                    label: *invoice,
                    checked: selected.read().contains(&index),
                    onchange: move |on: bool| {
                        let mut selected = selected.write();
                        if on { selected.push(index) } else { selected.retain(|i| *i != index) }
                    },
                }
            }
        }
        Float { fixed: true, placement: "bottom-center", offset_y: "-xl",
            Paper {
                role: "group",
                aria_label: "Selected invoices",
                bordered: true,
                shadow: "md",
                states: States::default().with("open", open).with("closed", !open),
                sx: bar_sx().padding("xs sm"),
                Flex { direction: "row", gap: "sm", align: "center",
                    Text { "{count} selected" }
                    Button { variant: "outlined", "Download" }
                    Button {
                        onclick: move |_| {
                            selected.write().clear();
                            // The bar hides with this button in it: hand focus
                            // back to the list rather than to the page.
                            let _ = list.query_selector("#invoice-0").and_then(|first| first.focus());
                        },
                        "Clear"
                    }
                }
            }
        }
    }
}"##;

const INVOICES: [&str; 3] = ["Invoice 1042", "Invoice 1043", "Invoice 1044"];
const REDUCED_MOTION: &str = "(prefers-reduced-motion: reduce)";

fn bar_sx() -> Sx {
    let motion = "opacity 200ms ease, translate 200ms ease";
    sx().when(
        "open",
        sx().opacity("1")
            .with("translate", "0 0")
            .visibility("visible")
            .transition(format!("{motion}, visibility 0s"))
            .media(REDUCED_MOTION, sx().transition("none")),
    )
    .when(
        "closed",
        sx().opacity("0")
            .with("translate", "0 8px")
            .visibility("hidden")
            .transition(format!("{motion}, visibility 0s linear 200ms"))
            .media(REDUCED_MOTION, sx().transition("none")),
    )
}

#[component]
fn Invoices() -> Element {
    let mut selected = use_signal(Vec::<usize>::new);
    let list = use_element();
    let count = selected.read().len();
    let open = count > 0;

    rsx! {
        div { onmounted: list.mount(),
            for (index, invoice) in INVOICES.iter().enumerate() {
                Checkbox {
                    id: "invoice-{index}",
                    label: *invoice,
                    checked: selected.read().contains(&index),
                    onchange: move |on: bool| {
                        let mut selected = selected.write();
                        if on { selected.push(index) } else { selected.retain(|i| *i != index) }
                    },
                }
            }
        }
        Float { fixed: true, placement: "bottom-center", offset_y: "-xl",
            Paper {
                role: "group",
                aria_label: "Selected invoices",
                bordered: true,
                shadow: "md",
                states: States::default().with("open", open).with("closed", !open),
                sx: bar_sx().padding("xs sm"),
                Flex { direction: "row", gap: "sm", align: "center",
                    Text { "{count} selected" }
                    Button { variant: "outlined", "Download" }
                    Button {
                        onclick: move |_| {
                            selected.write().clear();
                            // The bar hides with this button in it: hand focus
                            // back to the list rather than to the page.
                            let _ = list.query_selector("#invoice-0").and_then(|first| first.focus());
                        },
                        "Clear"
                    }
                }
            }
        }
    }
}

#[component]
pub fn FloatPage() -> Element {
    let theme = use_theme();

    rsx! {
        DocPage {
            title: "Float",
            source: "libero/src/components/layout/float.rs",
            markdown: "/md/float.md",
            properties: vec![props("Float", vec![
                prop("placement", "Placement")
                    .default("center-center")
                    .doc("Anchor corner/edge, e.g. `\"top-start\"`."),
                prop("offset_x", "ThemeAwareValue")
                    .default("0px")
                    .doc("Shift along the horizontal axis - a size token from the spacing scale (`\"md\"`, or `\"-md\"` for the other direction), or any CSS length."),
                prop("offset_y", "ThemeAwareValue")
                    .default("0px")
                    .doc("Shift along the vertical axis."),
                prop("z_index", "ThemeAwareValue")
                    .default("200")
                    .doc("Stacking order."),
                prop("children", "Element").doc("The anchored content."),
            ])],
            lead: rsx! {
                Text { "Anchors its child to a corner/edge of the nearest `position: relative` ancestor - e.g. a badge on an avatar. The parent must set `position: relative` itself. `offset_x`/`offset_y` take a size token from the spacing scale, or any CSS length, and shift it right/down along the page axes - negate the token (`\"-md\"`) to shift left/up instead." }
            },
            Demo {
                component: "Float",
                children_text: "",
                children_code: CHILD.to_string(),
                controls: vec![
                    Control::select(
                        "placement",
                        [
                            "top-start",
                            "top-center",
                            "top-end",
                            "center-start",
                            "center-center",
                            "center-end",
                            "bottom-start",
                            "bottom-center",
                            "bottom-end",
                        ],
                    )
                    .default(theme.float.placement.as_str()),
                    Control::slider(
                        "offset_x",
                        ["-lg", "-md", "-sm", "-xs", "auto", "xs", "sm", "md", "lg"],
                    )
                    .default("auto"),
                    Control::slider(
                        "offset_y",
                        ["-lg", "-md", "-sm", "-xs", "auto", "xs", "sm", "md", "lg"],
                    )
                    .default("auto"),
                ],
                render: move |values: DemoValues| rsx! {
                    Box {
                        sx: sx()
                            .position("relative")
                            .width("160px")
                            .height("120px")
                            .background("primary.1"),
                        Float {
                            placement: values.str("placement"),
                            offset_x: or_unset(values.str("offset_x")),
                            offset_y: or_unset(values.str("offset_y")),
                            Box { sx: sx().padding("4px 8px").background("primary"), "Badge" }
                        }
                    }
                },
                wrap: Wrap(wrap_anchor),
            }
            DocSection {
                title: "Fixed: an action bar",
                Text {
                    Code { source: "fixed: true" }
                    " places against the viewport instead of a positioned parent, so the bar "
                    "stays at the bottom of the window while the page scrolls. Tick an invoice: "
                    "a bar with the selection's actions slides in. It is a "
                    Code { source: "Paper" }
                    " with "
                    Code { source: "role=\"group\"" }
                    " and a label, not a toolbar - every button stays in the normal Tab order."
                }
                Text {
                    "The show and hide are CSS only: the bar stays mounted, and its "
                    Code { source: "open" }
                    "/"
                    Code { source: "closed" }
                    " state fades it and flips "
                    Code { source: "visibility" }
                    " once the fade has ended, which takes the hidden bar out of the tab order and "
                    "the accessibility tree. Reduced motion drops the animation. To unmount it "
                    "instead, use "
                    Code { source: "use_presence(open, \"opacity\")" }
                    ". A transformed, filtered or container-query ancestor becomes a fixed "
                    "element's containing block; render the bar through "
                    Code { source: "use_portal" }
                    " where you do not control the ancestors."
                }
                Invoices {}
                CodeBlock { source: ACTION_BAR, language: "rust" }
            }
        }
    }
}
