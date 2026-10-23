use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, indent, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Box, Button, Code, Input, LoadingOverlay, Overlay, Paper, Text},
    sx::sx,
};

/// On its own surface, so it reads at every dim.
const LOADING: &str = r#"Paper { role: "status", sx: sx().padding("8px 16px"), "Loading..." }"#;

const CARD: &str = r#"Paper { sx: sx().padding("16px"),
    Text { "Ada Lovelace" }
    Button { "Edit profile" }
}"#;

fn is_loading(values: &DemoValues) -> bool {
    values.str("mode") == "loading"
}

/// The `LoadingOverlay` card replaces the generated `Overlay` code.
fn wrap_mode(values: &DemoValues, source: &str) -> String {
    match is_loading(values) {
        true => format!(
            "LoadingOverlay {{\n    loading: {},\n{}}}",
            values.str("loading"),
            indent(CARD)
        ),
        false => source.to_string(),
    }
}

#[component]
pub fn OverlayPage() -> Element {
    rsx! {
        DocPage {
            title: "Overlay",
            source: "libero/src/components/overlay/overlay.rs",
            markdown: "/md/overlay.md",
            properties: vec![
                props("Overlay", vec![
                    prop("z_index", "ThemeAwareValue").default("300").doc("Stacking order of the layer."),
                    prop("opacity", "ThemeAwareValue").default("0.6").doc("How dark the dim is."),
                    prop("blur", "ThemeAwareValue").default("none").doc("How much the content behind is blurred. A bare number becomes `px`."),
                    prop("contained", "bool").default("false").doc("Covers its nearest positioned ancestor instead of the viewport. Give that box `position: relative` and a `z-index`, or the layer still stacks against the whole page."),
                    prop("onclick", "EventHandler<MouseEvent>").doc("Called on a click anywhere on the overlay, such as a backdrop click."),
                    prop("children", "Element").doc("Content centred on the overlay, such as a loading spinner."),
                ]),
                props("LoadingOverlay", vec![
                    prop("loading", "bool").default("false").doc("Covers the children with a dim and a `Loader`, makes them `inert` and the box `aria-busy`, and says \"Loading\" once."),
                    prop("children", "Element").default("required").doc("The content that loads. `sx`, `class` and extra attributes land on the box around it."),
                ]),
            ],
            accessibility: a11y()
                .handles([
                    "`LoadingOverlay` marks its box `aria-busy` and its children `inert` while loading, so Tab and a screen reader skip the covered content, and a status region outside the box says \"Loading\".",
                ])
                .must([
                    "For a modal backdrop, use a `Dialog` in `use_modal`, which brings its own overlay. An overlay does not trap focus or hide the page from a screen reader.",
                    "Give an overlay you close on a click a keyboard close as well, such as a `Button` inside it: a backdrop click alone leaves keyboard users no way out.",
                    "Move focus out of a `LoadingOverlay`'s children before it starts loading, or keep it for loads the user did not start there: focus inside `inert` content drops to the page.",
                ])
                .example("A profile card saving, `LoadingOverlay { loading: saving(), .. }` around it: the card dims under a spinner, Tab skips its buttons, and a screen reader hears \"Loading\" once."),
            lead: rsx! {
                Text {
                    "Dims and blurs whatever is behind it. A modal renders one behind its "
                    "content. There is no "
                    Code { source: "open" }
                    " prop, so render it conditionally. Its children are centred, which makes "
                    "it a loading screen as well as a backdrop. "
                    Code { source: "contained" }
                    " keeps it inside a box, and "
                    Code { source: "LoadingOverlay" }
                    " shows one box loading."
                }
            },
            Demo {
                component: "Overlay",
                children_text: "",
                children_code: LOADING.to_string(),
                fixed: vec!["contained: true".to_string()],
                controls: vec![
                    // Not a prop: swaps the dimmed box for the `LoadingOverlay` card.
                    Control::toggle("mode", ["overlay", "loading"])
                        .labels(["Overlay", "LoadingOverlay"])
                        .code(|_, _| vec![]),
                    Control::slider("opacity", ["0.2", "0.4", "0.6", "0.8"])
                        .default("0.6")
                        .hidden_when(is_loading),
                    Control::slider("blur", ["auto", "2px", "4px", "8px"]).hidden_when(is_loading),
                    Control::switch("loading")
                        .default("true")
                        .code(|_, _| vec![])
                        .hidden_when(|values| !is_loading(values)),
                ],
                wrap: Wrap(wrap_mode),
                render: move |values: DemoValues| match is_loading(&values) {
                    true => rsx! {
                        LoadingOverlay { loading: values.str("loading") == "true",
                            Paper { sx: sx().padding("16px"),
                                Text { "Ada Lovelace" }
                                Button { "Edit profile" }
                            }
                        }
                    },
                    false => rsx! {
                        Box {
                            sx: sx()
                                .position("relative")
                                .z_index("0")
                                .height("160px")
                                .width("100%")
                                .background("muted.2"),
                            Text { sx: sx().padding("16px"), "Content behind the overlay" }
                            Overlay {
                                contained: true,
                                opacity: values.str("opacity"),
                                blur: match values.str("blur").as_str() {
                                    "auto" => Input::None,
                                    blur => Input::from(blur),
                                },
                                Paper { role: "status", sx: sx().padding("8px 16px"), "Loading..." }
                            }
                        }
                    },
                },
            }
        }
    }
}
