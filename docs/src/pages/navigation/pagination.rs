use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Code, CodeBlock, Pagination, Text},
    use_theme,
};

const SIZES: [&str; 6] = ["xs", "sm", "md", "lg", "xl", "xxl"];

const RANGE_TABLE: &str = r#"total  siblings  boundaries  page   rendered
    7         1           1     4   1 2 3 4 5 6 7
   10         1           1     1   1 2 3 4 5 … 10
   10         1           1     5   1 … 4 5 6 … 10
   10         1           1     7   1 … 6 7 8 9 10
   20         2           1    10   1 … 8 9 10 11 12 … 20
   20         1           2    10   1 2 … 9 10 11 … 19 20
   11         0           1     6   1 … 6 … 11"#;

const LABEL_EXAMPLE: &str = r#"Pagination {
    total: 42,
    page: page(),
    onchange: move |next| page.set(next),
    aria_label: "Suchergebnisse",
    label: |label: PaginationLabel| match label {
        // The number does not have to go last.
        PaginationLabel::Page { number, current: true } => format!("Seite {number}, aktuell"),
        PaginationLabel::Page { number, .. } => format!("Seite {number}"),
        PaginationLabel::First => "Erste Seite".to_string(),
        PaginationLabel::Previous => "Vorherige Seite".to_string(),
        PaginationLabel::Next => "Nächste Seite".to_string(),
        PaginationLabel::Last => "Letzte Seite".to_string(),
    },
}"#;

const RANGE_EXAMPLE: &str = r#"use libero::components::{pagination_range, PaginationItem};

// The same arithmetic the component draws with, for a strip you draw
// yourself.
for item in pagination_range(42, 7, 1, 1) {
    match item {
        PaginationItem::Page(n) => { /* a control for page n */ }
        PaginationItem::Ellipsis => { /* a gap */ }
    }
}"#;

/// The demo is genuinely controlled: `page` lives here, so clicking through the
/// preview moves it the way it would in an application.
#[component]
pub fn PaginationPage() -> Element {
    let theme = use_theme();
    let mut page = use_signal(|| 1u32);

    rsx! {
        DocPage {
            title: "Pagination",
            source: "libero/src/components/navigation/pagination/pagination.rs",
            markdown: "/md/pagination.md",
            properties: vec![props("Pagination", vec![
                prop("total", "u32").default("required")
                    .doc("Page count. `0` renders nothing at all - an empty result set has no pages, and a lone disabled control implies otherwise."),
                prop("page", "u32").default("required")
                    .doc("The current page, 1-based and clamped into range. Strictly controlled, like `Tabs::value`."),
                prop("onchange", "EventHandler<u32>")
                    .doc("Asks for a new page. Without it the page can never change, and the component warns."),
                prop("aria_label", "String").default("required")
                    .doc("Names the `<nav>` landmark. Required, because two paginations on one page have to be distinguishable."),
                prop("siblings", "u8").default("1")
                    .doc("Pages either side of the current one."),
                prop("boundaries", "u8").default("1")
                    .doc("Pages pinned at each end. `0` is clamped to 1 - leading dots with nothing outside them hide a page to show a gap."),
                prop("size", "Size").default("md").doc("Control box and font size."),
                prop("radius", "Size").default("sm").doc("Corner radius, independent of `size`."),
                prop("color", "ThemeAwareValue").default("primary")
                    .doc("Fill of the current page. Its readable text comes from the matching `-contrast` twin, so there is no auto-contrast knob."),
                prop("disabled", "bool").default("false").doc("Disables every control at once."),
                prop("with_controls", "bool").default("true").doc("Previous and next."),
                prop("with_edges", "bool").default("false").doc("First and last."),
                prop("label", "Callback<PaginationLabel, String>")
                    .doc("Overrides every accessible name. Runs during render, so it can read a locale - and it can put the number somewhere other than last."),
            ])],
            lead: rsx! {
                Text {
                    "A row of page controls: a named "
                    Code { source: "nav" }
                    " landmark around a list of real buttons, with an ellipsis range that never "
                    "reflows as you click through it. Strictly controlled - "
                    Code { source: "page" }
                    " is yours and "
                    Code { source: "onchange" }
                    " asks for a new one."
                }
                Text {
                    "It renders buttons, not links. A pagination is state; whether a navigation "
                    "happens is not part of its contract, so a content listing that wants "
                    "shareable URLs wires its own links around this component."
                }
            },
            Demo {
                component: "Pagination",
                children_text: "",
                fixed: vec![
                    "page: page()".to_string(),
                    "onchange: move |next| page.set(next)".to_string(),
                ],
                controls: vec![
                    Control::select("total", ["1", "7", "10", "42"]).default("10"),
                    Control::slider("siblings", ["0", "1", "2", "3"]).default("1"),
                    Control::slider("boundaries", ["1", "2", "3"]).default("1"),
                    Control::slider("size", SIZES).default(theme.pagination.size.as_str()),
                    Control::slider("radius", SIZES).default(theme.pagination.radius.as_str()),
                    Control::color("color", ["primary", "secondary", "success", "error"]),
                    Control::switch("with_controls").default("true"),
                    Control::switch("with_edges"),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| {
                    let total = values.str("total").parse::<u32>().unwrap_or(10);
                    rsx! {
                        Pagination {
                            total,
                            page: page(),
                            onchange: move |next| page.set(next),
                            aria_label: "Demo pages",
                            siblings: values.str("siblings").parse::<u8>().unwrap_or(1),
                            boundaries: values.str("boundaries").parse::<u8>().unwrap_or(1),
                            size: values.str("size"),
                            radius: values.str("radius"),
                            color: values.str("color"),
                            with_controls: values.str("with_controls") == "true",
                            with_edges: values.str("with_edges") == "true",
                            disabled: (values.str("disabled") == "true").then_some(true),
                        }
                    }
                },
            }

            DocSection {
                title: "The range",
                Text {
                    "An ellipsis never stands for exactly one page. Hiding "
                    Code { source: "9" }
                    " behind a gap costs the same width as printing it, so the gap is only "
                    "drawn where it saves something. That also fixes the rendered width at "
                    Code { source: "2·siblings + 2·boundaries + 3" }
                    ", which is why the strip does not reflow while you click through it - the "
                    "comparison is the point here, so it is a table rather than a control."
                }
                CodeBlock { source: RANGE_TABLE, language: "text" }
                Text {
                    "The arithmetic is public. "
                    Code { source: "pagination_range" }
                    " is a pure function of four integers, so a caller drawing a custom strip "
                    "can reuse it rather than re-deriving the edge cases."
                }
                CodeBlock { source: RANGE_EXAMPLE, language: "rust" }
            }

            DocSection {
                title: "Accessible names",
                Text {
                    "The current page is named "
                    Code { source: "Page 4" }
                    " and every other "
                    Code { source: "Go to page 4" }
                    ". "
                    Code { source: "aria-current" }
                    " already says \"current\", so repeating \"go to\" on the page you are on "
                    "would be a lie. The ellipsis is "
                    Code { source: "aria-hidden" }
                    " and not focusable: it is a gap, not a control."
                }
                Text {
                    "Every string lives in "
                    Code { source: "theme.pagination_labels" }
                    ", which is English by default and swapped whole for a locale. That shape "
                    "assumes the number goes last, which is wrong in plenty of languages - "
                    Code { source: "label" }
                    " is the escape hatch, and it sees the five named controls and never the "
                    "ellipsis, so an exhaustive match has no dead branch."
                }
                CodeBlock { source: LABEL_EXAMPLE, language: "rust" }
            }

            DocSection {
                title: "Focus when a control disables under you",
                Text {
                    "Clicking previous until you reach page 1 disables the button your finger "
                    "is on. Nothing is removed, so focus does not move - it is left on a "
                    "disabled control, which browsers drop to the document. Focus goes to the "
                    "current page's button instead, which is why that button stays enabled "
                    "rather than being disabled as the page you are already on."
                }
                Text {
                    "The repair needs the platform to be able to find the focused element and "
                    "search a subtree. Where it cannot - a webview, where neither is available "
                    "- nothing moves and nothing else changes."
                }
                Text {
                    "Shrinking "
                    Code { source: "total" }
                    " is yours to handle, not the component's. A filter that takes a listing "
                    "from 40 pages to 3 destroys most of the page buttons, and if focus was on "
                    "one of them it falls to the document - nothing was clicked, so there is "
                    "nothing owing and no way to know a control rendered last time has gone. "
                    "If you narrow a result set while a pagination is on screen, move focus "
                    "yourself: to the filter the user just used, or to the heading above the "
                    "results."
                }
            }
        }
    }
}
