use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::components::{Chip, Code, Input, Text};

#[component]
pub fn ChipPage() -> Element {
    rsx! {
        DocPage {
            title: "Chip",
            source: "libero/src/components/form/chip.rs",
            markdown: "/md/chip.md",
            properties: vec![props("Chip", vec![
                prop("color", "ThemeAwareValue")
                    .default("primary")
                    .doc("Accent color; a theme color name or a literal CSS color."),
                prop("variant", "Variant")
                    .default("filled")
                    .doc("The unselected look; a checked chip is a tonal container whatever its variant."),
                prop("size", "Size").default("md").doc("Controls height, padding, and font size."),
                prop("radius", "Size")
                    .default("xl")
                    .doc("Corner radius, independent of size."),
                prop("checked", "bool")
                    .doc("Selection state - pair it with `onchange`. Left out, a chip with a `name` keeps its own state unless that name binds it to the form around it."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Disables interaction and dims the chip."),
                prop("onchange", "EventHandler<bool>")
                    .doc("Called with the value `checked` should take next. Its presence makes the chip a real checkbox."),
                prop("name", "FieldName<bool>")
                    .doc("Makes the chip a checkbox that posts under this name. A path - `Filters::FIELDS.open()` - also binds it to the surrounding `Form`'s value when the chip has no `onchange`, as on `Checkbox`."),
                prop("value", "String")
                    .default("on")
                    .doc("What the chip posts under its `name` when it is checked, so a row of filter chips can share one name. Left out, it posts the browser's `on`, as `Checkbox` does."),
                prop("onclick", "EventHandler<MouseEvent>")
                    .doc("A plain action; its presence makes the chip a `button`."),
                prop("to", "NavigationTarget")
                    .doc("Renders a router-aware link instead. Takes precedence over `onclick`."),
                prop("target", "String")
                    .doc("Link target, e.g. `_blank`. Only with `to`."),
                prop("icon", "Element")
                    .doc("Drawn before the label, with a gap; it never shrinks."),
                prop("trailing", "Element")
                    .doc("Drawn after the label, with a gap; it never shrinks - a remove x. Outside a checkbox chip's `label`, so it may be a button, but not on an `onclick` or `to` chip."),
                prop("children", "Element")
                    .doc("The label, laid out as the chip's own flex items and cut at its edge. Text and `Icon` only - a `<label>` hijacks clicks on nested controls."),
            ])],
            lead: rsx! {
                Text {
                    "A compact token. With "
                    Code { source: "onchange" }
                    " it is a real checkbox - a visually hidden "
                    Code { source: "input" }
                    " plus a "
                    Code { source: "label" }
                    ", so selection is announced and Space toggles it. Without it, a plain tag."
                }
            },
            // snippet: let mut selected = use_signal(|| false);
            Demo {
                component: "Chip",
                children_text: "rust",
                controls: vec![
                    Control::color("color"),
                    Control::toggle(
                        "variant",
                        ["filled", "tonal", "elevated", "outlined", "text"],
                    )
                    .labels(["Filled", "Tonal", "Elevated", "Outlined", "Text"]),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("xl"),
                    // What the chip is: a plain tag, a checkbox, a button or
                    // a link. The last three never combine.
                    Control::toggle("kind", ["tag", "filter", "action", "link"])
                        .labels(["Tag", "Filter", "Action", "Link"])
                        .code(|_, values| match values.str("kind").as_str() {
                            // Controlled state is `checked` + `onchange`; the
                            // library warns about one without the other.
                            "filter" => vec![
                                "checked: selected()".to_string(),
                                "onchange: move |next| selected.set(next)".to_string(),
                            ],
                            "action" => vec!["onclick: move |_| {}".to_string()],
                            "link" => vec![
                                r#"to: "https://dioxuslabs.com""#.to_string(),
                                r#"target: "_blank""#.to_string(),
                            ],
                            _ => vec![],
                        }),
                    // The preview writes `onchange` back into this switch;
                    // `kind` prints the pair a caller writes.
                    Control::switch("checked")
                        .hidden_when(|values| values.str("kind") != "filter")
                        .code(|_, _| vec![]),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| rsx! {
                    Chip {
                        color: values.str("color"),
                        variant: values.str("variant"),
                        size: values.str("size"),
                        radius: values.str("radius"),
                        // Both or neither: `checked` alone can never
                        // change, `onchange` alone can never look selected.
                        checked: (values.str("kind") == "filter")
                            .then(|| values.str("checked") == "true"),
                        onchange: (values.str("kind") == "filter").then(|| {
                            let values = values.clone();
                            EventHandler::new(move |next: bool| values.set("checked", next.to_string()))
                        }),
                        onclick: (values.str("kind") == "action")
                            .then(|| EventHandler::new(move |_: MouseEvent| {})),
                        to: match values.str("kind").as_str() {
                            "link" => Input::from("https://dioxuslabs.com"),
                            _ => Input::None,
                        },
                        target: (values.str("kind") == "link").then(|| "_blank".to_string()),
                        disabled: values.str("disabled") == "true",
                        "rust"
                    }
                },
            }
        }
    }
}
