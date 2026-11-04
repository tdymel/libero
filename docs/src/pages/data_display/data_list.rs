use crate::components::{Control, Demo, DemoFile, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::{Chip, Code, DataList, DataListItem, Orientation, Text};
use libero::theme::responsive;
use libero::use_theme;

/// The page's own source: the printed parts are cut from its live demo.
const FILE: DemoFile = DemoFile(include_str!("data_list.rs"));

/// The demo's orientation from the `md` breakpoint up, `vertical` below it.
fn responsive_on(values: &DemoValues) -> bool {
    values.str("responsive") == "true"
}

#[component]
pub fn DataListPage() -> Element {
    let theme = use_theme();
    let phones = vec!["555-1234", "555-5678"];

    rsx! {
        DocPage {
            title: "DataList",
            source: "libero/src/components/data_display/data_list",
            markdown: "/md/data_list.md",
            properties: vec![
                props("DataList", vec![
                    prop("orientation", "Responsive<Orientation>")
                        .default("vertical")
                        .doc("`horizontal` puts each description beside its term, `vertical` below it. Horizontal makes the `<dl>` a two-column grid, terms in the first column and every description in the second. Or one per breakpoint, `orientation: responsive(Orientation::Vertical).md(Orientation::Horizontal)`: the layout switches with the window from the first paint."),
                    prop("gap", "Responsive<ThemeAwareValue>")
                        .default(theme.data_list.size.as_str())
                        .doc("Gap between rows, or any CSS, e.g. `gap: \"0\"`, per breakpoint too. In a vertical list a description sits half that gap under its term. In a horizontal list the gap is also the space between the term and description columns."),
                    prop("children", "Element")
                        .default("required")
                        .doc("`DataListItem`s, or any `dt` and `dd` content."),
                ]),
                props("DataListItem", vec![
                    prop("label", "Element")
                        .default("required")
                        .doc("The term, a `<dt>`. `sx`, `class` and `states` style the term only."),
                    prop("children", "Element")
                        .default("required")
                        .doc("The term's descriptions, in one `<dd>`. To give a description an `id` (an `aria-describedby` target) or a style, wrap it in a `div` here: `div { id: \"plan-note\", \"Pro\" }`."),
                ]),
            ],
            accessibility: a11y()
                .handles(["Each term is a `<dt>` and its descriptions one `<dd>`, so a screen reader pairs them."])
                .must([
                    "Put each `DataListItem` directly inside the `DataList`. A wrapper element between them collapses the horizontal layout's two columns.",
                ])
                .example("An order summary with a `DataListItem` per row, such as \"Status\" and \"Shipped\": a screen reader pairs each term with its description."),
            lead: rsx! {
                Text {
                    "A "
                    Code { source: "dl" }
                    " of terms and descriptions. One term can carry several descriptions, "
                    "which share one "
                    Code { source: "dd" }
                    "."
                }
            },
            // snippet: let phones = ["+49 30 1234567"];
            Demo {
                component: "DataList",
                children_text: "",
                children_code: FILE.section("children"),
                controls: vec![
                    Control::toggle("orientation", ["horizontal", "vertical"])
                        .labels(["Horizontal", "Vertical"])
                        .default("vertical")
                        .code(|_, values| match (responsive_on(values), values.str("orientation").as_str()) {
                            (true, orientation) => vec![format!(
                                "orientation: responsive(Orientation::Vertical).md(Orientation::{:?})",
                                Orientation::from(orientation)
                            )],
                            (false, "vertical") => vec![],
                            (false, orientation) => vec![format!("orientation: {orientation:?}")],
                        }),
                    // On, so the breakpoint switch shows without another switch: resize the window.
                    Control::switch("responsive").default("true").code(|_, _| vec![]),
                    Control::slider("gap", ["0", "xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                ],
                render: move |values: DemoValues| rsx! {
                    DataList {
                        orientation: match (responsive_on(&values), Orientation::from(values.str("orientation").as_str())) {
                            (true, chosen) => responsive(Orientation::Vertical).md(chosen),
                            (false, chosen) => chosen.into(),
                        },
                        gap: values.str("gap"),
                        // demo-code: children start
                        DataListItem {
                            label: rsx! { "Status" },
                            Chip { variant: "filled", color: "success", size: "xs", "Active" }
                        }
                        DataListItem { label: rsx! { "Owner" }, "Jamie Chen" }
                        DataListItem {
                            label: rsx! { "Phone" },
                            for phone in &phones {
                                div { "{phone}" }
                            }
                        }
                        // demo-code: children end
                    }
                },
            }
        }
    }
}
