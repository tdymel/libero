use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Code, Pagination, PaginationPart, Text},
    use_theme,
};

/// `siblings` and `boundaries` are `u8`s, so they print unquoted.
fn unquoted(control: &Control, values: &DemoValues) -> Vec<String> {
    let value = values.str(control.name);
    match value == control.default {
        true => vec![],
        false => vec![format!("{}: {value}", control.name)],
    }
}

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
                    .doc("Page count. `0` renders nothing."),
                prop("page", "u32").default("required")
                    .doc("The current page, 1-based and clamped into range."),
                prop("onchange", "EventHandler<u32>")
                    .doc("Asks for a new page. Without it the page never changes, and the component warns."),
                prop("aria_label", "String").default("required")
                    .doc("Names the `<nav>` landmark, so two paginations on one page can be told apart."),
                prop("siblings", "u8").default(theme.pagination.siblings.to_string())
                    .doc("Pages on each side of the current one. Past `2·siblings + 2·boundaries + 3` pages the row is always that many items wide; fewer pages all show. An ellipsis never stands for a single page."),
                prop("boundaries", "u8").default(theme.pagination.boundaries.to_string())
                    .doc("Pages pinned at each end. `0` counts as 1."),
                prop("size", "Size").default(theme.pagination.size.as_str()).doc("Control box and font size."),
                prop("radius", "ThemeAwareValue").default(theme.pagination.radius.as_str()).doc("Corner radius, independent of `size`, or any CSS, e.g. `radius: \"0\"`."),
                prop("color", "ThemeAwareValue").default(theme.pagination.color.as_str())
                    .doc("Fill of the current page. A theme color gives its text the matching `-contrast` shade, a literal color black or white."),
                prop("disabled", "bool").default("false").doc("Disables every control."),
                prop("with_controls", "bool").default("true").doc("Shows the previous and next controls."),
                prop("with_edges", "bool").default("false").doc("Shows the first and last controls."),
                prop("label", "Callback<PaginationLabel, String>")
                    .doc("Overrides every accessible name. Runs during render, so it can read a locale."),
                prop("parts", "Parts<PaginationPart>")
                    .doc("Styles for the inner parts in the Style API tab, under `sx`."),
            ])
            .parts("PaginationPart", vec![
                (PaginationPart::List, "The list of controls."),
                (PaginationPart::Page, "A page button. The current one has `aria-current=\"page\"`."),
                (PaginationPart::Arrow, "The first, previous, next and last buttons."),
                (PaginationPart::Ellipsis, "The `…` between page ranges."),
            ])],
            accessibility: a11y()
                .handles([
                    "Every control is a button and a tab stop, except an arrow at its end, which is a natively disabled button. Under `disabled` every control is `aria-disabled` and stays a tab stop, so focus is not lost.",
                    "An arrow that disables itself on click, such as next on the last page, hands focus to the current page.",
                    "The page names come from the localization's `PaginationLabels`, and `label` overrides them.",
                ])
                .must([
                    "Keep `theme.pagination.gap` above zero: at `xs` the controls are 22px and meet the 24px target size only through the gap.",
                ])
                .example("A results pager: each page number and enabled arrow is a button and a tab stop, and the next arrow reads \"Go to next page\" from `PaginationLabels` instead of a bare arrow."),
            lead: rsx! {
                Text {
                    "A row of page buttons in a named "
                    Code { source: "nav" }
                    " landmark. The ellipsis keeps the row the same width as you click "
                    "through it. You own "
                    Code { source: "page" }
                    ", and "
                    Code { source: "onchange" }
                    " asks for a new one."
                }
                Text {
                    "It renders buttons, not links, so open in a new tab and crawlable "
                    "page URLs are not available. A listing that wants shareable URLs "
                    "navigates in "
                    Code { source: "onchange" }
                    "."
                }
            },
            // snippet: let mut page = use_signal(|| 1u32);
            Demo {
                component: "Pagination",
                children_text: "",
                fixed: vec![
                    "page: page()".to_string(),
                    "onchange: move |next| page.set(next)".to_string(),
                    // Required, so the snippet needs it even though no control
                    // sets it.
                    "aria_label: \"Demo pages\"".to_string(),
                ],
                controls: vec![
                    // Required as well, so it prints even at its default.
                    Control::select("total", ["1", "7", "10", "42"])
                        .default("10")
                        .code(|_, values| vec![format!("total: {}", values.str("total"))]),
                    Control::slider("siblings", ["0", "1", "2", "3"])
                        .default("1")
                        .code(unquoted),
                    Control::slider("boundaries", ["1", "2", "3"]).default("1").code(unquoted),
                    Control::sizes("size").default(theme.pagination.size.as_str()),
                    Control::sizes("radius").default(theme.pagination.radius.as_str()),
                    Control::color("color").default(theme.pagination.color.as_str()),
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
        }
    }
}
