use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        ColumnFilter, FilterOperator, Table, TableColumnsButton, TableDensityButton,
        TableExportButton, TableFilterButton, column,
    },
    theme::Size,
};

fn names() -> Vec<String> {
    vec!["Ada".to_string(), "Bea".to_string()]
}

/// 1416: the pieces render in a table's toolbar, and a density seed wins over `size`.
#[test]
fn the_toolbar_pieces_render_and_the_density_wins_over_size() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "People",
                    size: Size::Sm,
                    default_density: Size::Lg,
                    toolbar: rsx! {
                        TableColumnsButton {}
                        TableDensityButton {}
                        TableExportButton { onexport: |_| {} }
                    },
                    data: names(),
                    columns: vec![column("Name").value(|name: &String| name.clone())],
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    for (tool, label) in [
        ("columns", ">Columns<"),
        ("density", ">Density<"),
        ("export", ">Export<"),
    ] {
        assert!(
            body.contains(&format!("data-table-tool=\"{tool}\"")),
            "{tool}: {body}"
        );
        assert!(body.contains(label), "{label}: {body}");
    }
    let table = &body[body.find("<table").unwrap()..];
    let state = table
        .split("data-state=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .expect("the table's data-state");
    assert!(state.split(' ').any(|token| token == "size-lg"), "{state}");
    assert!(!state.split(' ').any(|token| token == "size-sm"), "{state}");
}

#[test]
fn a_piece_outside_a_table_renders_nothing() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                TableColumnsButton {}
                TableExportButton { onexport: |_| {} }
            }
        }
    }

    let html = render(app);
    assert!(!html.contains("data-table-tool"), "{html}");
}

/// 1400/1448: `TableFilterButton` puts a Filters button where the toolbar has it,
/// named with the active count; the count it shows is hidden from readers. The
/// dialog waits for a click.
#[test]
fn the_filter_button_sits_in_the_row_and_names_the_count() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "People",
                    filter_panel: true,
                    toolbar: rsx! {
                        TableFilterButton {}
                        TableExportButton { onexport: |_| {} }
                    },
                    default_column_filters: vec![
                        ColumnFilter::new("Name", FilterOperator::Contains, "a"),
                        ColumnFilter::new("Name", FilterOperator::Contains, ""),
                    ],
                    data: names(),
                    columns: vec![column("Name").value(|name: &String| name.clone())],
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let button = body.find("data-filter-panel-button").expect(&body);
    assert!(
        button < body.find("data-table-tool=\"export\"").unwrap(),
        "{body}"
    );
    assert!(body.contains("aria-label=\"Filters, 1 active\""), "{body}");
    assert!(body.contains("data-filter-count"), "{body}");
    assert!(!body.contains("data-filter-panel=true"), "{body}");
    assert_eq!(
        body.matches("data-filter-panel-button").count(),
        1,
        "{body}"
    );
}

/// 1448: with no `toolbar` the table puts the button there itself; with one,
/// only its `TableFilterButton` does. Without `filter_panel` the piece is empty.
#[test]
fn the_table_places_the_filter_button_only_without_a_toolbar() {
    fn table(panel: bool, toolbar: Option<Element>) -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "People",
                    filter_panel: panel,
                    toolbar,
                    data: names(),
                    columns: vec![column("Name").value(|name: &String| name.clone())],
                }
            }
        }
    }
    let cases: [(fn() -> Element, usize); 3] = [
        (|| table(true, None), 1),
        (
            || table(true, Some(rsx! { TableExportButton { onexport: |_| {} } })),
            0,
        ),
        (|| table(false, Some(rsx! { TableFilterButton {} })), 0),
    ];
    for (index, (app, buttons)) in cases.into_iter().enumerate() {
        let html = render(app);
        assert_eq!(
            html.matches("data-filter-panel-button").count(),
            buttons,
            "case {index}: {html}"
        );
    }
}
