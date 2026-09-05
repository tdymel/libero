use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::components::{ActionIcon, Code, Input, Text};

use crate::icons::CheckmarkIcon;

/// The svg the button wraps - a subtree, so the code block prints it verbatim.
const CHILDREN: &str = "CheckmarkIcon {}";

#[component]
pub fn ActionIconPage() -> Element {
    rsx! {
        DocPage {
            title: "ActionIcon",
            source: "libero/src/components/inputs/action_icon.rs",
            markdown: "/md/action_icon.md",
            properties: vec![props("ActionIcon", vec![
                prop("variant", "Variant")
                    .doc("Chrome, shared with `Button`: `filled`, `tonal`, `elevated`, `outlined`, `standard`. Unset, with `color` also unset, the button contributes no background or color of its own and inherits the surrounding text color."),
                prop("color", "ThemeAwareValue")
                    .doc("Accent color; a theme color name or a literal CSS color. Setting it turns on variant styling even if `variant` itself is unset (as `filled`)."),
                prop("size", "ThemeAwareValue")
                    .default("md")
                    .doc("Button size, independent of the wrapped icon's own size."),
                prop("radius", "ThemeAwareValue")
                    .default("sm")
                    .doc("Corner radius, independent of size."),
                prop("aria_label", "String")
                    .doc("Required: an icon-only button has no visible text for a screen reader to announce."),
                prop("selected", "bool")
                    .doc("Turns the button into a toggle, rendering `aria-pressed`, and the selected look once `variant` or `color` turns the chrome on. Omit to keep it a plain action."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Disables interaction and dims the button."),
                prop("loading", "bool")
                    .default("false")
                    .doc("Overlays a `Loader` on the icon and swallows clicks, but keeps the button focusable. The icon stays in the tree and holds the box; the button renders `aria-busy` and `aria-disabled`. Ignored on a link."),
                prop("onclick", "EventHandler<MouseEvent>")
                    .doc("Click handler; not called when the button renders as a link."),
                prop("to", "NavigationTarget")
                    .doc("Renders as a router-aware link instead of a `<button>`."),
                prop("target", "String")
                    .doc("The link's `target` attribute, when `to` is set."),
                prop("children", "Element").doc("The icon to show."),
            ])],
            lead: rsx! {
                Text {
                    Code { source: "Icon" }
                    "'s sizing, color, and variant system, rendered as a real "
                    Code { source: "<button>" }
                    " with click handling and required a11y - for icon-only actions like a "
                    "copy, close, or delete button. "
                    Code { source: "aria_label" }
                    " is required, not optional: an icon-only button has no visible text for "
                    "a screen reader to announce."
                }
                Text {
                    "With neither "
                    Code { source: "variant" }
                    " nor "
                    Code { source: "color" }
                    " set, it contributes no background or color of its own and inherits "
                    "the surrounding text color, rather than defaulting to a filled badge the "
                    "way "
                    Code { source: "Icon" }
                    " does. That is how "
                    Code { source: "Code" }
                    "'s own copy button is built."
                }
            },
            Demo {
                component: "ActionIcon",
                children_text: "",
                children_code: CHILDREN,
                fixed: vec!["aria_label: \"Confirm\"".to_string()],
                controls: vec![
                    Control::toggle("variant", ["filled", "tonal", "outlined", "standard"])
                        .labels(["Filled", "Tonal", "Outlined", "Standard"]),
                    // A bare `primary` is what an unset `color` resolves
                    // to, so that swatch prints nothing.
                    Control::color("color"),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("sm"),
                    Control::switch("selected"),
                    Control::switch("disabled"),
                    Control::switch("loading"),
                    // `to` and `target` together, since the preview's link
                    // must not navigate the docs away.
                    Control::switch("link").code(|_, values| match values.str("link").as_str() {
                        "true" => vec![
                            r#"to: "https://dioxuslabs.com""#.to_string(),
                            r#"target: "_blank""#.to_string(),
                        ],
                        _ => vec![],
                    }),
                ],
                render: move |values: DemoValues| rsx! {
                    ActionIcon {
                        variant: values.str("variant"),
                        color: match values.str("color").as_str() {
                            "primary" => Input::None,
                            color => Input::from(color),
                        },
                        size: values.str("size"),
                        radius: values.str("radius"),
                        // `Some(false)` is still a toggle (`aria-pressed="false"`);
                        // unset is not.
                        selected: match values.str("selected").as_str() {
                            "true" => Some(true),
                            _ => None,
                        },
                        disabled: values.str("disabled") == "true",
                        loading: values.str("loading") == "true",
                        to: match values.str("link").as_str() {
                            "true" => Input::from("https://dioxuslabs.com"),
                            _ => Input::None,
                        },
                        target: (values.str("link") == "true").then(|| "_blank".to_string()),
                        aria_label: "Confirm",
                        CheckmarkIcon {}
                    }
                },
            }
        }
    }
}
