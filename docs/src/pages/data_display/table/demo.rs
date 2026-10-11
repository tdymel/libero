use super::{
    MAX_HEIGHT, SCROLL_WIDTH, WINDOW_HEIGHT, WINDOW_HIDES, edits, from_server, no_rows, reorders,
    resizes, windowed,
};
use crate::components::DemoValues;
use dioxus::prelude::*;
use std::cell::OnceCell;
use std::rc::Rc;
use std::time::Duration;

use libero::chrono::{NaiveDate, TimeDelta};
use libero::components::{
    Aggregate, Button, Chip, Column, ColumnFilter, ColumnWidths, PinnedColumns, RowFn, RowUpdate,
    Table, TableColumnsButton, TableDensityButton, TableExportButton, TableFilterButton, column,
};
use libero::hooks::SortableMove;
use libero::platform::{TimerSubscription, save_file, timer};
use libero::sx::sx;

// demo-code: person start

#[derive(Clone, PartialEq)]
struct Person {
    name: String,
    role: String,
    joined: NaiveDate,
    review: Option<NaiveDate>,
    active: bool,
    bonus: Option<f64>,
}

fn day(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap_or_default()
}
// demo-code: person end

// demo-code: team start
/// Six rows, so the date filters have dates on both sides to keep and drop.
fn team() -> Vec<Person> {
    let person = |name: &str, role: &str, joined, review, active, bonus| Person {
        name: name.into(),
        role: role.into(),
        joined,
        review,
        active,
        bonus,
    };
    vec![
        person(
            "Ada Lovelace",
            "Owner",
            day(2019, 3, 4),
            Some(day(2025, 1, 15)),
            true,
            Some(12.5),
        ),
        person(
            "Grace Hopper",
            "Admin",
            day(2021, 7, 19),
            Some(day(2024, 11, 2)),
            true,
            Some(8.0),
        ),
        person("Alan Turing", "Viewer", day(2023, 2, 1), None, false, None),
        person(
            "Katherine Johnson",
            "Admin",
            day(2020, 10, 12),
            Some(day(2025, 4, 30)),
            true,
            Some(6.5),
        ),
        person(
            "Edsger Dijkstra",
            "Viewer",
            day(2024, 5, 27),
            None,
            false,
            Some(2.0),
        ),
        person(
            "Barbara Liskov",
            "Owner",
            day(2018, 1, 8),
            Some(day(2024, 9, 9)),
            true,
            Some(10.0),
        ),
    ]
}
// demo-code: team end

// demo-code: member start
const ROLES: [&str; 3] = ["Owner", "Admin", "Viewer"];

/// Made-up person `n` of ten thousand.
fn member(n: u32) -> Person {
    Person {
        name: format!("Person {n}"),
        role: ROLES[n as usize % ROLES.len()].into(),
        joined: day(2015, 1, 1) + TimeDelta::days(i64::from(n % 3_650)),
        review: (!n.is_multiple_of(3))
            .then(|| day(2025, 1, 1) + TimeDelta::days(i64::from(n % 365))),
        active: !n.is_multiple_of(5),
        bonus: (!n.is_multiple_of(4)).then(|| f64::from(n % 150) / 10.0),
    }
}
// demo-code: member end

// demo-code: crowd start
/// Ten thousand made-up people for the `virtual_row_height` switch.
fn crowd() -> Vec<Person> {
    (1..=10_000).map(member).collect()
}
// demo-code: crowd end

// demo-code: server start
const BATCH: usize = 100;
/// How long the fake server takes.
const LATENCY: Duration = Duration::from_millis(600);

/// The fake server: `BATCH` people from `from` on, of ten thousand.
fn fetch(from: usize) -> Vec<Person> {
    (1..=10_000).skip(from).take(BATCH).map(member).collect()
}
// demo-code: server end

#[derive(Clone, PartialEq)]
struct Staff {
    name: String,
    role: String,
}

fn staff() -> Vec<Staff> {
    [
        ("Ada", "owner"),
        ("Bo", "admin"),
        ("Cy", "viewer"),
        ("Di", "admin"),
        ("Ed", "viewer"),
    ]
    .map(|(name, role)| Staff {
        name: name.to_string(),
        role: role.to_string(),
    })
    .to_vec()
}

fn role_label(role: &str) -> &'static str {
    match role {
        "owner" => "Owner",
        "admin" => "Admin",
        _ => "Viewer",
    }
}

/// The select column example: stored role codes, shown as names, filtered by the codes.
#[component]
pub fn StaffTable() -> Element {
    rsx! {
        Table {
            caption: "Staff",
            data: staff(),
            row_key: |s: &Staff| s.name.clone(),
            header_filters: true,
            default_column_filters: vec![ColumnFilter::any_of("Role", ["owner", "admin"])],
            columns: vec![
                column("Name").value(|s: &Staff| s.name.clone()).row_header(),
                column("Role")
                    .value(|s: &Staff| s.role.clone())
                    .format(|s: &Staff| role_label(&s.role).to_string())
                    .options([("owner", "Owner"), ("admin", "Admin"), ("viewer", "Viewer")]),
            ],
        }
    }
}

#[derive(Clone, PartialEq)]
struct Task {
    id: String,
    parent: Option<String>,
    name: String,
    owner: String,
    days: u32,
}

fn tasks() -> Vec<Task> {
    [
        ("site", "", "Website relaunch", "Ada", 0),
        ("design", "site", "Design", "Bo", 0),
        ("wire", "design", "Wireframes", "Bo", 3),
        ("visual", "design", "Visual design", "Cy", 5),
        ("build", "site", "Build", "Di", 0),
        ("front", "build", "Frontend", "Di", 8),
        ("back", "build", "Backend", "Ed", 6),
        ("launch", "site", "Launch", "Ada", 1),
        ("retro", "", "Retrospective", "Cy", 0),
    ]
    .map(task)
    .to_vec()
}

fn task((id, parent, name, owner, days): (&str, &str, &str, &str, u32)) -> Task {
    Task {
        id: id.to_string(),
        parent: (!parent.is_empty()).then(|| parent.to_string()),
        name: name.to_string(),
        owner: owner.to_string(),
        days,
    }
}

/// The tree example: tasks under their parent task, two levels open, one row's
/// children loaded when it first opens, one subtree partly selected.
#[component]
pub fn TaskTable() -> Element {
    let mut loaded = use_signal(|| false);
    let mut rows = tasks();
    if loaded() {
        rows.extend(
            [
                ("notes", "retro", "Notes", "Cy", 1),
                ("follow", "retro", "Follow-ups", "Ed", 2),
            ]
            .map(task),
        );
    }
    rsx! {
        Table {
            caption: "Tasks",
            data: rows,
            row_key: |t: &Task| t.id.clone(),
            parent_key: |t: &Task| t.parent.clone(),
            default_node_expansion: 2,
            row_has_children: |t: &Task| t.id == "retro",
            onexpandednodeschange: move |open: Vec<String>| {
                if open.iter().any(|key| key == "retro") {
                    loaded.set(true);
                }
            },
            selectable: true,
            default_selection: vec!["wire".to_string()],
            show_quick_filter: true,
            columns: vec![
                column("Task").value(|t: &Task| t.name.clone()).sortable().row_header(),
                column("Owner").value(|t: &Task| t.owner.clone()).sortable(),
                column("Days").value(|t: &Task| t.days).sortable(),
            ],
        }
    }
}

fn team_columns(grouped: bool, editable: bool) -> Vec<Column<Person>> {
    let group = |column: Column<Person>| match grouped {
        true => column.group("Membership"),
        false => column,
    };
    let edit = |column: Column<Person>| match editable {
        true => column
            .editable(|p: &mut Person, bonus: Option<f64>| p.bonus = bonus)
            .validate(|_: &Person, bonus: &Option<f64>| {
                match bonus.is_none_or(|bonus| (0.0..=100.0).contains(&bonus)) {
                    true => Ok(()),
                    false => Err("A percentage from 0 to 100".into()),
                }
            }),
        false => column,
    };
    let edit_review = |column: Column<Person>| match editable {
        true => column.editable(|p: &mut Person, review: Option<NaiveDate>| p.review = review),
        false => column,
    };
    let edit_active = |column: Column<Person>| match editable {
        true => column.editable(|p: &mut Person, active: bool| p.active = active),
        false => column,
    };
    vec![
        column("Name")
            .value(|p: &Person| p.name.clone())
            .sortable()
            .row_header()
            .aggregate(Aggregate::Count),
        group(
            column("Role")
                .value(|p: &Person| p.role.clone())
                .sortable()
                .options(["Owner", "Admin", "Viewer"])
                .render(|p: &Person| rsx! { Chip { size: "xs", "{p.role}" } }),
        ),
        group(column("Joined").value(|p: &Person| p.joined).sortable()),
        group(edit_review(
            column("Review").value(|p: &Person| p.review).sortable(),
        )),
        group(edit_active(
            column("Active")
                .value(|p: &Person| p.active)
                .format(|p: &Person| (if p.active { "Yes" } else { "No" }).to_string())
                .sortable(),
        )),
        group(edit(
            column("Bonus")
                .value(|p: &Person| p.bonus)
                .format(|p: &Person| p.bonus.map(|b| format!("{b:.1} %")).unwrap_or_default())
                .aggregate(Aggregate::Avg)
                .aggregate_format(|b| format!("{b:.1} %"))
                .sortable(),
        )),
    ]
}

/// The main demo's table: a component, so reordered rows and resized widths have a signal to land in.
#[component]
pub fn TeamTable(values: DemoValues) -> Element {
    let mut rows = use_signal(team);
    // Built on the first flip only, then kept: a flip re-renders the table, not the data.
    let many = use_hook(|| Rc::new(OnceCell::<Vec<Person>>::new()));
    // The ten thousand once reordered, else `many` as built.
    let mut moved = use_signal(|| None::<Vec<Person>>);
    let mut widths = use_signal(ColumnWidths::new);
    // demo-code: loader start
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
    // demo-code: loader end
    let windowed = windowed(&values);
    let server = from_server(&values);
    // A switch the windowed table hides counts as off, `loading` too while the server sets it.
    let on = |name: &str| {
        values.str(name) == "true"
            && !(windowed && WINDOW_HIDES.contains(&name))
            && !(server && name == "loading")
    };
    // Keyed in a loop, as a root's own key never remounts: pages, pins and grouping seed once.
    let remount = format!(
        "{}-{}-{}-{windowed}-{server}",
        values.str("paginate"),
        values.str("pinned"),
        on("row_grouping")
    );
    rsx! {
        for remount in [remount] {
            Table {
                key: "{remount}",
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
                cell_focus: on("cell_focus"),
                column_menu: on("column_menu"),
                row_detail: match on("row_detail") {
                    true => (|p: &Person| Some(rsx! { "{p.name} joined as {p.role}." })).into(),
                    false => RowFn::default(),
                },
                onrowreorder: reorders(&values)
                    .then(|| {
                        let many = many.clone();
                        EventHandler::new(move |step: SortableMove| match windowed {
                            true => {
                                step.apply(
                                    moved
                                        .write()
                                        .get_or_insert_with(|| many.get_or_init(crowd).clone()),
                                )
                            }
                            false => step.apply(&mut rows.write()),
                        })
                    }),
                onrowupdate: edits(&values)
                    .then(|| {
                        let many = many.clone();
                        EventHandler::new(move |update: RowUpdate<Person>| match windowed {
                            true => {
                                moved
                                    .write()
                                    .get_or_insert_with(|| many.get_or_init(crowd).clone())[update
                                    .index] = update.row
                            }
                            false => rows.write()[update.index] = update.row,
                        })
                    }),
                default_row_grouping: match on("row_grouping") {
                    true => vec!["Role".to_string()],
                    false => Vec::new(),
                },
                default_group_expansion: 1,
                show_quick_filter: on("show_quick_filter"),
                toolbar: on("toolbar").then(|| rsx! {
                    TableColumnsButton {}
                    TableDensityButton {}
                    TableExportButton {
                        onexport: move |csv: String| {
                            spawn(async move {
                                save_file("team.csv", "text/csv", csv.into_bytes()).await;
                            });
                        },
                    }
                    if on("filter_panel") {
                        TableFilterButton {}
                    }
                    Button { variant: "outlined", size: "sm", "Add member" }
                }),
                loading: on("loading") || (server && loading()),
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
                columns: team_columns(on("column_groups"), edits(&values)),
            }
        }
    }
}
