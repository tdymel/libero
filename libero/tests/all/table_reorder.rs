use std::collections::BTreeMap;

use crate::common::{body, render, style_of, tag_with, tags_with};
use crate::table_fixture::{Person, people};

use dioxus::{
    core::{Mutation, Mutations},
    prelude::*,
};
use libero::{
    LiberoProvider,
    components::{
        Column, ColumnFilter, FilterOperator, PinnedColumns, SortDirection, Table, TableSort,
        column,
    },
};

fn columns() -> Vec<Column<Person>> {
    vec![
        column("Name")
            .value(|row: &Person| row.name.to_string())
            .sortable()
            .row_header(),
        column("Age").value(|row: &Person| row.age).sortable(),
        column("Initial").value(|row: &Person| row.name[..1].to_string()),
    ]
}

/// The sort names its column by header text, so it follows the column (4c).
#[test]
fn a_column_order_moves_headers_and_cells_and_the_sort_follows() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    data: people(),
                    columns: columns(),
                    default_column_order: vec!["Age".to_string(), "Initial".to_string()],
                    default_sort: vec![TableSort::new("Name", SortDirection::Descending)],
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let at = |text: &str| body.find(text).unwrap_or_else(|| panic!("{text}: {body}"));

    assert!(at("Age<") < at(">Initial<") && at(">Initial<") < at("Name<"));
    // The sorted header is still Name's.
    let sorted = at("aria-sort=\"descending\"");
    assert!(sorted > at(">Initial<") && sorted < at("Name<"));
    // Cells follow their header; Linus leads, sorted by name descending.
    assert!(at(">28<") < at(">L<") && at(">L<") < at(">Linus<"));
    assert!(at(">Linus<") < at(">Grace<"));
}

#[test]
fn row_reorder_adds_a_handle_column_with_move_buttons() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    data: people(),
                    columns: columns(),
                    row_key: |row: &Person| row.name.to_string(),
                    onrowreorder: |_| {},
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert!(body.contains(">Reorder<"), "{body}");
    assert_eq!(body.matches("data-reorder-handle").count(), 3);
    assert_eq!(body.matches("data-reorder-move=\"up\"").count(), 3);
    assert!(body.contains("Move Ada up"), "{body}");
    // Nothing is mid-drag, so no row is moved.
    let moved: Vec<_> = tags_with(&body, "style=")
        .into_iter()
        .filter(|tag| style_of(tag).contains_key("transform"))
        .collect();
    assert!(moved.is_empty(), "{moved:?}");
    // The first row has nothing above. "Down" waits for the rows to mount and count.
    assert!(tag_with(&body, r#"aria-label="Move Ada up""#).contains_key("disabled"));
    assert!(!tag_with(&body, r#"aria-label="Move Grace up""#).contains_key("disabled"));
}

/// The `onmounted` listeners a first render of `app` creates.
fn mounted_listeners(app: fn() -> Element) -> usize {
    let mut edits = Mutations::default();
    VirtualDom::new(app).rebuild(&mut edits);
    edits
        .edits
        .iter()
        .filter(|edit| matches!(edit, Mutation::NewEventListener { name, .. } if name == "mounted"))
        .count()
}

/// A WebView reports each `onmounted` back in a blocking round trip: a phone froze
/// on 4 per row while rows scrolled in (todo 2584). The rows now add none.
#[test]
fn reorder_rows_add_no_mounted_listeners() {
    fn plain() -> Element {
        rsx! {
            LiberoProvider {
                Table { aria_label: "People", data: people(), columns: columns(),
                    row_key: |row: &Person| row.name.to_string(),
                }
            }
        }
    }
    fn reordered() -> Element {
        rsx! {
            LiberoProvider {
                Table { aria_label: "People", data: people(), columns: columns(),
                    row_key: |row: &Person| row.name.to_string(),
                    onrowreorder: |_| {},
                }
            }
        }
    }

    let (plain, reordered) = (mounted_listeners(plain), mounted_listeners(reordered));
    // The list's own `tbody` only, not one per row of the 3.
    assert!(
        reordered <= plain + 1,
        "{plain} without reorder, {reordered} with"
    );
}

/// Every reorder control: each row's handle and its up and down moves.
fn reorder_controls(body: &str) -> Vec<BTreeMap<String, String>> {
    tags_with(body, "data-reorder-")
        .into_iter()
        .filter(|tag| {
            tag.contains_key("data-reorder-handle") || tag.contains_key("data-reorder-move")
        })
        .collect()
}

/// Whether each reorder control is off, in document order. Off is `aria-disabled`
/// described by the reason, still a Tab stop (1430).
fn disabled_controls(body: &str) -> Vec<bool> {
    let controls = reorder_controls(body);
    controls
        .iter()
        .map(|control| {
            let off = control.get("aria-disabled").map(String::as_str) == Some("true");
            if off {
                assert!(!control.contains_key("disabled"), "{control:?}");
                let reason = tag_with(body, &format!(r#"id="{}""#, control["aria-describedby"]));
                assert!(reason.contains_key("hidden"), "{reason:?}");
            }
            off
        })
        .collect()
}

/// The shown order is a view over `data` while sorted: nothing moves (4b).
#[test]
fn row_reorder_is_off_while_sorted() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    data: people(),
                    columns: columns(),
                    row_key: |row: &Person| row.name.to_string(),
                    default_sort: vec![TableSort::new("Age", SortDirection::Ascending)],
                    onrowreorder: |_| {},
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert_eq!(disabled_controls(&body), [true; 9], "{body}");
    assert!(
        html.contains(">Clear the sort and filters to reorder rows<"),
        "{html}"
    );
}

/// A column filter hides rows from the slots: nothing moves (1396).
#[test]
fn row_reorder_is_off_under_a_column_filter() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    data: people(),
                    columns: columns(),
                    row_key: |row: &Person| row.name.to_string(),
                    default_column_filters: vec![ColumnFilter::new("Age", FilterOperator::GreaterThan, "30")],
                    onrowreorder: |_| {},
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    // Two rows left, three controls each.
    assert!(!body.contains(">Linus<"), "{body}");
    assert_eq!(disabled_controls(&body), [true; 6], "{body}");
}

/// 1395: a column menu brings a pointer-only drag grip to each unpinned header.
#[test]
fn a_column_menu_puts_a_hidden_drag_grip_in_each_unpinned_header() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    column_menu: true,
                    default_pinned_columns: PinnedColumns::default().start(["Name"]),
                    data: people(),
                    columns: columns(),
                }
            }
        }
    }

    let html = render(app);
    let head = body(&html);
    let head = &head[head.find("<thead").unwrap()..head.find("</thead>").unwrap()];

    assert_eq!(head.matches("data-draggable=true").count(), 2, "{head}");
    let handles = tags_with(head, "data-drag-handle=true");
    assert_eq!(handles.len(), 2, "{head}");
    for handle in &handles {
        assert_eq!(handle["aria-hidden"], "true", "{handle:?}");
    }
    assert!(!head.contains("tabindex"), "{head}");
}

#[test]
fn without_a_column_menu_no_header_drags() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table { aria_label: "People", data: people(), columns: columns() }
            }
        }
    }

    assert!(!render(app).contains("data-drag-handle=true"));
}

/// Todo 2584: a column move carries each cell's body along. Unkeyed, the cells were
/// redrawn in place, so every row remounted the bodies that changed places.
#[test]
fn a_column_move_keeps_the_cell_bodies_mounted() {
    thread_local! {
        static MOUNTS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    }

    #[component]
    fn Badge(text: String) -> Element {
        use_hook(|| MOUNTS.set(MOUNTS.get() + 1));
        rsx! { "{text}" }
    }

    fn app() -> Element {
        let order =
            use_context_provider(|| Signal::new(vec!["Name".to_string(), "Age".to_string()]));
        let columns = vec![
            column("Name")
                .value(|row: &Person| row.name.to_string())
                .row_header(),
            column("Age").value(|row: &Person| row.age),
            column("Badge")
                .value(|row: &Person| row.name[..1].to_string())
                .render(|row: &Person| rsx! { Badge { text: row.name[..1].to_string() } }),
        ];
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    data: people(),
                    columns,
                    column_order: order(),
                }
            }
        }
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    let rows = people().len();
    assert_eq!(MOUNTS.get(), rows);

    let mut order = dom.in_scope(ScopeId::APP, consume_context::<Signal<Vec<String>>>);
    dom.in_runtime(|| order.set(vec!["Badge".to_string(), "Name".to_string()]));
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    let html = dioxus_ssr::render(&dom);
    let at = |text: &str| html.find(text).unwrap_or_else(|| panic!("{text}: {html}"));
    assert!(at(">Badge<") < at(">Name<"), "{html}");
    assert_eq!(MOUNTS.get(), rows);
}
