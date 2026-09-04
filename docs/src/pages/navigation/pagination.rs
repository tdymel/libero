use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Code, Pagination, Text},
    use_theme,
};

const SIZES: [&str; 6] = ["xs", "sm", "md", "lg", "xl", "xxl"];

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
                    Control::slider("size", SIZES).default(theme.pagination.size.as_str()),
                    Control::slider("radius", SIZES).default(theme.pagination.radius.as_str()),
                    Control::color("color"),
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
