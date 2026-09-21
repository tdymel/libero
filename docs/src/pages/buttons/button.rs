use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::{Button, Code, Input, Text};

fn is_link(values: &DemoValues) -> bool {
    values.str("link") == "true"
}

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
                    .doc("Accent color. A theme color name or any CSS color."),
                prop("variant", "Variant")
                    .default("filled")
                    .doc("Visual style, from most to least emphasis: `filled`, `tonal`, `elevated`, `outlined`, `standard`. `gradient` fills it with the theme's gradient."),
                prop("gradient", "Gradient")
                    .doc("With `variant: \"gradient\"`: this button's own stops and angle, such as `Gradient::default().from(\"success\").to(\"info\").deg(90)`. A literal CSS stop's label contrast is yours to check. Ignored by the other variants."),
                prop("radius", "Size")
                    .default("md")
                    .doc("Corner radius, independent of `size`."),
                prop("size", "Size")
                    .default("md")
                    .doc("Height, padding and font size."),
                prop("full_width", "bool")
                    .default("false")
                    .doc("Stretches the button to fill its container."),
                prop("selected", "bool")
                    .doc("Makes it a toggle button with the selected look. Leave it unset for a plain action."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Disables and dims the button."),
                prop("focusable_when_disabled", "bool")
                    .default("false")
                    .doc("With `disabled`: keeps the button in the Tab order. It renders `aria-disabled` rather than `disabled` and ignores presses."),
                prop("loading", "bool")
                    .default("false")
                    .doc("Shows a `Loader` over the label and ignores clicks. The button stays focusable and keeps its width. Ignored on a link."),
                prop("onclick", "EventHandler<MouseEvent>")
                    .doc("Click handler. Not called on a link."),
                prop("to", "NavigationTarget")
                    .doc("Renders a link instead of a `<button>`. Takes a path, a URL or a typed route (`Route::Foo {}`)."),
                prop("target", "String")
                    .doc("The link's `target` attribute."),
                prop("icon", "Element")
                    .doc("Drawn before the label. It never shrinks."),
                prop("children", "Element")
                    .default("required")
                    .doc("The label, on one line. A long one is cut at the edge."),
            ]).extends("button")],
            accessibility: a11y()
                .handles([
                    "A label cut at the edge is still the full accessible name.",
                    "`selected: Some(false)` announces a toggle that is off. An unset `selected` announces no state.",
                    "`focusable_when_disabled` keeps a disabled button in the Tab order, with `aria-disabled`.",
                ])
                .must(["Pass a label that may be cut as `title` too, so sighted users can read it on hover."]),
            lead: rsx! {
                Text {
                    "A clickable action, a toggle, or a link when "
                    Code { source: "to" }
                    " is set. It defaults to "
                    Code { source: "type=\"button\"" }
                    ", so it never submits a form by accident. A submit button sets "
                    Code { source: "r#type: \"submit\"" }
                    ". For an icon-only button, use "
                    Code { source: "ActionIcon" }
                    "."
                }
            },
            Demo {
                component: "Button",
                children_text: "Save changes",
                controls: vec![
                    Control::color("color"),
                    Control::toggle(
                        "variant",
                        ["filled", "tonal", "elevated", "outlined", "standard", "gradient"],
                    )
                    .labels(["Filled", "Tonal", "Elevated", "Outlined", "Standard", "Gradient"]),
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
                        // A link keeps the look but not `aria-pressed`, and warns.
                        .hidden_when(is_link)
                        .code(|_, values| match values.str("selected").as_str() {
                            "unset" => vec![],
                            value => vec![format!("selected: {value}")],
                        }),
                    Control::switch("disabled"),
                    Control::switch("focusable_when_disabled")
                        .hidden_when(|values| values.str("disabled") != "true"),
                    // An `<a>` has nothing to wait for: ignored, with a warning.
                    Control::switch("loading").hidden_when(is_link),
                    // `to` and `target` together, since the preview's link
                    // must not navigate the docs away.
                    Control::switch("link").code(|_, values| match values.str("link").as_str() {
                        "true" => vec![
                            r#"to: "https://dioxuslabs.com""#.to_string(),
                            r#"target: "_blank""#.to_string(),
                        ],
                        _ => vec![],
                    }),
                    // A `GlobalAttributes` pass-through, printed as the raw identifier. A link has no `type`.
                    Control::toggle("type", ["button", "submit", "reset"])
                        .labels(["Button", "Submit", "Reset"])
                        .hidden_when(is_link)
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
                            _ if is_link(&values) => None,
                            "true" => Some(true),
                            "false" => Some(false),
                            _ => None,
                        },
                        disabled: values.str("disabled") == "true",
                        focusable_when_disabled: values.str("focusable_when_disabled") == "true",
                        loading: values.str("loading") == "true" && !is_link(&values),
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
