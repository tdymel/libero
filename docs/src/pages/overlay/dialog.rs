use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, indent, prop, props};
use dioxus::prelude::*;
use libero::components::{Button, Code, Dialog, DialogPart, Text};
use libero::use_theme;

const CONTENT: &str = r#"Text { "notes.md has changes you have not saved." }"#;

/// Outside a modal the close button calls `onclose`, so the snippet owns the
/// open state and a way back.
fn wrap_open(values: &DemoValues, code: &str) -> String {
    if values.str("close_button") != "true" {
        return code.to_string();
    }
    format!(
        "let mut open = use_signal(|| true);\n\n\
         rsx! {{\n    \
             if open() {{\n{}    }} else {{\n        \
                 Button {{ variant: \"outlined\", onclick: move |_| open.set(true), \"Reopen\" }}\n    \
             }}\n\
         }}",
        indent(&indent(code)),
    )
}

/// The open state needs a scope of its own: `Demo` calls its `render` closure
/// from its own.
#[component]
fn DialogDemo(titled: bool, close_button: bool, size: String, radius: String) -> Element {
    let mut open = use_signal(|| true);

    // Without the button the snippet has no open state, so nor does this.
    if close_button && !open() {
        return rsx! {
            Button { variant: "outlined", onclick: move |_| open.set(true), "Reopen" }
        };
    }
    rsx! {
        Dialog {
            title: titled.then(|| "Unsaved changes".to_string()),
            aria_label: (!titled).then(|| "Unsaved changes".to_string()),
            onclose: close_button.then_some(EventHandler::new(move |_| open.set(false))),
            size,
            radius,
            Text { "notes.md has changes you have not saved." }
        }
    }
}

#[component]
pub fn DialogPage() -> Element {
    let theme = use_theme();
    let radius = theme.dialog.radius.unwrap_or(theme.paper.radius);
    rsx! {
        DocPage {
            title: "Dialog",
            source: "libero/src/components/overlay/dialog.rs",
            markdown: "/md/dialog.md",
            properties: vec![
                props("Dialog", vec![
                    prop("aria_label", "String").doc("The dialog's accessible name. Wins over `title`."),
                    prop("title", "String").doc("Heading, and the accessible name unless `aria_label` is set."),
                    prop("close_button", "bool")
                        .default("in a modal, or with onclose")
                        .doc("Header close button. Inside a modal it closes the modal, outside one it calls `onclose`."),
                    prop("onclose", "EventHandler<()>")
                        .doc("Called by the close button outside a modal."),
                    prop("ondismiss", "Callback<Dismiss, bool>")
                        .default("all close, an alertdialog's backdrop does not")
                        .doc("Inside a modal: asked before Escape, the backdrop or Back closes it. Return `false` to keep it open, such as a form with unsaved input. The close button and your own `close()` are not asked."),
                    prop("close_label", "String").default("\"Close\"").doc("The close button's accessible name, such as \"Close cart\". Unset, the localization's `common.close`."),
                    prop("radius", "Size").default(radius.as_str()).doc("Corner radius from the radius scale. Other values go through `sx`."),
                    prop("size", "ThemeAwareValue")
                        .default(theme.dialog.size.as_str())
                        .doc("Caps the width from the dialog scale. `md` is 510px."),
                    prop("variables", "Variables")
                        .doc("CSS variables layered onto the dialog's own, as `Drawer` does."),
                    prop("parts", "Parts<DialogPart>")
                        .doc("Styles for the inner parts in the Style API tab, under `sx`: `Parts::new().part(DialogPart::Title, sx().font_size(\"lg\"))`."),
                    prop("children", "Element").default("required").doc("The dialog's content."),
                ])
                .parts("DialogPart", vec![
                    (DialogPart::Header, "The row holding the title and the close button. Rendered only with a title or a close button."),
                    (DialogPart::Title, "The title heading."),
                    (DialogPart::Close, "The close button."),
                ]),
            ],
            accessibility: a11y()
                .handles([
                    "Outside a modal, a close button without `onclose` warns in debug builds.",
                    "`role: \"alertdialog\"`, as a spread attribute, makes it an alert dialog for a message that needs an answer, such as a delete confirmation. It ignores a click on the backdrop: a stray click is no answer (APG). Any other `role` stays `dialog`.",
                ])
                .must([
                    "Name it with `title` or `aria_label`.",
                    "Give an `alertdialog` an `aria_describedby` pointing at its message, so a screen reader reads it on open.",
                    "Open it in a modal: the focus trap, Escape and backdrop dismissal come from the modal. A `Dialog` on its own has none of them.",
                    "Outside a modal, give a close button `onclose`, or it closes nothing.",
                ])
                .example("A confirm dialog, `Dialog { title: \"Delete file?\", role: \"alertdialog\", aria_describedby: \"delete-message\" }` inside a modal: the title names it, the message describes it, and the modal traps focus and closes it on Escape."),
            lead: rsx! {
                Text {
                    "The dialog surface, a "
                    Code { source: "Paper" }
                    " with a header, padding and "
                    Code { source: "role=\"dialog\"" }
                    ". It does no positioning. Open it in a modal with "
                    Code { source: "use_modal" }
                    ", and it names itself from "
                    Code { source: "title" }
                    " and closes the modal from its header button. Outside a modal, as here, "
                    "the button calls "
                    Code { source: "onclose" }
                    "."
                }
            },
            Demo {
                component: "Dialog",
                children_text: "",
                children_code: CONTENT.to_string(),
                controls: vec![
                    Control::sizes("size").default(theme.dialog.size.as_str()),
                    Control::sizes("radius").default(radius.as_str()),
                    // Unnamed, a dialog is announced as just "dialog", so off
                    // names it through `aria_label` instead.
                    Control::switch("title")
                        .default("true")
                        .code(|_, values| match values.str("title").as_str() {
                            "true" => vec![r#"title: "Unsaved changes""#.to_string()],
                            _ => vec![r#"aria_label: "Unsaved changes""#.to_string()],
                        }),
                    // On by default once `onclose` is set, so that is the line.
                    Control::switch("close_button")
                        .default("true")
                        .code(|_, values| match values.str("close_button").as_str() {
                            "true" => vec!["onclose: move |_| open.set(false)".to_string()],
                            _ => vec![],
                        }),
                ],
                render: move |values: DemoValues| rsx! {
                    DialogDemo {
                        titled: values.str("title") == "true",
                        close_button: values.str("close_button") == "true",
                        size: values.str("size"),
                        radius: values.str("radius"),
                    }
                },
                wrap: Wrap(wrap_open),
                // The size scale runs to 900px, so side by side every step
                // from md up is the pane's width.
                wide_preview: true,
            }
        }
    }
}
