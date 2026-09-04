use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, indent, prop, props};
use dioxus::prelude::*;
use libero::components::{Box, Code, Flex, Input, Loader, Text, VisuallyHidden};

/// Beside its own text, the text is the message and the loader stays silent.
/// As the sole content of a busy region it stays silent too, and a status
/// region outside that region says the text.
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
                    .doc("The shape: `oval` (a ring with a gap, rotating), `bars` (three bars rising in turn) or `dots` (three dots pulsing)."),
                prop("size", "Size")
                    .default("md")
                    .doc("The square edge, 18px at `xs` to 72px at `xxl`. Every part of the shape is a fraction of it."),
                prop("color", "ThemeAwareValue")
                    .default("primary")
                    .doc("The ink; a theme color name or a literal CSS color."),
                prop("label", "Option<String>")
                    .doc("Makes the loader its own `role=\"status\"` live region holding this text. It mounts with the text already in it, which some screen readers do not announce, so to announce a wait use an always-mounted status region outside the busy element instead."),
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
                    " carries it. As the sole content of a region, mark the region "
                    Code { source: "aria-busy" }
                    " and keep the loader silent there too. The text goes into a "
                    Code { source: "role=\"status\"" }
                    " region that is always mounted and sits outside the busy element, and "
                    "is filled only while loading. A region that mounts with its text, or "
                    "changes inside a busy element, may never be read. Switch "
                    Code { source: "Beside text" }
                    " and "
                    Code { source: "Sole content" }
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
                    // Neither is a prop: they place the loader beside its
                    // own text, or alone in a busy region.
                    Control::switch("beside_text").code(|_, _| vec![]),
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
