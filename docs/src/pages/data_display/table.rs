use crate::components::{
    Control, Demo, DemoFile, DemoValues, DocPage, DocSection, Wrap, a11y, prop, props,
};
use dioxus::prelude::*;
use libero::components::{Code, Text};
use libero::use_theme;

mod demo;
use demo::{StaffTable, TaskTable, TeamTable};

/// The live table and its data, printed from its text by `wrap_data`.
const FILE: DemoFile = DemoFile(include_str!("table/demo.rs"));

/// People a batch at a time from the fake server, with `virtual_row_height`.
fn from_server(values: &DemoValues) -> bool {
    windowed(values) && values.str("onbottomreached") == "true"
}

// snippet: item use libero::chrono::NaiveDate;
// snippet: item #[derive(Clone, PartialEq)] struct Person { name: String, role: String, joined: NaiveDate, review: Option<NaiveDate>, active: bool, bonus: Option<f64> }
// snippet: let people: Vec<Person> = Vec::new();
// snippet: in Table { caption: "Team members", data: people, .. }
const COLUMNS: &str = r#"columns: vec![
    column("Name")
        .value(|p: &Person| p.name.clone())
        .sortable()
        .row_header()
        .aggregate(Aggregate::Count),
    column("Role")
        .value(|p: &Person| p.role.clone())
        .sortable()
        .options(["Owner", "Admin", "Viewer"])
        .render(|p: &Person| rsx! { Chip { size: "xs", "{p.role}" } }),
    column("Joined").value(|p: &Person| p.joined).sortable(),
    column("Review").value(|p: &Person| p.review).sortable(),
    column("Active")
        .value(|p: &Person| p.active)
        .format(|p: &Person| (if p.active { "Yes" } else { "No" }).to_string())
        .sortable(),
    column("Bonus")
        .value(|p: &Person| p.bonus)
        .format(|p: &Person| p.bonus.map(|b| format!("{b:.1} %")).unwrap_or_default())
        .aggregate(Aggregate::Avg)
        .aggregate_format(|b| format!("{b:.1} %"))
        .sortable(),
]"#;

// snippet: item use libero::chrono::NaiveDate;
// snippet: item #[derive(Clone, PartialEq)] struct Person { name: String, role: String, joined: NaiveDate, review: Option<NaiveDate>, active: bool, bonus: Option<f64> }
// snippet: let people: Vec<Person> = Vec::new();
// snippet: in Table { caption: "Team members", data: people, .. }
const GROUPED_COLUMNS: &str = r#"columns: vec![
    column("Name")
        .value(|p: &Person| p.name.clone())
        .sortable()
        .row_header()
        .aggregate(Aggregate::Count),
    column("Role")
        .value(|p: &Person| p.role.clone())
        .sortable()
        .options(["Owner", "Admin", "Viewer"])
        .render(|p: &Person| rsx! { Chip { size: "xs", "{p.role}" } })
        .group("Membership"),
    column("Joined").value(|p: &Person| p.joined).sortable().group("Membership"),
    column("Review").value(|p: &Person| p.review).sortable().group("Membership"),
    column("Active")
        .value(|p: &Person| p.active)
        .format(|p: &Person| (if p.active { "Yes" } else { "No" }).to_string())
        .sortable()
        .group("Membership"),
    column("Bonus")
        .value(|p: &Person| p.bonus)
        .format(|p: &Person| p.bonus.map(|b| format!("{b:.1} %")).unwrap_or_default())
        .aggregate(Aggregate::Avg)
        .aggregate_format(|b| format!("{b:.1} %"))
        .sortable()
        .group("Membership"),
]"#;

const SELECT_FIXED: [&str; 6] = [
    r#"caption: "Staff""#,
    "data: staff",
    r#"row_key: |s: &Staff| s.name.clone()"#,
    "header_filters: true",
    r#"default_column_filters: vec![ColumnFilter::any_of("Role", ["owner", "admin"])]"#,
    r#"columns: vec![
    column("Name").value(|s: &Staff| s.name.clone()).row_header(),
    column("Role")
        .value(|s: &Staff| s.role.clone())
        .format(|s: &Staff| role_label(&s.role).to_string())
        .options([("owner", "Owner"), ("admin", "Admin"), ("viewer", "Viewer")]),
]"#,
];

const TREE_FIXED: [&str; 11] = [
    r#"caption: "Tasks""#,
    "data: tasks",
    r#"row_key: |t: &Task| t.id.clone()"#,
    r#"parent_key: |t: &Task| t.parent.clone()"#,
    "default_node_expansion: 2",
    r#"row_has_children: |t: &Task| t.id == "retro""#,
    r#"onexpandednodeschange: move |open: Vec<String>| {
    if open.iter().any(|key| key == "retro") {
        loaded.set(true);
    }
}"#,
    "selectable: true",
    r#"default_selection: vec!["wire".to_string()]"#,
    "show_quick_filter: true",
    r#"columns: vec![
    column("Task").value(|t: &Task| t.name.clone()).sortable().row_header(),
    column("Owner").value(|t: &Task| t.owner.clone()).sortable(),
    column("Days").value(|t: &Task| t.days).sortable(),
]"#,
];

/// The rows are the fixture, and the snippet only compiles with them, so the
/// code block carries the struct and the data above the `Table` itself.
fn wrap_data(values: &DemoValues, code: &str) -> String {
    let mut items = vec![
        match windowed(values) {
            true => "use libero::chrono::{NaiveDate, TimeDelta};",
            false => "use libero::chrono::NaiveDate;",
        }
        .to_string(),
        FILE.section("person"),
    ];
    let rows = match windowed(values) {
        true => "crowd",
        false => "team",
    };
    // Reordered or edited rows and resized widths need a signal to land in.
    let mut people = match (no_rows(values), reorders(values) || edits(values)) {
        _ if from_server(values) => {
            items.extend(["member", "server"].map(|name| FILE.section(name)));
            format!("{}\nlet people = loaded();", FILE.section("loader"))
        }
        (true, false) => "let people: Vec<Person> = Vec::new();".to_string(),
        (false, false) => format!("let people = {rows}();"),
        (true, true) => {
            "let mut rows = use_signal(Vec::<Person>::new);\nlet people = rows();".to_string()
        }
        (false, true) => format!("let mut rows = use_signal({rows});\nlet people = rows();"),
    };
    if !from_server(values) && !no_rows(values) {
        match windowed(values) {
            true => items.extend(["member", "crowd"].map(|name| FILE.section(name))),
            false => items.push(FILE.section("team")),
        }
    }
    if resizes(values) {
        people.push_str("\nlet mut widths = use_signal(ColumnWidths::new);");
    }
    if values.str("toolbar") == "true" {
        people.push_str(
            "\n// The export's CSV, yours to save.\nlet mut csv = use_signal(String::new);",
        );
    }
    items.push(people);
    items.push(code.to_string());
    items.join("\n\n")
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

// snippet: item #[derive(Clone, PartialEq)] struct Person { name: String }
// snippet: in Table { caption: "Team members", data: Vec::<Person>::new(), columns: vec![], .. }
const GROUPING: &str = r#"default_row_grouping: vec!["Role".to_string()],
default_group_expansion: 1"#;

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

/// The server's batches have no signal of the demo's own to store an edit in.
fn edits(values: &DemoValues) -> bool {
    values.str("editable") == "true" && !from_server(values)
}

// snippet: item #[derive(Clone, PartialEq)] struct Person { name: String }
// snippet: let mut rows = use_signal(Vec::<Person>::new);
// snippet: in Table { caption: "Team members", data: rows(), columns: vec![], row_key: |p: &Person| p.name.clone(), .. }
const EDIT: &str =
    "onrowupdate: move |update: RowUpdate<Person>| rows.write()[update.index] = update.row";

/// The Bonus column's setter, after its `.sortable()`.
const BONUS_EDIT: &str = "\n        .editable(|p: &mut Person, bonus: Option<f64>| p.bonus = bonus)\n        .validate(|_: &Person, bonus: &Option<f64>| match bonus.is_none_or(|b| (0.0..=100.0).contains(&b)) {\n            true => Ok(()),\n            false => Err(\"A percentage from 0 to 100\".into()),\n        })";

/// The Review column's setter, after its `.sortable()`.
const REVIEW_EDIT: &str =
    "\n        .editable(|p: &mut Person, review: Option<NaiveDate>| p.review = review)";

const REVIEW_COLUMN: &str = "column(\"Review\").value(|p: &Person| p.review).sortable()";

/// The Active column up to its `.sortable()`, and its setter.
const ACTIVE_COLUMN: &str = "column(\"Active\")\n        .value(|p: &Person| p.active)\n        .format(|p: &Person| (if p.active { \"Yes\" } else { \"No\" }).to_string())\n        .sortable()";
const ACTIVE_EDIT: &str = "\n        .editable(|p: &mut Person, active: bool| p.active = active)";

/// `columns`, the Review, Active and Bonus columns editable with the `editable` switch.
fn columns_code(columns: &str, values: &DemoValues) -> String {
    match edits(values) {
        true => {
            let columns = columns
                .replacen(REVIEW_COLUMN, &format!("{REVIEW_COLUMN}{REVIEW_EDIT}"), 1)
                .replacen(ACTIVE_COLUMN, &format!("{ACTIVE_COLUMN}{ACTIVE_EDIT}"), 1);
            let at = columns
                .rfind(".sortable()")
                .map_or(columns.len(), |at| at + ".sortable()".len());
            format!("{}{BONUS_EDIT}{}", &columns[..at], &columns[at..])
        }
        false => columns.to_string(),
    }
}

#[component]
pub fn TablePage() -> Element {
    let theme = use_theme();
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
                    prop("toolbar", "Option<Element>").default("None").doc("A row above the table for your own controls, say an export or add button. With `show_quick_filter` the search field joins it at the end. It wraps on a narrow screen and stays put while the table scrolls. `TableColumnsButton`, `TableDensityButton`, `TableFilterButton` and `TableExportButton` work only in here. `TableColumnsButton` opens a checklist that shows and hides the hideable columns; `TableDensityButton` picks compact, standard or comfortable rows, `size` small, medium or large."),
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
                    prop("onrowreorder", "EventHandler<SortableMove>").default("None").doc("Adds a leading column with a drag handle and Move up and Move down buttons per row. Called with a move by positions in `data`; apply it with `step.apply(&mut rows)`, the table shows the old order until you do. Off while the rows are sorted, filtered or grouped. Paged, a row moves within its page. Set `row_key` with it."),
                    prop("onrowupdate", "EventHandler<RowUpdate<T>>").default("None").doc("Called when an `.editable(..)` cell commits, with a `RowUpdate`: the row's `key`, its `index` in `data`, the edited `column`'s header and `row`, a clone of the row with the new value set. Store it, `rows.write()[update.index] = update.row`; the table keeps no copy, so the cell shows `data` again until you do. An unchanged value calls nothing."),
                    prop("row_grouping", "Option<Vec<String>>").default("None").doc("Groups the rows by the cell text of these headers, outermost first: `vec![\"Role\".into()]`. Each group gets a row of its own, with a toggle in the first shown column naming its text and row count; rows with an empty cell form one group, \"(empty)\". Groups follow the sorted rows, so sorting by the grouped column orders them, and a filter drops the groups it empties. The grouped column stays shown. With `selectable`, each group row gets a box that selects or clears the group's rows, open or not, checked when all are selected and mixed when some are. With `virtual_row_height`, group rows take one row height each and count in `aria-rowindex`. Rows don't reorder while grouped. Paged, group rows take places on a page like rows, and a group split by a page break gets no second group row. The footer and an export cover the rows alone. Set, the grouping is controlled: pair it with `onrowgroupingchange`."),
                    prop("default_row_grouping", "Vec<String>").default("[]").doc("Seeds the grouping once. Ignored when `row_grouping` is set."),
                    prop("onrowgroupingchange", "EventHandler<Vec<String>>").default("None").doc("Called with the grouping a pick asks for."),
                    prop("expanded_groups", "Option<Vec<String>>").default("None").doc("The open groups' ids: their texts from the outermost group in, joined by `/`, as `\"Admin\"` or `\"Admin/2024\"`; a `/` or `\\` in a text is escaped by `\\`, and the empty group's text is empty. Set, it is controlled: pair it with `onexpandedgroupschange`."),
                    prop("default_expanded_groups", "Vec<String>").default("[]").doc("Seeds the open groups once. Ignored when `expanded_groups` is set."),
                    prop("onexpandedgroupschange", "EventHandler<Vec<String>>").default("None").doc("Called with the open groups a group toggle asks for."),
                    prop("default_group_expansion", "usize").default("0").doc("How many levels of groups start open, the first time there are rows to group, beside `default_expanded_groups`: 1 opens the outermost groups. 0 starts every group closed. Ignored with `expanded_groups`."),
                    prop("parent_key", "RowFn<T, Option<String>>").default("None").doc("A row's parent's `row_key`, `None` for a root: the rows nest under their parents, each level in the sort, behind a toggle in the first shown column named by the row and its child count. A key no row has, or a cycle, makes roots; a debug build warns. A filter keeps the kept rows' parents and opens them. Set `row_key` with it. Over `row_grouping`, which it ignores and drops from the column menu; a tree shows no `row_detail` and its rows don't reorder. With `virtual_row_height` its rows window as flat rows do. With `selectable`, a row's box selects the row and its descendants the filter keeps; a parent turns selected once all its descendants are and clears when one clears, and its box is mixed while only some of the row and its descendants are selected. A row's `aria-selected` tells its own selection. The table is a `role=\"treegrid\"` whose rows are its stops: the arrows move between them, open and close them."),
                    prop("row_has_children", "RowFn<T, bool>").default("None").doc("Whether a tree row has children not in `data` yet, from a `|row: &T| ..` closure: the row gets a closed toggle, named by the row alone until its children load. Opening it calls `onexpandednodeschange` with its key: append its children to `data`, their `parent_key` naming it, or return `false` for it once it has none. Until either, the open row stays `aria-busy`. A row with children in `data` is a parent whatever this returns."),
                    prop("expanded_nodes", "Option<Vec<String>>").default("None").doc("The `row_key`s of the open tree rows. Set, it is controlled: pair it with `onexpandednodeschange`."),
                    prop("default_expanded_nodes", "Vec<String>").default("[]").doc("Seeds the open tree rows once. Ignored when `expanded_nodes` is set."),
                    prop("onexpandednodeschange", "EventHandler<Vec<String>>").default("None").doc("Called with the open tree rows a toggle asks for."),
                    prop("default_node_expansion", "usize").default("0").doc("How many tree levels start open, the first time there are rows, beside `default_expanded_nodes`: 1 opens the roots. 0 starts every row closed. Ignored with `expanded_nodes`."),
                    prop("cell_focus", "bool").default("false").doc("Makes the table a `role=\"grid\"` whose header and body cells are one tab stop: Tab enters on one cell, the first header cell until focus moves, and the arrows, Home, End, Ctrl+Home, Ctrl+End, PageUp and PageDown move it. On a header cell Enter or Space sorts and Ctrl+Enter opens its column menu; on a body cell Shift+Space selects the row with `selectable`. The stop stays where it was left and follows its row through a sort. A button or field in a cell stays a tab stop of its own, and a widget that takes the arrows keeps them. The leading checkbox, detail and reorder cells, group rows, the filter row and the footer stay out of the key map. Paged, `aria-rowcount` counts every page's rows; with a hidden column each cell carries its `aria-colindex`. An editable column turns it on. Give the rows a `.row_header()` column."),
                    prop("size", "Size").default(theme.table.size.as_str()).doc("Cell padding and font size."),
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
                    prop("manual_pagination", "bool").default("false").doc("`data` is the current page only: the table draws the page controls and leaves the slicing to you. A column's `aggregate` then covers this page only."),
                    prop("row_count", "Option<usize>").default("data's length").doc("Rows over all pages with `manual_pagination`, for the page count and the range text."),
                    prop("hidden_columns", "Option<Vec<String>>").default("None").doc("The hidden columns' headers. Set, visibility is controlled: pair it with `onhiddencolumnschange`. A hidden sorted column keeps sorting."),
                    prop("default_hidden_columns", "Vec<String>").default("[]").doc("Seeds the hidden columns once. Ignored when `hidden_columns` is set."),
                    prop("onhiddencolumnschange", "EventHandler<Vec<String>>").default("None").doc("Called with the hidden columns a column menu pick asks for."),
                    prop("quick_filter", "Option<String>").default("None").doc("The quick filter's text. A row stays when every word occurs, ignoring case, in the text of one of its shown, `filterable` cells. Set, the filter is controlled: pair it with `onquickfilterchange`. The rows are filtered, then sorted, then paged."),
                    prop("default_quick_filter", "String").default("\"\"").doc("Seeds the quick filter once. Ignored when `quick_filter` is set."),
                    prop("onquickfilterchange", "EventHandler<String>").default("None").doc("Called with the text typed into the quick-filter field."),
                    prop("show_quick_filter", "bool").default("false").doc("Puts a search field above the table that drives the quick filter."),
                    prop("manual_filter", "bool").default("false").doc("`data` comes filtered, say from a server: the quick filter and the column filters only report through `onquickfilterchange` and `oncolumnfilterschange`. Pair it with `manual_pagination` and `row_count` when paged."),
                    prop("column_filters", "Option<Vec<ColumnFilter>>").default("None").doc("Filters by header: `ColumnFilter::new(\"Age\", FilterOperator::GreaterThan, \"30\")`, or `ColumnFilter::between(\"Joined\", \"2026-01-01\", \"2026-06-30\")` for a date range. A row stays when it passes all of them (or any, with `filter_logic`) and the quick filter. Dates take an ISO day. Text operators ignore case, `Equals` too; number operators compare the value, not its formatted text, and take `1,5` as 1.5. An empty value, a number that does not parse, or an operator the column's type does not offer keeps every row. A hidden column's filter keeps filtering. Set, the filters are controlled: pair them with `oncolumnfilterschange`. The operators follow the column's type, such as contains or starts with for text, greater than for numbers, before, after, on or before, on or after and between for a `chrono::NaiveDate`, yes or no for booleans, is, is not and is any of for a select column (`ColumnFilter::any_of(\"Role\", [\"Admin\", \"Owner\"])`). A date's value field is a `DateField`, the native date input on a WebView. A `between` end left empty is open, and ends the wrong way round are swapped. A column may hold several filters: the popover and the header field edit its first, `filter_panel` all of them."),
                    prop("default_column_filters", "Vec<ColumnFilter>").default("[]").doc("Seeds the column filters once. Ignored when `column_filters` is set."),
                    prop("oncolumnfilterschange", "EventHandler<Vec<ColumnFilter>>").default("None").doc("Called with the filters a filter popover or a header filter asks for. Typed values arrive once typing pauses for 300 ms, an operator or yes/no pick at once."),
                    prop("filter_logic", "Option<FilterLogic>").default("None").doc("How the column filters join: `FilterLogic::And` keeps a row that passes all of them, `Or` one that passes any. The quick filter applies on top either way. Set, it is controlled."),
                    prop("default_filter_logic", "FilterLogic").default("And").doc("Seeds the filter logic once. Ignored when `filter_logic` is set."),
                    prop("onfilterlogicchange", "EventHandler<FilterLogic>").default("None").doc("Called with the filter logic a pick asks for."),
                    prop("header_filters", "bool").default("false").doc("Adds a row of filter fields under the headers, one per `filterable` column: a text field, Any/Yes/No for a boolean column, or Any and the options for a select column (one multi-select showing the first pick and a count once the popover picked Is any of). A field edits its column's filter with the operator the filter popover set, else the type's first: Contains for text, Equals for numbers."),
                    prop("filter_panel", "bool").default("false").doc("A Filters button at the start of the toolbar row, showing the active filters' count, opens a dialog of every column filter; with a `toolbar`, put `TableFilterButton` in it where the button goes instead, one line each: column, operator, value and a remove button. Add filter appends a line for the first column without one; from two lines a Match pick sets `filter_logic`. Lines apply as the popover does. `column_menu`'s Filter then opens the panel on its column's line instead of the popover."),
                    prop("pinned_columns", "Option<PinnedColumns>").default("None").doc("The columns held at the table's start and end edges while the rest scroll sideways, by header: `PinnedColumns::default().start([..]).end([..])`. Start is the left in a left-to-right page, the right in a right-to-left one. Set, pinning is controlled: pair it with `onpinnedcolumnschange`. Pair it with `scroll` or `max_height`, and give every pinned column but the outermost on its side a `width`, which it then keeps exactly. Pins that leave the other columns under a third of the scroll area's width are released, the innermost first and the end side before the start, and hold again once it grows; the checkbox column stays pinned."),
                    prop("default_pinned_columns", "PinnedColumns").default("none pinned").doc("Seeds the pinned columns once. Ignored when `pinned_columns` is set."),
                    prop("onpinnedcolumnschange", "EventHandler<PinnedColumns>").default("None").doc("Called with the pinned columns a column menu pick asks for."),
                    prop("column_order", "Option<Vec<String>>").default("None").doc("The headers in display order. Unlisted columns follow the listed ones in `columns` order, and pinned columns keep their pinned order. Set, the order is controlled: pair it with `oncolumnorderchange`. The sort, hidden and pinned columns name headers, so they follow a moved column. With `column_menu`, a grip at each unpinned header's start edge also drags the column to a gap."),
                    prop("default_column_order", "Vec<String>").default("[]").doc("Seeds the column order once. Ignored when `column_order` is set."),
                    prop("oncolumnorderchange", "EventHandler<Vec<String>>").default("None").doc("Called with the order a column menu's Move left or Move right, or a header drag, asks for, every header listed."),
                    prop("resizable_columns", "bool").default("false").doc("Puts a visible grip on each header's end edge, 24px wide on a touch screen, and Widen column, Narrow column (50px steps, the menu stays open) and Reset width in its `column_menu`, the drag-free way. Drag a grip, or double-click it to reset. A focused grip takes the keys: Left and Right move it 10px the way they point, 50px with Shift, Home and End go to the limits. A column opts out with `.resizable(false)` and sets its range with `.resize_limits(min, max)` in px, 50 to unbounded by default. Auto layout never draws a column narrower than its content. Blitz: use the menu, the grip does not drag reliably there."),
                    prop("column_widths", "Option<ColumnWidths>").default("None").doc("Resized widths in px by header, a `BTreeMap<String, f64>`, over the columns' own `width`. Set, the widths are controlled: pair them with `oncolumnwidthschange`."),
                    prop("default_column_widths", "ColumnWidths").default("{}").doc("Seeds the widths once. Ignored when `column_widths` is set."),
                    prop("oncolumnwidthschange", "EventHandler<ColumnWidths>").default("None").doc("Called with the widths a grip drag asks for when it ends, each grip key, and with each Widen, Narrow or Reset width."),
                    prop("column_menu", "bool").default("false").doc("Puts a menu button in each header: sort ascending or descending, unsort, add to the sort with `multi_sort`, filter a `filterable` column, group the rows by it or stop (a new grouping level goes innermost), move the column left or right past the next shown one, widen, narrow or reset it with `resizable_columns`, pin to the start or end or unpin, hide the column, and a Columns submenu that shows or hides the others. A pinned column has no move entries. Filter opens a popover with the operators of the column's type, a value and Clear; a filtered column's header then shows a filter button that reopens it."),
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
                    prop("header_render", "fn() -> Element").default("None").doc("Replaces the header's body, inside the sort button when sortable, so keep it non-interactive then. The header text stays the column's name in `TableSort`. Capture signals, not values: the closure is not compared, so a changed value does not redraw the header."),
                    prop("of", "&ColumnType<V>").default("None").doc("Before `value`: starts the column from a shared `const` type, its alignment, widths and a `format` over the value. `V` must match `value`'s; the column's own settings win."),
                    prop("row_header", "bool").default("false").doc("Renders the column's cells as `th scope=\"row\"`, so a screen reader names each row by it. One per table, usually the first."),
                    prop("editable", "fn(&mut T, V)").default("None").doc("Makes the column's cells editable, turning on `cell_focus`: a double click, a second tap, or Enter or F2 on the focused cell, opens a text field over the cell's value, Enter, Tab or leaving it commits through `onrowupdate`, Escape puts it back. `V: EditValue` parses the text: text, finite numbers, `bool` and `Option<bool>` (a checkbox: Space toggles, an `Option<bool>` one cycles empty, true, false), a `NaiveDate` (`2026-03-01`, the field names its form as a placeholder), and `Option<V>`, empty for `None`. Text that does not parse keeps the field open, marked invalid; leaving the field instead puts the old value back and announces it. The other cells read as read-only."),
                    prop("validate", "fn(&T, &V) -> Result<(), String>").default("None").doc("After `editable`: checks a parsed value against the row before the commit; an `Err` keeps the field open, marked invalid."),
                    prop("editor", "fn(EditCtx) -> Element").default("the text field").doc("After `editable`: draws your own editor in the cell. `EditCtx` gives the start `value` as text, a `label` to name it by, \"Edit Age, Ana\", and `commit` (the new text) and `cancel`. Focus it on mount; a commit that does not parse or validate keeps it open and sets `error`, the message the table shows under it: mark the field invalid with it, point its `aria-describedby` at `error_id` while `error` is set, and call `changed` when its text changes to clear it."),
                    prop("filterable", "bool").default("true").doc("Whether the quick filter searches the column's cell text, and whether it takes a column filter. Off for ids and codes that would match by accident."),
                    prop("options", "Vec<SelectOption>").default("[]").doc("Makes it a select column: its filter picks among these options, with the operators is, is not and is any of. A `SelectOption` is a `&str`, or a `(value, label)` pair: values match the raw cell text, before `format`, and are what a `ColumnFilter` holds; labels are shown. The quick filter and sorting still read the cell."),
                    prop("hideable", "bool").default("true").doc("Whether the column menu offers to hide the column. `hidden_columns` still hides it."),
                    prop("row_groupable", "bool").default("true").doc("Whether the column menu offers to group the rows by the column. `row_grouping` still groups by it."),
                    prop("group", "String").default("None").doc("Puts the column under a group header, shared with the adjacent columns of the same groups. Call it once per level, outermost first. The same name under another parent is another group, and a hidden column leaves its group."),
                    prop("col_span", "fn(&T) -> usize").default("None").doc("How many shown columns a row's cell covers, from this one on, say a total row's label. The covered cells are left out; the span stops at the row's end. Capture signals, not values: the closure is not compared."),
                    prop("aggregate", "Aggregate").default("None").doc("Adds a footer cell under the column that sums up its rows: `Sum`, `Avg`, `Min` or `Max` of the numeric cells, or `Count` of the filled ones. It covers the rows that pass the quick filter and the column filters, on every page. With `row_grouping`, each group row shows it too, over the group's rows, open or not. With `manual_pagination` it covers the loaded rows only: the table shows no total of rows it does not hold. A column without an aggregate gets an empty footer cell; with none at all there is no footer. The footer sticks to the bottom of a `max_height` table, as the header sticks to the top. Each cell names its aggregate before the value, from the localized `table.aggregate_name`."),
                    prop("aggregate_format", "fn(f64) -> String").default("the number").doc("The footer cell's text from the aggregate, say a price with its currency; by default the number as Rust prints it, so format a `Sum` or `Avg` of fractions. Not compared: capture signals, not values."),
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
                .key(["Enter", "Space"], "With `row_grouping`, on a group's toggle: opens or closes the group. Focus stays on the toggle.")
                .key(["Enter", "Space", "Down"], "With `column_menu`, on a header's menu button: opens the column menu, keyed like `Menu`.")
                .key(["Space", "Enter"], "With `onrowreorder`, on a row's handle: lifts the row, then drops it. Up and Down move the lifted row, Home and End to the first or last place, Escape puts it back.")
                .key(["Left", "Right"], "With `resizable_columns`, on a header's resize grip: moves it 10px the way the arrow points, 50px with Shift.")
                .key(["Home", "End"], "On a resize grip: narrows the column to its minimum, or widens it to its maximum when it has one.")
                .key(["Left", "Right", "Up", "Down"], "With `cell_focus`, on a header or body cell or a button in one: moves to the next cell that way, stopping at the edges. A cell spanning columns or header rows is one stop.")
                .key(["Home", "End"], "With `cell_focus`, on a cell: moves to the row's first or last cell.")
                .key(["Ctrl+Home", "Ctrl+End"], "With `cell_focus`: moves to the first header cell or the last row's last cell, scrolling it into view.")
                .key(["PageUp", "PageDown"], "With `cell_focus`: moves up or down by the rows one view shows, 10 when the view cannot be measured.")
                .key(["Enter", "Space"], "With `cell_focus`, on a sortable header cell itself: sorts by that column, as its button does. Focus stays on the cell.")
                .key(["Ctrl+Enter"], "With `cell_focus` and `column_menu`, on a header cell: opens its column menu.")
                .key(["Shift+Space"], "With `cell_focus` and `selectable`, on a body cell: selects or deselects its row.")
                .key(["Right"], "With `parent_key`, on a row: opens a closed row; on an open one moves to its first child, or with `cell_focus` to its first cell.")
                .key(["Left"], "With `parent_key`, on a row: closes an open row, else moves to its parent. With `cell_focus`, on a row's first cell: moves to the row.")
                .key(["Up", "Down", "Home", "End", "PageUp", "PageDown"], "With `parent_key`, on a row: moves to the row before or after it, the first or last row, or by the rows one view shows. Ctrl+Home and Ctrl+End do as Home and End.")
                .key(["Enter"], "With `parent_key` and `onrowclick`, on a row: what a click on it does.")
                .key(["Enter", "F2"], "On an editable cell: opens its editor, the field focused and its text selected, so typing replaces it. A second tap on the stop cell does the same on a touch screen.")
                .key(["Enter", "Shift+Enter"], "In a cell's editor: commits the value and moves to the cell below or above. A value that does not parse or validate keeps the editor open, with its message under it.")
                .key(["Tab", "Shift+Tab"], "In a cell's editor: commits the value and moves to the next or previous cell; the editor of that one opens with `Enter`.")
                .key(["Space"], "In a `bool` or `Option<bool>` cell's editor, a checkbox: toggles it. An `Option<bool>` one cycles empty, true, false.")
                .key(["Escape"], "In a cell's editor: puts the value back and returns focus to the cell.")
                .key(["Escape"], "In a filter popover: closes it and returns focus to the button that opened it: the column's menu button, or its filtered-header button.")
                .key(["Tab", "Shift+Tab"], "In a filter popover: moves between its fields; past either end it closes and Tab goes on from the button that opened it.")
                .handles([
                    "A debug build warns about an unnamed table, about `selectable`, `row_detail` or `onrowreorder` without `row_key`, about `selectable` with a first row whose name cell has no text, about an editable column without `onrowupdate`, and about a pinned column after one with no `width`.",
                    "`.row_header()` cells are `th scope=\"row\"`, so a screen reader reads the row's name as it moves down any other column. A group header is a `th scope=\"colgroup\"`, read with each cell under it.",
                    "With `scroll` or `max_height`, an overflowing table with no button in it is a named region and a tab stop, so the arrows can scroll it. With header buttons there is no extra stop: the arrows scroll from a focused button.",
                    "Only sorted headers carry `aria-sort`. With `multi_sort`, each sort button's name adds its place, \"sort order 2\", and a tap always adds the column, as touch has no Shift.",
                    "Each row's checkbox is named \"Select\" plus the row's name: the row header's text, else the first cell's, else the row's key (\"Select 0\"), so give that column text. The header checkbox reads mixed while some rows are selected. A selected row carries `aria-selected=\"true\"`.",
                    "Each detail toggle is a button named \"Details for\" plus the row's name, with `aria-expanded` and, while open, `aria-controls`.",
                    "Each group toggle is a button named by the group's text and row count, \"Admin, 2 rows\", with `aria-expanded`; opening or closing it is said in a polite live region, \"Admin expanded\". The table keeps its table role: group rows are rows with a button, not a tree.",
                    "A polite live region says what changed: the selected count, \"2 rows selected\"; the page range, \"4–6 of 7\"; the rows left after a filter, \"2 rows\"; the rows loaded by `onbottomreached`, \"200 rows\"; an export, \"Exported 12 rows\". The first render says nothing.",
                    "Page buttons sit in a `nav` named after the caption, and the page-size picker is labelled. The quick-filter is a labelled search field described by the table's name, so two tables' fields differ.",
                    "Filter popovers and the filter panel are named dialogs with labelled fields. Focus moves in on open, and Escape closes them onto their button. A filtered header reads \"Age is filtered\". A select column's Is any of is a named group of checkboxes, one per option, and in the header filters row a named multi-select.",
                    "Each column menu button is named after its column, \"Age column options\". It shows on hover or focus, always on touch. A move, pin, hide (from any Columns menu too) or width change is said in a polite live region, and the last shown column cannot be hidden: its controls say why.",
                    "Pinned and moved columns move in the DOM too, so Tab and a screen reader meet the cells in the order they are seen.",
                    "Rows and columns reorder without a drag (WCAG 2.5.7): a row has a named handle plus Move up and Move down buttons, a column moves from its menu. The header drag grip is pointer only and hidden from screen readers. While sorted or filtered the row controls stay tab stops but are `aria-disabled` and say why.",
                    "Each resize grip is a tab stop, a vertical `role=\"separator\"` named \"Resize\" plus the column, with its width and limits in `aria-valuenow`, `aria-valuemin` and `aria-valuemax`.",
                    "`loading` marks an empty table `aria-busy`. With rows shown it adds a progress bar named \"Loading rows\" instead, as some screen readers hold back a busy table's rows.",
                    "With `virtual_row_height`, `aria-rowcount` and `aria-rowindex` give the full count and each row's place, \"row 5 001 of 10 001\". Tab walks past the rendered rows, and the row holding focus stays rendered.",
                    "A footer cell reads its aggregate's name before the value, \"Average 7.8 %\", and sits in the column its header names. In a `virtual_row_height` table the footer is the last counted row, and a focus scroll stops above it.",
                    "The `toolbar` is a plain row, not a `role=\"toolbar\"`: Tab moves through it as anywhere else.",
                    "With `cell_focus`, the table is a `role=\"grid\"` and one header or body cell is its tab stop, `tabindex=\"0\"`, the rest `-1`, so Tab passes the grid in one step. Cells, column and row headers keep their roles; in a `virtual_row_height` table the focused cell's row stays rendered. The grid is `aria-readonly` without an editable column, `aria-multiselectable` with `selectable`.",
                    "With `parent_key`, the table is a `role=\"treegrid\"`: each body row carries `aria-level`, `aria-posinset`, `aria-setsize`, and `aria-expanded` when it has children or `row_has_children` says it will, `aria-busy` while open before they load, and one row is the tab stop, the first until focus moves. The row toggles are no Tab stops, as Left and Right do their work, and stay in the reading order. A collapse or filter that hides the focused row moves the stop, and the focus, to its nearest shown parent. With `cell_focus` its cells are stops too and the first header cell is the first.",
                    "In a grid with an editable column, the other cells are `aria-readonly`. The editor is a text field named \"Edit\" plus the column and the row's name, \"Edit Bonus, Ana\", a number column's with a decimal keypad and a `bool` column's a checkbox, mixed while an `Option<bool>` is empty. A value that does not parse or validate marks it `aria-invalid`, shows the `validate` message (or \"Not a valid value\") under it, linked by `aria-describedby`, and announces it once; typing clears it. A custom `editor` gets the message as `EditCtx::error` and clears it with `EditCtx::changed`. On a touch screen, which has no Escape, a Cancel edit button sits beside it.",
                ])
                .must([
                    "Name every table: `caption` shows a title, `aria_labelledby` points at a heading already on the page, `aria_label` names it without text.",
                    "Set `scroll: true` on a table wider than its container, or `max_height` on a long one.",
                    "Mark the column that names a row with `.row_header()`.",
                    "With `onrowclick`, also put a button or link for that action in a cell. A row is not a tab stop, so a keyboard cannot click it.",
                    "With `selectable` or `row_detail`, give the rows a `.row_header()` column, so each checkbox and toggle gets a unique name.",
                    "Before a CSV of text users typed goes to a spreadsheet, neutralise cells that start with `=`, `+`, `-` or `@` (CSV injection, OWASP). `table_text`'s docs show a three-line guard.",
                ])
                .example("A stock list with `caption: \"Stock\"`, `row_detail` and `.row_header()` on the Name column. A screen reader announces the \"Stock\" table, reads \"Cherry\" with each cell of that row, and names the row's toggle \"Details for Cherry\".")
                .limits([
                    "On Blitz, once a `max_height` table's rows scroll, a click on a header's sort or menu button misses: Blitz hit-tests the header where it sat before the scroll. Tab to the button and press Enter instead.",
                    "On Blitz, the same holds for a pinned column's cells once the table scrolls sideways. In a right-to-left page Blitz cannot scroll a wide table at all (todo 707), so pinning shows no effect there.",
                    "On Blitz, a `max_height` table's scrollbar is the native one and runs past the header too.",
                    "On Blitz, a `max_height` table with column groups keeps only its last header row in place; the group rows scroll away with the rows. The same holds for the `header_filters` row.",
                    "On Blitz, a row's handle does not drag: Blitz paints no moved table row. The keyboard and the move buttons reorder there.",
                    "On Blitz, a header has no drag grip: columns reorder from the column menu there.",
                    "The footer and the leading checkbox, detail and reorder cells carry no `aria-colindex`; the leading cells sit at the first places.",
                    "On Android, the arrows, the page keys, Enter and F2 do not reach a `cell_focus` grid or a tree's rows in the emulator's WebView (todo 958); a tap, a double click and a second tap do.",
                    "How TalkBack reads a `cell_focus` grid or a treegrid is not checked yet.",
                    "On Android, the filter popover cannot look into itself, so Tab does not close it at its ends and focus lands on the popover rather than its value field; Escape and Back close it.",
                ]),
            lead: rsx! {
                Text {
                    "A data table over your own row type. "
                    Code { source: "data" }
                    " holds the rows, and each "
                    Code { source: "column(..)" }
                    " in "
                    Code { source: "columns" }
                    " reads one cell, whose type sets its sort order and alignment. Sorting, "
                    "selection, paging, filters, a column menu, pinned, resized and reordered "
                    "columns, row groups, tree rows, row details, cell editing and virtual rows are each a prop "
                    "away, their state held by the table or by you."
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
                        .default(theme.table.size.as_str()),
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
                    // On, so the group rows' boxes show without another switch.
                    Control::switch("selectable").default("true"),
                    Control::switch("multi_sort"),
                    Control::switch("column_menu").default("true"),
                    // On, so the grid's focus ring shows on a click without another switch.
                    Control::switch("cell_focus").default("true"),
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
                    // On, so the Role column's select filter shows without another switch.
                    Control::switch("header_filters").default("true").code(|_, values| match values.str("header_filters") == "true" {
                        true => vec!["header_filters: true".to_string()],
                        false => vec![],
                    }),
                    Control::switch("filter_panel"),
                    Control::switch("paginate").code(|_, values| match values.str("paginate") == "true" {
                        true => vec!["page_sizes: vec![2, 5, 10]".to_string()],
                        false => vec![],
                    }).hidden_when(windowed),
                    Control::switch("empty").code(|_, values| match no_rows(values) {
                        true => vec![r#"empty: rsx! { "No team members yet." }"#.to_string()],
                        false => vec![],
                    }).hidden_when(windowed),
                    // On, so the group rows show without another switch; every group starts open.
                    Control::switch("row_grouping").default("true").code(|_, values| match values.str("row_grouping") == "true" {
                        true => vec![GROUPING.to_string()],
                        false => vec![],
                    }),
                    Control::switch("reorder_rows").code(|_, values| match reorders(values) {
                        true => vec![REORDER.to_string()],
                        false => vec![],
                    }).hidden_when(from_server),
                    // On, so a double click edits a Bonus cell without another switch.
                    Control::switch("editable").default("true").code(|_, values| match edits(values) {
                        true => vec![EDIT.to_string()],
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
                        true => vec![columns_code(GROUPED_COLUMNS, values)],
                        false => vec![columns_code(COLUMNS, values)],
                    }),
                ],
                wrap: Wrap(wrap_data),
                render: move |values: DemoValues| rsx! {
                    TeamTable { values }
                },
            }

            DocSection {
                title: "Select columns",
                Text {
                    Code { source: ".options(..)" }
                    " makes a column's filter pick among values, each with a label: a "
                    Code { source: "(value, label)" }
                    " pair, or a bare string for both. The values match the cell's raw text, "
                    "before "
                    Code { source: ".format(..)" }
                    ", and are what a "
                    Code { source: "ColumnFilter" }
                    " holds, so stored codes shown as names still filter. With "
                    Code { source: "header_filters" }
                    ", Is any of is one field showing the first label and a count."
                }
                // snippet: item #[derive(Clone, PartialEq)] struct Staff { name: String, role: String }
                // snippet: item fn role_label(role: &str) -> &'static str { "" }
                // snippet: let staff: Vec<Staff> = Vec::new();
                Demo {
                    component: "Table",
                    title: "Select column",
                    wide_preview: true,
                    children_text: "",
                    fixed: SELECT_FIXED.map(str::to_string).to_vec(),
                    controls: Vec::new(),
                    render: move |_: DemoValues| rsx! { StaffTable {} },
                }
            }

            DocSection {
                title: "Tree rows",
                Text {
                    Code { source: "parent_key" }
                    " names each row's parent by its "
                    Code { source: "row_key" }
                    ": the rows nest under their parents, each level in the sort, and a row whose "
                    "parent is missing shows as a root. A toggle in the first column opens a row; "
                    Code { source: "default_node_expansion" }
                    " opens the first levels once. A filter keeps the matches' parents, open. A tree "
                    "replaces "
                    Code { source: "row_grouping" }
                    " and windows with "
                    Code { source: "virtual_row_height" }
                    " as a flat table does."
                }
                Text {
                    "With "
                    Code { source: "selectable" }
                    ", a row's box selects the row and its descendants; a parent's box fills once all "
                    "of them are selected and shows mixed with some, as Website relaunch and Design do "
                    "here. To load children on demand, "
                    Code { source: "row_has_children" }
                    " gives a row a toggle before any child is in "
                    Code { source: "data" }
                    ": opening it asks through "
                    Code { source: "onexpandednodeschange" }
                    ", and the row reads as busy until you append its children, as Retrospective does, "
                    "or answer "
                    Code { source: "false" }
                    " for a row that turns out to have none."
                }
                Text {
                    "The tree is a "
                    Code { source: "treegrid" }
                    " whose rows are its stops: Right opens a closed row and steps into an open one, "
                    "Left closes an open row or steps out to its parent, and Up, Down, Home, End and "
                    "the page keys move between the rows. With "
                    Code { source: "cell_focus" }
                    ", Right on an open row goes on to its first cell and Left on a first cell back to the row."
                }
                // snippet: item #[derive(Clone, PartialEq)] struct Task { id: String, parent: Option<String>, name: String, owner: String, days: u32 }
                // snippet: let tasks: Vec<Task> = Vec::new();
                // snippet: let mut loaded = use_signal(|| false);
                Demo {
                    component: "Table",
                    title: "Tree rows",
                    wide_preview: true,
                    children_text: "",
                    fixed: TREE_FIXED.map(str::to_string).to_vec(),
                    controls: Vec::new(),
                    render: move |_: DemoValues| rsx! { TaskTable {} },
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::dioxus_core::NoOpMutations;
    use libero::context::LiberoProvider;
    use std::cell::Cell;

    thread_local! {
        static WINDOWED: Cell<bool> = const { Cell::new(false) };
    }

    #[test]
    fn flipping_virtual_row_height_drops_the_seeded_grouping() {
        fn app() -> Element {
            let values = DemoValues::defaults(&[
                Control::switch("row_grouping").default("true"),
                Control::switch("virtual_row_height"),
            ])
            .with("virtual_row_height", &WINDOWED.get().to_string());
            rsx! {
                LiberoProvider { TeamTable { values } }
            }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let grouped = dioxus_ssr::render(&dom);
        assert!(grouped.contains("aria-expanded"), "{grouped}");
        WINDOWED.set(true);
        dom.mark_dirty(ScopeId::APP);
        dom.render_immediate(&mut NoOpMutations);
        // Kept, the grouping renders all ten thousand rows past the window.
        let rows = dioxus_ssr::render(&dom).matches("<tr").count();
        assert!(rows < 100, "{rows} rows");
    }
}
