use crate::components::{Child, Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::{Anchor, AnchorPart, Code, Text};
use libero::use_theme;

const HREF: &str = "https://dioxuslabs.com";
const EXTERNAL_LABEL: &str = "Read the Dioxus docs";
const INTERNAL_LABEL: &str = "Back to Getting Started";
const ACTION_LABEL: &str = "Show details";

/// A typed route reads differently from a URL, so the label follows `to`.
fn link_label(values: &DemoValues) -> String {
    if values.str("action") == "true" {
        return ACTION_LABEL.to_string();
    }
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
                    .doc("A path, a URL or a typed route (`Route::Foo {}`). Unset, the anchor renders a `<button type=\"button\">` with the same look, for an action such as \"Show details\"; `target`, `new_tab_hint` and `new_tab_icon` then do nothing. With a router mounted and `target` unset, an internal target navigates without a page reload; with `\"_blank\"` it opens a new tab with a full page load. A `javascript:`, `data:` or `vbscript:` URL is blocked, in every link `to` and in `MenuItem::href`: the element renders without an `href`, so a click does nothing, and a warning is logged."),
                prop("onclick", "EventHandler<MouseEvent>")
                    .doc("Runs on a click, or on Enter and Space without `to`. With `to` it runs too, after the router's own navigation."),
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
                    "Without `to` it is a real `<button>`: a screen reader announces a button, and Enter and Space run `onclick`.",
                    "A `target: \"_blank\"` link reads a hidden \"(opens in a new tab)\". `new_tab_icon: true` also draws a small external icon, for sighted users. `new_tab_hint: false` drops both, for a link whose text already says it.",
                    "In a filled, tonal or gradient `Alert`, a colored `Header` and a `Mark`, a link takes the text color, so `underline: \"hover\"` draws its underline at rest there too.",
                ])
                .must([
                    "Leave `to` unset for an action that does not go anywhere, so it is announced as a button, not a link.",
                    "Make the link text say where the link goes: it is the accessible name.",
                    "Set `underline: \"always\"` on a link inside a paragraph: the default `hover` draws it only on hover and `never` not at all, so at rest the link stands out by color alone.",
                    "Give a nav row of Anchors its own padding for a 44 px finger target. Running text is exempt from WCAG 2.5.8, and an Anchor cannot tell which it is in.",
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
                    ". Without "
                    Code { source: "to" }
                    " it renders a button with the same look, for an action."
                }
            },
            // snippet: item #[derive(Clone, PartialEq, Routable)] enum Route { #[route("/")] GettingStarted {} }
            // snippet: item #[component] fn GettingStarted() -> Element { rsx! {} }
            Demo {
                component: "Anchor",
                children_text: "",
                controls: vec![
                    Control::switch("action").code(|_, values| {
                        match values.str("action").as_str() {
                            "true" => vec!["onclick: move |_| {}".to_string()],
                            _ => vec![],
                        }
                    }),
                    // `to` always prints for a link. This switch varies it.
                    Control::switch("internal_route")
                        .hidden_when(|values| values.str("action") == "true")
                        .code(|_, values| {
                        match (values.str("action").as_str(), values.str("internal_route").as_str()) {
                            ("true", _) => vec![],
                            (_, "true") => vec!["to: Route::GettingStarted {}".to_string()],
                            _ => vec![
                                format!("to: {HREF:?}"),
                                "target: \"_blank\"".to_string(),
                            ],
                        }
                    }),
                    Control::switch("new_tab_hint")
                        .default("true")
                        .hidden_when(|values| values.str("action") == "true")
                        .code(|_, values| {
                        match (values.str("internal_route").as_str(), values.str("new_tab_hint").as_str()) {
                            _ if values.str("action") == "true" => vec![],
                            ("true", _) | (_, "true") => vec![],
                            _ => vec!["new_tab_hint: false".to_string()],
                        }
                    }),
                    // On here so the icon shows; the component's own default is off.
                    Control::switch("new_tab_icon")
                        .default("true")
                        .hidden_when(|values| {
                            values.str("action") == "true"
                                || values.str("internal_route") == "true"
                                || values.str("new_tab_hint") != "true"
                        })
                        .code(|_, values| {
                            match (
                                values.str("internal_route").as_str(),
                                values.str("new_tab_hint").as_str(),
                                values.str("new_tab_icon").as_str(),
                            ) {
                                _ if values.str("action") == "true" => vec![],
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
                    if values.str("action") == "true" {
                        return rsx! {
                            Anchor { onclick: move |_| {}, underline, size, {ACTION_LABEL} }
                        };
                    }
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
