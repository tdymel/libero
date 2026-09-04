use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, indent, prop, props};
use dioxus::prelude::*;
use libero::components::{Box, Code, Flex, Input, Loader, Text};

/// Beside its own text, the text is the message and the loader stays silent.
/// With a `label` it is the sole content of a region, which carries
/// `aria-busy`.
fn wrap_context(values: &DemoValues, code: &str) -> String {
    if beside_text(values) {
        format!(
            "Flex {{ direction: \"row\", align: \"center\", gap: \"sm\",\n{}    Text {{ \"Uploading…\" }}\n}}",
            indent(code)
        )
    } else if labelled(values) {
        format!("Box {{ \"aria-busy\": \"true\",\n{}}}", indent(code))
    } else {
        code.to_string()
    }
}

fn beside_text(values: &DemoValues) -> bool {
    values.str("beside_text") == "true"
}

/// A loader beside its own text never takes a label: that would announce the
/// same state twice.
fn labelled(values: &DemoValues) -> bool {
    values.str("label") == "true" && !beside_text(values)
}

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
                    .doc("Makes the loader a `role=\"status\"` live region that announces this text. Pass it only when the loader is the sole content of the area that is loading; mark that area `aria-busy` while it waits."),
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
                Text {
                    "Who announces the wait depends on where the loader sits. Beside its own "
                    "visible text, the text is the message; inside a named control - a "
                    Code { source: "Button" }
                    " with "
                    Code { source: "loading" }
                    " - the control's name plus "
                    Code { source: "aria-busy" }
                    " carries it. Leave "
                    Code { source: "label" }
                    " unset in both. Only as the sole content of a region does the loader "
                    "speak: with a "
                    Code { source: "label" }
                    " it renders "
                    Code { source: "role=\"status\"" }
                    " and the label as a visually hidden text node, and the region is marked "
                    Code { source: "aria-busy" }
                    ". Switch "
                    Code { source: "Beside text" }
                    " and "
                    Code { source: "Label" }
                    " to see both."
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
                    Control::color("color"),
                    // Not a prop: it puts the loader beside its own text,
                    // where `label` does not belong.
                    Control::switch("beside_text").code(|_, _| vec![]),
                    Control::switch("label")
                        .code(|_, values| {
                            if labelled(values) {
                                vec![r#"label: "Loading""#.to_string()]
                            } else {
                                vec![]
                            }
                        })
                        .hidden_when(beside_text),
                ],
                wrap: Wrap(wrap_context),
                render: move |values: DemoValues| {
                    let loader = rsx! {
                        Loader {
                            variant: values.str("variant"),
                            size: values.str("size"),
                            color: match values.str("color").as_str() {
                                "primary" => Input::None,
                                color => Input::from(color),
                            },
                            label: labelled(&values).then(|| "Loading".to_string()),
                        }
                    };
                    match (beside_text(&values), labelled(&values)) {
                        (true, _) => rsx! {
                            Flex { direction: "row", align: "center", gap: "sm",
                                {loader}
                                Text { "Uploading…" }
                            }
                        },
                        (false, true) => rsx! {
                            Box { "aria-busy": "true", {loader} }
                        },
                        (false, false) => loader,
                    }
                },
            }
        }
    }
}
