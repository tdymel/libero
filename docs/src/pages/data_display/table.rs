use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::{Chip, Code, Table, Text, column};
use libero::sx::sx;

#[derive(Clone, PartialEq)]
struct Person {
    name: String,
    role: String,
    bonus: Option<f64>,
}

fn people() -> Vec<Person> {
    vec![
        Person {
            name: "Ada Lovelace".into(),
            role: "Owner".into(),
            bonus: Some(12.5),
        },
        Person {
            name: "Grace Hopper".into(),
            role: "Admin".into(),
            bonus: Some(8.0),
        },
        Person {
            name: "Alan Turing".into(),
            role: "Viewer".into(),
            bonus: None,
        },
    ]
}

// snippet: item #[derive(Clone, PartialEq)] struct Person { name: String, role: String, bonus: Option<f64> }
// snippet: let people: Vec<Person> = Vec::new();
// snippet: in Table { caption: "Team members", data: people, .. }
const COLUMNS: &str = r#"columns: vec![
        column("Name").value(|p: &Person| p.name.clone()).sortable().row_header(),
        column("Role")
            .value(|p: &Person| p.role.clone())
            .sortable()
            .render(|p: &Person| rsx! { Chip { size: "xs", "{p.role}" } }),
        column("Bonus")
            .value(|p: &Person| p.bonus)
            .format(|p: &Person| p.bonus.map(|b| format!("{b:.1} %")).unwrap_or_default())
            .sortable(),
    ]"#;

/// The rows are the fixture, and the snippet only compiles with them, so the
/// code block carries the struct and the data above the `Table` itself.
fn wrap_data(values: &DemoValues, code: &str) -> String {
    let people = match no_rows(values) {
        true => "let people: Vec<Person> = Vec::new();",
        false => {
            r#"let people = vec![
    Person { name: "Ada Lovelace".into(), role: "Owner".into(), bonus: Some(12.5) },
    Person { name: "Grace Hopper".into(), role: "Admin".into(), bonus: Some(8.0) },
    Person { name: "Alan Turing".into(), role: "Viewer".into(), bonus: None },
];"#
        }
    };
    format!(
        r#"#[derive(Clone, PartialEq)]
struct Person {{
    name: String,
    role: String,
    bonus: Option<f64>,
}}

{people}

{code}"#
    )
}

/// Wider than the preview at any width, so `scroll` has something to scroll.
const SCROLL_WIDTH: &str = "640px";

/// The `empty` switch empties `data` too, or the slot would never show.
fn no_rows(values: &DemoValues) -> bool {
    values.str("empty") == "true"
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
                    prop("empty", "Option<Element>").default("None").doc("Shown in one full-width row when `data` is empty. Unset, the row reads the localized `table.no_rows`, \"No rows\"."),
                    prop("scroll", "bool").default("false").doc("Wraps the table in a named, focusable region that scrolls sideways. `class`, `sx` and `attributes` stay on the table."),
                    prop("sort", "Option<Vec<TableSort>>").default("None").doc("The sorted columns, empty for source order. Set, the sort is controlled: pair it with `onsortchange`. Without `multi_sort`, one column sorts, the first entry naming a sortable header."),
                    prop("default_sort", "Vec<TableSort>").default("[]").doc("Seeds the sort once. Ignored when `sort` is set."),
                    prop("onsortchange", "EventHandler<Vec<TableSort>>").default("None").doc("Called with the sort a header click asks for: ascending, then descending, then empty."),
                    prop("multi_sort", "bool").default("false").doc("Sorts by several columns, the first entry first. Shift, Ctrl or Cmd with a header click, or any tap on a touch screen, adds the column after the sorted ones, then flips and removes it. A plain click sorts by that column alone. Each sorted header shows its place."),
                    prop("selectable", "bool").default("false").doc("Adds a checkbox column, with a select-all box in its header. Set `row_key` with it, or the selection sticks to positions in `data`."),
                    prop("selection", "Option<Vec<String>>").default("None").doc("The selected rows' `row_key`s. Set, the selection is controlled: pair it with `onselectionchange`. It survives a sort."),
                    prop("default_selection", "Vec<String>").default("[]").doc("Seeds the selection once. Ignored when `selection` is set."),
                    prop("onselectionchange", "EventHandler<Vec<String>>").default("None").doc("Called with the selection a checkbox asks for. Select-all covers every row of `data` and keeps keys of rows not in it, say from another page of a server."),
                    prop("row_key", "RowFn<T, String>").default("the row's index").doc("A row's identity, unique per row, from a `|row: &T| ..` closure. Its DOM node follows it through a sort or a data change, so focus and state inside a row stay with it."),
                    prop("onrowclick", "EventHandler<T>").default("None").doc("Called with the clicked row. Pointer only: give keyboard users a button or link in a cell for the same action."),
                    prop("row_states", "RowFn<T, States>").default("None").doc("A row's states, rendered as its `data-state`. Style them with `sx().selector(\"& tbody tr\", sx().when(..))`."),
                    prop("row_attrs", "RowFn<T, Vec<Attribute>>").default("None").doc("Extra attributes on a row's `tr`."),
                    prop("size", "Size").default("theme (md)").doc("Cell padding and font size."),
                    prop("striped", "bool").default("false").doc("Shades every other body row."),
                    prop("page", "Option<u32>").default("None").doc("The shown page, 1-based. Set, the page is controlled: pair it with `onpagechange`. A page past the end shows the last one."),
                    prop("default_page", "u32").default("1").doc("Seeds the page once. Ignored when `page` is set."),
                    prop("onpagechange", "EventHandler<u32>").default("None").doc("Called with the page a page button asks for. A sort change asks for page 1, and a new page size for the page that keeps the first shown row."),
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
                    prop("column_menu", "bool").default("false").doc("Puts a menu button in each header: sort ascending or descending, unsort, add to the sort with `multi_sort`, hide the column, and a Columns submenu that shows or hides the others."),
                    prop("column_menu_parts", "Parts<MenuPart>").default("none").doc("The column menus' `parts`, the `Menu` page's Style API. The menus open in a portal, out of the table's `sx`."),
                ]),
                props("column()", vec![
                    prop("header", "String").default("required").doc("The column's title, the argument to `column(..)`."),
                    prop("value", "fn(&T) -> V").default("required").doc("Reads one cell out of a row. `V` sets the sort order and alignment. Text sorts as text and aligns left, numbers sort numerically and align right, and `Option<V>` renders `None` empty and sorts it last. Your own type joins them with one `impl CellValue`."),
                    prop("sortable", "bool").default("false").doc("Turns the header into a sort button."),
                    prop("render", "fn(&T) -> Element").default("None").doc("Replaces the cell body. Sorting still uses `value`."),
                    prop("format", "fn(&T) -> String").default("None").doc("Replaces the cell text, say a price with its currency. Sorting and alignment still follow `value`."),
                    prop("align", "CellAlign").default("follows the cell type").doc("Overrides the alignment the cell type chose and `column_defaults`."),
                    prop("width", "String").default("None").doc("The column's width, any CSS length. Columns without one share the rest."),
                    prop("min_width", "String").default("None").doc("The narrowest the column gets, any CSS length."),
                    prop("header_render", "fn() -> Element").default("None").doc("Replaces the header's body, inside the sort button when sortable. The header text stays the column's name in `TableSort`. Capture signals, not values: the closure is not compared, so a changed value does not redraw the header."),
                    prop("of", "&ColumnType<V>").default("None").doc("Before `value`: starts the column from a shared `const` type, its alignment, widths and a `format` over the value. `V` must match `value`'s; the column's own settings win."),
                    prop("row_header", "bool").default("false").doc("Renders the column's cells as `th scope=\"row\"`, so a screen reader names each row by it. One per table, usually the first."),
                    prop("hideable", "bool").default("true").doc("Whether the column menu offers to hide the column. `hidden_columns` still hides it."),
                ]).without_base_props(),
            ],
            accessibility: a11y()
                .key(["Tab"], "With `scroll: true`: enters the scroll region, a tab stop.")
                .key(["Left", "Right", "Up", "Down"], "In the scroll region: scrolls the table.")
                .key(["Enter", "Space"], "On a sortable header, a button: sorts by that column, flips it, then unsorts.")
                .key(["Shift+Enter", "Shift+Space"],"With `multi_sort`, on a sortable header: adds that column after the sorted ones.")
                .key(["Space"], "On a row's checkbox: selects or deselects the row. On the header checkbox: selects or clears every row.")
                .key(["Enter", "Space", "Down"], "With `column_menu`, on a header's menu button: opens the column menu, keyed like `Menu`.")
                .handles([
                    "An unnamed table warns in a debug build.",
                    "With `scroll: true` the wrapper is a `role=\"region\"` named like the table.",
                    "Only sorted headers carry `aria-sort`. With several, each sort button's name adds its place, \"sort order 2\".",
                    "Each row's checkbox is named \"Select\" plus its row header's text, else its first cell's. The header checkbox reads mixed while some rows are selected.",
                    "A selected row carries `aria-selected=\"true\"`, and a polite live region says the new count, \"2 rows selected\", after each change.",
                    "A tap on a touch screen has no Shift key, so with `multi_sort` a tap always adds the column.",
                    "`selectable` without `row_key` warns in a debug build.",
                    "`.row_header()` cells render as `th scope=\"row\"`, so a screen reader reads that name as it moves down any other column. They look like the other cells.",
                    "Paginated, the page buttons sit in a `nav` named after the caption, the page-size picker is labelled, and a page change announces the new range, \"4–6 of 7\", politely. The first render announces nothing.",
                    "With `column_menu`, each menu button is named after its column, \"Age column options\", and the header keeps its text as its name. The Columns submenu lists checkbox items, and the last shown column cannot be hidden.",
                ])
                .must([
                    "Name every table. `caption` shows a title and names it, `aria_labelledby` points at a heading already on the page, and `aria_label` names it without text.",
                    "Set `scroll: true` on a table wider than its container.",
                    "Mark the column that names a row with `.row_header()`.",
                    "With `onrowclick`, also put a button or link for that action in a cell. A row is not a tab stop, so a keyboard cannot click it.",
                    "With `selectable`, give the rows a `.row_header()` column, so each checkbox is named by something unique.",
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
                    Code { source: "scroll" }
                    " wraps a table wider than its container in a region that scrolls "
                    "sideways. The demo's switch also sets "
                    Code { source: "sx().min_width(\"640px\")" }
                    ", so the three columns overflow at any width."
                }
            },
            Demo {
                component: "Table",
                children_text: "",
                fixed: vec![
                    r#"caption: "Team members""#.to_string(),
                    "data: people".to_string(),
                    COLUMNS.to_string(),
                ],
                controls: vec![
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::switch("striped"),
                    Control::switch("scroll").code(|_, values| match values.str("scroll") == "true" {
                        true => vec![
                            "scroll: true".to_string(),
                            format!("sx: sx().min_width({SCROLL_WIDTH:?})"),
                        ],
                        false => vec![],
                    }),
                    Control::switch("selectable").code(|_, values| match values.str("selectable") == "true" {
                        true => vec![
                            "selectable: true".to_string(),
                            "row_key: |p: &Person| p.name.clone()".to_string(),
                        ],
                        false => vec![],
                    }),
                    Control::switch("multi_sort"),
                    Control::switch("column_menu"),
                    Control::switch("paginate").code(|_, values| match values.str("paginate") == "true" {
                        true => vec!["page_sizes: vec![2, 5, 10]".to_string()],
                        false => vec![],
                    }),
                    Control::switch("empty").code(|_, values| match no_rows(values) {
                        true => vec![r#"empty: rsx! { "No team members yet." }"#.to_string()],
                        false => vec![],
                    }),
                ],
                wrap: Wrap(wrap_data),
                render: move |values: DemoValues| rsx! {
                    Table {
                        // A new table per switch: `page_sizes` seeds the page size once.
                        key: "{values.str(\"paginate\")}",
                        caption: "Team members",
                        size: values.str("size"),
                        striped: values.str("striped") == "true",
                        scroll: values.str("scroll") == "true",
                        sx: match values.str("scroll") == "true" {
                            true => sx().min_width(SCROLL_WIDTH),
                            false => sx(),
                        },
                        selectable: values.str("selectable") == "true",
                        multi_sort: values.str("multi_sort") == "true",
                        column_menu: values.str("column_menu") == "true",
                        row_key: |p: &Person| p.name.clone(),
                        page_sizes: if values.str("paginate") == "true" { vec![2, 5, 10] } else { vec![] },
                        empty: no_rows(&values).then(|| rsx! { "No team members yet." }),
                        data: if no_rows(&values) { Vec::new() } else { people() },
                        columns: vec![
                            column("Name").value(|p: &Person| p.name.clone()).sortable().row_header(),
                            column("Role")
                                .value(|p: &Person| p.role.clone())
                                .sortable()
                                .render(|p: &Person| rsx! { Chip { size: "xs", "{p.role}" } }),
                            column("Bonus")
                                .value(|p: &Person| p.bonus)
                                .format(|p: &Person| p.bonus.map(|b| format!("{b:.1} %")).unwrap_or_default())
                                .sortable(),
                        ],
                    }
                },
            }
        }
    }
}
