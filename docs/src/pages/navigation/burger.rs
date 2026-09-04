use crate::components::{Control, Demo, DemoValues, DocPage, UNSET, Wrap, indent, prop, props};
use dioxus::prelude::*;
use libero::components::{Burger, Code, Flex, Input, Paper, Text};
use libero::sx::sx;

/// The panel the preview's `aria-controls` points at. A `Burger` whose
/// `opened` is `Some` and whose `aria-controls` names nothing is exactly what
/// the component warns about, so the demo has to own a real one - and
/// `opened` is a relation to a neighbour, which the preview needs anyway.
const PANEL_ID: &str = "burger-demo-panel";

// Rendered on both states, hidden rather than absent: `aria-controls`
// pointing at an element that is not in the document is a dangling
// reference, and a disclosure's panel is the thing that exists and is
// hidden.
const PANEL: &str = r#"Paper {
    id: "burger-demo-panel",
    sx: sx().padding("sm").display(if opened() { "block" } else { "none" }),
    "Navigation"
}"#;

/// The preview is a pair - the button and the thing it expands - so the block
/// has to be that pair too, signal and all. The signal is the preview's own:
/// nothing outside the block owns `opened`.
fn wrap(_values: &DemoValues, source: &str) -> String {
    let mut code = String::from("let mut opened = use_signal(|| false);\n\nrsx! {\n");
    code.push_str(&indent(source));
    code.push_str(&indent(PANEL));
    code.push('}');
    code
}

/// The preview owns `opened` itself, so the burger in it is the control.
///
/// `Demo` hands `render` a snapshot of the control values and no way to write
/// back, which is why a *control* could never have driven this. It does not
/// have to: `render` returns an `Element`, and an element may hold state even
/// though the closure that built it may not. Everything the controls vary
/// arrives as props, so a control change re-renders this scope without
/// resetting the signal.
#[component]
fn BurgerPreview(size: String, color: String, disabled: bool) -> Element {
    let mut opened = use_signal(|| false);

    rsx! {
        Flex {
            direction: "row",
            align: "center",
            gap: "md",
            Burger {
                opened: opened(),
                "aria-controls": PANEL_ID,
                onclick: move |_| opened.toggle(),
                size,
                color: match color.as_str() {
                    UNSET => Input::None,
                    color => Input::from(color.to_string()),
                },
                disabled,
            }
            Paper {
                id: PANEL_ID,
                sx: sx().padding("sm").display(if opened() { "block" } else { "none" }),
                "Navigation"
            }
        }
    }
}

#[component]
pub fn BurgerPage() -> Element {
    rsx! {
        DocPage {
            title: "Burger",
            source: "libero/src/components/navigation/burger.rs",
            markdown: "/md/burger.md",
            properties: vec![props("Burger", vec![
                prop("opened", "bool")
                    .doc("`true` draws the X, and either value emits `aria-expanded`, which makes the button a disclosure. Omit it entirely for a burger that opens something that is not one - a modal."),
                prop("onclick", "EventHandler<MouseEvent>")
                    .doc("`Burger` never owns the open state; the panel does, and the caller already holds that signal to drive the panel itself."),
                prop("label", "Callback<bool, String>")
                    .doc("Replaces the theme's two labels, keyed by `opened`. It runs during render, so it can read a live locale."),
                prop("size", "ThemeAwareValue")
                    .default("md")
                    .doc("The glyph's width and height. The bars are a twelfth of it thick, and the button around it is one `spacing.xs` larger."),
                prop("color", "ThemeAwareValue")
                    .default("currentColor")
                    .doc("The bars. Unset they inherit, so styling the button's `color` reaches them."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Passed through to the underlying button."),
            ])],
            lead: rsx! {
                Text {
                    "Three bars that morph into an X. It renders an "
                    Code { source: "ActionIcon" }
                    " - a real "
                    Code { source: "<button type=\"button\">" }
                    " with the ripple and the disabled handling already right - and adds the "
                    "glyph plus the three ARIA facts a burger is usually missing: "
                    Code { source: "aria-expanded" }
                    ", an accessible name that changes with the state, and the "
                    Code { source: "aria-controls" }
                    " you spread to name the panel. The morph animates "
                    Code { source: "background-color" }
                    " and "
                    Code { source: "transform" }
                    " only, and is dropped entirely under "
                    Code { source: "prefers-reduced-motion: reduce" }
                    " - the two states differ in shape, not just in position, so snapping "
                    "between them stays legible."
                }
            },
            Demo {
                component: "Burger",
                children_text: "",
                // The three props the preview owns rather than varies.
                // `opened` is no longer a control - the burger toggles itself -
                // but it is still the prop a caller has to hold, so the block
                // prints it together with the handler that writes it and the
                // `aria-controls` that names what it expands. That attribute is
                // a `GlobalAttributes` pass-through, not a prop, and it is the
                // half of the contract no control could ever have shown.
                fixed: vec![
                    "opened: opened()".to_string(),
                    format!("{:?}: {PANEL_ID:?}", "aria-controls"),
                    "onclick: move |_| opened.toggle()".to_string(),
                ],
                controls: vec![
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("md"),
                    // Unset is `currentColor`, not a palette colour, so the
                    // swatch is painted rather than dropped - see
                    // `codebase/docs/demo-controls`.
                    Control::color("color").with_unset()
                    .unset_swatch("currentColor"),
                    Control::switch("disabled"),
                ],
                wrap: Wrap(wrap),
                render: move |values: DemoValues| rsx! {
                    BurgerPreview {
                        size: values.str("size"),
                        color: values.str("color"),
                        disabled: values.str("disabled") == "true",
                    }
                },
            }
        }
    }
}
