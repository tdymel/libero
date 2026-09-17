use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, prop, props};
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
            source: "libero/src/components/buttons/action_icon.rs",
            markdown: "/md/action_icon.md",
            properties: vec![props("ActionIcon", vec![
                prop("variant", "Variant")
                    .doc("Visual style, shared with `Button`: `filled`, `tonal`, `elevated`, `outlined`, `standard`. With `color` also unset, the button takes the surrounding text color."),
                prop("color", "ThemeAwareValue")
                    .doc("Accent color. A theme color name or any CSS color. Set alone, it gives the theme's default variant, `filled`."),
                prop("size", "ThemeAwareValue")
                    .default("md")
                    .doc("Button size, independent of the icon's own size."),
                prop("radius", "ThemeAwareValue")
                    .default("sm")
                    .doc("Corner radius, independent of `size`."),
                prop("aria_label", "String")
                    .default("required")
                    .doc("The button's accessible name."),
                prop("selected", "bool")
                    .doc("Makes it a toggle button. The selected look shows once `variant` or `color` is set. Leave it unset for a plain action."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Disables and dims the button."),
                prop("loading", "bool")
                    .default("false")
                    .doc("Shows a `Loader` over the icon and ignores clicks. The button stays focusable. Ignored on a link."),
                prop("onclick", "EventHandler<MouseEvent>")
                    .doc("Click handler. Not called on a link."),
                prop("to", "NavigationTarget")
                    .doc("Renders a link instead of a `<button>`."),
                prop("target", "String")
                    .doc("The link's `target` attribute."),
                prop("children", "Element").default("required").doc("The icon."),
            ])],
            lead: rsx! {
                Text {
                    "An icon-only button for actions like copy, close or delete. It renders a "
                    Code { source: "<button>" }
                    ", or a link when "
                    Code { source: "to" }
                    " is set. "
                    Code { source: "aria_label" }
                    " is required, because the icon gives a screen reader nothing to read."
                }
                Text {
                    "With neither "
                    Code { source: "variant" }
                    " nor "
                    Code { source: "color" }
                    " set, it has no background of its own and takes the surrounding text color."
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
            DocSection { title: "Accessibility",
                Text {
                    "Below 24px ("
                    Code { source: "xs" }
                    " and "
                    Code { source: "sm" }
                    ") the button still takes presses in a 24x24 box centred on it. Keep other targets 2px ("
                    Code { source: "sm" }
                    ") or 4px ("
                    Code { source: "xs" }
                    ") away, or the one drawn later takes the overlap."
                }
            }
        }
    }
}
