use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, CopyButton, Input, Text};

#[component]
pub fn CopyButtonPage() -> Element {
    rsx! {
        DocPage {
            title: "CopyButton",
            source: "libero/src/components/buttons/copy_button.rs",
            markdown: "/md/copy_button.md",
            properties: vec![props("CopyButton", vec![
                prop("value", "String")
                    .doc("Required. The text a press writes to the clipboard."),
                prop("variant", "Variant")
                    .doc("Visual style, as on `ActionIcon`. With `color` also unset, the button draws no chrome of its own."),
                prop("color", "ThemeAwareValue")
                    .doc("Accent color. A theme color name or any CSS color."),
                prop("size", "ThemeAwareValue")
                    .default("md")
                    .doc("Button size."),
                prop("radius", "ThemeAwareValue")
                    .default("sm")
                    .doc("Corner radius, independent of `size`."),
                prop("aria_label", "String")
                    .default("\"Copy\"")
                    .doc("The button's name, such as \"Copy link\". Unset, the localization's."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Disables and dims the button."),
            ])],
            accessibility: a11y()
                .handles([
                    "An always-mounted `role=\"status\"` region says \"Copied\", or that the copy failed, once the platform answers. A second copy announces again.",
                    "The icon and the status reset when the pointer or the focus leaves.",
                    "Unset, `aria_label` is the localization's `copy`. The words are `CopyButtonLabels`.",
                ])
                .must([
                    "Name it after what it copies with `aria_label`, such as \"Copy link\".",
                    "For a copy control of your own on `use_clipboard()`, say the result in a status region that is already mounted.",
                ]),
            lead: rsx! {
                Text {
                    "An icon button that copies a value to the clipboard. Once the write "
                    "landed, the icon turns into a check and a screen reader hears "
                    "\"Copied\". Both reset when the pointer or the focus leaves. "
                    Code { source: "CodeBlock" }
                    "'s copy control is one."
                }
                Text {
                    "Name it after what it copies with "
                    Code { source: "aria_label" }
                    ", such as \"Copy link\". For a copy control of your own, build on "
                    Code { source: "use_clipboard()" }
                    ": "
                    Code { source: "copy(text)" }
                    " starts the write, "
                    Code { source: "copied()" }
                    " or "
                    Code { source: "failed()" }
                    " rises once the platform answers, and "
                    Code { source: "reset()" }
                    " clears both. Say the result in a status region that is already "
                    "mounted. On the web the browser allows the write only over HTTPS or on "
                    "localhost, inside a user action; natively it needs the "
                    Code { source: "native" }
                    " feature."
                }
            },
            Demo {
                component: "CopyButton",
                children_text: "",
                fixed: vec![
                    "value: \"cargo add libero\"".to_string(),
                    "aria_label: \"Copy the install command\"".to_string(),
                ],
                controls: vec![
                    Control::toggle(
                        "variant",
                        ["filled", "tonal", "elevated", "outlined", "standard"],
                    )
                    .labels(["Filled", "Tonal", "Elevated", "Outlined", "Standard"]),
                    // A bare `primary` is what an unset `color` resolves
                    // to, so that swatch prints nothing.
                    Control::color("color"),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("sm"),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| rsx! {
                    CopyButton {
                        value: "cargo add libero",
                        aria_label: "Copy the install command",
                        variant: values.str("variant"),
                        color: match values.str("color").as_str() {
                            "primary" => Input::None,
                            color => Input::from(color),
                        },
                        size: values.str("size"),
                        radius: values.str("radius"),
                        disabled: values.str("disabled") == "true",
                    }
                },
            }
        }
    }
}
