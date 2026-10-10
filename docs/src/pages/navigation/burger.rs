use crate::components::{
    Control, Demo, DemoValues, DocPage, UNSET, Wrap, a11y, indent, prop, props,
};
use dioxus::prelude::*;
use libero::components::{Burger, BurgerPart, Code, Flex, Input, Paper, Text};
use libero::sx::sx;
use libero::use_theme;

/// The panel `aria-controls` names: with `open` set and no real target, `Burger` warns.
const PANEL_ID: &str = "burger-demo-panel";

// Hidden rather than absent, or `aria-controls` dangles.
// snippet: let open = use_signal(|| false);
const PANEL: &str = r#"Paper {
    id: "burger-demo-panel",
    sx: sx().padding("sm").display(if open() { "block" } else { "none" }),
    "Navigation"
}"#;

/// Prints the pair, the button and what it expands, with the `open` signal.
fn wrap(_values: &DemoValues, source: &str) -> String {
    let mut code = String::from("let mut open = use_signal(|| false);\n\nrsx! {\n");
    code.push_str(&indent(source));
    code.push_str(&indent(PANEL));
    code.push('}');
    code
}

/// Owns `open`, so the burger is its own control. A component, since `render` can't hold state;
/// control changes arrive as props without resetting the signal.
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
    let theme = use_theme();
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
                    .default(theme.burger.size.as_str())
                    .doc("The glyph's width and height. The button is one `spacing.xs` step larger. Below 24px it still takes presses in a 24x24 box."),
                prop("color", "ThemeAwareValue")
                    .default("currentColor")
                    .doc("The bars. Unset, they follow the button's `color`."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Disables the button."),
                prop("parts", "Parts<BurgerPart>")
                    .doc("Styles for the inner parts in the Style API tab, under `sx`."),
            ])
            .parts("BurgerPart", vec![
                (BurgerPart::Glyph, "The middle bar. The outer two are its `::before` and `::after`."),
            ])],
            accessibility: a11y()
                .handles([
                    "The name is \"Toggle navigation\" while `open` is set, since `aria-expanded` carries the state, and \"Open navigation\" otherwise. Translate both in the localization's `BurgerLabels`, or per burger with `label`.",
                    "`Burger` warns when `open` is set without `aria-controls`.",
                    "Focus stays on the burger when the panel opens. Moving it is the panel's job, and `Drawer` already traps it.",
                ])
                .must([
                    "Set `open` only when the burger expands a panel, and spread `aria-controls` with that panel's id.",
                    "Leave `open` unset for a burger that opens a modal, since a modal is not expanded by its trigger. Its name is \"Open navigation\" by default, so set `label` when the modal is not navigation.",
                ])
                .example("A burger that opens the site menu panel, with `open: menu_open()` and `\"aria-controls\": \"site-menu\"`: a screen reader reads \"Toggle navigation\", expanded or collapsed."),
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
                // Owned, not varied: `open`, its handler, and the `aria-controls` pass-through.
                fixed: vec![
                    "open: open()".to_string(),
                    format!("{:?}: {PANEL_ID:?}", "aria-controls"),
                    "onclick: move |_| open.toggle()".to_string(),
                ],
                controls: vec![
                    Control::sizes("size").default(theme.burger.size.as_str()),
                    // Unset is `currentColor`, so the swatch is painted (`codebase/docs/demo-controls`).
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
