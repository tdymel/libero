use crate::components::{
    Control, Demo, DemoValues, DocPage, DocSection, a11y, gradient_controls, gradient_value,
    not_gradient_variant, prop, props,
};
use dioxus::prelude::*;
use libero::components::{Button, ButtonPart, Code, Input, Text};

fn is_link(values: &DemoValues) -> bool {
    values.str("link") == "true"
}

#[component]
pub fn ButtonPage() -> Element {
    let [gradient_to, gradient_deg] = gradient_controls(not_gradient_variant);
    rsx! {
        DocPage {
            title: "Button",
            source: "libero/src/components/buttons/button.rs",
            markdown: "/md/button.md",
            properties: vec![props("Button", vec![
                prop("color", "ThemeAwareValue")
                    .default("primary")
                    .doc("Accent color. A theme color name or any CSS color. Under a gradient, its first stop. Unset, a `standard` or `outlined` button on a gradient `Paper`, a coloured or gradient `Header`, a filled, tonal or gradient `Alert` or a `Mark` takes that surface's text color."),
                prop("variant", "Variant")
                    .default("filled")
                    .doc("Visual style, from most to least emphasis: `filled`, `tonal`, `elevated`, `outlined`, `standard`. `gradient` fills it from `color` into the theme's second stop."),
                prop("gradient", "Gradient")
                    .doc("With `variant: \"gradient\"`: the second stop and the angle, as `(\"info\", 90)` or `Gradient::default().to(\"info\").deg(90)`. The first stop is `color`. A literal CSS stop's label contrast is yours to check. Ignored by the other variants."),
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
                prop("parts", "Parts<ButtonPart>")
                    .doc("Styles for the inner parts in the Style API tab, under `sx`."),
                prop("children", "Element")
                    .default("required")
                    .doc("The label, on one line. A long one is cut at the edge."),
            ]).extends("button")
            .parts("ButtonPart", vec![
                (ButtonPart::Icon, "The `icon` wrapper."),
            ])],
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
                    gradient_to,
                    gradient_deg,
                    Control::sizes("size")
                        .default("md"),
                    Control::sizes("radius")
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
                        gradient: gradient_value(&values),
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

            DocSection {
                title: "Labels and icons",
                Text {
                    "A "
                    Code { source: "Button" }
                    " shows a label, with an optional "
                    Code { source: "icon" }
                    " before it. "
                    Code { source: "ActionIcon" }
                    " is the same button reduced to a square icon: it requires an "
                    Code { source: "aria_label" }
                    ", keeps a 24px target and defaults to no background. A "
                    Code { source: "Button" }
                    " with no text would be a wide pill without an accessible name."
                }
            }
        }
    }
}
