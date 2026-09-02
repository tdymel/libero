use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, CodeBlock, Input, Loader, Text};

/// The region is the caller's: it carries `aria-busy`, and the loader inside it
/// is the only content, so the loader is the one that speaks.
const REGION: &str = r#"let results = use_resource(search);

rsx! {
    Box {
        "aria-busy": results.read().is_none(),
        match &*results.read() {
            None => rsx! { Loader { label: "Loading results" } },
            Some(rows) => rsx! { ResultList { rows: rows.clone() } },
        }
    }
}"#;

/// Beside its own text, the text is the message and the loader stays silent.
const BESIDE_TEXT: &str = r#"Flex { direction: "row", align: "center", gap: "sm",
    Loader { variant: "dots", size: "sm" }
    Text { "Uploading…" }
}"#;

#[component]
pub fn LoaderPage() -> Element {
    rsx! {
        DocPage {
            title: "Loader",
            source: "libero/src/components/feedback/loader.rs",
            markdown: "/md/loader.md",
            properties: vec![props("Loader", vec![
                prop("variant", "LoaderVariant")
                    .default("oval")
                    .doc("The shape: `oval` (a ring with a gap, rotating), `bars` (three bars rising in turn) or `dots` (three dots pulsing)."),
                prop("size", "Size")
                    .default("md")
                    .doc("The square edge, 18px at `xs` to 72px at `xxl`. Every part of the shape is a fraction of it."),
                prop("color", "ThemeAwareValue")
                    .default("primary")
                    .doc("The ink; a theme color name or a literal CSS color."),
                prop("label", "Option<String>")
                    .doc("Makes the loader a `role=\"status\"` live region that announces this text. Pass it only when the loader is the sole content of the area that is loading - see \"Announcing it\"."),
            ])],
            lead: rsx! {
                Text {
                    "An indeterminate busy indicator: it says something is happening, never "
                    "how much is left. The root is a "
                    Code { source: "<span>" }
                    ", so a loader is legal inside a paragraph or a button. It is silent by "
                    "default - "
                    Code { source: "aria-hidden=\"true\"" }
                    " - because something else on screen usually already says what is going "
                    "on. Under "
                    Code { source: "prefers-reduced-motion: reduce" }
                    " all three shapes stop and stay visible: a still ring with its gap, three "
                    "full-height bars, three dots."
                }
            },
            Demo {
                component: "Loader",
                children_text: "",
                controls: vec![
                    Control::toggle("variant", ["oval", "bars", "dots"])
                        .labels(["Oval", "Bars", "Dots"]),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("md"),
                    // A bare `primary` is what an unset `color` resolves to,
                    // so that swatch prints nothing.
                    Control::color(
                        "color",
                        ["primary", "secondary", "success", "error", "warning", "info", "neutral"],
                    ),
                    Control::switch("label").code(|_, values| {
                        if values.str("label") == "true" {
                            vec![r#"label: "Loading""#.to_string()]
                        } else {
                            vec![]
                        }
                    }),
                ],
                render: move |values: DemoValues| rsx! {
                    Loader {
                        variant: values.str("variant"),
                        size: values.str("size"),
                        color: match values.str("color").as_str() {
                            "primary" => Input::None,
                            color => Input::from(color),
                        },
                        label: (values.str("label") == "true").then(|| "Loading".to_string()),
                    }
                },
            }
            DocSection {
                title: "Announcing it",
                Text {
                    "Who announces the wait depends on where the loader sits. Beside its own "
                    "visible text, the text is the message. Inside a control that already has "
                    "a name - a "
                    Code { source: "Button" }
                    " with "
                    Code { source: "loading" }
                    " - the control's name plus "
                    Code { source: "aria-busy" }
                    " carries it. In both cases leave "
                    Code { source: "label" }
                    " unset: a second announcement of the same state is noise."
                }
                CodeBlock { source: BESIDE_TEXT, language: "rust" }
                Text {
                    "Only when the loader is the sole content of a region does it speak. Give "
                    "it a "
                    Code { source: "label" }
                    " and it renders "
                    Code { source: "role=\"status\"" }
                    " with the label as a visually hidden text node - a live region announces "
                    "its content, so an "
                    Code { source: "aria-label" }
                    " would announce nothing. Mark the region itself "
                    Code { source: "aria-busy" }
                    " while it waits."
                }
                CodeBlock { source: REGION, language: "rust" }
            }
        }
    }
}
