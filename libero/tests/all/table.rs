use std::collections::BTreeMap;

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Checkbox, ColumnDefaults, Mark, SortDirection, States, Table, TableSort, column},
    localization::Localization,
    theme::Size,
};

#[test]
fn a_table_marks_only_sortable_headers_and_aligns_by_cell_type() {
    #[derive(Clone, PartialEq)]
    struct Row {
        name: &'static str,
        age: u32,
    }

    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    data: vec![
                        Row { name: "Ada", age: 36 },
                        Row { name: "Grace", age: 45 },
                    ],
                    columns: vec![
                        column("Name").value(|row: &Row| row.name.to_string()),
                        column("Age").value(|row: &Row| row.age).sortable(),
                    ],
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    // Two rows of data, plus the header row.
    assert_eq!(body.matches("<tr").count(), 3);
    assert_eq!(body.matches("<td").count(), 4);
    assert!(body.contains("Ada"));
    assert!(body.contains("36"));

    // Only the sortable column is a button; nothing is sorted, so no header
    // carries `aria-sort` (APG: the sorted column only).
    assert_eq!(body.matches("data-sortable").count(), 1);
    assert!(!body.contains("aria-sort"));
    assert_eq!(body.matches("<button").count(), 1);
    // The arrow is always in the markup, so sorting can't resize the header.
    assert_eq!(body.matches("<svg").count(), 1);

    // The numeric column aligns itself; the text column doesn't say anything.
    assert_eq!(body.matches("data-align=\"end\"").count(), 3);
    assert!(!body.contains("data-align=\"start\""));
    assert_eq!(attributes_of(&html, "table")["aria-label"], "People");
    assert_eq!(body.matches("<th scope=\"col\"").count(), 2);
}

/// A screen reader names each row by its row header while moving down another
/// column (todo 742).
#[test]
fn a_row_header_column_renders_th_scope_row() {
    #[derive(Clone, PartialEq)]
    struct Row {
        name: &'static str,
        age: u32,
    }

    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    data: vec![
                        Row { name: "Ada", age: 36 },
                        Row { name: "Grace", age: 45 },
                    ],
                    columns: vec![
                        column("Name").value(|row: &Row| row.name.to_string()).row_header(),
                        column("Age").value(|row: &Row| row.age),
                    ],
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert_eq!(body.matches("<th scope=\"row\"").count(), 2, "{body}");
    assert!(body.contains("<th scope=\"row\">Ada</th>"), "{body}");
    assert_eq!(body.matches("<td").count(), 2);
    assert_eq!(body.matches("<th scope=\"col\"").count(), 2);
}

#[test]
fn a_custom_render_replaces_the_cell_body() {
    #[derive(Clone, PartialEq)]
    struct Row {
        score: u32,
    }

    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    data: vec![Row { score: 7 }],
                    columns: vec![
                        column("Score")
                            .value(|row: &Row| row.score)
                            .render(|row: &Row| rsx! { Mark { "{row.score} pts" } }),
                    ],
                }
            }
        }
    }

    let body = body(&render(app));

    assert!(body.contains("7 pts"));
    assert!(body.contains("<mark"));
}

#[derive(Clone, PartialEq)]
struct Item {
    name: &'static str,
}

/// The attributes of the scroll region's `div`, wherever `role` sits in it.
fn region(html: &str) -> BTreeMap<String, String> {
    let at = html.find("role=\"region\"").expect("no scroll region");
    let start = html[..at].rfind("<div").unwrap();
    attributes_of(&html[start..], "div")
}

#[test]
fn a_caption_names_the_table_and_the_scroll_region() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "Stock",
                    scroll: true,
                    data: vec![Item { name: "Apple" }],
                    columns: vec![column("Name").value(|item: &Item| item.name.to_string())],
                }
            }
        }
    }

    let html = render(app);
    let caption = attributes_of(&html, "caption");
    let region = region(&html);

    assert!(body(&html).contains(">Stock</caption>"));
    assert_eq!(region["aria-labelledby"], caption["id"]);
    assert_eq!(region["tabindex"], "0");
}

#[test]
fn a_scroll_region_copies_the_callers_label() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Stock",
                    scroll: true,
                    data: vec![Item { name: "Apple" }],
                    columns: vec![column("Name").value(|item: &Item| item.name.to_string())],
                }
            }
        }
    }

    let html = render(app);

    assert_eq!(region(&html)["aria-label"], "Stock");
    assert_eq!(attributes_of(&html, "table")["aria-label"], "Stock");
}

#[test]
fn no_scroll_means_no_region_and_no_caption() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Stock",
                    data: vec![Item { name: "Apple" }],
                    columns: vec![column("Name").value(|item: &Item| item.name.to_string())],
                }
            }
        }
    }

    let body = body(&render(app));

    assert!(!body.contains("role=\"region\""));
    assert!(!body.contains("<caption"));
}

/// The first column's text per body row, in order.
fn names(body: &str) -> Vec<&str> {
    body.split("<tbody")
        .nth(1)
        .unwrap_or_default()
        .split("<td")
        .skip(1)
        .map(|cell| &cell[cell.find('>').unwrap() + 1..cell.find("</td>").unwrap()])
        .collect()
}

fn fruit(sort: Option<Vec<TableSort>>, default_sort: Vec<TableSort>) -> Element {
    rsx! {
        LiberoProvider {
            Table {
                aria_label: "Stock",
                data: vec![Item { name: "Cherry" }, Item { name: "Apple" }, Item { name: "Banana" }],
                columns: vec![column("Name").value(|item: &Item| item.name.to_string()).sortable()],
                sort,
                default_sort,
                onsortchange: |_| {},
            }
        }
    }
}

#[test]
fn a_default_sort_orders_the_first_render() {
    let html = render(|| {
        fruit(
            None,
            vec![TableSort::new("Name", SortDirection::Descending)],
        )
    });
    let body = body(&html);

    assert_eq!(names(&body), ["Cherry", "Banana", "Apple"], "{body}");
    assert!(body.contains("aria-sort=\"descending\""), "{body}");
}

#[test]
fn a_controlled_sort_wins_over_the_default_even_when_unsorted() {
    fn default() -> Vec<TableSort> {
        vec![TableSort::new("Name", SortDirection::Descending)]
    }

    let sorted = render(|| {
        fruit(
            Some(vec![TableSort::new("Name", SortDirection::Ascending)]),
            default(),
        )
    });
    assert_eq!(names(&body(&sorted)), ["Apple", "Banana", "Cherry"]);

    let unsorted = render(|| fruit(Some(Vec::new()), default()));
    let body = body(&unsorted);
    assert_eq!(names(&body), ["Cherry", "Apple", "Banana"]);
    assert!(!body.contains("aria-sort"));
}

#[test]
fn the_empty_slot_spans_every_column_only_without_rows() {
    fn table(data: Vec<Item>) -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Stock",
                    empty: rsx! { "Nothing in stock" },
                    data,
                    columns: vec![
                        column("Name").value(|item: &Item| item.name.to_string()),
                        column("Also").value(|item: &Item| item.name.to_string()),
                    ],
                }
            }
        }
    }

    let empty = body(&render(|| table(Vec::new())));
    assert!(empty.contains("colspan=\"2\""), "{empty}");
    assert!(empty.contains("Nothing in stock"));

    let full = body(&render(|| table(vec![Item { name: "Apple" }])));
    assert!(!full.contains("Nothing in stock"));
    assert!(!full.contains("data-empty"));
}

#[derive(Clone, PartialEq)]
struct Stock {
    id: u32,
    name: &'static str,
    cents: u32,
}

fn stock(size: Option<Size>, striped: bool) -> Element {
    rsx! {
        LiberoProvider {
            Table {
                aria_label: "Stock",
                data: vec![
                    Stock { id: 7, name: "Apple", cents: 120 },
                    Stock { id: 9, name: "Pear", cents: 5 },
                ],
                columns: vec![
                    column("Name").value(|s: &Stock| s.name.to_string()),
                    column("Price")
                        .value(|s: &Stock| s.cents)
                        .format(|s: &Stock| format!("${}.{:02}", s.cents / 100, s.cents % 100)),
                ],
                row_key: |s: &Stock| s.id.to_string(),
                row_states: |s: &Stock| States::new().with("cheap", s.cents < 100),
                row_attrs: |s: &Stock| vec![Attribute::new("data-id", s.id.to_string(), None, false)],
                size,
                striped,
            }
        }
    }
}

#[test]
fn rows_carry_their_states_and_attributes_and_cells_their_format() {
    let body = body(&render(|| stock(None, false)));

    assert!(body.contains("$1.20") && body.contains("$0.05"), "{body}");
    assert_eq!(body.matches("data-state=\"cheap\"").count(), 1, "{body}");
    assert!(
        body.contains("data-id=\"7\"") && body.contains("data-id=\"9\""),
        "{body}"
    );
}

#[test]
fn size_and_stripes_are_table_states() {
    let html = render(|| stock(None, false));
    assert_eq!(attributes_of(&html, "table")["data-state"], "size-md");

    let html = render(|| stock(Some(Size::Sm), true));
    assert_eq!(
        attributes_of(&html, "table")["data-state"],
        "size-sm striped"
    );
}

#[test]
fn an_empty_table_says_so_in_the_active_language() {
    fn table() -> Element {
        rsx! {
            Table {
                aria_label: "Stock",
                data: Vec::<Item>::new(),
                columns: vec![column("Name").value(|item: &Item| item.name.to_string())],
            }
        }
    }

    let english = body(&render(|| rsx! { LiberoProvider { {table()} } }));
    assert!(english.contains(">No rows</td>"), "{english}");

    let german = body(&render(|| {
        rsx! { LiberoProvider { localization: &Localization::GERMAN, {table()} } }
    }));
    assert!(german.contains(">Keine Zeilen</td>"), "{german}");
}

fn selectable(selection: Vec<String>) -> Element {
    rsx! {
        LiberoProvider {
            Table {
                aria_label: "Stock",
                selectable: true,
                default_selection: selection,
                data: vec![
                    Stock { id: 7, name: "Apple", cents: 120 },
                    Stock { id: 9, name: "Pear", cents: 5 },
                ],
                columns: vec![
                    column("Name").value(|s: &Stock| s.name.to_string()).row_header(),
                    column("Price").value(|s: &Stock| s.cents),
                ],
                row_key: |s: &Stock| s.id.to_string(),
            }
        }
    }
}

#[test]
fn a_selectable_table_marks_selected_rows_and_mixes_select_all() {
    let body = body(&render(|| selectable(vec!["9".into()])));

    assert_eq!(
        body.matches("aria-label=\"Select all rows\"").count(),
        1,
        "{body}"
    );
    assert!(body.contains("aria-label=\"Select Apple\""), "{body}");
    assert!(body.contains("aria-label=\"Select Pear\""), "{body}");
    assert_eq!(body.matches("aria-selected=\"true\"").count(), 1, "{body}");
    assert_eq!(body.matches("aria-selected=\"false\"").count(), 1, "{body}");
    assert_eq!(body.matches("aria-checked=\"mixed\"").count(), 1, "{body}");
    assert_eq!(body.matches("role=\"status\"").count(), 1, "{body}");
    assert_eq!(body.matches("<th scope=\"col\"").count(), 3, "{body}");
}

#[test]
fn select_all_is_checked_only_with_every_row_selected() {
    let all = body(&render(|| {
        selectable(vec!["7".into(), "9".into(), "x".into()])
    }));
    assert!(!all.contains("aria-checked=\"mixed\""), "{all}");
    assert_eq!(all.matches("aria-selected=\"true\"").count(), 2, "{all}");

    let none = body(&render(|| selectable(Vec::new())));
    assert!(!none.contains("aria-checked=\"mixed\""), "{none}");
    assert!(!none.contains("aria-selected=\"true\""), "{none}");
}

#[test]
fn a_row_box_renders_as_a_plain_checkbox() {
    fn table() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Stock",
                    size: Size::Sm,
                    selectable: true,
                    default_selection: vec!["7".to_string()],
                    data: vec![Stock { id: 7, name: "Apple", cents: 120 }],
                    columns: vec![column("Name").value(|s: &Stock| s.name.to_string()).row_header()],
                    row_key: |s: &Stock| s.id.to_string(),
                }
            }
        }
    }
    fn checkbox() -> Element {
        rsx! {
            LiberoProvider {
                Checkbox {
                    aria_label: "Select Apple",
                    size: Size::Sm,
                    checked: true,
                    onchange: |_| {},
                }
            }
        }
    }
    // The table's lighter box (todo 1195) must stay the same markup, ids aside.
    let table = body(&render(table));
    let cell = table.split("<td data-select=true>").nth(1).unwrap();
    let cell = &cell[..cell.find("</td>").unwrap()];
    let plain = render(checkbox);
    // The provider's empty portal follows the box.
    let plain = body(&plain)
        .strip_suffix("<div></div>")
        .unwrap()
        .to_string();
    let id = plain.find(" id=\"").unwrap();
    let end = id + 5 + plain[id + 5..].find('"').unwrap() + 1;
    assert_eq!(cell, format!("{}{}", &plain[..id], &plain[end..]));
}

#[test]
fn a_table_without_selectable_has_no_checkbox_column() {
    let body = body(&render(|| stock(None, false)));

    assert!(!body.contains("aria-selected"), "{body}");
    assert!(!body.contains("data-select"), "{body}");
    assert!(!body.contains("role=\"status\""), "{body}");
}

fn ranked(multi_sort: bool) -> Element {
    rsx! {
        LiberoProvider {
            Table {
                aria_label: "Stock",
                multi_sort,
                default_sort: vec![
                    TableSort::new("Price", SortDirection::Descending),
                    TableSort::new("Name", SortDirection::Ascending),
                ],
                data: vec![
                    Stock { id: 1, name: "Pear", cents: 5 },
                    Stock { id: 2, name: "Apple", cents: 120 },
                    Stock { id: 3, name: "Fig", cents: 5 },
                ],
                columns: vec![
                    column("Name").value(|s: &Stock| s.name.to_string()).sortable(),
                    column("Price").value(|s: &Stock| s.cents).sortable(),
                ],
            }
        }
    }
}

#[test]
fn multi_sort_orders_by_every_sorted_column_and_ranks_the_headers() {
    let body = body(&render(|| ranked(true)));

    let (apple, fig, pear) = (
        body.find("Apple").unwrap(),
        body.find("Fig").unwrap(),
        body.find("Pear").unwrap(),
    );
    assert!(apple < fig && fig < pear, "{body}");
    assert_eq!(body.matches("aria-sort=").count(), 2, "{body}");
    assert!(
        body.contains("sort order 1") && body.contains("sort order 2"),
        "{body}"
    );
    assert_eq!(body.matches("data-sort-order").count(), 2, "{body}");
}

#[test]
fn without_multi_sort_only_the_first_entry_sorts() {
    let body = body(&render(|| ranked(false)));

    let (fig, pear) = (body.find("Fig").unwrap(), body.find("Pear").unwrap());
    assert!(pear < fig, "{body}");
    assert_eq!(body.matches("aria-sort=").count(), 1, "{body}");
    assert!(!body.contains("data-sort-order"), "{body}");
}

#[test]
fn widths_defaults_and_a_header_render_reach_the_header_cells() {
    #[derive(Clone, PartialEq)]
    struct Row {
        name: &'static str,
        age: u32,
    }

    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    column_defaults: ColumnDefaults::new().min_width("4rem"),
                    data: vec![Row { name: "Ada", age: 36 }],
                    columns: vec![
                        column("Name").value(|row: &Row| row.name.to_string()).width("12rem"),
                        column("Age")
                            .value(|row: &Row| row.age)
                            .min_width("2rem")
                            .sortable()
                            .header_render(|| rsx! { "Age " small { "(years)" } }),
                    ],
                }
            }
        }
    }

    let body = body(&render(app));

    assert_eq!(body.matches("data-sortable").count(), 1, "{body}");
    assert!(
        body.contains("style=\"box-sizing:border-box;width:12rem;min-width:4rem;\""),
        "{body}"
    );
    assert!(
        body.contains("style=\"box-sizing:border-box;min-width:2rem;\""),
        "{body}"
    );
    assert!(body.contains("<small>(years)</small>"), "{body}");
}
