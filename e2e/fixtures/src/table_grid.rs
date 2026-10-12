//! `Table` with `cell_focus` (1156 D1-D3): a grid of people between two buttons,
//! one with a button in each row, one selectable with column menus, a windowed
//! one of 500 rows, and one with editable Age and City cells, which stores its
//! edits or (`/kept`) only logs them, `/button` with a button in each City cell,
//! `/custom` with its own Age editor, `/windowed` with fixed 28px rows, `/formats`
//! with empty yes/no and date cells.

use dioxus::prelude::*;
use libero::{
    chrono::NaiveDate,
    components::{EditCtx, RowUpdate, Table, column},
    hooks::use_localization_handle,
    localization::Localization,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/table-grid", || rsx! { GridPage {} }),
    ("/table-grid/widgets", || rsx! { GridPage { widgets: true } }),
    ("/table-grid/selectable", || rsx! { GridPage { selectable: true } }),
    ("/table-grid/windowed", || rsx! { WindowedGridPage {} }),
    ("/table-grid/editable", || rsx! { EditableGridPage { store: true } }),
    ("/table-grid/editable/kept", || rsx! { EditableGridPage { store: false } }),
    ("/table-grid/editable/button", || rsx! { EditableGridPage { store: true, button: true } }),
    ("/table-grid/editable/custom", || rsx! { EditableGridPage { store: true, custom: true } }),
    ("/table-grid/editable/windowed", || rsx! { EditableGridPage { store: true, windowed: true } }),
    ("/table-grid/editable/formats", || rsx! { FormatsGridPage {} }),
    ("/table-grid/collation", || rsx! { CollationGridPage {} }),
];

#[derive(Clone, PartialEq)]
struct Person {
    name: &'static str,
    age: u32,
    city: &'static str,
}

fn people() -> Vec<Person> {
    [
        ("Ana", 31, "Lisbon"),
        ("Ben", 42, "Berlin"),
        ("Cleo", 27, "Cairo"),
        ("Dev", 55, "Delhi"),
        ("Eli", 38, "Essen"),
    ]
    .into_iter()
    .map(|(name, age, city)| Person { name, age, city })
    .collect()
}

#[component]
fn GridPage(#[props(default)] widgets: bool, #[props(default)] selectable: bool) -> Element {
    let mut columns = vec![
        column("Name").value(|p: &Person| p.name.to_string()).sortable().row_header(),
        column("Age").value(|p: &Person| p.age).sortable(),
        column("City").value(|p: &Person| p.city.to_string()),
    ];
    if widgets {
        columns.push(column("Action").value(|p: &Person| p.name.to_string()).render(|p: &Person| {
            rsx! {
                button { r#type: "button", "Greet {p.name}" }
            }
        }));
    }
    rsx! {
        button { id: "before", "Before" }
        Table {
            caption: "Team",
            cell_focus: true,
            selectable,
            column_menu: selectable,
            data: people(),
            columns,
            row_key: |p: &Person| p.name.to_string(),
        }
        button { id: "after", "After" }
    }
}

/// A custom Age editor: it marks itself invalid by `error` and clears it by `changed`.
#[component]
fn AgeEditor(ctx: EditCtx) -> Element {
    let mut draft = use_signal(|| ctx.value.clone());
    let commit = ctx.commit;
    let changed = ctx.changed;
    rsx! {
        input {
            id: "custom-editor",
            // iOS zooms the page into a smaller field and keeps the zoom for the unit's next scenarios.
            font_size: "16px",
            aria_label: ctx.label.clone(),
            aria_invalid: ctx.error.is_some().then_some("true"),
            aria_describedby: ctx.error.is_some().then(|| ctx.error_id.clone()),
            value: draft(),
            onmounted: move |mounted| async move {
                let _ = mounted.data().set_focus(true).await;
            },
            oninput: move |event| {
                draft.set(event.value());
                changed.call(());
            },
            onkeydown: move |event| {
                if event.key() == Key::Enter {
                    event.stop_propagation();
                    commit.call(draft());
                }
            },
        }
    }
}

#[component]
fn EditableGridPage(
    store: bool,
    #[props(default)] button: bool,
    #[props(default)] custom: bool,
    #[props(default)] windowed: bool,
) -> Element {
    let mut rows = use_signal(people);
    let mut log = use_signal(String::new);
    let age = column("Age")
        .value(|p: &Person| p.age)
        .editable(|p: &mut Person, age: u32| p.age = age)
        .validate(|_: &Person, age: &u32| match *age <= 150 {
            true => Ok(()),
            false => Err("Too old".into()),
        });
    let age = match custom {
        true => age.editor(|ctx: EditCtx| rsx! { AgeEditor { ctx } }),
        false => age,
    };
    let max_height = windowed.then(|| "300px".to_string());
    let virtual_row_height = windowed.then_some(28.0);
    let city = column("City")
        .value(|p: &Person| p.city.to_string())
        .editable(|p: &mut Person, city: String| p.city = city.leak());
    let city = match button {
        true => city.render(move |p: &Person| {
            let name = p.name;
            rsx! {
                button { r#type: "button", onclick: move |_| log.set(format!("Pinged {name}")), "Ping {name}" }
            }
        }),
        false => city,
    };
    rsx! {
        button { id: "before", "Before" }
        Table {
            caption: "Team",
            max_height,
            virtual_row_height,
            data: rows(),
            columns: vec![
                column("Name").value(|p: &Person| p.name.to_string()).row_header(),
                age,
                city,
            ],
            row_key: |p: &Person| p.name.to_string(),
            onrowupdate: move |update: RowUpdate<Person>| {
                log.set(format!(
                    "{} {} {}: {} {}",
                    update.key, update.index, update.column, update.row.age, update.row.city,
                ));
                if store {
                    rows.write()[update.index] = update.row;
                }
            },
        }
        output { id: "log", "{log}" }
        button { id: "after", "After" }
    }
}

#[derive(Clone, PartialEq)]
struct Task {
    title: &'static str,
    done: Option<bool>,
    due: Option<NaiveDate>,
}

#[component]
fn FormatsGridPage() -> Element {
    let mut rows = use_signal(|| {
        vec![Task {
            title: "Ship",
            done: None,
            due: None,
        }]
    });
    rsx! {
        Table {
            caption: "Tasks",
            data: rows(),
            columns: vec![
                column("Title").value(|t: &Task| t.title.to_string()).row_header(),
                column("Done")
                    .value(|t: &Task| t.done)
                    .editable(|t: &mut Task, done: Option<bool>| t.done = done),
                column("Due")
                    .value(|t: &Task| t.due)
                    .editable(|t: &mut Task, due: Option<NaiveDate>| t.due = due),
            ],
            row_key: |t: &Task| t.title.to_string(),
            onrowupdate: move |update: RowUpdate<Task>| rows.write()[update.index] = update.row,
        }
    }
}

/// A sortable word column in Swedish, where "Ä" sorts after "Z" (todo 3294).
#[component]
fn CollationGridPage() -> Element {
    let localization = use_localization_handle();
    use_effect(move || {
        localization.set(&Localization {
            lang: "sv",
            ..Localization::ENGLISH
        })
    });
    rsx! {
        Table {
            caption: "Words",
            data: vec!["Zoe", "Äpple", "Anna"],
            columns: vec![
                column("Word").value(|word: &&str| word.to_string()).sortable().row_header(),
            ],
            row_key: |word: &&str| word.to_string(),
        }
    }
}

#[derive(Clone, PartialEq)]
struct Item {
    id: u32,
}

#[component]
fn WindowedGridPage() -> Element {
    rsx! {
        Table {
            aria_label: "Items",
            cell_focus: true,
            max_height: "300px",
            virtual_row_height: 40.0,
            data: (1..=500).map(|id| Item { id }).collect::<Vec<_>>(),
            columns: vec![
                column("Id").value(|item: &Item| item.id).row_header(),
                column("Double").value(|item: &Item| item.id * 2),
                column("Square").value(|item: &Item| item.id * item.id),
            ],
            row_key: |item: &Item| item.id.to_string(),
        }
    }
}
