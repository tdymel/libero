use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Table, TableColumnsButton, TableDensityButton, TableExportButton, column},
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
