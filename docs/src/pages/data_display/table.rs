use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, prop, props};
use dioxus::prelude::*;
use std::cell::OnceCell;
use std::rc::Rc;
use std::time::Duration;

use libero::chrono::{NaiveDate, TimeDelta};
use libero::components::{
    Button, Chip, Code, Column, ColumnWidths, PinnedColumns, RowFn, Table, TableColumnsButton,
    TableDensityButton, TableExportButton, TableFilterButton, Text, column,
};
use libero::hooks::SortableMove;
use libero::platform::{TimerSubscription, save_file, timer};
use libero::sx::sx;

#[derive(Clone, PartialEq)]
struct Person {
    name: String,
    role: String,
    joined: NaiveDate,
    review: Option<NaiveDate>,
    bonus: Option<f64>,
}

fn day(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap_or_default()
}

/// Six rows, so the date filters have dates on both sides to keep and drop.
fn people() -> Vec<Person> {
    let person = |name: &str, role: &str, joined, review, bonus| Person {
        name: name.into(),
        role: role.into(),
        joined,
        review,
        bonus,
    };
    vec![
        person(
            "Ada Lovelace",
            "Owner",
            day(2019, 3, 4),
            Some(day(2025, 1, 15)),
            Some(12.5),
        ),
        person(
            "Grace Hopper",
            "Admin",
            day(2021, 7, 19),
            Some(day(2024, 11, 2)),
            Some(8.0),
        ),
        person("Alan Turing", "Viewer", day(2023, 2, 1), None, None),
        person(
            "Katherine Johnson",
            "Admin",
            day(2020, 10, 12),
            Some(day(2025, 4, 30)),
            Some(6.5),
        ),
        person(
            "Edsger Dijkstra",
            "Viewer",
            day(2024, 5, 27),
            None,
            Some(2.0),
        ),
        person(
            "Barbara Liskov",
            "Owner",
            day(2018, 1, 8),
            Some(day(2024, 9, 9)),
            Some(10.0),
        ),
    ]
}

const ROLES: [&str; 3] = ["Owner", "Admin", "Viewer"];

/// Made-up person `n` of the crowd, as `wrap_data` prints it.
fn member(n: u32) -> Person {
    Person {
        name: format!("Person {n}"),
        role: ROLES[n as usize % ROLES.len()].into(),
        joined: day(2015, 1, 1) + TimeDelta::days(i64::from(n % 3_650)),
        review: (!n.is_multiple_of(3))
            .then(|| day(2025, 1, 1) + TimeDelta::days(i64::from(n % 365))),
        bonus: (!n.is_multiple_of(4)).then(|| f64::from(n % 150) / 10.0),
    }
}

/// Ten thousand made-up people for the `virtual_row_height` switch.
fn crowd() -> Vec<Person> {
    (1..=10_000).map(member).collect()
}

const BATCH: usize = 100;
const LATENCY: Duration = Duration::from_millis(600);

/// The `onbottomreached` switch's fake server, as `SERVER` prints it.
fn fetch(from: usize) -> Vec<Person> {
    (1..=10_000).skip(from).take(BATCH).map(member).collect()
}

/// The fake server and the state the `onbottomreached` props read, printed so
/// the snippet calls nothing it does not show.
// snippet: item use libero::chrono::{NaiveDate, TimeDelta};
// snippet: item #[derive(Clone, PartialEq)] struct Person { name: String, role: String, joined: NaiveDate, review: Option<NaiveDate>, bonus: Option<f64> }
// snippet: item fn day(year: i32, month: u32, day: u32) -> NaiveDate { NaiveDate::from_ymd_opt(year, month, day).unwrap_or_default() }
const SERVER: &str = r#"const BATCH: usize = 100;
/// How long the fake server takes.
const LATENCY: Duration = Duration::from_millis(600);

/// The fake server: `BATCH` people from `from` on, of ten thousand.
fn fetch(from: usize) -> Vec<Person> {
    (1..=10_000u32)
        .skip(from)
        .take(BATCH)
        .map(|n| Person {
            name: format!("Person {n}"),
            role: ["Owner", "Admin", "Viewer"][n as usize % 3].into(),
            joined: day(2015, 1, 1) + TimeDelta::days(i64::from(n % 3_650)),
            review: (!n.is_multiple_of(3)).then(|| day(2025, 1, 1) + TimeDelta::days(i64::from(n % 365))),
            bonus: (!n.is_multiple_of(4)).then(|| f64::from(n % 150) / 10.0),
        })
        .collect()
}

let mut loaded = use_signal(|| fetch(0));
let mut loading = use_signal(|| false);
let mut pending = use_signal(|| None::<Box<dyn TimerSubscription>>);
use_drop(move || pending.set(None));
// The next batch, appended once the fake server answers.
let mut load = move || {
    loading.set(true);
    pending.set(timer().map(|timer| {
        timer.after(
            LATENCY,
            Box::new(move || {
                let batch = fetch(loaded.peek().len());
                loaded.write().extend(batch);
                loading.set(false);
            }),
        )
    }));
};
let people = loaded();"#;

/// People a batch at a time from the fake server, with `virtual_row_height`.
fn from_server(values: &DemoValues) -> bool {
    windowed(values) && values.str("onbottomreached") == "true"
}

// snippet: item use libero::chrono::NaiveDate;
// snippet: item #[derive(Clone, PartialEq)] struct Person { name: String, role: String, joined: NaiveDate, review: Option<NaiveDate>, bonus: Option<f64> }
// snippet: let people: Vec<Person> = Vec::new();
// snippet: in Table { caption: "Team members", data: people, .. }
const COLUMNS: &str = r#"columns: vec![
    column("Name").value(|p: &Person| p.name.clone()).sortable().row_header(),
    column("Role")
        .value(|p: &Person| p.role.clone())
        .sortable()
        .render(|p: &Person| rsx! { Chip { size: "xs", "{p.role}" } }),
    column("Joined").value(|p: &Person| p.joined).sortable(),
    column("Review").value(|p: &Person| p.review).sortable(),
    column("Bonus")
        .value(|p: &Person| p.bonus)
        .format(|p: &Person| p.bonus.map(|b| format!("{b:.1} %")).unwrap_or_default())
        .sortable(),
]"#;

// snippet: item use libero::chrono::NaiveDate;
// snippet: item #[derive(Clone, PartialEq)] struct Person { name: String, role: String, joined: NaiveDate, review: Option<NaiveDate>, bonus: Option<f64> }
// snippet: let people: Vec<Person> = Vec::new();
// snippet: in Table { caption: "Team members", data: people, .. }
const GROUPED_COLUMNS: &str = r#"columns: vec![
    column("Name").value(|p: &Person| p.name.clone()).sortable().row_header(),
    column("Role")
        .value(|p: &Person| p.role.clone())
        .sortable()
        .render(|p: &Person| rsx! { Chip { size: "xs", "{p.role}" } })
        .group("Membership"),
    column("Joined").value(|p: &Person| p.joined).sortable().group("Membership"),
    column("Review").value(|p: &Person| p.review).sortable().group("Membership"),
    column("Bonus")
        .value(|p: &Person| p.bonus)
        .format(|p: &Person| p.bonus.map(|b| format!("{b:.1} %")).unwrap_or_default())
        .sortable()
        .group("Membership"),
]"#;

fn team_columns(grouped: bool) -> Vec<Column<Person>> {
    let group = |column: Column<Person>| match grouped {
        true => column.group("Membership"),
        false => column,
    };
    vec![
        column("Name")
            .value(|p: &Person| p.name.clone())
            .sortable()
            .row_header(),
        group(
            column("Role")
                .value(|p: &Person| p.role.clone())
                .sortable()
                .render(|p: &Person| rsx! { Chip { size: "xs", "{p.role}" } }),
        ),
        group(column("Joined").value(|p: &Person| p.joined).sortable()),
        group(column("Review").value(|p: &Person| p.review).sortable()),
        group(
            column("Bonus")
                .value(|p: &Person| p.bonus)
                .format(|p: &Person| p.bonus.map(|b| format!("{b:.1} %")).unwrap_or_default())
                .sortable(),
        ),
    ]
}

/// The rows are the fixture, and the snippet only compiles with them, so the
/// code block carries the struct and the data above the `Table` itself.
fn wrap_data(values: &DemoValues, code: &str) -> String {
    const ROWS: &str = r#"vec![
    Person { name: "Ada Lovelace".into(), role: "Owner".into(), joined: day(2019, 3, 4), review: Some(day(2025, 1, 15)), bonus: Some(12.5) },
    Person { name: "Grace Hopper".into(), role: "Admin".into(), joined: day(2021, 7, 19), review: Some(day(2024, 11, 2)), bonus: Some(8.0) },
    Person { name: "Alan Turing".into(), role: "Viewer".into(), joined: day(2023, 2, 1), review: None, bonus: None },
    Person { name: "Katherine Johnson".into(), role: "Admin".into(), joined: day(2020, 10, 12), review: Some(day(2025, 4, 30)), bonus: Some(6.5) },
    Person { name: "Edsger Dijkstra".into(), role: "Viewer".into(), joined: day(2024, 5, 27), review: None, bonus: Some(2.0) },
    Person { name: "Barbara Liskov".into(), role: "Owner".into(), joined: day(2018, 1, 8), review: Some(day(2024, 9, 9)), bonus: Some(10.0) },
]"#;
    const CROWD: &str = r#"(1..=10_000)
    .map(|n: u32| Person {
        name: format!("Person {n}"),
        role: ["Owner", "Admin", "Viewer"][n as usize % 3].into(),
        joined: day(2015, 1, 1) + TimeDelta::days(i64::from(n % 3_650)),
        review: (!n.is_multiple_of(3)).then(|| day(2025, 1, 1) + TimeDelta::days(i64::from(n % 365))),
        bonus: (!n.is_multiple_of(4)).then(|| f64::from(n % 150) / 10.0),
    })
    .collect::<Vec<_>>()"#;
    let rows = match windowed(values) {
        true => CROWD,
        false => ROWS,
    };
    // Reordered rows and resized widths need a signal to land in.
    let mut people = match (no_rows(values), reorders(values)) {
        _ if from_server(values) => SERVER.to_string(),
        (true, false) => "let people: Vec<Person> = Vec::new();".to_string(),
        (false, false) => format!("let people = {rows};"),
        (true, true) => {
            "let mut rows = use_signal(Vec::<Person>::new);\nlet people = rows();".to_string()
        }
        (false, true) => format!("let mut rows = use_signal(|| {rows});\nlet people = rows();"),
    };
    if resizes(values) {
        people.push_str("\nlet mut widths = use_signal(ColumnWidths::new);");
    }
    if values.str("toolbar") == "true" {
        people.push_str(
            "\n// The export's CSV, yours to save.\nlet mut csv = use_signal(String::new);",
        );
    }
    let uses = match windowed(values) {
        true => "use libero::chrono::{NaiveDate, TimeDelta};",
        false => "use libero::chrono::NaiveDate;",
    };
    format!(
        r#"{uses}

#[derive(Clone, PartialEq)]
struct Person {{
    name: String,
    role: String,
    joined: NaiveDate,
    review: Option<NaiveDate>,
    bonus: Option<f64>,
}}

fn day(year: i32, month: u32, day: u32) -> NaiveDate {{
    NaiveDate::from_ymd_opt(year, month, day).unwrap_or_default()
}}

{people}

{code}"#
    )
}

// snippet: item #[derive(Clone, PartialEq)] struct Person { name: String, role: String }
// snippet: in Table { caption: "Team members", data: Vec::<Person>::new(), columns: vec![], row_key: |p: &Person| p.name.clone(), .. }
const DETAIL: &str = r#"row_detail: |p: &Person| Some(rsx! { "{p.name} joined as {p.role}." })"#;

// snippet: item #[derive(Clone, PartialEq)] struct Person { name: String }
// snippet: in Table { caption: "Team members", data: Vec::<Person>::new(), columns: vec![], .. }
const TOOLBAR: &str = r#"toolbar: rsx! {
    TableColumnsButton {}
    TableDensityButton {}
    TableExportButton {
        onexport: move |csv: String| {
            spawn(async move {
                save_file("team.csv", "text/csv", csv.into_bytes()).await;
            });
        }
    }
    Button { variant: "outlined", size: "sm", "Add member" }
}"#;

// snippet: item #[derive(Clone, PartialEq)] struct Person { name: String }
// snippet: in Table { caption: "Team members", data: Vec::<Person>::new(), columns: vec![], filter_panel: true, .. }
const TOOLBAR_FILTERS: &str = r#"toolbar: rsx! {
    TableColumnsButton {}
    TableDensityButton {}
    TableExportButton {
        onexport: move |csv: String| {
            spawn(async move {
                save_file("team.csv", "text/csv", csv.into_bytes()).await;
            });
        }
    }
    TableFilterButton {}
    Button { variant: "outlined", size: "sm", "Add member" }
}"#;

/// Wider than the preview at any width, so `scroll` has something to scroll.
const SCROLL_WIDTH: &str = "1400px";

/// Shorter than the header and two rows, so `max_height` has something to scroll.
const MAX_HEIGHT: &str = "100px";

/// Name, the outermost start column, needs no `width`; nor does Bonus at the end.
// snippet: item #[derive(Clone, PartialEq)] struct Person { name: String, role: String, bonus: Option<f64> }
// snippet: let people: Vec<Person> = Vec::new();
// snippet: in Table { caption: "Team members", data: people, columns: Vec::new(), .. }
const PINNED: &str =
    r#"default_pinned_columns: PinnedColumns::default().start(["Name"]).end(["Bonus"])"#;

/// The `virtual_row_height` switch's cap: a dozen 40px rows.
const WINDOW_HEIGHT: &str = "320px";

/// The switches `virtual_row_height` hides: it sets the cap, the rest render every row.
const WINDOW_HIDES: [&str; 4] = ["max_height", "row_detail", "paginate", "empty"];

/// Ten thousand rows, windowed.
fn windowed(values: &DemoValues) -> bool {
    values.str("virtual_row_height") == "true"
}

/// The `empty` switch empties `data` too, or the slot would never show.
fn no_rows(values: &DemoValues) -> bool {
    values.str("empty") == "true" && !windowed(values)
}

/// The server's batches have no signal of the demo's own to reorder.
fn reorders(values: &DemoValues) -> bool {
    values.str("reorder_rows") == "true" && !from_server(values)
}

// snippet: item #[derive(Clone, PartialEq)] struct Person { name: String }
// snippet: let mut rows = use_signal(Vec::<Person>::new);
// snippet: in Table { caption: "Team members", data: rows(), columns: vec![], row_key: |p: &Person| p.name.clone(), .. }
const REORDER: &str = "onrowreorder: move |step: SortableMove| step.apply(&mut rows.write())";

fn resizes(values: &DemoValues) -> bool {
    values.str("resizable_columns") == "true"
}

/// The main demo's table: a component, so reordered rows and resized widths have a signal to land in.
#[component]
fn TeamTable(values: DemoValues) -> Element {
    let mut rows = use_signal(people);
    // Built on the first flip only, then kept: a flip re-renders the table, not the data.
    let many = use_hook(|| Rc::new(OnceCell::<Vec<Person>>::new()));
    // The ten thousand once reordered, else `many` as built.
    let mut moved = use_signal(|| None::<Vec<Person>>);
    let mut widths = use_signal(ColumnWidths::new);
    let mut loaded = use_signal(|| fetch(0));
    let mut fetching = use_signal(|| false);
    let mut pending = use_signal(|| None::<Box<dyn TimerSubscription>>);
    use_drop(move || pending.set(None));
    let mut load = move || {
        fetching.set(true);
        pending.set(timer().map(|timer| {
            timer.after(
                LATENCY,
                Box::new(move || {
                    let batch = fetch(loaded.peek().len());
                    loaded.write().extend(batch);
                    fetching.set(false);
                }),
            )
        }));
    };
    let windowed = windowed(&values);
    let server = from_server(&values);
    // A switch the windowed table hides counts as off, `loading` too while the server sets it.
    let on = |name: &str| {
        values.str(name) == "true"
            && !(windowed && WINDOW_HIDES.contains(&name))
            && !(server && name == "loading")
    };
    rsx! {
        Table {
            // A new table per switch: `page_sizes` and the pins seed once.
            key: "{values.str(\"paginate\")}-{values.str(\"pinned\")}-{windowed}-{server}",
            caption: "Team members",
            size: values.str("size"),
            striped: on("striped"),
            scroll: on("scroll"),
            sx: match on("scroll") {
                true => sx().min_width(SCROLL_WIDTH),
                false => sx(),
            },
            max_height: match windowed {
                true => Some(WINDOW_HEIGHT.to_string()),
                false => on("max_height").then(|| MAX_HEIGHT.to_string()),
            },
            virtual_row_height: windowed.then_some(40.0),
            default_pinned_columns: match on("pinned") {
                true => PinnedColumns::default().start(["Name"]).end(["Bonus"]),
                false => PinnedColumns::default(),
            },
            selectable: on("selectable"),
            multi_sort: on("multi_sort"),
            column_menu: on("column_menu"),
            row_detail: match on("row_detail") {
                true => (|p: &Person| Some(rsx! { "{p.name} joined as {p.role}." })).into(),
                false => RowFn::default(),
            },
            onrowreorder: reorders(&values).then(|| {
                let many = many.clone();
                EventHandler::new(move |step: SortableMove| match windowed {
                    true => step.apply(
                        moved.write().get_or_insert_with(|| many.get_or_init(crowd).clone()),
                    ),
                    false => step.apply(&mut rows.write()),
                })
            }),
            show_quick_filter: on("show_quick_filter"),
            toolbar: on("toolbar").then(|| rsx! {
                TableColumnsButton {}
                TableDensityButton {}
                TableExportButton {
                    onexport: move |csv: String| {
                        spawn(async move {
                            save_file("team.csv", "text/csv", csv.into_bytes()).await;
                        });
                    }
                }
                if on("filter_panel") {
                    TableFilterButton {}
                }
                Button { variant: "outlined", size: "sm", "Add member" }
            }),
            loading: on("loading") || (server && fetching()),
            onbottomreached: server.then(|| EventHandler::new(move |()| load())),
            header_filters: on("header_filters"),
            filter_panel: on("filter_panel"),
            resizable_columns: resizes(&values),
            column_widths: resizes(&values).then(|| widths.cloned()),
            oncolumnwidthschange: move |next: ColumnWidths| widths.set(next),
            row_key: |p: &Person| p.name.clone(),
            page_sizes: if on("paginate") { vec![2, 5, 10] } else { vec![] },
            empty: no_rows(&values).then(|| rsx! { "No team members yet." }),
            data: match (windowed, no_rows(&values)) {
                _ if server => loaded(),
                (true, _) => moved().unwrap_or_else(|| many.get_or_init(crowd).clone()),
                (false, true) => Vec::new(),
                (false, false) => rows(),
            },
            columns: team_columns(on("column_groups")),
        }
    }
}

#[component]
pub fn TablePage() -> Element {
    rsx! {
        DocPage {
            title: "Table",
            source: "libero/src/components/data_display/table",
            markdown: "/md/table.md",
            properties: vec![
                props("Table", vec![
                    prop("data", "Vec<T>").default("required").doc("One row each, in source order until a column is sorted."),
                    prop("columns", "Vec<Column<T>>").default("required").doc("Built with `column(..)`."),
                    prop("column_defaults", "ColumnDefaults").default("none").doc("Settings every column starts from: `ColumnDefaults::new().align(..).width(..).min_width(..)`. A column's own setting wins."),
                    prop("caption", "Option<String>").default("None").doc("A visible title above the header row, and the table's accessible name."),
                    prop("empty", "Option<Element>").default("None").doc("Shown in one full-width row when `data` is empty. Unset, the row reads the localized `table.no_rows`, \"No rows\". When the quick filter or the column filters leave no rows, the row reads `table.no_results`, \"No matching rows\", instead."),
                    prop("no_results", "Option<Element>").default("None").doc("Shown in one full-width row when the quick filter or the column filters leave no rows, over the localized `table.no_results`."),
                    prop("loading", "bool").default("false").doc("Rows are on their way. While no row shows, placeholder rows fill the body, a page of them when paged, else five, and the table is `aria-busy`. With rows shown, they stay usable under a thin progress bar over the table's top edge, named by the localized `table.loading`, \"Loading rows\". The empty row waits until loading ends."),
                    prop("toolbar", "Option<Element>").default("None").doc("A row above the table for your own controls, say an export or add button. With `show_quick_filter` the search field joins it at the end. It wraps on a narrow screen and stays put while the table scrolls. `TableColumnsButton`, `TableDensityButton`, `TableFilterButton` and `TableExportButton` work only in here."),
                    prop("scroll", "bool").default("false").doc("Wraps the table in a `ScrollArea` that scrolls sideways. `class`, `sx` and `attributes` stay on the table."),
                    prop("max_height", "Option<String>").default("None").doc("Caps the table's height, any CSS length. The rows scroll in a `ScrollArea`, both ways, under a header that stays put, with the scrollbar beside the rows only. The `caption` sits above the scrolled box. The header takes the page surface's colour: on another background, set it with `sx().selector(\"& thead th\", ..)`."),
                    prop("virtual_row_height", "Option<f64>").default("None").doc("With `max_height`, renders only the rows in view plus a few beyond each edge, so ten thousand rows scroll like fifty. Every body row is clipped to this height in px: one line per cell, longer text ends in an ellipsis. The table then lays out fixed, columns without a `width` sharing the rest evenly. A row holding focus stays rendered while it scrolls away. With `onrowreorder` a keyboard move scrolls to its slot, and a row dragged to the top or bottom edge scrolls the rows on. Ignored with `row_detail`, which renders every row; a debug build warns."),
                    prop("onbottomreached", "EventHandler<()>").default("None").doc("With `max_height`, called when the rows scroll to their bottom, by wheel, drag or End: append the next batch to `data`. Called again at the new bottom once rows were added. Not called while `loading` is set, so set it during a fetch to ask once. Without `max_height` it never fires; a debug build warns."),
                    prop("sort", "Option<Vec<TableSort>>").default("None").doc("The sorted columns, empty for source order. Set, the sort is controlled: pair it with `onsortchange`. Without `multi_sort`, one column sorts, the first entry naming a sortable header."),
                    prop("default_sort", "Vec<TableSort>").default("[]").doc("Seeds the sort once. Ignored when `sort` is set."),
                    prop("onsortchange", "EventHandler<Vec<TableSort>>").default("None").doc("Called with the sort a header click asks for: ascending, then descending, then empty."),
                    prop("multi_sort", "bool").default("false").doc("Sorts by several columns, the first entry first. Shift, Ctrl or Cmd with a header click, or any tap on a touch screen, adds the column after the sorted ones, then flips and removes it. A plain click sorts by that column alone. Each sorted header shows its place."),
                    prop("selectable", "bool").default("false").doc("Adds a checkbox column, with a select-all box in its header. Set `row_key` with it, or the selection sticks to positions in `data`."),
                    prop("selection", "Option<Vec<String>>").default("None").doc("The selected rows' `row_key`s. Set, the selection is controlled: pair it with `onselectionchange`. It survives a sort."),
                    prop("default_selection", "Vec<String>").default("[]").doc("Seeds the selection once. Ignored when `selection` is set."),
                    prop("onselectionchange", "EventHandler<Vec<String>>").default("None").doc("Called with the selection a checkbox asks for. Select-all covers every row of `data` the quick filter keeps, and keeps keys of the other rows, say from another page of a server."),
                    prop("row_key", "RowFn<T, String>").default("the row's index").doc("A row's identity, unique per row, from a `|row: &T| ..` closure. Its DOM node follows it through a sort or a data change, so focus and state inside a row stay with it."),
                    prop("onrowclick", "EventHandler<T>").default("None").doc("Called with the clicked row. Pointer only: give keyboard users a button or link in a cell for the same action."),
                    prop("row_states", "RowFn<T, States>").default("None").doc("A row's states, rendered as its `data-state`. Style them with `sx().selector(\"& tbody tr\", sx().when(..))`."),
                    prop("row_attrs", "RowFn<T, Vec<Attribute>>").default("None").doc("Extra attributes on a row's `tr`."),
                    prop("row_detail", "RowFn<T, Option<Element>>").default("None").doc("A row's detail, from a `|row: &T| ..` closure. `Some` gives the row a toggle in a leading column; open, the detail shows in a full-width row under it. Called for the shown rows on every render, or only the open ones with `row_has_detail`. Set `row_key` with it, or open details stick to positions in `data`."),
                    prop("row_has_detail", "RowFn<T, bool>").default("None").doc("Whether a row has a detail, without building it. Set, it decides the toggles, and `row_detail` is called for the open rows only: for large tables."),
                    prop("animate_details", "bool").default("false").doc("Slides the detail rows open and shut, as a `Collapse`; at once under reduced motion. Not with `onrowreorder`."),
                    prop("expanded", "Option<Vec<String>>").default("None").doc("The `row_key`s of the rows whose detail shows. Set, it is controlled: pair it with `onexpandedchange`. It survives a sort."),
                    prop("default_expanded", "Vec<String>").default("[]").doc("Seeds the open details once. Ignored when `expanded` is set."),
                    prop("onexpandedchange", "EventHandler<Vec<String>>").default("None").doc("Called with the open details a toggle asks for."),
                    prop("onrowreorder", "EventHandler<SortableMove>").default("None").doc("Adds a leading column with a drag handle and Move up and Move down buttons per row. Called with a move by positions in `data`; apply it with `step.apply(&mut rows)`, the table shows the old order until you do. Off while the rows are sorted or the quick filter has text. Paged, a row moves within its page. Set `row_key` with it."),
                    prop("size", "Size").default("theme (md)").doc("Cell padding and font size."),
                    prop("density", "Option<Size>").default("None").doc("The size a `TableDensityButton` picked, over `size`; set, it is controlled."),
                    prop("default_density", "Option<Size>").default("None").doc("Seeds the density once. Ignored when `density` is set."),
                    prop("ondensitychange", "Option<EventHandler<Size>>").default("None").doc("The density a `TableDensityButton` pick asks for."),
                    prop("striped", "bool").default("false").doc("Shades every other body row."),
                    prop("page", "Option<u32>").default("None").doc("The shown page, 1-based. Set, the page is controlled: pair it with `onpagechange`. A page past the end shows the last one."),
                    prop("default_page", "u32").default("1").doc("Seeds the page once. Ignored when `page` is set."),
                    prop("onpagechange", "EventHandler<u32>").default("None").doc("Called with the page a page button asks for. A sort or quick-filter change asks for page 1, and a new page size for the page that keeps the first shown row."),
                    prop("page_size", "Option<usize>").default("None").doc("Rows per page. Set, the size is controlled: pair it with `onpagesizechange`. Turns on pagination."),
                    prop("default_page_size", "Option<usize>").default("None").doc("Seeds the page size once. Turns on pagination."),
                    prop("onpagesizechange", "EventHandler<usize>").default("None").doc("Called with the size picked in the page-size picker."),
                    prop("page_sizes", "Vec<usize>").default("[]").doc("The page-size picker's choices; empty hides the picker. Turns on pagination, the first one seeding the size. No cap."),
                    prop("manual_sort", "bool").default("false").doc("`data` comes sorted, say from a server: a header click only reports through `onsortchange`."),
                    prop("manual_pagination", "bool").default("false").doc("`data` is the current page only: the table draws the page controls and leaves the slicing to you."),
                    prop("row_count", "Option<usize>").default("data's length").doc("Rows over all pages with `manual_pagination`, for the page count and the range text."),
                    prop("hidden_columns", "Option<Vec<String>>").default("None").doc("The hidden columns' headers. Set, visibility is controlled: pair it with `onhiddencolumnschange`. A hidden sorted column keeps sorting."),
                    prop("default_hidden_columns", "Vec<String>").default("[]").doc("Seeds the hidden columns once. Ignored when `hidden_columns` is set."),
                    prop("onhiddencolumnschange", "EventHandler<Vec<String>>").default("None").doc("Called with the hidden columns a column menu pick asks for."),
                    prop("quick_filter", "Option<String>").default("None").doc("The quick filter's text. A row stays when every word occurs, ignoring case, in the text of one of its shown, `filterable` cells. Set, the filter is controlled: pair it with `onquickfilterchange`."),
                    prop("default_quick_filter", "String").default("\"\"").doc("Seeds the quick filter once. Ignored when `quick_filter` is set."),
                    prop("onquickfilterchange", "EventHandler<String>").default("None").doc("Called with the text typed into the quick-filter field."),
                    prop("show_quick_filter", "bool").default("false").doc("Puts a search field above the table that drives the quick filter."),
                    prop("manual_filter", "bool").default("false").doc("`data` comes filtered, say from a server: the quick filter and the column filters only report through `onquickfilterchange` and `oncolumnfilterschange`. Pair it with `manual_pagination` and `row_count` when paged."),
                    prop("column_filters", "Option<Vec<ColumnFilter>>").default("None").doc("Filters by header: `ColumnFilter::new(\"Age\", FilterOperator::GreaterThan, \"30\")`, or `ColumnFilter::between(\"Joined\", \"2026-01-01\", \"2026-06-30\")` for a date range. A row stays when it passes all of them (or any, with `filter_logic`) and the quick filter. Dates take an ISO day. Text operators ignore case, `Equals` too; number operators compare the value, not its formatted text, and take `1,5` as 1.5. An empty value, a number that does not parse, or an operator the column's type does not offer keeps every row. A hidden column's filter keeps filtering. Set, the filters are controlled: pair them with `oncolumnfilterschange`."),
                    prop("default_column_filters", "Vec<ColumnFilter>").default("[]").doc("Seeds the column filters once. Ignored when `column_filters` is set."),
                    prop("oncolumnfilterschange", "EventHandler<Vec<ColumnFilter>>").default("None").doc("Called with the filters a filter popover or a header filter asks for. Typed values arrive once typing pauses for 300 ms, an operator or yes/no pick at once."),
                    prop("filter_logic", "Option<FilterLogic>").default("None").doc("How the column filters join: `FilterLogic::And` keeps a row that passes all of them, `Or` one that passes any. The quick filter applies on top either way. Set, it is controlled."),
                    prop("default_filter_logic", "FilterLogic").default("And").doc("Seeds the filter logic once. Ignored when `filter_logic` is set."),
                    prop("onfilterlogicchange", "EventHandler<FilterLogic>").default("None").doc("Called with the filter logic a pick asks for."),
                    prop("header_filters", "bool").default("false").doc("Adds a row of filter fields under the headers, one per `filterable` column: a text field, or Any/Yes/No for a boolean column. A field edits its column's filter with the operator the filter popover set, else the type's first: Contains for text, Equals for numbers."),
                    prop("filter_panel", "bool").default("false").doc("A Filters button at the start of the toolbar row, showing the active filters' count, opens a dialog of every column filter; with a `toolbar`, put `TableFilterButton` in it where the button goes instead, one line each: column, operator, value and a remove button. Add filter appends a line for the first column without one; from two lines a Match pick sets `filter_logic`. Lines apply as the popover does. `column_menu`'s Filter then opens the panel on its column's line instead of the popover."),
                    prop("pinned_columns", "Option<PinnedColumns>").default("None").doc("The columns held at the table's start and end edges while the rest scroll sideways, by header: `PinnedColumns::default().start([..]).end([..])`. Start is the left in a left-to-right page, the right in a right-to-left one. Set, pinning is controlled: pair it with `onpinnedcolumnschange`. Pair it with `scroll` or `max_height`, and give every pinned column but the outermost on its side a `width`, which it then keeps exactly."),
                    prop("default_pinned_columns", "PinnedColumns").default("none pinned").doc("Seeds the pinned columns once. Ignored when `pinned_columns` is set."),
                    prop("onpinnedcolumnschange", "EventHandler<PinnedColumns>").default("None").doc("Called with the pinned columns a column menu pick asks for."),
                    prop("column_order", "Option<Vec<String>>").default("None").doc("The headers in display order. Unlisted columns follow the listed ones in `columns` order, and pinned columns keep their pinned order. Set, the order is controlled: pair it with `oncolumnorderchange`. The sort, hidden and pinned columns name headers, so they follow a moved column."),
                    prop("default_column_order", "Vec<String>").default("[]").doc("Seeds the column order once. Ignored when `column_order` is set."),
                    prop("oncolumnorderchange", "EventHandler<Vec<String>>").default("None").doc("Called with the order a column menu's Move left or Move right, or a header drag, asks for, every header listed."),
                    prop("resizable_columns", "bool").default("false").doc("Puts a visible grip on each header's end edge, 24px wide on a touch screen, and Widen column, Narrow column (50px steps, the menu stays open) and Reset width in its `column_menu`, the drag-free way. Drag a grip, or double-click it to reset. A focused grip takes the keys: Left and Right move it 10px the way they point, 50px with Shift, Home and End go to the limits. A column opts out with `.resizable(false)` and sets its range with `.resize_limits(min, max)` in px, 50 to unbounded by default. Auto layout never draws a column narrower than its content. Blitz: use the menu, the grip does not drag reliably there."),
                    prop("column_widths", "Option<ColumnWidths>").default("None").doc("Resized widths in px by header, a `BTreeMap<String, f64>`, over the columns' own `width`. Set, the widths are controlled: pair them with `oncolumnwidthschange`."),
                    prop("default_column_widths", "ColumnWidths").default("{}").doc("Seeds the widths once. Ignored when `column_widths` is set."),
                    prop("oncolumnwidthschange", "EventHandler<ColumnWidths>").default("None").doc("Called with the widths a grip drag asks for when it ends, each grip key, and with each Widen, Narrow or Reset width."),
                    prop("column_menu", "bool").default("false").doc("Puts a menu button in each header: sort ascending or descending, unsort, add to the sort with `multi_sort`, filter a `filterable` column, move the column left or right past the next shown one, widen, narrow or reset it with `resizable_columns`, pin to the start or end or unpin, hide the column, and a Columns submenu that shows or hides the others. A pinned column has no move entries. Filter opens a popover with the operators of the column's type, a value and Clear; a filtered column's header then shows a filter button that reopens it."),
                    prop("column_menu_parts", "Parts<MenuPart>").default("none").doc("The column menus' `parts`, the `Menu` page's Style API. The menus open in a portal, out of the table's `sx`."),
                ]),
                props("column()", vec![
                    prop("header", "String").default("required").doc("The column's title, the argument to `column(..)`."),
                    prop("value", "fn(&T) -> V").default("required").doc("Reads one cell out of a row. `V` sets the sort order and alignment. Text sorts as text and aligns left, numbers sort numerically and align right, and `Option<V>` renders `None` empty and sorts it last. Your own type joins them with one `impl CellValue`."),
                    prop("sortable", "bool").default("false").doc("Turns the header into a sort button."),
                    prop("render", "fn(&T) -> Element").default("None").doc("Replaces the cell body. Sorting still uses `value`. Capture signals: a closure that captures a plain value redraws only rows whose data changed."),
                    prop("format", "fn(&T) -> String").default("None").doc("Replaces the cell text, say a price with its currency. Sorting and alignment still follow `value`."),
                    prop("align", "CellAlign").default("follows the cell type").doc("Overrides the alignment the cell type chose and `column_defaults`."),
                    prop("width", "String").default("None").doc("The column's width, any CSS length. Columns without one share the rest."),
                    prop("min_width", "String").default("None").doc("The narrowest the column gets, any CSS length."),
                    prop("resizable", "bool").default("true").doc("Whether the table's `resizable_columns` gives it a grip and menu entries."),
                    prop("resize_limits", "(f64, f64)").default("(50, unbounded)").doc("How narrow and how wide a resize takes the column, in px. `min_width` still floors it on screen."),
                    prop("header_render", "fn() -> Element").default("None").doc("Replaces the header's body, inside the sort button when sortable. The header text stays the column's name in `TableSort`. Capture signals, not values: the closure is not compared, so a changed value does not redraw the header."),
                    prop("of", "&ColumnType<V>").default("None").doc("Before `value`: starts the column from a shared `const` type, its alignment, widths and a `format` over the value. `V` must match `value`'s; the column's own settings win."),
                    prop("row_header", "bool").default("false").doc("Renders the column's cells as `th scope=\"row\"`, so a screen reader names each row by it. One per table, usually the first."),
                    prop("filterable", "bool").default("true").doc("Whether the quick filter searches the column's cell text, and whether it takes a column filter. Off for ids and codes that would match by accident."),
                    prop("hideable", "bool").default("true").doc("Whether the column menu offers to hide the column. `hidden_columns` still hides it."),
                    prop("group", "String").default("None").doc("Puts the column under a group header, shared with the adjacent columns of the same groups. Call it once per level, outermost first. The same name under another parent is another group, and a hidden column leaves its group."),
                    prop("col_span", "fn(&T) -> usize").default("None").doc("How many shown columns a row's cell covers, from this one on, say a total row's label. The covered cells are left out; the span stops at the row's end. Capture signals, not values: the closure is not compared."),
                ]).without_base_props(),
                props("table_csv()", vec![
                    prop("table_csv", "fn(&[Column<T>], &[T]) -> String").default("none").doc("The header row and one line per row as CSV (RFC 4180, CRLF), each cell as its column's text, `format` applied and `render` ignored. Pass the rows and columns in the order you want; `save_file` saves it as a file."),
                    prop("table_text", "fn(&[Column<T>], &[T]) -> Vec<Vec<String>>").default("none").doc("The same cells unjoined, the header row first, for your own writer."),
                ]).without_base_props(),
                props("TableExportButton", vec![
                    prop("onexport", "EventHandler<String>").default("required").doc("Called with the table as CSV, as `table_csv` writes it: the filtered, sorted rows of every page, in the shown columns. `save_file` saves it as a file. `TableColumnsButton`, `TableDensityButton` and `TableFilterButton` take no props, and none of the four takes `class`, `sx` or `attributes`."),
                ]).without_base_props(),
            ],
            accessibility: a11y()
                .key(["Tab"], "With `scroll` or `max_height`, when the table overflows and has no button in it: enters the scroll region, a tab stop.")
                .key(["Left", "Right", "Up", "Down", "PageUp", "PageDown"], "In the scroll region, or on a header button inside it: scrolls the table.")
                .key(["Enter", "Space"], "On a sortable header, a button: sorts by that column, flips it, then unsorts.")
                .key(["Shift+Enter", "Shift+Space"],"With `multi_sort`, on a sortable header: adds that column after the sorted ones.")
                .key(["Space"], "On a row's checkbox: selects or deselects the row. On the header checkbox: selects or clears every row.")
                .key(["Enter", "Space"], "With `row_detail`, on a row's toggle: opens or closes its detail. Tab then goes into the open detail.")
                .key(["Enter", "Space", "Down"], "With `column_menu`, on a header's menu button: opens the column menu, keyed like `Menu`.")
                .key(["Space", "Enter"], "With `onrowreorder`, on a row's handle: lifts the row, then drops it. Up and Down move the lifted row, Home and End to the first or last place, Escape puts it back.")
                .key(["Left", "Right"], "With `resizable_columns`, on a header's resize grip: moves it 10px the way the arrow points, 50px with Shift.")
                .key(["Home", "End"], "On a resize grip: narrows the column to its minimum, or widens it to its maximum when it has one.")
                .key(["Escape"], "In a filter popover: closes it and returns focus to the column's menu button.")
                .key(["Tab", "Shift+Tab"], "In a filter popover: moves between its fields; past either end it closes and Tab goes on from the menu button.")
                .handles([
                    "An unnamed table warns in a debug build.",
                    "With `scroll` or `max_height`, an overflowing scroll area with no button in it is a tab stop, a `role=\"region\"` named like the table. With sort or menu buttons in the header it is no stop: the arrows scroll it from a focused button.",
                    "The column menu button shows on its header's hover or focus with a mouse, always on a touch screen. An end-aligned header puts it first, in the DOM too, so Tab follows what is seen.",
                    "Only sorted headers carry `aria-sort`. With several, each sort button's name adds its place, \"sort order 2\".",
                    "Each row's checkbox is named \"Select\" plus its row header's text, else its first cell's. The header checkbox reads mixed while some rows are selected.",
                    "A selected row carries `aria-selected=\"true\"`, and a polite live region says the new count, \"2 rows selected\", after each change.",
                    "A tap on a touch screen has no Shift key, so with `multi_sort` a tap always adds the column.",
                    "`selectable` or `row_detail` without `row_key` warns in a debug build.",
                    "Each detail toggle is a button named \"Details for\" plus the row's name, like its checkbox, with `aria-expanded`, and `aria-controls` on the detail row while it is open. The toggle column's header reads \"Details\" to a screen reader only.",
                    "`.row_header()` cells render as `th scope=\"row\"`, so a screen reader reads that name as it moves down any other column. They look like the other cells.",
                    "Paginated, the page buttons sit in a `nav` named after the caption, the page-size picker is labelled, and a page change announces the new range, \"4–6 of 7\", politely. The first render announces nothing.",
                    "The quick-filter field is a labelled `type=\"search\"` input, \"Search\", described by the table's caption, else its `aria_labelledby` or `aria_label`, so two tables' fields differ. Once typing pauses for half a second, a polite live region says how many rows are left, \"2 rows\".",
                    "With `column_menu`, each menu button is named after its column, \"Age column options\", and the header keeps its text as its name. The Columns submenu lists checkbox items, and the last shown column cannot be hidden: its checkbox and Hide column are described by why, \"One column stays shown\". After a pin moves the column, focus stays on its menu button; after Hide column it moves to the next shown column's, else the previous one's.",
                    "The filter popover is a `role=\"dialog\"` named \"Filter\" plus the column, with labelled Operator and Value fields; focus moves to the value on open, or to the operator when it takes none. A filtered header's button reads \"Age is filtered\". Each header filter field is named \"Filter\" plus its column.",
                    "The filter panel is a `role=\"dialog\"` named \"Filters\"; its button reads \"Filters, 2 active\", the count it shows hidden from readers. Each line's controls are named with their column, \"Name: operator\", \"Name: value\", \"Remove Name filter\". Opening focuses the first line, else Add filter; Add focuses the new line, a remove the line before it, else Add filter; Escape, or Tab past either end, closes it onto its button.",
                    "A column filter change announces the rows left, \"2 rows\", in the same polite live region, once it settles.",
                    "A group header is a `th scope=\"colgroup\"` over its columns, so a screen reader reads it with each of their cells. A column outside any group, and the select-all box, span every header row.",
                    "Pinned columns move to their edge in the DOM too, so Tab and a screen reader meet the cells in the order they are seen. The detail toggle and checkbox columns pin with the start ones.",
                    "A pinned column past one without a `width` warns in a debug build: its offset is unknown, so it would overlap.",
                    "A group or a `col_span` stops at a pin edge: a group over pinned and scrolled columns shows as two headers, and the one over the pinned columns pins with them.",
                    "With `onrowreorder`, each row has a drag handle named \"Reorder\" plus the row's name, described by the keyboard steps, and Move up and Move down buttons, so a single pointer reorders without a drag (WCAG 2.5.7). A touch drags only from the handle; elsewhere it scrolls. Each lift, move and drop is said in a polite live region. While sorted or filtered the controls are off, `aria-disabled`, yet stay tab stops described by why: \"Clear the sort and filters to reorder rows\".",
                    "`onrowreorder` without `row_key` warns in a debug build.",
                    "The column menu's Move left and Move right name the screen sides in either text direction. A moved column moves in the DOM too, so Tab and a screen reader follow it, and a column moved out of its group splits the group.",
                            "Each move, pin, unpin and Hide column in the column menu is said in a polite live region, \"Name moved to column 2 of 5\", \"Name pinned to start\", \"Name hidden\": the menu closes over the change.",
                    "The header drag grip is pointer only and hidden from screen readers, with no tab stop: the column menu's moves are its keyboard and drag-free way (WCAG 2.5.7). Escape, or a drop outside the table, cancels the drag. Blitz has no grip; the menu moves columns there.",
                    "Each resize grip is a tab stop, a vertical `role=\"separator\"` named \"Resize\" plus the column, with `aria-valuenow` its width in px and `aria-valuemin` and `aria-valuemax` its limits.",
                    "Each Widen, Narrow or Reset width in the column menu says the column's new width in a polite live region, \"Name: 170 px\": the menu stays open over the change.",
                    "`loading` without shown rows marks the table `aria-busy` and hides its placeholder rows from screen readers. With rows shown it adds a progress bar named \"Loading rows\" and leaves the table unbusy, as some screen readers hold back a busy table's rows.",
                    "The `toolbar` is a plain row, not a `role=\"toolbar\"`: Tab moves through its controls as anywhere else. `TableExportButton` says the rows it handed over, \"Exported 12 rows\", in the polite live region; `TableColumnsButton` describes the last shown column's checkbox as the column menu does.",
                    "With `virtual_row_height`, the table carries `aria-rowcount`, every row it holds, the other pages' too when paged, and each rendered row its `aria-rowindex`, so a screen reader says \"row 5 001 of 10 001\" though only a screenful is in the DOM. The scrolled-away rows leave no empty rows behind.",
                    "With `virtual_row_height`, Tab and Shift+Tab walk the rows' controls past the rendered ones: the focused row scrolls into view and the next one renders. The row holding focus stays rendered when it scrolls away, Blitz included.",
                    "With `onbottomreached`, once the rows it asked for arrive a polite live region says the new count, \"200 rows\". `aria-rowcount` counts the rows loaded so far.",
                ])
                .must([
                    "Name every table. `caption` shows a title and names it, `aria_labelledby` points at a heading already on the page, and `aria_label` names it without text.",
                    "Set `scroll: true` on a table wider than its container, or `max_height` on a long one.",
                    "Mark the column that names a row with `.row_header()`.",
                    "With `onrowclick`, also put a button or link for that action in a cell. A row is not a tab stop, so a keyboard cannot click it.",
                    "With `selectable` or `row_detail`, give the rows a `.row_header()` column, so each checkbox and toggle is named by something unique.",
                    "Before a CSV of text users typed goes to a spreadsheet, neutralise cells that start with `=`, `+`, `-` or `@` (CSV injection, OWASP). `table_text`'s docs show a three-line guard.",
                ])
                .limits([
                    "On Blitz, once a `max_height` table's rows scroll, a click on a header's sort or menu button misses: Blitz hit-tests the header where it sat before the scroll. Tab to the button and press Enter instead.",
                    "On Blitz, the same holds for a pinned column's cells once the table scrolls sideways. In a right-to-left page Blitz cannot scroll a wide table at all (todo 707), so pinning shows no effect there.",
                    "On Blitz, a `max_height` table's scrollbar is the native one and runs past the header too.",
                    "On Blitz, a `max_height` table with column groups keeps only its last header row in place; the group rows scroll away with the rows. The same holds for the `header_filters` row.",
                    "On Blitz, a row's handle does not drag: Blitz paints no moved table row. The keyboard and the move buttons reorder there.",
                    "On Blitz, a header has no drag grip: columns reorder from the column menu there.",
                    "On Android, the filter popover cannot look into itself, so Tab does not close it at its ends and focus lands on the popover rather than its value field; Escape and Back close it.",
                ]),
            lead: rsx! {
                Text {
                    "A table built from "
                    Code { source: "data" }
                    " and "
                    Code { source: "columns" }
                    ". Each column comes from "
                    Code { source: "column" }
                    ", with a header and a "
                    Code { source: "value" }
                    " that reads one cell out of a row. The cell's type sets the sort order "
                    "and alignment, so a numeric column sorts numerically and aligns right "
                    "on its own."
                }
                Text {
                    Code { source: "sortable" }
                    " turns a header into a button. The first click sorts ascending, the "
                    "next flips it, and a third restores source order. "
                    Code { source: "render" }
                    " changes only what a cell draws, so the Role column below still sorts "
                    "by its text. "
                    Code { source: "format" }
                    " does the same for the cell's text: the Bonus column prints a percent "
                    "and still sorts by number."
                }
                Text {
                    Code { source: "default_sort" }
                    " sorts the first render. To hold the sort yourself, for a server-side "
                    "query or a saved view, pass "
                    Code { source: "sort" }
                    " and update it from "
                    Code { source: "onsortchange" }
                    "."
                }
                Text {
                    Code { source: "row_key" }
                    " gives each row an identity, so its DOM node follows it when rows "
                    "are added, removed or sorted. "
                    Code { source: "size" }
                    " sets the cell padding and font size, and "
                    Code { source: "striped" }
                    " shades every other row."
                }
                Text {
                    Code { source: "width" }
                    " and "
                    Code { source: "min_width" }
                    " size a column, "
                    Code { source: "header_render" }
                    " draws its header, and "
                    Code { source: "column_defaults" }
                    " sets what every column starts from, say a minimum width. Columns of one "
                    "kind, say prices, share a "
                    Code { source: "const ColumnType" }
                    " with "
                    Code { source: "column(\"Price\").of(&MONEY).value(..)" }
                    "."
                }
                Text {
                    Code { source: "selectable" }
                    " adds a checkbox per row and a select-all box. The selection is a list of "
                    Code { source: "row_key" }
                    "s, so it stays with its rows through a sort. Hold it yourself with "
                    Code { source: "selection" }
                    " and "
                    Code { source: "onselectionchange" }
                    ". "
                    Code { source: "multi_sort" }
                    " lets Shift-click, or a tap on a touch screen, sort by one more column."
                }
                Text {
                    Code { source: "page_sizes" }
                    " pages the rows after sorting them, with a page-size picker, the shown "
                    "range and page buttons under the table. "
                    Code { source: "page" }
                    " and "
                    Code { source: "page_size" }
                    " hold the state yourself, as "
                    Code { source: "sort" }
                    " does. For server-side data set "
                    Code { source: "manual_sort" }
                    " and "
                    Code { source: "manual_pagination" }
                    ", pass the current page as "
                    Code { source: "data" }
                    " and the total as "
                    Code { source: "row_count" }
                    ". Select-all covers the rows on every page."
                }
                Text {
                    Code { source: "show_quick_filter" }
                    " puts a search field above the table. A row stays when every typed word "
                    "occurs in one of its shown cells; "
                    Code { source: ".filterable(false)" }
                    " leaves a column out. The rows are filtered, then sorted, then paged. Hold "
                    "the text yourself with "
                    Code { source: "quick_filter" }
                    " and "
                    Code { source: "onquickfilterchange" }
                    ", or filter on a server with "
                    Code { source: "manual_filter" }
                    "."
                }
                Text {
                    Code { source: "toolbar" }
                    " puts your own controls in a row above the table, the search field at its "
                    "end. "
                    Code { source: "loading" }
                    " shows placeholder rows while there are none yet, and a progress bar over "
                    "the rows while new ones load. "
                    Code { source: "empty" }
                    " and "
                    Code { source: "no_results" }
                    " replace the text of the row shown without data and without matches."
                }
                Text {
                    "Four ready pieces go in the "
                    Code { source: "toolbar" }
                    ", and only there. "
                    Code { source: "TableColumnsButton" }
                    " opens a checklist that shows and hides the hideable columns. "
                    Code { source: "TableDensityButton" }
                    " picks compact, standard or comfortable rows, the "
                    Code { source: "size" }
                    " small, medium or large; the pick wins over "
                    Code { source: "size" }
                    ". "
                    Code { source: "TableExportButton" }
                    " hands "
                    Code { source: "onexport" }
                    " the filtered, sorted rows of every page as CSV, in the shown columns; "
                    Code { source: "save_file" }
                    " saves it, a download on the web, a save dialog on the desktop, the share sheet on Android. "
                    Code { source: "TableFilterButton" }
                    " opens "
                    Code { source: "filter_panel" }
                    "'s dialog from where you put it; with a "
                    Code { source: "toolbar" }
                    ", the table leaves the Filters button to it. Only "
                    Code { source: "TableExportButton" }
                    " takes a prop, "
                    Code { source: "onexport" }
                    "; the demo's "
                    Code { source: "toolbar" }
                    " switch, on at first, shows them, "
                    Code { source: "TableFilterButton" }
                    " with "
                    Code { source: "filter_panel" }
                    " on."
                }
                Text {
                    Code { source: "column_menu" }
                    "'s Filter entry, and "
                    Code { source: "header_filters" }
                    "' row of fields under the headers, filter one column each, with operators "
                    "for its type: contains or starts with for text, greater than for numbers, "
                    "before, after or between for dates, as the Joined and Review columns show, "
                    "yes or no for booleans. The filters all apply, together with the quick filter. "
                    "Hold them yourself with "
                    Code { source: "column_filters" }
                    " and "
                    Code { source: "oncolumnfilterschange" }
                    "."
                }
                Text {
                    "A date column, one whose value is a "
                    Code { source: "chrono::NaiveDate" }
                    ", filters with before, after, on or before, on or after, and between. Its "
                    "value field is a "
                    Code { source: "DateField" }
                    ", whose calendar opens inside the filter; on a WebView (desktop, Android) it is the native date input. Both hold an ISO day ("
                    Code { source: "2026-10-03" }
                    "). "
                    Code { source: "ColumnFilter::between(\"Joined\", from, to)" }
                    " sets both ends, in "
                    Code { source: "value" }
                    " and "
                    Code { source: "value_to" }
                    ": an empty end leaves that side open, and ends given the wrong way round "
                    "are swapped. "
                    Code { source: "filter_logic: FilterLogic::Or" }
                    " keeps a row that passes any one column filter instead of all; the quick "
                    "filter still applies on top. A column may hold several filters; the popover "
                    "and the header field edit its first, and "
                    Code { source: "filter_panel" }
                    " all of them."
                }
                Text {
                    Code { source: "column_menu" }
                    " adds a menu to each header to sort, hide the column, or show and hide "
                    "the others. "
                    Code { source: ".hideable(false)" }
                    " keeps a column out of it. To hold the hidden columns yourself, pass "
                    Code { source: "hidden_columns" }
                    " and update it from "
                    Code { source: "onhiddencolumnschange" }
                    "."
                }
                Text {
                    Code { source: "column(\"Q1\").value(..).group(\"Revenue\")" }
                    " puts a column under a group header, shared with its neighbours of the "
                    "same group; call "
                    Code { source: "group" }
                    " again for a nested one. "
                    Code { source: "col_span" }
                    " lets a row's cell cover the next columns, say a total row's label."
                }
                Text {
                    Code { source: "row_detail" }
                    " gives a row a toggle that opens a full-width detail row under it, as in a "
                    "master-detail view. Rows the closure answers "
                    Code { source: "None" }
                    " for get no toggle. The open details are "
                    Code { source: "row_key" }
                    "s, held yourself with "
                    Code { source: "expanded" }
                    " and "
                    Code { source: "onexpandedchange" }
                    "."
                }
                Text {
                    Code { source: "onrowreorder" }
                    " gives each row a drag handle and move buttons. The table hands you the "
                    "move, by positions in "
                    Code { source: "data" }
                    ", and "
                    Code { source: "step.apply(&mut rows.write())" }
                    " applies it. Sorted or filtered, the shown order is not your data's, so "
                    "the controls turn off. With "
                    Code { source: "column_menu" }
                    ", Move left and Move right reorder the columns, and a grip at each unpinned "
                    "header's start edge drags one to a gap between the others; hold the order "
                    "yourself with "
                    Code { source: "column_order" }
                    " and "
                    Code { source: "oncolumnorderchange" }
                    ". A sorted column stays sorted wherever it moves."
                }
                Text {
                    Code { source: "table_csv(&columns, &rows)" }
                    " writes the cells as CSV, each as its column shows it as text, and "
                    Code { source: "table_text" }
                    " hands them over unjoined for another format. Saving is yours. A cell "
                    "that starts with "
                    Code { source: "=" }
                    " runs as a formula in a spreadsheet, so neutralise text users typed first."
                }
                Text {
                    Code { source: "scroll" }
                    " wraps a table wider than its container in a "
                    Code { source: "ScrollArea" }
                    " that scrolls sideways. The demo's switch also sets "
                    Code { source: format!("sx().min_width({SCROLL_WIDTH:?})") }
                    ", so the columns overflow at any width. "
                    Code { source: "max_height" }
                    " caps a long table's height: its rows scroll under a header that stays put."
                }
                Text {
                    "For thousands of rows add "
                    Code { source: "virtual_row_height" }
                    " to "
                    Code { source: "max_height" }
                    ": only the rows in view render, each that tall. The demo's switch swaps in "
                    "ten thousand people, 40px a row under a 320px cap, and hides the switches "
                    "that render every row. Sorting, filtering, selection and pinning work as before."
                }
                Text {
                    Code { source: "onbottomreached" }
                    " asks for more rows once the table scrolls to its bottom, by wheel, drag or "
                    "End. The demo's switch, shown with "
                    Code { source: "virtual_row_height" }
                    ", fetches the people from a fake server instead, a hundred at a time, and "
                    Code { source: "loading" }
                    " holds off another ask until the batch arrives."
                }
                Text {
                    Code { source: "default_pinned_columns" }
                    " holds columns at the start or end edge while the rest scroll under "
                    "them, and the column menu pins and unpins them. Start and end follow "
                    "the page's direction. Give each pinned column but the outermost a "
                    Code { source: "width" }
                    ", so the next one knows where to sit."
                }
                Text {
                    Code { source: "resizable_columns" }
                    " puts a grip on each header's end edge, a short line that lights up on "
                    "hover: drag it, or double-click it to reset the width. Tab reaches it too, "
                    "and the arrow keys step the width, Home and End to its limits. With "
                    Code { source: "column_menu" }
                    ", Widen column, Narrow column and Reset width do the same without a drag. "
                    "The demo holds the widths itself with "
                    Code { source: "column_widths" }
                    " and "
                    Code { source: "oncolumnwidthschange" }
                    ", say to save them with a view."
                }
            },
            Demo {
                component: "Table",
                children_text: "",
                wide_preview: true,
                fixed: vec![
                    r#"caption: "Team members""#.to_string(),
                    "row_key: |p: &Person| p.name.clone()".to_string(),
                    "data: people".to_string(),
                ],
                controls: vec![
                    Control::sizes("size")
                        .default("md"),
                    Control::switch("striped"),
                    Control::switch("scroll").code(|_, values| match values.str("scroll") == "true" {
                        true => vec![
                            "scroll: true".to_string(),
                            format!("sx: sx().min_width({SCROLL_WIDTH:?})"),
                        ],
                        false => vec![],
                    }),
                    Control::switch("max_height").code(|_, values| match values.str("max_height") == "true" {
                        true => vec![format!("max_height: {MAX_HEIGHT:?}")],
                        false => vec![],
                    }).hidden_when(windowed),
                    Control::switch("virtual_row_height").code(|_, values| match windowed(values) {
                        true => vec![
                            format!("max_height: {WINDOW_HEIGHT:?}"),
                            "virtual_row_height: 40.0".to_string(),
                        ],
                        false => vec![],
                    }),
                    Control::switch("onbottomreached").code(|_, values| match from_server(values) {
                        true => vec![
                            "loading: loading()".to_string(),
                            "onbottomreached: move |_| load()".to_string(),
                        ],
                        false => vec![],
                    }).hidden_when(|values| !windowed(values)),
                    Control::switch("pinned").code(|_, values| match values.str("pinned") == "true" {
                        true => vec![PINNED.to_string()],
                        false => vec![],
                    }),
                    Control::switch("selectable"),
                    Control::switch("multi_sort"),
                    Control::switch("column_menu"),
                    Control::switch("row_detail").code(|_, values| match values.str("row_detail") == "true" {
                        true => vec![DETAIL.to_string()],
                        false => vec![],
                    }).hidden_when(windowed),
                    Control::switch("show_quick_filter"),
                    // On, so the toolbar pieces and Export show without another switch.
                    Control::switch("toolbar").default("true").code(|_, values| match (values.str("toolbar") == "true", values.str("filter_panel") == "true") {
                        (true, true) => vec![TOOLBAR_FILTERS.to_string()],
                        (true, false) => vec![TOOLBAR.to_string()],
                        (false, _) => vec![],
                    }),
                    Control::switch("loading").hidden_when(from_server),
                    Control::switch("header_filters"),
                    Control::switch("filter_panel"),
                    Control::switch("paginate").code(|_, values| match values.str("paginate") == "true" {
                        true => vec!["page_sizes: vec![2, 5, 10]".to_string()],
                        false => vec![],
                    }).hidden_when(windowed),
                    Control::switch("empty").code(|_, values| match no_rows(values) {
                        true => vec![r#"empty: rsx! { "No team members yet." }"#.to_string()],
                        false => vec![],
                    }).hidden_when(windowed),
                    Control::switch("reorder_rows").code(|_, values| match reorders(values) {
                        true => vec![REORDER.to_string()],
                        false => vec![],
                    }).hidden_when(from_server),
                    Control::switch("resizable_columns").code(|_, values| match resizes(values) {
                        true => vec![
                            "resizable_columns: true".to_string(),
                            "column_widths: widths()".to_string(),
                            "oncolumnwidthschange: move |next: ColumnWidths| widths.set(next)".to_string(),
                        ],
                        false => vec![],
                    }),
                    // Last, as it prints the columns: no `column_groups` prop exists.
                    Control::switch("column_groups").code(|_, values| match values.str("column_groups") == "true" {
                        true => vec![GROUPED_COLUMNS.to_string()],
                        false => vec![COLUMNS.to_string()],
                    }),
                ],
                wrap: Wrap(wrap_data),
                render: move |values: DemoValues| rsx! {
                    TeamTable { values }
                },
            }
        }
    }
}
