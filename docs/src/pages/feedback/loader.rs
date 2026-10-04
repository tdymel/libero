use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, indent, prop, props};
use dioxus::prelude::*;
use libero::components::{Box, Code, Flex, Input, Loader, Text, VisuallyHidden};

/// The loader stays silent either way: beside text the text speaks; alone in a busy region,
/// a status region outside it does.
fn wrap_context(values: &DemoValues, code: &str) -> String {
    if beside_text(values) {
        format!(
            "Flex {{ direction: \"row\", align: \"center\", gap: \"sm\",\n{}    Text {{ \"Uploading…\" }}\n}}",
            indent(code)
        )
    } else if in_region(values) {
        format!(
            "Box {{ \"aria-busy\": \"true\",\n{}}}\n\
             // Always mounted, outside the busy element. Empty once loading ends.\n\
             VisuallyHidden {{ role: \"status\", \"{LOADING}\" }}",
            indent(code)
        )
    } else {
        code.to_string()
    }
}

const LOADING: &str = "Loading results";

fn beside_text(values: &DemoValues) -> bool {
    values.str("beside_text") == "true"
}

/// A loader beside its own text needs no status region: the text is already
/// on screen.
fn in_region(values: &DemoValues) -> bool {
    values.str("sole_content") == "true" && !beside_text(values)
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
                    .doc("The shape: `oval`, `bars` or `dots`."),
                prop("size", "Size")
                    .default("md")
                    .doc("The edge of the square, 18px at `xs` to 72px at `xxl`."),
                prop("color", "ThemeAwareValue")
                    .default("primary")
                    .doc("A theme color name or any CSS color."),
            ])],
            accessibility: a11y()
                .handles([
                    "The loader is hidden from screen readers.",
                    "With reduced motion it stops moving but stays visible.",
                ])
                .must([
                    "Say the wait with something else. Beside its own text, the text says it. Inside a control, such as a `Button` with `loading`, the control does.",
                    "As the only content of a region, mark the region `aria-busy` and put the text in a `role=\"status\"` region outside it.",
                    "Mount that status region up front and fill it only while loading, or a screen reader may skip it. The demo's `Beside text` and `Sole content` switches show both setups.",
                ])
                .example("A `Loader` beside the text \"Loading results\": a screen reader reads the text and skips the loader. With reduced motion the loader stops moving."),
            lead: rsx! {
                Text {
                    "An indeterminate busy indicator. It says something is happening, not how "
                    "much is left. The root is a "
                    Code { source: "<span>" }
                    ", so it fits inside a paragraph or a button. With reduced motion it stops "
                    "moving but stays visible."
                }
            },
            Demo {
                component: "Loader",
                children_text: "",
                controls: vec![
                    Control::toggle("variant", ["oval", "bars", "dots"])
                        .labels(["Oval", "Bars", "Dots"]),
                    Control::sizes("size").default("md"),
                    // A bare `primary` is what an unset `color` resolves to,
                    // so that swatch prints nothing.
                    Control::color("color"),
                    // Neither is a prop: they place the loader beside its
                    // own text, or alone in a busy region.
                    // On by default, so the first code block is the safe setup.
                    Control::switch("beside_text").default("true").code(|_, _| vec![]),
                    Control::switch("sole_content")
                        .code(|_, _| vec![])
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
                        }
                    };
                    let in_region = in_region(&values);
                    let shown = match (beside_text(&values), in_region) {
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
                    };
                    // Mounted in every arm, so switching `Sole content` on
                    // changes the text of a region that already exists.
                    rsx! {
                        {shown}
                        VisuallyHidden { role: "status", if in_region { "{LOADING}" } }
                    }
                },
            }
        }
    }
}
