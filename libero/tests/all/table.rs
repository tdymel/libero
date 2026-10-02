//! Rendering, scrolling, sorting, the empty and loading states, and the
//! column menu. Selection, filters, pinning and windowing have their own
//! `table_*.rs`; the shared rows and pickers are in `table_fixture.rs`.

use std::collections::BTreeMap;

use crate::common::{attributes_of, body, css_rules_for, render, style_of, tags_with};
use crate::table_fixture::{Item, Person, Stock, region, table_rule, table_state};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{ColumnDefaults, Mark, SortDirection, States, Table, TableSort, column},
    localization::Localization,
    theme::Size,
};

#[test]
fn a_table_marks_only_sortable_headers_and_aligns_by_cell_type() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    data: vec![
                        Person { name: "Ada", age: 36 },
                        Person { name: "Grace", age: 45 },
                    ],
                    columns: vec![
                        column("Name").value(|row: &Person| row.name.to_string()),
                        column("Age").value(|row: &Person| row.age).sortable(),
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
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    data: vec![
                        Person { name: "Ada", age: 36 },
                        Person { name: "Grace", age: 45 },
                    ],
                    columns: vec![
                        column("Name").value(|row: &Person| row.name.to_string()).row_header(),
                        column("Age").value(|row: &Person| row.age),
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
    struct Person {
        score: u32,
    }

    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    data: vec![Person { score: 7 }],
                    columns: vec![
                        column("Score")
                            .value(|row: &Person| row.score)
                            .render(|row: &Person| rsx! { Mark { "{row.score} pts" } }),
                    ],
                }
            }
        }
    }

    let body = body(&render(app));

    assert!(body.contains("7 pts"));
    assert!(body.contains("<mark"));
}

#[test]
fn a_scrolling_table_sits_in_a_scroll_area_that_is_no_stop_until_it_overflows() {
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
    let region = region(&html);

    assert!(body(&html).contains(">Stock</caption>"));
    // Measured once mounted: the e2e covers the named stop of an overflowing one.
    assert_eq!(region["tabindex"], "-1");
    assert!(!region.contains_key("role"), "{region:?}");
    assert!(!body(&html).contains("sticky-header"));
}

#[test]
fn max_height_bounds_the_area_and_sticks_the_header() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Stock",
                    max_height: "240px",
                    data: vec![Item { name: "Apple" }],
                    columns: vec![column("Name").value(|item: &Item| item.name.to_string())],
                }
            }
        }
    }

    let html = render(app);
    let region = region(&html);

    assert!(region["data-state"].contains("axis-both"), "{region:?}");
    let bounds: Vec<_> = css_rules_for(&html, &region)
        .into_iter()
        .filter_map(|rule| rule.declarations.get("max-height").cloned())
        .collect();
    assert_eq!(bounds, ["240px"], "{region:?}");
    assert!(
        table_state(&html).contains(&"sticky-header".to_string()),
        "{html}"
    );
    let header = table_rule(
        &html,
        r#"[data-state~="sticky-header"] thead th:not([data-group])"#,
    );
    assert_eq!(header["position"], "sticky", "{header:?}");
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

fn loading_table(data: Vec<Item>, page_size: Option<usize>) -> Element {
    rsx! {
        LiberoProvider {
            Table {
                aria_label: "Stock",
                loading: true,
                page_size,
                data,
                columns: vec![
                    column("Name").value(|item: &Item| item.name.to_string()),
                    column("Also").value(|item: &Item| item.name.to_string()),
                ],
            }
        }
    }
}

#[test]
fn a_loading_table_without_rows_shows_hidden_skeleton_rows_and_is_busy() {
    let html = render(|| loading_table(Vec::new(), None));
    let rows = body(&html);
    assert_eq!(rows.matches("data-skeleton").count(), 5, "{rows}");
    assert_eq!(rows.matches("<td").count(), 10);
    assert!(!rows.contains("data-empty"));
    assert!(!rows.contains("data-loading-bar"));
    assert_eq!(attributes_of(&html, "table")["aria-busy"], "true");

    // Paged: a page of placeholders.
    let paged = body(&render(|| loading_table(Vec::new(), Some(3))));
    assert_eq!(paged.matches("data-skeleton").count(), 3);
}

#[test]
fn a_loading_table_with_rows_keeps_them_under_a_named_bar() {
    let html = render(|| loading_table(vec![Item { name: "Apple" }], None));
    let body = body(&html);
    assert!(body.contains("Apple"));
    assert!(!body.contains("data-skeleton"));
    assert!(body.contains("data-loading-bar"));
    assert!(body.contains("aria-label=\"Loading rows\""), "{body}");
    // The rows stay usable, so the table is not busy.
    assert!(!attributes_of(&html, "table").contains_key("aria-busy"));
}

#[test]
fn the_toolbar_holds_the_callers_controls_then_the_quick_filter() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Stock",
                    show_quick_filter: true,
                    toolbar: rsx! { button { "Export" } },
                    data: vec![Item { name: "Apple" }],
                    columns: vec![column("Name").value(|item: &Item| item.name.to_string())],
                }
            }
        }
    }

    let body = body(&render(app));
    let toolbar = body.find("data-toolbar=").expect(&body);
    let export = body.find("Export").unwrap();
    let search = body.find("data-toolbar-end").unwrap();
    assert!(toolbar < export && export < search && search < body.find("<table").unwrap());
    assert!(!body.contains("role=\"toolbar\""));
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
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    column_defaults: ColumnDefaults::new().min_width("4rem"),
                    data: vec![Person { name: "Ada", age: 36 }],
                    columns: vec![
                        column("Name").value(|row: &Person| row.name.to_string()).width("12rem"),
                        column("Age")
                            .value(|row: &Person| row.age)
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
    let sized: Vec<_> = tags_with(&body, "<th").iter().map(style_of).collect();
    let style = |pairs: &[(&str, &str)]| {
        pairs
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect::<BTreeMap<_, _>>()
    };
    assert_eq!(
        sized,
        [
            style(&[
                ("box-sizing", "border-box"),
                ("width", "12rem"),
                ("min-width", "4rem")
            ]),
            style(&[("box-sizing", "border-box"), ("min-width", "2rem")]),
        ],
        "{body}"
    );
    assert!(body.contains("<small>(years)</small>"), "{body}");
}

#[test]
fn a_hidden_column_draws_no_cells_but_keeps_its_sort() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    default_hidden_columns: vec!["Age".to_string()],
                    default_sort: vec![TableSort::new("Age", SortDirection::Descending)],
                    data: vec![
                        Person { name: "Ada", age: 36 },
                        Person { name: "Grace", age: 45 },
                    ],
                    columns: vec![
                        column("Name").value(|row: &Person| row.name.to_string()),
                        column("Age").value(|row: &Person| row.age).sortable(),
                    ],
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert_eq!(body.matches("<th scope=\"col\"").count(), 1, "{body}");
    assert_eq!(body.matches("<td").count(), 2, "{body}");
    assert!(!body.contains("36"), "{body}");
    assert!(
        body.find("Grace").unwrap() < body.find("Ada").unwrap(),
        "{body}"
    );
}

#[test]
fn a_column_menu_puts_a_named_menu_button_in_each_shown_header() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    column_menu: true,
                    data: vec![Person { name: "Ada", age: 36 }],
                    columns: vec![
                        column("Name").value(|row: &Person| row.name.to_string()),
                        column("Age").value(|row: &Person| row.age).sortable(),
                    ],
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert_eq!(body.matches("data-column-menu").count(), 2, "{body}");
    assert!(
        body.contains("aria-label=\"Name column options\""),
        "{body}"
    );
    assert!(body.contains("aria-label=\"Age column options\""), "{body}");
    assert!(body.contains("aria-haspopup=\"menu\""), "{body}");
    assert_eq!(body.matches("data-menu").count(), 2, "{body}");
    // The header's name stays its text, without the menu button's label.
    assert!(body.contains("aria-label=\"Age\""), "{body}");
    // One sort button, beside its menu button.
    assert_eq!(body.matches("data-sort-button").count(), 1, "{body}");
    // The end-aligned number column puts its menu first in the DOM, so Tab
    // follows the visual order (todo 1261); the start-aligned one after its text.
    let age = &body[body.find("aria-label=\"Age\"").unwrap()..];
    assert!(
        age.find("data-column-menu") < age.find("data-sort-button"),
        "{age}"
    );
    let name = &body[body.find("aria-label=\"Name\"").unwrap()..];
    assert!(
        name.find("data-header-text") < name.find("data-column-menu"),
        "{name}"
    );
}

/// Events dispatched the way a renderer does, through [`crate::dispatch`].
mod dispatched {
    use crate::common::body;
    use crate::dispatch::*;
    use dioxus::prelude::*;
    use libero::{
        LiberoProvider,
        components::{Table, column},
    };

    /// Todo 428: the sort stays on its column when the columns move.
    #[test]
    fn a_table_sort_follows_its_column_through_a_reorder() {
        #[derive(Clone, PartialEq)]
        struct Person {
            name: &'static str,
            age: u32,
        }

        fn app() -> Element {
            let layout = use_context_provider(|| Signal::new(vec!["Name", "Age"]));
            let columns = layout()
                .into_iter()
                .map(|header| match header {
                    "Name" => column("Name")
                        .value(|p: &Person| p.name.to_string())
                        .sortable(),
                    _ => column("Age").value(|p: &Person| p.age).sortable(),
                })
                .collect::<Vec<_>>();

            rsx! {
                LiberoProvider {
                    Table {
                        data: vec![
                            Person { name: "Grace", age: 45 },
                            Person { name: "Linus", age: 28 },
                            Person { name: "Ada", age: 36 },
                        ],
                        columns,
                    }
                }
            }
        }

        fn row_order(html: &str) -> Vec<&'static str> {
            let mut names = ["Grace", "Linus", "Ada"];
            names.sort_by_key(|name| html.find(&format!(">{name}<")).expect("a missing row"));
            names.to_vec()
        }

        dioxus::html::set_event_converter(Box::new(TestConverter));
        let mut dom = VirtualDom::new(app);
        let mut find = FindClickListener::default();
        dom.rebuild(&mut find);
        let age = find.element("click", "text", "Age");

        dom.runtime()
            .handle_event("click", Event::new(click_event(), true), age);
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        assert_eq!(
            row_order(&dioxus_ssr::render(&dom)),
            ["Linus", "Ada", "Grace"]
        );

        let mut layout = dom.in_scope(ScopeId::APP, consume_context::<Signal<Vec<&str>>>);
        dom.in_runtime(|| layout.set(vec!["Age", "Name"]));
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        let html = dioxus_ssr::render(&dom);
        assert_eq!(row_order(&html), ["Linus", "Ada", "Grace"]);
        assert!(
            html.contains(
                r#"aria-sort="ascending"><button type="button" data-sort-button=true>Age"#
            ),
            "{html}"
        );

        // Without its column the sort is gone, not moved onto "Name".
        dom.in_runtime(|| layout.set(vec!["Name"]));
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        let html = dioxus_ssr::render(&dom);
        assert_eq!(row_order(&html), ["Grace", "Linus", "Ada"]);
        assert!(
            html.contains(r#"<button type="button" data-sort-button=true>Name"#),
            "{html}"
        );
        assert!(!body(&html).contains("aria-sort"), "{html}");
    }
}
