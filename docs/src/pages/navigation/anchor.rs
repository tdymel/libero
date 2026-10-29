use crate::components::{Child, Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::{Anchor, AnchorPart, Code, Text};
use libero::use_theme;

const HREF: &str = "https://dioxuslabs.com";
const EXTERNAL_LABEL: &str = "Read the Dioxus docs";
const INTERNAL_LABEL: &str = "Back to Getting Started";

/// A typed route reads differently from a URL, so the label follows `to`.
fn link_label(values: &DemoValues) -> String {
    match values.str("internal_route").as_str() {
        "true" => INTERNAL_LABEL.to_string(),
        _ => EXTERNAL_LABEL.to_string(),
    }
}

#[component]
pub fn AnchorPage() -> Element {
    let theme = use_theme();
    rsx! {
        DocPage {
            title: "Anchor",
            source: "libero/src/components/navigation/anchor.rs",
            markdown: "/md/anchor.md",
            properties: vec![props("Anchor", vec![
                prop("size", "Size").default(theme.anchor.size.as_str()).doc("Text size."),
                prop("to", "NavigationTarget")
                    .default("required")
                    .doc("A path, a URL or a typed route (`Route::Foo {}`). With a router mounted and `target` unset, an internal target navigates without a page reload; with `\"_blank\"` it opens a new tab with a full page load. A `javascript:`, `data:` or `vbscript:` URL is blocked, in every link `to` and in `MenuItem::href`: the element renders without an `href`, so a click does nothing, and a warning is logged."),
                prop("target", "String").doc("The link's `target` attribute. `\"_blank\"` adds a hidden \"(opens in a new tab)\"."),
                prop("new_tab_hint", "bool")
                    .default("true")
                    .doc("`false` drops the hidden text a `\"_blank\"` target adds, and the icon."),
                prop("new_tab_icon", "bool")
                    .default(if theme.anchor.new_tab_icon { "true" } else { "false" })
                    .doc("With the hint, draws a small external icon after the text. Unset, the theme's `anchor.new_tab_icon`, which is off. `Button`, `ActionIcon`, `Chip`, `NavLink`, `Repository` and `MenuItem` take it too."),
                prop("underline", "AnchorUnderline")
                    .default("hover")
                    .doc("When the underline draws: `always`, `hover` (only under the pointer), `not-hover` (at rest, gone under the pointer) or `never`. On a touch screen nothing hovers, so `hover` draws no underline and `not-hover` always does."),
                prop("children", "Element").default("required").doc("The link's content."),
                prop("parts", "Parts<AnchorPart>")
                    .doc("Styles for the inner parts in the Style API tab, under `sx`."),
            ])
            .parts("AnchorPart", vec![
                (AnchorPart::NewTab, "The new-tab icon after the text, with `target: \"_blank\"` and `new_tab_icon` only."),
            ])],
            accessibility: a11y()
                .handles([
                    "A `target: \"_blank\"` link reads a hidden \"(opens in a new tab)\". `new_tab_icon: true` also draws a small external icon, for sighted users. `new_tab_hint: false` drops both, for a link whose text already says it.",
                    "In a filled, tonal or gradient `Alert`, a colored `Header` and a `Mark`, a link takes the text color, so `underline: \"hover\"` draws its underline at rest there too.",
                ])
                .must([
                    "Make the link text say where the link goes: it is the accessible name.",
                    "Set `underline: \"always\"` on a link inside a paragraph: the default `hover` draws it only on hover and `never` not at all, so at rest the link stands out by color alone.",
                ])
                .example("A link in a paragraph, `Anchor { to: \"https://www.rust-lang.org\", target: \"_blank\", underline: \"always\", \"The Rust website\" }`: it keeps its underline, and a screen reader reads \"The Rust website (opens in a new tab)\"."),
            lead: rsx! {
                Text {
                    "A link styled and sized like "
                    Code { source: "Text" }
                    ". A typed route in "
                    Code { source: "to" }
                    " navigates through the app's router without a page reload. Without a "
                    "router it renders a plain "
                    Code { source: "href" }
                    "."
                }
            },
            // snippet: item #[derive(Clone, PartialEq, Routable)] enum Route { #[route("/")] GettingStarted {} }
            // snippet: item #[component] fn GettingStarted() -> Element { rsx! {} }
            Demo {
                component: "Anchor",
                children_text: "",
                controls: vec![
                    // `to` is required, so it always prints. This switch varies it.
                    Control::switch("internal_route").code(|_, values| {
                        match values.str("internal_route").as_str() {
                            "true" => vec!["to: Route::GettingStarted {}".to_string()],
                            _ => vec![
                                format!("to: {HREF:?}"),
                                "target: \"_blank\"".to_string(),
                            ],
                        }
                    }),
                    Control::switch("new_tab_hint").default("true").code(|_, values| {
                        match (values.str("internal_route").as_str(), values.str("new_tab_hint").as_str()) {
                            ("true", _) | (_, "true") => vec![],
                            _ => vec!["new_tab_hint: false".to_string()],
                        }
                    }),
                    // On here so the icon shows; the component's own default is off.
                    Control::switch("new_tab_icon")
                        .default("true")
                        .hidden_when(|values| {
                            values.str("internal_route") == "true"
                                || values.str("new_tab_hint") != "true"
                        })
                        .code(|_, values| {
                            match (
                                values.str("internal_route").as_str(),
                                values.str("new_tab_hint").as_str(),
                                values.str("new_tab_icon").as_str(),
                            ) {
                                ("true", _, _) | (_, "false", _) | (_, _, "false") => vec![],
                                _ => vec!["new_tab_icon: true".to_string()],
                            }
                        }),
                    // `not-hover` first, so the preview shows it; the component's own default is `hover`.
                    Control::toggle("underline", ["not-hover", "hover", "always", "never"])
                        .labels(["At rest", "Hover", "Always", "Never"])
                        .code(|_, values| match values.str("underline").as_str() {
                            "hover" => vec![],
                            value => vec![format!("underline: {value:?}")],
                        }),
                    Control::sizes("size")
                        .default(theme.anchor.size.as_str()),
                ],
                render: move |values: DemoValues| {
                    let underline = values.str("underline");
                    let size = values.str("size");
                    let new_tab_hint = values.str("new_tab_hint") == "true";
                    let new_tab_icon = values.str("new_tab_icon") == "true";
                    match values.str("internal_route").as_str() {
                        "true" => rsx! {
                            Anchor {
                                to: crate::Route::GettingStarted {},
                                underline,
                                size,
                                {INTERNAL_LABEL}
                            }
                        },
                        _ => rsx! {
                            Anchor {
                                to: HREF,
                                target: "_blank",
                                new_tab_hint,
                                new_tab_icon,
                                underline,
                                size,
                                {EXTERNAL_LABEL}
                            }
                        },
                    }
                },
                child: Child(link_label),
            }
        }
    }
}
