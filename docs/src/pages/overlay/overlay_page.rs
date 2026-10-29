use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, indent, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Box, Button, Code, Flex, Input, LoadingOverlay, Overlay, Paper, Text},
    hooks::use_timeout,
    sx::sx,
};

/// On its own surface, so it reads at every dim. No live role: one mounted with its
/// text is not announced, `LoadingOverlay` is (todo 4970).
const LOADING: &str = r#"Paper { sx: sx().padding("8px 16px"), "Loading..." }"#;

/// Tall enough that the centred Loader clears the name and the buttons (todo 4972).
// snippet: let mut saving = use_signal(|| true);
const CARD: &str = r#"Paper { sx: sx().padding("16px"),
    Flex { direction: "column", justify: "space-between", sx: sx().min_height("128px"),
        Text { "Ada Lovelace" }
        Flex { direction: "row", gap: "sm",
            Button { variant: "outlined", color: "muted", "Edit profile" }
            Button { onclick: move |_| saving.set(true), "Save" }
        }
    }
}"#;

/// How long the demo's Save pretends to save.
const SAVE_MS: u64 = 2000;

/// The card, whose Save starts a short load from inside, so the held focus can be
/// tried (todo 4973). The Loading switch shows the same state.
#[component]
fn SavingCard(values: DemoValues) -> Element {
    let done = {
        let values = values.clone();
        use_timeout(move || values.set("loading", "false"), SAVE_MS)
    };
    let loading = values.str("loading") == "true";
    rsx! {
        LoadingOverlay { loading,
            Paper { sx: sx().padding("16px"),
                Flex {
                    direction: "column",
                    justify: "space-between",
                    sx: sx().min_height("128px"),
                    Text { "Ada Lovelace" }
                    Flex { direction: "row", gap: "sm",
                        Button { variant: "outlined", color: "muted", "Edit profile" }
                        Button {
                            onclick: move |_| {
                                values.set("loading", "true");
                                done.start();
                            },
                            "Save"
                        }
                    }
                }
            }
        }
    }
}

fn is_loading(values: &DemoValues) -> bool {
    values.str("mode") == "loading"
}

/// The `LoadingOverlay` card replaces the generated `Overlay` code.
fn wrap_mode(values: &DemoValues, source: &str) -> String {
    match is_loading(values) {
        true => format!(
            "LoadingOverlay {{\n    loading: saving(),\n{}}}",
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
                    prop("onclick", "EventHandler<MouseEvent>").doc("Called on a click anywhere on the overlay, such as a backdrop click. A click inside its children bubbles here too, as in Mantine: stop it on the content, such as `onclick: move |event| event.stop_propagation()` on a card, to keep the overlay open."),
                    prop("children", "Element").doc("Content centred on the overlay, such as a loading spinner."),
                ]),
                props("LoadingOverlay", vec![
                    prop("loading", "bool").default("false").doc("Covers the children with a dim and a `Loader`, makes them `inert` and the box `aria-busy`, and says \"Loading\" once. Its feature is `loading-overlay`, off by default."),
                    prop("children", "Element").default("required").doc("The content that loads. `sx`, `class` and extra attributes land on the box around it."),
                ]),
            ],
            accessibility: a11y()
                .handles([
                    "`LoadingOverlay` marks its box `aria-busy` and its children `inert` while loading, so Tab and a screen reader skip the covered content, and a status region outside the box says \"Loading\".",
                    "Focus inside a `LoadingOverlay`'s children moves to the box while it loads, so a keyboard keeps its place, and goes back to the same control when loading ends, unless the user moved it meanwhile.",
                ])
                .must([
                    "For a modal backdrop, use a `Dialog` in `use_modal`, which brings its own overlay. An overlay does not trap focus or hide the page from a screen reader.",
                    "Give an overlay you close on a click a keyboard close as well, such as a `Button` inside it: a backdrop click alone leaves keyboard users no way out.",
                    "An `Overlay` announces nothing. For a load a screen reader hears, use a `LoadingOverlay`, or keep a `role: \"status\"` region mounted and set its text when the load starts.",
                    "For a long load started inside a `LoadingOverlay`, offer a way out outside it, such as a Cancel button: until it ends, focus waits on the busy box.",
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
            // snippet: let mut saving = use_signal(|| true);
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
                    Control::slider("blur", ["none", "2px", "4px", "8px"]).hidden_when(is_loading),
                    // The card's Save turns it on for a moment too.
                    Control::switch("loading")
                        .default("true")
                        .code(|_, _| vec![])
                        .hidden_when(|values| !is_loading(values)),
                ],
                wrap: Wrap(wrap_mode),
                render: move |values: DemoValues| match is_loading(&values) {
                    true => rsx! { SavingCard { values: values.clone() } },
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
                                    "none" => Input::None,
                                    blur => Input::from(blur),
                                },
                                Paper { sx: sx().padding("8px 16px"), "Loading..." }
                            }
                        }
                    },
                },
            }
        }
    }
}
