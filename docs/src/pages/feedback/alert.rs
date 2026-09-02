use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, prop, props};
use crate::icons::CheckmarkIcon;
use dioxus::prelude::*;
use libero::{
    components::{Alert, Button, Code, CodeBlock, Fields, Form, Rule, Text, TextField, not_empty},
    use_theme,
};

const TITLE: &str = "Card expiring";
const MESSAGE: &str = "Your card ends 09/26. Update it before the next invoice.";

const ICON_CODE: &str = r#"Alert {
    color: "success",
    title: "Saved",
    icon: rsx! { CheckmarkIcon {} },
    "Your changes are live."
}"#;

const FORM_CODE: &str = r#"Form {
    value: contact,
    summary_title: "Please fix these first:",
    TextField { label: "Name", name: Contact::FIELDS.name(), validate: not_empty.error("Enter your name.") }
    Button { r#type: "submit", "Send" }
}"#;

#[derive(Clone, PartialEq, Default, Fields)]
pub struct Contact {
    pub name: String,
}

#[component]
fn ContactForm() -> Element {
    let contact = use_store(Contact::default);

    rsx! {
        Form {
            value: contact,
            summary_title: "Please fix these first:",
            TextField {
                label: "Name",
                name: Contact::FIELDS.name(),
                validate: not_empty.error("Enter your name."),
            }
            Button { r#type: "submit", "Send" }
        }
    }
}

#[component]
pub fn AlertPage() -> Element {
    let theme = use_theme();

    rsx! {
        DocPage {
            title: "Alert",
            source: "libero/src/components/feedback/alert.rs",
            markdown: "/md/alert.md",
            properties: vec![props("Alert", vec![
                prop("title", "String")
                    .doc("The heading, and the alert's accessible name through `aria-labelledby`. Text, not markup - markup in a name is dropped from it."),
                prop("icon", "Element")
                    .doc("A leading glyph, rendered `aria-hidden`. The library ships no icon set; this is your own."),
                prop("color", "ThemeAwareValue")
                    .default("info")
                    .doc("The tint; a theme color name or a literal CSS color. Severity is yours to state - `Form`'s summary passes `error`."),
                prop("variant", "ButtonVariant")
                    .default("tonal")
                    .doc("Chrome, shared with `Button` and `Badge`: `filled`, `tonal`, `elevated`, `outlined`, `standard`. No hover response - an alert is not a target."),
                prop("radius", "ThemeAwareValue")
                    .default("md")
                    .doc("A size step or any CSS length."),
                prop("onclose", "EventHandler<()>")
                    .doc("Its presence is what shows the close button. Closing is yours: unmount the alert."),
                prop("close_label", "String")
                    .default("Close")
                    .doc("The close button's accessible name."),
                prop("children", "Element").doc("The message, and the alert's description through `aria-describedby`."),
            ])],
            lead: rsx! {
                Text {
                    "A tinted surface for something the reader has to know. It renders "
                    Code { source: "role=\"alert\"" }
                    " as a default your own "
                    Code { source: "role" }
                    " replaces - a message that should wait its turn takes "
                    Code { source: "role: \"status\"" }
                    ". It takes no focus and does not close on Escape: it is not an overlay."
                }
            },
            Demo {
                component: "Alert",
                children_text: MESSAGE,
                controls: vec![
                    Control::toggle(
                        "variant",
                        ["filled", "tonal", "elevated", "outlined", "standard"],
                    )
                    .labels(["Filled", "Tonal", "Elevated", "Outlined", "Standard"])
                    .default(theme.alert.variant.as_str()),
                    Control::color(
                        "color",
                        ["primary", "secondary", "success", "error", "warning", "info", "neutral"],
                    )
                    .default(theme.alert.color),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default(theme.alert.radius.as_str()),
                    Control::switch("title").default("true").code(|_, values| {
                        match values.str("title").as_str() {
                            "true" => vec![format!("title: {TITLE:?}")],
                            _ => vec![],
                        }
                    }),
                    // A closure is not a value a control can hold, so this
                    // prints the honest minimum.
                    Control::switch("onclose").code(|_, values| {
                        match values.str("onclose").as_str() {
                            "true" => vec!["onclose: move |_| {}".to_string()],
                            _ => vec![],
                        }
                    }),
                ],
                render: move |values: DemoValues| rsx! {
                    Alert {
                        variant: values.str("variant"),
                        color: values.str("color"),
                        radius: values.str("radius"),
                        title: (values.str("title") == "true").then(|| TITLE.to_string()),
                        onclose: (values.str("onclose") == "true")
                            .then(|| EventHandler::new(|_| {})),
                        {MESSAGE}
                    }
                },
            }
            DocSection { title: "With an icon",
                Text {
                    "The icon slot is "
                    Code { source: "aria-hidden" }
                    ": the glyph repeats what the title already says."
                }
                Alert {
                    color: "success",
                    title: "Saved",
                    icon: rsx! { CheckmarkIcon {} },
                    "Your changes are live."
                }
                CodeBlock { source: ICON_CODE, language: "rust" }
            }
            DocSection { title: "In a form",
                Text {
                    "A blocked submit shows "
                    Code { source: "Form" }
                    "'s error summary, which is an "
                    Code { source: "Alert" }
                    " with "
                    Code { source: "color: \"error\"" }
                    ". The form moves focus to it, and each line focuses its field. Submit this one empty."
                }
                ContactForm {}
                CodeBlock { source: FORM_CODE, language: "rust" }
            }
        }
    }
}
