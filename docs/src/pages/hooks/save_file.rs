use crate::components::{Demo, DemoValues, DocPage, Wrap, a11y};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, Flex, Text},
    platform::{SaveOutcome, save_file},
};

/// The demo in one component, as `SaveDemo` renders it.
// snippet: mirrors SaveDemo
fn code(_: &DemoValues, _: &str) -> String {
    r#"let mut outcome = use_signal(|| None::<SaveOutcome>);

rsx! {
    Flex { gap: "sm", align: "center", wrap: "wrap",
        Button {
            onclick: move |_| async move {
                let csv = "name,role\nAda,Engineer\n";
                let saved = save_file("team.csv", "text/csv", csv.into()).await;
                outcome.set(Some(saved));
            },
            "Save team.csv"
        }
        if let Some(outcome) = outcome() {
            Text { size: "sm", "{outcome:?}" }
        }
    }
}"#
    .to_string()
}

#[component]
fn SaveDemo() -> Element {
    let mut outcome = use_signal(|| None::<SaveOutcome>);
    rsx! {
        Flex { gap: "sm", align: "center", wrap: "wrap",
            Button {
                onclick: move |_| async move {
                    let csv = "name,role\nAda,Engineer\n";
                    let saved = save_file("team.csv", "text/csv", csv.into()).await;
                    outcome.set(Some(saved));
                },
                "Save team.csv"
            }
            if let Some(outcome) = outcome() {
                Text { size: "sm", "{outcome:?}" }
            }
        }
    }
}

#[component]
pub fn SaveFilePage() -> Element {
    rsx! {
        DocPage {
            title: "Save file",
            source: "libero/src/platform/save_file.rs",
            markdown: "/md/save_file.md",
            accessibility: a11y()
                .handles([
                    "The save dialog and the share sheet are the system's own, with its keyboard and screen reader support.",
                ])
                .must([
                    "Say what happened: the browser's download shows no dialog, so name the file in your button or confirm the save in text, as `TableExportButton` announces its rows.",
                ])
                .example("An \"Export report.csv\" button that calls the save: the file name is in the button's text, and a status line says \"Saved report.csv\" once it reports `Saved`, since a browser download shows no dialog.")
                .limits([
                    "Android shares text only, under about 500 KB; other bytes come back `Failed`.",
                    "The web cannot tell a finished download from a blocked one: it reports `Saved` once the browser has the file.",
                ]),
            lead: rsx! {
                Text {
                    Code { source: "save_file(name, mime, bytes).await" }
                    " saves bytes your app made, an export or a report, as a file the user keeps. "
                    "The web downloads it, the desktop and Blitz open a save dialog, and Android "
                    "opens the share sheet for text. It returns a "
                    Code { source: "SaveOutcome" }
                    ": "
                    Code { source: "Saved" }
                    ", "
                    Code { source: "Shared" }
                    ", "
                    Code { source: "Cancelled" }
                    " or "
                    Code { source: "Failed(reason)" }
                    "."
                }
            },

            Demo {
                component: "save_file",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { SaveDemo {} },
                wrap: Wrap(code),
            }
        }
    }
}
