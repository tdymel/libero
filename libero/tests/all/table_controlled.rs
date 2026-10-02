//! Controlled state props: each one is wired to its own slice, beats its
//! `default_*`, and is followed when the caller changes it after mount.

use crate::common::{body, render, style_of};
use crate::table_fixture::{Person, cell_of, order_of, people, person_columns, table_state};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        ColumnFilter, ColumnWidths, FilterLogic, FilterOperator, PinnedColumns, SortDirection,
        Table, TableSort,
    },
    theme::Size,
};

const NAMES: [&str; 3] = ["Ada", "Grace", "Linus"];

/// What the caller holds for the four props a user changes most.
#[derive(Clone, Copy)]
struct Held {
    sort: Signal<Vec<TableSort>>,
    selection: Signal<Vec<String>>,
    expanded: Signal<Vec<String>>,
    hidden: Signal<Vec<String>>,
}

fn held() -> Element {
    let held = use_context_provider(|| Held {
        sort: Signal::new(Vec::new()),
        selection: Signal::new(Vec::new()),
        expanded: Signal::new(Vec::new()),
        hidden: Signal::new(Vec::new()),
    });

    rsx! {
        LiberoProvider {
            Table {
                aria_label: "People",
                data: people(),
                columns: person_columns(),
                row_key: |row: &Person| row.name.to_string(),
                selectable: true,
                row_detail: |row: &Person| Some(rsx! { p { "{row.name} is {row.age}" } }),
                sort: (held.sort)(),
                onsortchange: |_| {},
                selection: (held.selection)(),
                onselectionchange: |_| {},
                expanded: (held.expanded)(),
                onexpandedchange: |_| {},
                hidden_columns: (held.hidden)(),
                onhiddencolumnschange: |_| {},
            }
        }
    }
}

/// The names of the selected rows, in row order.
fn selected(body: &str) -> Vec<&'static str> {
    NAMES
        .into_iter()
        .filter(|name| {
            body.contains(&format!(r#"aria-label="Select {name}""#))
                && cell_of(body, "tr", name)["aria-selected"] == "true"
        })
        .collect()
}

/// Applies `change` and renders once, as a caller's state change would.
fn step(dom: &mut VirtualDom, change: impl FnOnce()) -> String {
    dom.in_runtime(change);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    body(&dioxus_ssr::render(dom))
}

#[test]
fn controlled_props_follow_the_caller_after_mount() {
    let mut dom = VirtualDom::new(held);
    dom.rebuild_in_place();
    let Held {
        mut sort,
        mut selection,
        mut expanded,
        mut hidden,
    } = dom.in_scope(ScopeId::APP, consume_context::<Held>);

    let html = step(&mut dom, || {});
    assert_eq!(order_of(&html, &NAMES), NAMES, "{html}");
    assert!(selected(&html).is_empty(), "{html}");

    let html = step(&mut dom, || {
        sort.set(vec![TableSort::new("Age", SortDirection::Descending)])
    });
    assert_eq!(order_of(&html, &NAMES), ["Grace", "Ada", "Linus"], "sort");

    let html = step(&mut dom, || selection.set(vec!["Linus".into()]));
    assert_eq!(selected(&html), ["Linus"], "selection: {html}");

    let html = step(&mut dom, || expanded.set(vec!["Ada".into()]));
    assert!(html.contains("Ada is 36"), "expanded: {html}");
    assert!(!html.contains("Grace is 45"), "expanded: {html}");

    let html = step(&mut dom, || hidden.set(vec!["Age".into()]));
    assert!(!html.contains(">45<"), "hidden_columns: {html}");

    // And back: each one follows the caller both ways.
    let html = step(&mut dom, || {
        sort.set(Vec::new());
        selection.set(Vec::new());
        expanded.set(Vec::new());
        hidden.set(Vec::new());
    });
    assert_eq!(order_of(&html, &NAMES), NAMES, "{html}");
    assert!(selected(&html).is_empty(), "{html}");
    assert!(!html.contains("Ada is 36"), "{html}");
    assert!(html.contains(">45<"), "{html}");
}

// A macro: `render` takes a fn pointer, which captures nothing.
macro_rules! people_html {
    ($($prop:ident: $value:expr),* $(,)?) => {
        body(&render(|| {
            rsx! {
                LiberoProvider {
                    Table {
                        aria_label: "People",
                        data: people(),
                        columns: person_columns(),
                        $($prop: $value,)*
                    }
                }
            }
        }))
    };
}

/// Each prop's controlled value is neither its default's nor the unset
/// state's, so a prop wired to the wrong slice, or not at all, fails.
#[test]
fn a_controlled_prop_beats_its_default() {
    use FilterOperator::StartsWith;

    let html = people_html!(
        column_order: vec!["Age".into(), "Name".into()],
        default_column_order: vec!["Name".into(), "Age".into()],
        oncolumnorderchange: |_| {},
    );
    assert_eq!(
        order_of(&html, &["Name", "Age"]),
        ["Age", "Name"],
        "column_order"
    );

    let html = people_html!(
        column_filters: vec![ColumnFilter::new("Name", StartsWith, "G")],
        default_column_filters: vec![ColumnFilter::new("Name", StartsWith, "A")],
        oncolumnfilterschange: |_| {},
    );
    assert_eq!(order_of(&html, &NAMES), ["Grace"], "column_filters");

    let html = people_html!(
        default_column_filters: vec![
            ColumnFilter::new("Name", StartsWith, "A"),
            ColumnFilter::new("Name", StartsWith, "G"),
        ],
        filter_logic: FilterLogic::Or,
        default_filter_logic: FilterLogic::And,
        onfilterlogicchange: |_| {},
    );
    assert_eq!(order_of(&html, &NAMES), ["Ada", "Grace"], "filter_logic");

    let html = people_html!(
        pinned_columns: PinnedColumns::default().start(["Age"]),
        default_pinned_columns: PinnedColumns::default().start(["Name"]),
        onpinnedcolumnschange: |_| {},
    );
    assert_eq!(
        cell_of(&html, "th", "Age")["data-pin"],
        "start",
        "pinned_columns"
    );
    assert!(
        !cell_of(&html, "th", "Ada").contains_key("data-pin"),
        "{html}"
    );

    let html = people_html!(
        column_widths: ColumnWidths::from([("Name".to_string(), 240.0)]),
        default_column_widths: ColumnWidths::from([("Name".to_string(), 100.0)]),
        oncolumnwidthschange: |_| {},
    );
    assert_eq!(
        style_of(&cell_of(&html, "th", "Name"))["width"],
        "240px",
        "column_widths"
    );

    let html = people_html!(
        density: Size::Lg,
        default_density: Size::Sm,
        ondensitychange: |_| {},
    );
    assert!(
        table_state(&html).contains(&"size-lg".to_string()),
        "density: {html}"
    );

    let html = people_html!(
        quick_filter: "lin",
        default_quick_filter: "ada",
        onquickfilterchange: |_| {},
    );
    assert_eq!(order_of(&html, &NAMES), ["Linus"], "quick_filter");
}

thread_local! {
    static CELLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// The cell bodies one caller change builds, counted from zero.
fn cells_built(dom: &mut VirtualDom, change: impl FnOnce()) -> usize {
    CELLS.with(|cells| cells.set(0));
    step(dom, change);
    CELLS.with(std::cell::Cell::get)
}

/// A render-count budget (todo 1981): a selection, a detail toggle or a sort redraws no
/// cell, a column change redraws them all. The first test checks what they show.
#[test]
fn a_row_state_change_redraws_no_cell() {
    fn app() -> Element {
        let held = use_context_provider(|| Held {
            sort: Signal::new(Vec::new()),
            selection: Signal::new(Vec::new()),
            expanded: Signal::new(Vec::new()),
            hidden: Signal::new(Vec::new()),
        });
        let mut columns = person_columns();
        columns[0] = columns[0].clone().render(|row: &Person| {
            CELLS.with(|cells| cells.set(cells.get() + 1));
            rsx! { "{row.name}" }
        });
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    data: people(),
                    columns,
                    row_key: |row: &Person| row.name.to_string(),
                    selectable: true,
                    row_detail: |row: &Person| Some(rsx! { p { "{row.name} is {row.age}" } }),
                    sort: (held.sort)(),
                    selection: (held.selection)(),
                    expanded: (held.expanded)(),
                    hidden_columns: (held.hidden)(),
                }
            }
        }
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    let Held {
        mut sort,
        mut selection,
        mut expanded,
        mut hidden,
    } = dom.in_scope(ScopeId::APP, consume_context::<Held>);

    assert_eq!(
        cells_built(&mut dom, || selection.set(vec!["Linus".into()])),
        0
    );
    assert_eq!(
        cells_built(&mut dom, || expanded.set(vec!["Ada".into()])),
        0
    );
    let by_age = vec![TableSort::new("Age", SortDirection::Descending)];
    assert_eq!(cells_built(&mut dom, || sort.set(by_age)), 0);
    assert_eq!(cells_built(&mut dom, || hidden.set(vec!["Age".into()])), 3);
}

/// A memoized row still follows a signal its column reads (todo 1981).
#[test]
fn a_signal_a_cell_reads_redraws_every_row() {
    fn app() -> Element {
        let unit = use_context_provider(|| Signal::new("years"));
        let mut columns = person_columns();
        columns[1] = columns[1]
            .clone()
            .render(move |row: &Person| rsx! { "{row.age} {unit}" });
        rsx! {
            LiberoProvider {
                Table { aria_label: "People", data: people(), columns }
            }
        }
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    let mut unit = dom.in_scope(ScopeId::APP, consume_context::<Signal<&str>>);
    let html = step(&mut dom, || unit.set("yrs"));
    assert_eq!(html.matches(" yrs").count(), 3, "{html}");
    assert!(!html.contains("years"), "{html}");
}
