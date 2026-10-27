//! What the table says and keeps across a host's changes: sort news only for the
//! reader's own accepted picks, a removed column's sort dropped, keyed cells and the page.

use std::cell::{Cell, RefCell};

use crate::common::body;
use crate::dispatch::Page;
use crate::table_fixture::{Person, people};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Column, ColumnFilter, FilterOperator, SortDirection, Table, TableSort, column},
};

thread_local! {
    static SORT: Cell<Option<Signal<Vec<TableSort>>>> = const { Cell::new(None) };
    static ACCEPTS: Cell<bool> = const { Cell::new(true) };
    static COLUMNS: Cell<Option<Signal<Vec<&'static str>>>> = const { Cell::new(None) };
    static FILTERS: Cell<Option<Signal<Vec<ColumnFilter>>>> = const { Cell::new(None) };
    static PAGES: RefCell<Vec<u32>> = const { RefCell::new(Vec::new()) };
}

fn age_sorted(direction: SortDirection) -> Vec<TableSort> {
    vec![TableSort::new("Age", direction)]
}

fn columns(names: &[&'static str]) -> Vec<Column<Person>> {
    names
        .iter()
        .map(|&name| {
            let column = match name {
                "Age" => column("Age").value(|row: &Person| row.age).sortable(),
                "Name" => column("Name").value(|row: &Person| row.name.to_string()),
                other => column(other).value(|_: &Person| String::new()),
            };
            // Email's the only filter field.
            column.filterable(name == "Email")
        })
        .collect()
}

/// A host that holds the sort and accepts a pick only while `ACCEPTS` holds.
fn held_sort() -> Element {
    let mut sort = use_signal(Vec::new);
    use_hook(|| SORT.set(Some(sort)));
    rsx! {
        LiberoProvider {
            Table {
                aria_label: "People",
                data: people(),
                columns: columns(&["Name", "Age"]),
                sort: sort(),
                onsortchange: move |next| {
                    if ACCEPTS.get() {
                        sort.set(next);
                    }
                },
            }
        }
    }
}

fn host_sort(page: &mut Page, next: Vec<TableSort>) {
    let sort = SORT.get().expect("the host's sort");
    page.dom.in_runtime(|| sort.clone().set(next));
    settle(page);
}

fn settle(page: &mut Page) {
    for _ in 0..3 {
        page.dom.process_events();
        page.render();
    }
}

fn html(page: &Page) -> String {
    body(&dioxus_ssr::render(&page.dom))
}

/// Age's sort button: the one sortable column's.
fn click_sort(page: &mut Page) {
    let [button] = page.rec.registered_for("click")[..] else {
        panic!(
            "one sort button, got {:?}",
            page.rec.registered_for("click")
        );
    };
    page.click(button);
    settle(page);
}

/// 4408: an accepted pick is said; a sort the host sets is not.
#[test]
fn an_accepted_pick_is_said_and_a_host_sort_is_not() {
    ACCEPTS.set(true);
    let mut page = Page::mount(held_sort);
    host_sort(&mut page, age_sorted(SortDirection::Descending));
    assert!(!html(&page).contains("Sorted by"), "{}", html(&page));

    host_sort(&mut page, Vec::new());
    click_sort(&mut page);
    assert!(
        html(&page).contains("Sorted by Age, ascending"),
        "{}",
        html(&page)
    );
}

/// 4416: a pick the host refused is never said, not even when the host sets that sort later.
#[test]
fn a_refused_pick_is_never_said() {
    ACCEPTS.set(false);
    let mut page = Page::mount(held_sort);
    click_sort(&mut page);
    assert!(!html(&page).contains("Sorted by"), "{}", html(&page));

    host_sort(&mut page, age_sorted(SortDirection::Ascending));
    let html = html(&page);
    assert!(html.contains("aria-sort=\"ascending\""), "{html}");
    assert!(!html.contains("Sorted by"), "{html}");
}

/// A host whose columns come from `COLUMNS`, sorted by Age from the start.
fn held_columns() -> Element {
    let names = use_signal(|| vec!["Name", "Age"]);
    use_hook(|| COLUMNS.set(Some(names)));
    rsx! {
        LiberoProvider {
            Table {
                aria_label: "People",
                data: people(),
                columns: columns(&names()),
                default_sort: age_sorted(SortDirection::Descending),
                header_filters: true,
            }
        }
    }
}

fn host_columns(page: &mut Page, next: Vec<&'static str>) {
    let names = COLUMNS.get().expect("the host's columns");
    page.dom.in_runtime(|| names.clone().set(next));
    settle(page);
}

/// 4411: a removed column's sort is dropped without a word, so it does not return with it.
#[test]
fn a_removed_column_drops_its_sort_silently() {
    let mut page = Page::mount(held_columns);
    assert!(html(&page).contains("aria-sort=\"descending\""));

    host_columns(&mut page, vec!["Name"]);
    host_columns(&mut page, vec!["Name", "Age"]);
    let html = html(&page);
    assert!(!html.contains("aria-sort"), "{html}");
    assert!(!html.contains("no longer sorted"), "{html}");
}

/// 4410, 4414: a column the host inserts mounts its own header and filter cells only.
#[test]
fn an_inserted_column_remounts_no_other_cell() {
    let mut page = Page::mount(held_columns);
    host_columns(&mut page, vec!["Name", "Age", "Email"]);
    page.rec = Default::default();
    host_columns(&mut page, vec!["Phone", "Name", "Age", "Email"]);

    // Age's sort button and Email's filter field stay mounted: no listener is set again.
    assert!(
        page.rec.registered_for("click").is_empty(),
        "{:?}",
        page.rec.registered
    );
    assert!(
        page.rec.registered_for("input").is_empty(),
        "{:?}",
        page.rec.registered
    );
}

/// A host holding the page, one row a page, and the column filters.
fn held_page() -> Element {
    let mut page = use_signal(|| 3);
    let filters = use_signal(Vec::new);
    use_hook(|| FILTERS.set(Some(filters)));
    rsx! {
        LiberoProvider {
            Table {
                aria_label: "People",
                data: people(),
                columns: columns(&["Name", "Age"]),
                page: page(),
                onpagechange: move |next| {
                    PAGES.with_borrow_mut(|pages| pages.push(next));
                    page.set(next);
                },
                page_size: 1,
                onpagesizechange: |_| {},
                column_filters: filters(),
                oncolumnfilterschange: |_| {},
            }
        }
    }
}

/// 4415: a filter from page 3 to two pages lands on page 1, not on the old page clamped.
#[test]
fn a_filter_past_the_page_goes_back_to_page_one() {
    let mut page = Page::mount(held_page);
    assert!(html(&page).contains(">Linus<"), "{}", html(&page));

    let filters = FILTERS.get().expect("the host's filters");
    let names = ColumnFilter::new("Name", FilterOperator::Contains, "a");
    page.dom.in_runtime(|| filters.clone().set(vec![names]));
    settle(&mut page);
    let html = html(&page);
    assert!(
        html.contains(">Ada<") && !html.contains(">Grace<"),
        "{html}"
    );
    assert_eq!(PAGES.with_borrow(Vec::clone), [1]);
}

/// A host that holds the open groups and keeps a toggle only while `ACCEPTS` holds.
fn held_groups() -> Element {
    let mut open = use_signal(Vec::<String>::new);
    rsx! {
        LiberoProvider {
            Table {
                aria_label: "People",
                data: people(),
                columns: vec![
                    column("Name").value(|row: &Person| row.name.to_string()),
                    column("Age").value(|row: &Person| row.age),
                ],
                default_row_grouping: vec!["Name".to_string()],
                expanded_groups: open(),
                onexpandedgroupschange: move |next| {
                    if ACCEPTS.get() {
                        open.set(next);
                    }
                },
            }
        }
    }
}

/// 4445: a group toggle is said once the host keeps it, and not when it refuses.
#[test]
fn a_group_toggle_is_said_only_once_kept() {
    for accepts in [false, true] {
        ACCEPTS.set(accepts);
        let mut page = Page::mount(held_groups);
        let toggles = page.rec.registered_for("click");
        page.click(toggles[0]);
        settle(&mut page);
        let html = html(&page);
        assert_eq!(html.contains("Ada expanded"), accepts, "{html}");
    }
}

/// 4442: a plain click that clears a multi-sort is said by the clicked column.
#[test]
fn a_cleared_multi_sort_names_the_clicked_column() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    data: people(),
                    columns: crate::table_fixture::person_columns(),
                    multi_sort: true,
                    default_sort: vec![
                        TableSort::new("Age", SortDirection::Ascending),
                        TableSort::new("Name", SortDirection::Descending),
                    ],
                }
            }
        }
    }
    let mut page = Page::mount(app);
    // Name's sort button comes first.
    let buttons = page.rec.registered_for("click");
    page.click(buttons[0]);
    settle(&mut page);
    let html = html(&page);
    assert!(!html.contains("aria-sort"), "{html}");
    assert!(html.contains("Name no longer sorted"), "{html}");
    assert!(!html.contains("Age no longer sorted"), "{html}");
}

/// 4444: with a hidden column, the filter row's cells carry the header cells' `aria-colindex`.
#[test]
fn the_filter_row_skips_a_hidden_columns_index() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    data: people(),
                    columns: columns(&["Name", "Phone", "Email"]),
                    header_filters: true,
                    cell_focus: true,
                    default_hidden_columns: vec!["Phone".to_string()],
                }
            }
        }
    }
    let page = Page::mount(app);
    let html = html(&page);
    let filters = &html[html.find("data-filters").expect("a filters row")..];
    let filters = &filters[..filters.find("</tr>").unwrap()];
    assert!(filters.contains("aria-colindex=\"3\""), "{filters}");
    assert!(!filters.contains("aria-colindex=\"2\""), "{filters}");
}
