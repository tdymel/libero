//! The quick filter, column filters and header filters, and how they meet
//! the empty state, sorting, paging and selection.

use crate::common::{attributes_of, body, render};
use crate::table_fixture::Item;

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Column, ColumnFilter, FilterOperator, SortDirection, Table, TableSort, column},
};

#[test]
fn the_no_results_slot_replaces_the_text_when_the_filter_leaves_nothing() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Stock",
                    empty: rsx! { "Nothing in stock" },
                    no_results: rsx! { "Try another word" },
                    quick_filter: "pear",
                    data: vec![Item { name: "Apple" }],
                    columns: vec![column("Name").value(|item: &Item| item.name.to_string())],
                }
            }
        }
    }

    let body = body(&render(app));
    assert!(body.contains("Try another word"), "{body}");
    assert!(!body.contains("Nothing in stock"));
    assert!(!body.contains("No matching rows"));
}

#[derive(Clone, PartialEq)]
struct City {
    name: &'static str,
    country: &'static str,
    code: &'static str,
}

fn cities() -> Vec<City> {
    vec![
        City {
            name: "London",
            country: "United Kingdom",
            code: "LON",
        },
        City {
            name: "Paris",
            country: "France",
            code: "PAR",
        },
        City {
            name: "Lyon",
            country: "France",
            code: "LYS",
        },
    ]
}

fn city_columns() -> Vec<Column<City>> {
    vec![
        column("City")
            .value(|c: &City| c.name)
            .row_header()
            .sortable(),
        column("Country").value(|c: &City| c.country),
        column("Code").value(|c: &City| c.code).filterable(false),
    ]
}

#[test]
fn the_quick_filter_keeps_rows_whose_shown_filterable_cells_hold_every_word() {
    let html = render(|| {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Cities",
                    data: cities(),
                    columns: city_columns(),
                    default_quick_filter: "FRANCE  ly",
                    show_quick_filter: true,
                }
            }
        }
    });
    let body = body(&html);
    assert!(body.contains("Lyon"), "{body}");
    assert!(
        !body.contains("Paris") && !body.contains("London"),
        "{body}"
    );
    let input = attributes_of(&html, "input");
    assert_eq!(input["type"], "search");
    assert_eq!(input["value"], "FRANCE  ly");
    assert!(body.contains(">Search<"), "{body}");
    assert_eq!(body.matches("role=\"status\"").count(), 1, "{body}");

    // `filterable(false)` and hidden columns are not searched.
    let html = render(|| {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Cities",
                    data: cities(),
                    columns: city_columns(),
                    default_quick_filter: "LYS",
                }
            }
        }
    });
    assert!(!html.contains("Lyon"), "{html}");
    let html = render(|| {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Cities",
                    data: cities(),
                    columns: city_columns(),
                    default_quick_filter: "france",
                    default_hidden_columns: vec!["Country".into()],
                }
            }
        }
    });
    assert!(!html.contains("Paris"), "{html}");
}

/// Each table's "Search" field is told apart by its caption.
#[test]
fn the_quick_filter_field_is_described_by_the_caption() {
    let html = render(|| {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "Cities",
                    data: cities(),
                    columns: city_columns(),
                    show_quick_filter: true,
                }
            }
        }
    });
    let described = attributes_of(&html, "input")["aria-describedby"].clone();
    assert_eq!(attributes_of(&html, "caption")["id"], described, "{html}");
}

#[test]
fn a_filter_that_leaves_nothing_says_no_results_not_empty() {
    let html = render(|| {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Cities",
                    data: cities(),
                    columns: city_columns(),
                    empty: rsx! { "No cities yet." },
                    default_quick_filter: "Berlin",
                }
            }
        }
    });
    assert!(html.contains("No matching rows"), "{html}");
    assert!(!html.contains("No cities yet."), "{html}");

    let html = render(|| {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Cities",
                    data: Vec::<City>::new(),
                    columns: city_columns(),
                    empty: rsx! { "No cities yet." },
                    default_quick_filter: "Berlin",
                }
            }
        }
    });
    assert!(html.contains("No cities yet."), "{html}");

    // Filtered by a server: an empty page for a query has no results.
    let html = render(|| {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Cities",
                    data: Vec::<City>::new(),
                    columns: city_columns(),
                    default_quick_filter: "Berlin",
                    manual_filter: true,
                }
            }
        }
    });
    assert!(html.contains("No matching rows"), "{html}");
}

#[test]
fn manual_filter_draws_data_as_given() {
    let html = render(|| {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Cities",
                    data: cities(),
                    columns: city_columns(),
                    default_quick_filter: "Berlin",
                    manual_filter: true,
                }
            }
        }
    });
    assert_eq!(
        body(&html).matches("<th scope=\"row\"").count(),
        3,
        "{html}"
    );
}

#[test]
fn filter_sort_and_page_compose_and_select_all_covers_the_kept_rows() {
    let html = render(|| {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Cities",
                    data: cities(),
                    columns: city_columns(),
                    default_quick_filter: "france",
                    default_sort: vec![TableSort::new("City", SortDirection::Ascending)],
                    default_page_size: 1usize,
                    selectable: true,
                    // Paris and Lyon by index: every kept row, so select-all is checked.
                    default_selection: vec!["1".to_string(), "2".to_string()],
                }
            }
        }
    });
    let body = body(&html);
    assert!(body.contains("Lyon") && !body.contains("Paris"), "{body}");
    assert!(body.contains("1–1 of 2"), "{body}");
    assert!(!body.contains("aria-checked=\"mixed\""), "{body}");
}

#[derive(Clone, PartialEq)]
struct Produce {
    item: &'static str,
    count: u32,
    sold_out: bool,
}

fn produce() -> Vec<Produce> {
    vec![
        Produce {
            item: "Apples",
            count: 12,
            sold_out: false,
        },
        Produce {
            item: "Pears",
            count: 3,
            sold_out: false,
        },
        Produce {
            item: "Plums",
            count: 0,
            sold_out: true,
        },
    ]
}

fn produce_columns() -> Vec<Column<Produce>> {
    vec![
        column("Item").value(|p: &Produce| p.item).row_header(),
        column("Count")
            .value(|p: &Produce| p.count)
            .format(|p: &Produce| format!("{} pcs", p.count)),
        column("Sold out").value(|p: &Produce| p.sold_out),
    ]
}

// A macro: `render` takes a fn pointer, which captures nothing.
macro_rules! produce_html {
    ($($filter:expr),+ $(,)?) => {
        render(|| {
            rsx! {
                LiberoProvider {
                    Table {
                        aria_label: "Produce",
                        data: produce(),
                        columns: produce_columns(),
                        default_column_filters: vec![$($filter),+],
                    }
                }
            }
        })
    };
}

#[test]
fn column_filters_compare_by_type_and_all_apply() {
    use FilterOperator::*;
    // By value, not the "pcs" text, and with a decimal comma.
    let html = produce_html!(ColumnFilter::new("Count", GreaterThan, "2,5"));
    assert!(html.contains("Apples") && html.contains("Pears"), "{html}");
    assert!(!html.contains("Plums"), "{html}");

    let html = produce_html!(
        ColumnFilter::new("Count", GreaterThan, "2"),
        ColumnFilter::new("Item", StartsWith, "P"),
    );
    assert!(html.contains("Pears") && !html.contains("Apples"), "{html}");

    let html = produce_html!(ColumnFilter::new("Sold out", Is, "true"));
    assert!(html.contains("Plums") && !html.contains("Pears"), "{html}");

    // An unfinished filter keeps every row.
    let html = produce_html!(ColumnFilter::new("Count", LessThan, "-"));
    assert_eq!(
        body(&html).matches("<th scope=\"row\"").count(),
        3,
        "{html}"
    );

    let html = produce_html!(ColumnFilter::new("Item", Equals, "kiwis"));
    assert!(html.contains("No matching rows"), "{html}");
}

#[test]
fn header_filters_draw_a_labelled_field_per_filterable_column() {
    let html = render(|| {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Cities",
                    data: cities(),
                    columns: city_columns(),
                    header_filters: true,
                    default_column_filters: vec![ColumnFilter::new("Country", FilterOperator::Contains, "fra")],
                }
            }
        }
    });
    let head = &html[html.find("<thead").unwrap()..html.find("</thead>").unwrap()];
    assert_eq!(head.matches("data-filter-cell").count(), 3, "{head}");
    assert!(head.contains("aria-label=\"Filter City\""), "{head}");
    assert!(head.contains("value=\"fra\""), "{head}");
    // `filterable(false)` leaves its cell empty.
    assert!(!head.contains("aria-label=\"Filter Code\""), "{head}");
    assert!(!body(&html).contains("London"), "{html}");
}

#[test]
fn a_filtered_column_shows_a_button_beside_its_menu() {
    let html = render(|| {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Cities",
                    data: cities(),
                    columns: city_columns(),
                    column_menu: true,
                    default_column_filters: vec![
                        ColumnFilter::new("Country", FilterOperator::Contains, "fra"),
                        // No value yet: nothing filters, so no button.
                        ColumnFilter::new("City", FilterOperator::Contains, ""),
                    ],
                }
            }
        }
    });
    assert_eq!(body(&html).matches("data-filtered").count(), 1, "{html}");
    assert!(
        html.contains("aria-label=\"Country is filtered\""),
        "{html}"
    );
}
