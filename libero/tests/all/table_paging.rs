use crate::common::{body, render};
use crate::table_fixture::{Person, order_of};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{SortDirection, Table, TableSort, column},
    localization::Localization,
};

fn people() -> Vec<Person> {
    ["Ada", "Bea", "Cy", "Dan", "Eve"]
        .into_iter()
        .zip([36, 28, 51, 19, 44])
        .map(|(name, age)| Person { name, age })
        .collect()
}

/// Names in the order the body shows them.
fn shown(body: &str) -> Vec<&'static str> {
    order_of(body, &["Ada", "Bea", "Cy", "Dan", "Eve"])
}

#[test]
fn a_table_without_page_props_draws_no_page_controls() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "People",
                    data: people(),
                    columns: vec![column("Name").value(|row: &Person| row.name.to_string())],
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    assert!(!body.contains("<nav"), "{body}");
    assert_eq!(shown(&body).len(), 5);
}

#[test]
fn a_default_page_shows_its_slice_and_range() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "People",
                    data: people(),
                    columns: vec![column("Name").value(|row: &Person| row.name.to_string())],
                    default_page_size: 2,
                    default_page: 2,
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    assert_eq!(shown(&body), ["Cy", "Dan"], "{body}");
    assert!(body.contains("3–4 of 5"), "{body}");
    assert!(body.contains("aria-label=\"Pages of People\""), "{body}");
    assert!(
        body.contains("data-slot=\"range\">3–4 of 5</span>"),
        "{body}"
    );
    // The live region starts empty: the first range is no news.
    assert!(!body.contains("role=\"status\">3"), "{body}");
    // No `page_sizes`, no picker.
    assert!(!body.contains("Rows per page"), "{body}");
}

#[test]
fn the_table_sorts_before_it_pages() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    data: people(),
                    columns: vec![
                        column("Name").value(|row: &Person| row.name.to_string()),
                        column("Age").value(|row: &Person| row.age).sortable(),
                    ],
                    default_sort: vec![TableSort::new("Age", SortDirection::Descending)],
                    default_page_size: 2,
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    assert_eq!(shown(&body), ["Cy", "Eve"], "{body}");
    assert!(body.contains("aria-label=\"Table pages\""), "{body}");
}

#[test]
fn a_controlled_page_past_the_end_shows_the_last_page() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    data: people(),
                    columns: vec![column("Name").value(|row: &Person| row.name.to_string())],
                    page: 9,
                    onpagechange: |_| {},
                    page_size: 2,
                    onpagesizechange: |_| {},
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    assert_eq!(shown(&body), ["Eve"], "{body}");
    assert!(body.contains("5–5 of 5"), "{body}");
    assert!(body.contains("aria-current=\"page\""), "{body}");
}

#[test]
fn manual_stages_draw_data_as_given() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    data: people().into_iter().take(2).collect::<Vec<_>>(),
                    columns: vec![
                        column("Name").value(|row: &Person| row.name.to_string()),
                        column("Age").value(|row: &Person| row.age).sortable(),
                    ],
                    sort: vec![TableSort::new("Age", SortDirection::Descending)],
                    onsortchange: |_| {},
                    manual_sort: true,
                    page: 3,
                    onpagechange: |_| {},
                    page_size: 2,
                    onpagesizechange: |_| {},
                    manual_pagination: true,
                    row_count: 50,
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    // Source order, though the header says descending: the caller sorted.
    assert_eq!(shown(&body), ["Ada", "Bea"], "{body}");
    assert!(body.contains("aria-sort=\"descending\""), "{body}");
    assert!(body.contains("5–6 of 50"), "{body}");
}

#[test]
fn page_sizes_draw_a_localized_picker() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { localization: &Localization::GERMAN,
                Table {
                    caption: "Leute",
                    data: people(),
                    columns: vec![column("Name").value(|row: &Person| row.name.to_string())],
                    page_sizes: vec![3, 10],
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    // The first size seeds the page.
    assert_eq!(shown(&body), ["Ada", "Bea", "Cy"], "{body}");
    assert!(body.contains("Zeilen pro Seite"), "{body}");
    assert!(body.contains("1–3 von 5"), "{body}");
    assert!(body.contains("aria-label=\"Seiten von Leute\""), "{body}");
}
