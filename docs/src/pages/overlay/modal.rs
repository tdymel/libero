use crate::components::{
    Control, Demo, DemoValues, DocPage, DocSection, Wrap, indent, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, Dialog, Flex, Text},
    hooks::{ModalScope, use_modal},
};

const SIZES: [&str; 6] = ["xs", "sm", "md", "lg", "xl", "xxl"];

const CHOICE: &str = "#[derive(Clone, Copy, PartialEq)]
enum SaveChoice {
    Save,
    Discard,
}";

const ANSWER: &str = "answer.set(match result {
    Some(SaveChoice::Save) => \"saved\",
    Some(SaveChoice::Discard) => \"discarded\",
    None => \"dismissed\",
});";

/// `generate_code` prints one component's props, but this page shows a hook
/// call and its trigger. So the snippet is built by hand.
fn wrap_hook_call(values: &DemoValues, _generated: &str) -> String {
    let onclick = if values.str("await") == "true" {
        format!(
            "onclick: move |_| async move {{\n    \
                 let result = prompt.open_with(\"notes.md\").await;\n{}}},",
            indent(ANSWER)
        )
    } else {
        format!(
            "onclick: move |_| {{\n    \
                 prompt.open_with(\"notes.md\").onresult(move |result| {{\n{}    }});\n\
             }},",
            indent(&indent(ANSWER))
        )
    };
    let button = format!(
        "Button {{\n    variant: \"outlined\",\n{}    \"Close editor\"\n}}",
        indent(&onclick)
    );
    format!(
        "{CHOICE}\n\n\
         let prompt = use_modal(move |s: ModalScope<String, SaveChoice>| {{\n    \
             let document = s.args();\n\n    \
             rsx! {{\n        \
                 Dialog {{\n            \
                     title: \"Unsaved changes\",\n            \
                     size: \"{}\",\n            \
                     Flex {{\n                \
                         direction: \"column\",\n                \
                         gap: \"md\",\n                \
                         Text {{ \"{{document}} has changes you have not saved.\" }}\n                \
                         Flex {{\n                    \
                             direction: \"row\",\n                    \
                             gap: \"sm\",\n                    \
                             wrap: \"wrap\",\n                    \
                             Button {{ variant: \"text\", onclick: move |_| s.close(), \"Keep editing\" }}\n                    \
                             Button {{\n                        \
                                 variant: \"filled\",\n                        \
                                 color: \"error\",\n                        \
                                 onclick: move |_| s.resolve(SaveChoice::Discard),\n                        \
                                 \"Discard\"\n                    \
                             }}\n                    \
                             Button {{\n                        \
                                 variant: \"filled\",\n                        \
                                 color: \"primary\",\n                        \
                                 onclick: move |_| s.resolve(SaveChoice::Save),\n                        \
                                 \"Save\"\n                    \
                             }}\n                \
                         }}\n            \
                     }}\n        \
                 }}\n    \
             }}\n\
         }});\n\
         let mut answer = use_signal(|| \"none yet\");\n\n\
         rsx! {{\n    \
             Flex {{\n        \
                 direction: \"row\",\n        \
                 align: \"center\",\n        \
                 gap: \"md\",\n\
         {}        \
                 Text {{ \"Last answer: {{answer}}\" }}\n    \
             }}\n\
         }}",
        values.str("size"),
        indent(&indent(&button)),
    )
}

#[derive(Clone, Copy, PartialEq)]
enum SaveChoice {
    Save,
    Discard,
}

fn describe(result: Option<SaveChoice>) -> &'static str {
    match result {
        Some(SaveChoice::Save) => "saved",
        Some(SaveChoice::Discard) => "discarded",
        None => "dismissed",
    }
}

/// The hook call lives here rather than in `Demo`'s render closure: a hook
/// there would land in `Demo`'s own hook slots.
#[component]
fn ModalDemo(size: String, awaited: bool) -> Element {
    let prompt = use_modal(move |s: ModalScope<String, SaveChoice>| {
        let document = s.args();

        rsx! {
            Dialog {
                title: "Unsaved changes",
                size: size.clone(),
                Flex {
                    direction: "column",
                    gap: "md",
                    Text { "{document} has changes you have not saved." }
                    Flex {
                        direction: "row",
                        gap: "sm",
                        // Wraps rather than truncating the labels on an xs/sm dialog.
                        wrap: "wrap",
                        Button { variant: "text", onclick: move |_| s.close(), "Keep editing" }
                        Button {
                            variant: "filled",
                            color: "error",
                            onclick: move |_| s.resolve(SaveChoice::Discard),
                            "Discard"
                        }
                        Button {
                            variant: "filled",
                            color: "primary",
                            onclick: move |_| s.resolve(SaveChoice::Save),
                            "Save"
                        }
                    }
                }
            }
        }
    });
    let mut answer = use_signal(|| "none yet");

    rsx! {
        Flex {
            direction: "row",
            align: "center",
            gap: "md",
            if awaited {
                Button {
                    variant: "outlined",
                    onclick: move |_| async move {
                        let result = prompt.open_with("notes.md").await;
                        answer.set(describe(result));
                    },
                    "Close editor"
                }
            } else {
                Button {
                    variant: "outlined",
                    onclick: move |_| {
                        prompt
                            .open_with("notes.md")
                            .onresult(move |result| answer.set(describe(result)));
                    },
                    "Close editor"
                }
            }
            Text { "Last answer: {answer}" }
        }
    }
}

#[component]
pub fn ModalPage() -> Element {
    rsx! {
        DocPage {
            title: "Modal",
            source: "libero/src/components/overlay/use_modal.rs",
            markdown: "/md/modal.md",
            properties: vec![
                props("use_modal", vec![
                    prop("render", "impl FnMut(ModalScope<S, R>) -> Element")
                        .default("required")
                        .doc("Builds the content, usually a `Dialog`, while the modal is open. Returns a `ModalHandle<S, R>`. `R` defaults to `()`, for a modal that answers nothing. Call it under `LiberoProvider`, in a component that outlives every trigger."),
                ]).without_base_props(),
                props("ModalHandle<S, R>", vec![
                    prop("open_with", "fn(impl Into<S>) -> Opening<R>")
                        .doc("Opens with these arguments, replacing whatever was showing."),
                    prop("open", "fn() -> Opening<R>")
                        .doc("Opens with `S::default()`. Needs `S: Default`."),
                    prop("close", "fn()").doc("Dismisses whatever this modal is showing."),
                    prop("is_open", "fn() -> bool").doc("Whether this modal is showing."),
                ]).without_base_props(),
                props("ModalScope<S, R>", vec![
                    prop("args", "fn() -> S").doc("The arguments this opening was given."),
                    prop("close", "fn()").doc("Ends it as a dismissal, the same as Escape."),
                    prop("resolve", "fn(R)").doc("Ends it with an answer for the caller."),
                ]).without_base_props(),
                props("Opening<R>", vec![
                    prop("onresult", "fn(impl FnMut(Option<R>)) -> Self")
                        .doc("Runs when this opening settles, with `None` if it was dismissed. A replaced opening never runs it."),
                    prop("close", "fn()").doc("Closes this opening, if it is still the one showing."),
                    prop(".await", "Option<R>").doc("The same outcome, as a future. Use it when async work waits on the answer, or for dialogs in sequence."),
                ]).without_base_props(),
                props("ModalContext", vec![
                    prop("is_modal", "fn() -> bool")
                        .doc("Whether the content is in a modal. Every modal provides this context to its content."),
                    prop("close", "fn()")
                        .doc("Dismisses the modal around the content. `use_modal_close()` is the shorthand, for a component outside the render closure."),
                ]).without_base_props(),
            ],
            lead: rsx! {
                Text {
                    "A modal is a hook, not a component. "
                    Code { source: "use_modal" }
                    " takes a render closure and returns a handle that opens it. The content "
                    "is built only while it shows. Each opening passes its arguments to the "
                    "closure's "
                    Code { source: "ModalScope" }
                    ", and settles with what "
                    Code { source: "resolve" }
                    " answered, or "
                    Code { source: "None" }
                    " when dismissed."
                }
                Text {
                    "Make the answer the dialog's own enum, not a "
                    Code { source: "bool" }
                    ". The handle is "
                    Code { source: "Copy" }
                    ", so a trigger elsewhere in the tree can take it as a prop or from "
                    "context. A "
                    Code { source: "Dialog" }
                    " inside closes the modal from its own close button."
                }
            },
            Demo {
                component: "ModalDemo",
                children_text: "",
                controls: vec![
                    Control::slider("size", SIZES).default("md"),
                    // An `Opening` is also a future, for an answer that gates
                    // work which is already async.
                    Control::switch("await"),
                ],
                render: move |values: DemoValues| rsx! {
                    ModalDemo {
                        size: values.str("size"),
                        awaited: values.str("await") == "true",
                    }
                },
                wrap: Wrap(wrap_hook_call),
            }

            DocSection {
                title: "Accessibility",
                Text {
                    "Escape and a backdrop click dismiss the modal, settling the "
                    Code { source: "Opening" }
                    " with "
                    Code { source: "None" }
                    ", so a handler written for an answer never runs on a dismissal. Focus "
                    "moves into the modal and back to the trigger once it closes. Name the "
                    Code { source: "Dialog" }
                    " with its "
                    Code { source: "title" }
                    ", or "
                    Code { source: "aria_label" }
                    "."
                }
            }
        }
    }
}
