use crate::components::{
    Control, Demo, DemoValues, DocPage, UNSET, Wrap, a11y, indent, prop, props,
};
use dioxus::prelude::*;
use libero::components::{Burger, Code, Flex, Input, Paper, Text};
use libero::sx::sx;

/// The panel the preview's `aria-controls` points at. A `Burger` whose
/// `open` is `Some` and whose `aria-controls` names nothing is exactly what
/// the component warns about, so the demo has to own a real one - and
/// `open` is a relation to a neighbour, which the preview needs anyway.
const PANEL_ID: &str = "burger-demo-panel";

// Rendered on both states, hidden rather than absent: `aria-controls`
// pointing at an element that is not in the document is a dangling
// reference, and a disclosure's panel is the thing that exists and is
// hidden.
// snippet: let open = use_signal(|| false);
const PANEL: &str = r#"Paper {
    id: "burger-demo-panel",
    sx: sx().padding("sm").display(if open() { "block" } else { "none" }),
    "Navigation"
}"#;

/// The preview is a pair - the button and the thing it expands - so the block
/// has to be that pair too, signal and all. The signal is the preview's own:
/// nothing outside the block owns `open`.
fn wrap(_values: &DemoValues, source: &str) -> String {
    let mut code = String::from("let mut open = use_signal(|| false);\n\nrsx! {\n");
    code.push_str(&indent(source));
    code.push_str(&indent(PANEL));
    code.push('}');
    code
}

/// The preview owns `open` itself, so the burger in it is the control.
///
/// `Demo` hands `render` a snapshot of the control values and no way to write
/// back, which is why a *control* could never have driven this. It does not
/// have to: `render` returns an `Element`, and an element may hold state even
/// though the closure that built it may not. Everything the controls vary
/// arrives as props, so a control change re-renders this scope without
/// resetting the signal.
#[component]
fn BurgerPreview(size: String, color: String, disabled: bool) -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Flex {
            direction: "row",
            align: "center",
            gap: "md",
            Burger {
                open: open(),
                "aria-controls": PANEL_ID,
                onclick: move |_| open.toggle(),
                size,
                color: match color.as_str() {
                    UNSET => Input::None,
                    color => Input::from(color.to_string()),
                },
                disabled,
            }
            Paper {
                id: PANEL_ID,
                sx: sx().padding("sm").display(if open() { "block" } else { "none" }),
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
                prop("open", "bool")
                    .doc("`true` draws the X. Set either way, it makes the button a disclosure with `aria-expanded`. Leave it unset when the burger opens a modal."),
                prop("onclick", "EventHandler<MouseEvent>")
                    .doc("Click handler. The caller holds the open state and toggles it here."),
                prop("label", "Callback<bool, String>")
                    .doc("Replaces the localization's labels. Called with the open state during render, so it can read a live locale."),
                prop("size", "ThemeAwareValue")
                    .default("md")
                    .doc("The glyph's width and height. The button is one `spacing.xs` step larger. Below 24px it still takes presses in a 24x24 box."),
                prop("color", "ThemeAwareValue")
                    .default("currentColor")
                    .doc("The bars. Unset, they follow the button's `color`."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Disables the button."),
            ])],
            accessibility: a11y()
                .handles([
                    "The name is \"Toggle navigation\" while `open` is set, since `aria-expanded` carries the state, and \"Open navigation\" otherwise. Translate both in the localization's `BurgerLabels`, or per burger with `label`.",
                    "`Burger` warns when `open` is set without `aria-controls`.",
                    "Focus stays on the burger when the panel opens. Moving it is the panel's job, and `Drawer` already traps it.",
                ])
                .must([
                    "Set `open` only when the burger expands a panel, and spread `aria-controls` with that panel's id.",
                    "Leave `open` unset for a burger that opens a modal, since a modal is not expanded by its trigger.",
                ]),
            lead: rsx! {
                Text {
                    "Three bars that morph into an X. It renders an "
                    Code { source: "ActionIcon" }
                    " with "
                    Code { source: "aria-expanded" }
                    " and a name that stays the same in both states. You spread "
                    Code { source: "aria-controls" }
                    " to name the panel it opens. Under reduced motion the bars snap instead "
                    "of morphing."
                }
            },
            Demo {
                component: "Burger",
                children_text: "",
                // The three props the preview owns rather than varies.
                // `open` is no longer a control - the burger toggles itself -
                // but it is still the prop a caller has to hold, so the block
                // prints it together with the handler that writes it and the
                // `aria-controls` that names what it expands. That attribute is
                // a `GlobalAttributes` pass-through, not a prop, and it is the
                // half of the contract no control could ever have shown.
                fixed: vec![
                    "open: open()".to_string(),
                    format!("{:?}: {PANEL_ID:?}", "aria-controls"),
                    "onclick: move |_| open.toggle()".to_string(),
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
