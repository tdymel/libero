use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::components::{Button, Code, Input, Text};

#[component]
pub fn ButtonPage() -> Element {
    rsx! {
        DocPage {
            title: "Button",
            source: "libero/src/components/buttons/button.rs",
            markdown: "/md/button.md",
            properties: vec![props("Button", vec![
                prop("color", "ThemeAwareValue")
                    .default("primary")
                    .doc("Accent color; a theme color name or a literal CSS color."),
                prop("variant", "Variant")
                    .default("filled")
                    .doc("Visual style, in Material 3's descending emphasis order: `filled`, `tonal`, `elevated`, `outlined`, `standard`."),
                prop("radius", "Size")
                    .default("md")
                    .doc("Corner radius, independent of size."),
                prop("size", "Size")
                    .default("md")
                    .doc("Controls height, padding, and font size."),
                prop("full_width", "bool")
                    .default("false")
                    .doc("Stretches the button to fill its container."),
                prop("selected", "bool")
                    .doc("Turns the button into a toggle, rendering `aria-pressed` and the selected look. Omit to keep it a plain action."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Disables interaction and dims the button."),
                prop("loading", "bool")
                    .default("false")
                    .doc("Overlays a `Loader` on the label and swallows clicks, but keeps the button focusable. The label stays in the tree as the accessible name and holds the width; the button renders `aria-busy` and `aria-disabled`. Ignored on a link."),
                prop("onclick", "EventHandler<MouseEvent>")
                    .doc("Click handler; not called when the button renders as a link."),
                prop("to", "NavigationTarget")
                    .doc("Renders as a router-aware link instead of a `<button>`. A path/URL or a typed route (`Route::Foo {}`)."),
                prop("target", "String")
                    .doc("The link's `target` attribute, when `to` is set."),
                prop("icon", "Element")
                    .doc("Drawn before the label, with a gap; it never shrinks."),
                prop("children", "Element")
                    .doc("The button's label, laid out as its own flex items, on one line: a long one is cut at the edge and stays the full accessible name."),
            ]).extends("button")],
            lead: rsx! {
                Text {
                    "A clickable control, or a router-aware link when "
                    Code { source: "to" }
                    " is set. A plain "
                    Code { source: "<button>" }
                    " submits an enclosing form; ours defaults to "
                    Code { source: "type=\"button\"" }
                    " instead, so a submit or reset button says so."
                }
            },
            Demo {
                component: "Button",
                children_text: "Save changes",
                controls: vec![
                    Control::color("color"),
                    Control::toggle(
                        "variant",
                        ["filled", "tonal", "elevated", "outlined", "standard"],
                    )
                    .labels(["Filled", "Tonal", "Elevated", "Outlined", "Standard"]),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::switch("full_width"),
                    // Off is still a toggle (`aria-pressed="false"`); unset
                    // is a plain action.
                    Control::toggle("selected", ["unset", "false", "true"])
                        .labels(["Unset", "Off", "On"])
                        .default("unset")
                        .code(|_, values| match values.str("selected").as_str() {
                            "unset" => vec![],
                            value => vec![format!("selected: {value}")],
                        }),
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
                    // A `GlobalAttributes` pass-through rather than a
                    // prop, so it prints as the raw identifier. A link has
                    // no `type`.
                    Control::toggle("type", ["button", "submit", "reset"])
                        .hidden_when(|values| values.str("link") == "true")
                        .code(
                        |control, values| match values.str("type") {
                            _ if values.str("link") == "true" => vec![],
                            value if value == control.default => vec![],
                            value => vec![format!("r#type: {value:?}")],
                        },
                    ),
                ],
                render: move |values: DemoValues| rsx! {
                    Button {
                        color: values.str("color"),
                        variant: values.str("variant"),
                        size: values.str("size"),
                        radius: values.str("radius"),
                        full_width: values.str("full_width") == "true",
                        selected: match values.str("selected").as_str() {
                            "true" => Some(true),
                            "false" => Some(false),
                            _ => None,
                        },
                        disabled: values.str("disabled") == "true",
                        loading: values.str("loading") == "true",
                        r#type: values.str("type"),
                        to: match values.str("link").as_str() {
                            "true" => Input::from("https://dioxuslabs.com"),
                            _ => Input::None,
                        },
                        target: (values.str("link") == "true").then(|| "_blank".to_string()),
                        "Save changes"
                    }
                },
            }
        }
    }
}
