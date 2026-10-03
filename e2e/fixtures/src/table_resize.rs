//! `Table` column resize (1156-4d): the resized widths printed below.

use dioxus::prelude::*;
use libero::components::{ColumnWidths, Table, column};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/table-resize", || rsx! { ResizeTablePage {} }),
    ("/table-resize/wide", || rsx! { WideResizePage {} }),
];

/// Eight unsized columns of long unbreakable words, wider than any viewport: the
/// table overflows (2017).
#[component]
fn WideResizePage() -> Element {
    let mut widths = use_signal(ColumnWidths::new);
    let shown: Vec<String> = widths
        .read()
        .iter()
        .map(|(header, width)| format!("{header}={width}"))
        .collect();
    let columns = ["Name", "Origin", "Season", "Colour", "Taste", "Storage", "Supplier", "Notes"]
        .map(|header| {
            column(header).value(move |row: &usize| {
                format!("{header}-{row}-from-the-far-north-of-the-steppe")
            })
        })
        .to_vec();
    rsx! {
        Table {
            aria_label: "Fruit",
            scroll: true,
            data: vec![1usize, 2],
            columns,
            resizable_columns: true,
            column_menu: true,
            column_widths: widths(),
            oncolumnwidthschange: move |next| widths.set(next),
        }
        p { id: "widths", {shown.join(" ")} }
    }
}

#[derive(Clone, PartialEq)]
struct Fruit {
    name: &'static str,
    stock: u32,
    origin: &'static str,
}

#[component]
fn ResizeTablePage() -> Element {
    let mut widths = use_signal(ColumnWidths::new);
    let data = vec![
        Fruit {
            name: "Cherry",
            stock: 3,
            origin: "Turkey",
        },
        Fruit {
            name: "Apple",
            stock: 12,
            origin: "Kazakhstan",
        },
    ];
    let shown: Vec<String> = widths
        .read()
        .iter()
        .map(|(header, width)| format!("{header}={width}"))
        .collect();
    rsx! {
        Table {
            aria_label: "Fruit",
            scroll: true,
            data,
            columns: vec![
                column("Name")
                    .value(|fruit: &Fruit| fruit.name.to_string())
                    .row_header()
                    // Clear of the header's min-content (117px here), which a wider fallback
                    // font pushed past 120px on CI and off the resize base (todo 2103).
                    .width("150px")
                    .resize_limits(100.0, 300.0),
                column("Stock").value(|fruit: &Fruit| fruit.stock).sortable(),
                column("Origin").value(|fruit: &Fruit| fruit.origin.to_string()).resizable(false),
            ],
            row_key: |fruit: &Fruit| fruit.name.to_string(),
            resizable_columns: true,
            column_menu: true,
            column_widths: widths(),
            oncolumnwidthschange: move |next| widths.set(next),
        }
        p { id: "widths", {shown.join(" ")} }
    }
}
