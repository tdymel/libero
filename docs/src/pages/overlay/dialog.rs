use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, indent, prop, props};
use dioxus::prelude::*;
use libero::components::{Button, Code, Dialog, DialogPart, Text};
use libero::{hooks::use_element, platform::ElementApi, use_theme};

const CONTENT: &str = r#"Text { "notes.md has changes you have not saved." }"#;

/// Outside a modal the close button calls `onclose`, so the snippet owns the
/// open state and a way back.
fn wrap_open(values: &DemoValues, code: &str) -> String {
    if values.str("close_button") != "true" {
        return code.to_string();
    }
    format!(
        "let mut open = use_signal(|| true);\n\
         let mut moved = use_signal(|| false);\n\
         let (dialog, reopen) = (use_element(), use_element());\n\
         // Closing and reopening each unmount the focused button: hand the focus on.\n\
         use_effect(move || {{\n    \
             let target = if open() {{ dialog }} else {{ reopen }};\n    \
             if moved() && target.is_mounted() {{\n        \
                 let _ = target.focus();\n    \
             }}\n\
         }});\n\n\
         rsx! {{\n    \
             if open() {{\n{}    }} else {{\n        \
                 Button {{\n            \
                     variant: \"outlined\",\n            \
                     onmounted: reopen.mount(),\n            \
                     onclick: move |_| open.set(true),\n            \
                     \"Reopen\"\n        \
                 }}\n    \
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
    let mut moved = use_signal(|| false);
    let (dialog, reopen) = (use_element(), use_element());
    // Closing and reopening each unmount the focused button: the focus goes to its replacement.
    use_effect(move || {
        let target = if open() { dialog } else { reopen };
        if moved() && target.is_mounted() {
            let _ = target.focus();
        }
    });

    // Without the button the snippet has no open state, so nor does this.
    if close_button && !open() {
        return rsx! {
            Button {
                variant: "outlined",
                onmounted: reopen.mount(),
                onclick: move |_| open.set(true),
                "Reopen"
            }
        };
    }
    rsx! {
        Dialog {
            title: titled.then(|| "Unsaved changes".to_string()),
            aria_label: (!titled).then(|| "Unsaved changes".to_string()),
            onclose: close_button.then_some(EventHandler::new(move |_| {
                moved.set(true);
                open.set(false);
            })),
            tabindex: close_button.then_some("-1"),
            onmounted: dialog.mount(),
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
                        .doc("Inside a modal: asked before Escape, the backdrop, Back, a bottom drawer's swipe or the close button (`Dismiss::CloseButton`) closes it. Return `false` to keep it open, such as a form with unsaved input. Your own `close()` is not asked."),
                    prop("close_label", "String").default("\"Close\"").doc("The close button's accessible name, such as \"Close cart\". Unset, the localization's `common.close`."),
                    prop("radius", "ThemeAwareValue").default(radius.as_str()).doc("Corner radius from the radius scale, or any CSS, e.g. `radius: \"0\"`."),
                    prop("size", "ThemeAwareValue")
                        .default(theme.dialog.size.as_str())
                        .doc("Caps the width from the dialog scale. `md` is 510px."),
                    prop("full_screen", "DialogFullScreen")
                        .default("never")
                        .doc("Inside a modal, fills the screen edge to edge with square corners instead of centring: `\"phone\"` below the `sm` breakpoint, where a centred dialog leaves slivers of page, `\"always\"` at every width. Outside a modal it does nothing. The Modal page's demo shows it."),
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
                    "Keep the visible title's words in an `aria_label` set beside a `title`: the label replaces the title as the name, so a speech-input user who says the heading finds no match otherwise (WCAG 2.5.3).",
                    "Give an `alertdialog` an `aria_describedby` pointing at its message, so a screen reader reads it on open.",
                    "Open it in a modal: the focus trap, Escape and backdrop dismissal come from the modal. A `Dialog` on its own has none of them.",
                    "Outside a modal, give a close button `onclose`, or it closes nothing.",
                    "Move the focus somewhere sensible in `onclose`: closing removes the focused close button.",
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
                            "true" => vec![
                                "onclose: move |_| { moved.set(true); open.set(false) }".to_string(),
                                r#"tabindex: "-1""#.to_string(),
                                "onmounted: dialog.mount()".to_string(),
                            ],
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
