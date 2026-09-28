use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, Wrap, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::{Button, Chip, Code, PinnedColumns, RowFn, Table, Text, column};
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

// snippet: item #[derive(Clone, PartialEq)] struct Person { name: String, role: String }
// snippet: in Table { caption: "Team members", data: Vec::<Person>::new(), columns: vec![], row_key: |p: &Person| p.name.clone(), .. }
const DETAIL: &str = r#"row_detail: |p: &Person| Some(rsx! { "{p.name} joined as {p.role}." })"#;

// snippet: item #[derive(Clone, PartialEq)] struct Person { name: String }
// snippet: in Table { caption: "Team members", data: Vec::<Person>::new(), columns: vec![], .. }
const TOOLBAR: &str =
    r#"toolbar: rsx! { Button { variant: "outlined", size: "sm", "Add member" } }"#;

/// Wider than the preview at any width, so `scroll` has something to scroll.
const SCROLL_WIDTH: &str = "640px";

/// Shorter than the header and three rows, so `max_height` has something to scroll.
const MAX_HEIGHT: &str = "100px";

/// Name, the outermost start column, needs no `width`; nor does Bonus at the end.
// snippet: item #[derive(Clone, PartialEq)] struct Person { name: String, role: String, bonus: Option<f64> }
// snippet: let people: Vec<Person> = Vec::new();
// snippet: in Table { caption: "Team members", data: people, columns: Vec::new(), .. }
const PINNED: &str =
    r#"default_pinned_columns: PinnedColumns::default().start(["Name"]).end(["Bonus"])"#;

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
                    prop("empty", "Option<Element>").default("None").doc("Shown in one full-width row when `data` is empty. Unset, the row reads the localized `table.no_rows`, \"No rows\". When the quick filter or the column filters leave no rows, the row reads `table.no_results`, \"No matching rows\", instead."),
                    prop("no_results", "Option<Element>").default("None").doc("Shown in one full-width row when the quick filter or the column filters leave no rows, over the localized `table.no_results`."),
                    prop("loading", "bool").default("false").doc("Rows are on their way. While no row shows, placeholder rows fill the body, a page of them when paged, else five, and the table is `aria-busy`. With rows shown, they stay usable under a thin progress bar over the table's top edge, named by the localized `table.loading`, \"Loading rows\". The empty row waits until loading ends."),
                    prop("toolbar", "Option<Element>").default("None").doc("A row above the table for your own controls, say an export or add button. With `show_quick_filter` the search field joins it at the end. It wraps on a narrow screen and stays put while the table scrolls."),
                    prop("scroll", "bool").default("false").doc("Wraps the table in a `ScrollArea` that scrolls sideways. `class`, `sx` and `attributes` stay on the table."),
                    prop("max_height", "Option<String>").default("None").doc("Caps the table's height, any CSS length. The rows scroll in a `ScrollArea`, both ways, under a header that stays put. The header takes the page surface's colour: on another background, set it with `sx().selector(\"& thead th\", ..)`."),
                    prop("virtual_row_height", "Option<f64>").default("None").doc("With `max_height`, renders only the rows in view plus a few beyond each edge, so ten thousand rows scroll like fifty. Every body row is clipped to this height in px: one line per cell, longer text ends in an ellipsis. The table then lays out fixed, columns without a `width` sharing the rest evenly. A row holding focus stays rendered while it scrolls away. Ignored with `row_detail` or `onrowreorder`, which render every row; a debug build warns."),
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
                    prop("row_detail", "RowFn<T, Option<Element>>").default("None").doc("A row's detail, from a `|row: &T| ..` closure. `Some` gives the row a toggle in a leading column; open, the detail shows in a full-width row under it. Called for the shown rows on every render. Set `row_key` with it, or open details stick to positions in `data`."),
                    prop("expanded", "Option<Vec<String>>").default("None").doc("The `row_key`s of the rows whose detail shows. Set, it is controlled: pair it with `onexpandedchange`. It survives a sort."),
                    prop("default_expanded", "Vec<String>").default("[]").doc("Seeds the open details once. Ignored when `expanded` is set."),
                    prop("onexpandedchange", "EventHandler<Vec<String>>").default("None").doc("Called with the open details a toggle asks for."),
                    prop("onrowreorder", "EventHandler<SortableMove>").default("None").doc("Adds a leading column with a drag handle and Move up and Move down buttons per row. Called with a move by positions in `data`; apply it with `step.apply(&mut rows)`, the table shows the old order until you do. Off while the rows are sorted or the quick filter has text. Paged, a row moves within its page. Set `row_key` with it."),
                    prop("size", "Size").default("theme (md)").doc("Cell padding and font size."),
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
                    prop("column_filters", "Option<Vec<ColumnFilter>>").default("None").doc("One filter per column, by header: `ColumnFilter::new(\"Age\", FilterOperator::GreaterThan, \"30\")`. A row stays when it passes all of them and the quick filter. Text operators ignore case, `Equals` too; number operators compare the value, not its formatted text, and take `1,5` as 1.5. An empty value, a number that does not parse, or an operator the column's type does not offer keeps every row. A hidden column's filter keeps filtering. Set, the filters are controlled: pair them with `oncolumnfilterschange`."),
                    prop("default_column_filters", "Vec<ColumnFilter>").default("[]").doc("Seeds the column filters once. Ignored when `column_filters` is set."),
                    prop("oncolumnfilterschange", "EventHandler<Vec<ColumnFilter>>").default("None").doc("Called with the filters a filter popover or a header filter asks for. Typed values arrive once typing pauses for 300 ms, an operator or yes/no pick at once."),
                    prop("header_filters", "bool").default("false").doc("Adds a row of filter fields under the headers, one per `filterable` column: a text field, or Any/Yes/No for a boolean column. A field edits its column's filter with the operator the filter popover set, else the type's first: Contains for text, Equals for numbers."),
                    prop("pinned_columns", "Option<PinnedColumns>").default("None").doc("The columns held at the table's start and end edges while the rest scroll sideways, by header: `PinnedColumns::default().start([..]).end([..])`. Start is the left in a left-to-right page, the right in a right-to-left one. Set, pinning is controlled: pair it with `onpinnedcolumnschange`. Pair it with `scroll` or `max_height`, and give every pinned column but the outermost on its side a `width`, which it then keeps exactly."),
                    prop("default_pinned_columns", "PinnedColumns").default("none pinned").doc("Seeds the pinned columns once. Ignored when `pinned_columns` is set."),
                    prop("onpinnedcolumnschange", "EventHandler<PinnedColumns>").default("None").doc("Called with the pinned columns a column menu pick asks for."),
                    prop("column_order", "Option<Vec<String>>").default("None").doc("The headers in display order. Unlisted columns follow the listed ones in `columns` order, and pinned columns keep their pinned order. Set, the order is controlled: pair it with `oncolumnorderchange`. The sort, hidden and pinned columns name headers, so they follow a moved column."),
                    prop("default_column_order", "Vec<String>").default("[]").doc("Seeds the column order once. Ignored when `column_order` is set."),
                    prop("oncolumnorderchange", "EventHandler<Vec<String>>").default("None").doc("Called with the order a column menu's Move left or Move right asks for, every header listed."),
                    prop("resizable_columns", "bool").default("false").doc("Puts a drag grip on each header's end edge, and Widen column, Narrow column (50px steps, the menu stays open) and Reset width in its `column_menu`, the keyboard and drag-free way. Double-click a grip to reset. A column opts out with `.resizable(false)` and sets its range with `.resize_limits(min, max)` in px, 50 to unbounded by default. Auto layout never draws a column narrower than its content. Blitz: use the menu, the grip does not drag reliably there."),
                    prop("column_widths", "Option<ColumnWidths>").default("None").doc("Resized widths in px by header, a `BTreeMap<String, f64>`, over the columns' own `width`. Set, the widths are controlled: pair them with `oncolumnwidthschange`."),
                    prop("default_column_widths", "ColumnWidths").default("{}").doc("Seeds the widths once. Ignored when `column_widths` is set."),
                    prop("oncolumnwidthschange", "EventHandler<ColumnWidths>").default("None").doc("Called with the widths a grip drag asks for when it ends, and with each Widen, Narrow or Reset width."),
                    prop("column_menu", "bool").default("false").doc("Puts a menu button in each header: sort ascending or descending, unsort, add to the sort with `multi_sort`, filter a `filterable` column, move the column left or right past the next shown one, widen, narrow or reset it with `resizable_columns`, pin to the start or end or unpin, hide the column, and a Columns submenu that shows or hides the others. A pinned column has no move entries. Filter opens a popover with the operators of the column's type, a value and Clear; a filtered column's header then shows a filter button that reopens it."),
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
                    prop("table_csv", "fn(&[Column<T>], &[T]) -> String").default("none").doc("The header row and one line per row as CSV (RFC 4180, CRLF), each cell as its column's text, `format` applied and `render` ignored. Pass the rows and columns in the order you want; saving the file is yours."),
                    prop("table_text", "fn(&[Column<T>], &[T]) -> Vec<Vec<String>>").default("none").doc("The same cells unjoined, the header row first, for your own writer."),
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
                    "The quick-filter field is a labelled `type=\"search\"` input, \"Search\", described by the table's caption. Once typing pauses for half a second, a polite live region says how many rows are left, \"2 rows\".",
                    "With `column_menu`, each menu button is named after its column, \"Age column options\", and the header keeps its text as its name. The Columns submenu lists checkbox items, and the last shown column cannot be hidden.",
                    "The filter popover is a `role=\"dialog\"` named \"Filter\" plus the column, with labelled Operator and Value fields; focus moves to the value on open, or to the operator when it takes none. A filtered header's button reads \"Age is filtered\". Each header filter field is named \"Filter\" plus its column.",
                    "A column filter change announces the rows left, \"2 rows\", in the same polite live region, once it settles.",
                    "A group header is a `th scope=\"colgroup\"` over its columns, so a screen reader reads it with each of their cells. A column outside any group, and the select-all box, span every header row.",
                    "Pinned columns move to their edge in the DOM too, so Tab and a screen reader meet the cells in the order they are seen. The detail toggle and checkbox columns pin with the start ones.",
                    "A pinned column past one without a `width` warns in a debug build: its offset is unknown, so it would overlap.",
                    "A group or a `col_span` stops at a pin edge: a group over pinned and scrolled columns shows as two headers, and a group header never pins.",
                    "With `onrowreorder`, each row has a drag handle named \"Reorder\" plus the row's name, described by the keyboard steps, and Move up and Move down buttons, so a single pointer reorders without a drag (WCAG 2.5.7). A touch drags only from the handle; elsewhere it scrolls. Each lift, move and drop is said in a polite live region. While sorted or filtered the controls are disabled.",
                    "`onrowreorder` without `row_key` warns in a debug build.",
                    "The column menu's Move left and Move right name the screen sides in either text direction. A moved column moves in the DOM too, so Tab and a screen reader follow it, and a column moved out of its group splits the group.",
                    "`loading` without shown rows marks the table `aria-busy` and hides its placeholder rows from screen readers. With rows shown it adds a progress bar named \"Loading rows\" and leaves the table unbusy, as some screen readers hold back a busy table's rows.",
                    "The `toolbar` is a plain row, not a `role=\"toolbar\"`: Tab moves through its controls as anywhere else.",
                    "With `virtual_row_height`, the table carries `aria-rowcount`, every row it holds, and each rendered row its `aria-rowindex`, so a screen reader says \"row 5 001 of 10 001\" though only a screenful is in the DOM. The scrolled-away rows leave no empty rows behind.",
                    "With `virtual_row_height`, Tab and Shift+Tab walk the rows' controls past the rendered ones: the focused row scrolls into view and the next one renders. The row holding focus stays rendered when it scrolls away, Blitz included.",
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
                    "On Blitz, a `max_height` table with column groups keeps only its last header row in place; the group rows scroll away with the rows. The same holds for the `header_filters` row.",
                    "On Blitz, a row's handle does not drag: Blitz paints no moved table row. The keyboard and the move buttons reorder there.",
                    "Columns reorder from the column menu only, not by dragging a header.",
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
                    Code { source: "column_menu" }
                    "'s Filter entry, and "
                    Code { source: "header_filters" }
                    "' row of fields under the headers, filter one column each, with operators "
                    "for its type: contains or starts with for text, greater than for numbers, "
                    "yes or no for booleans. The filters all apply, together with the quick filter. "
                    "Hold them yourself with "
                    Code { source: "column_filters" }
                    " and "
                    Code { source: "oncolumnfilterschange" }
                    "."
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
                    ", Move left and Move right reorder the columns; hold the order yourself with "
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
                    Code { source: "sx().min_width(\"640px\")" }
                    ", so the three columns overflow at any width. "
                    Code { source: "max_height" }
                    " caps a long table's height: its rows scroll under a header that stays put."
                }
                Text {
                    "For thousands of rows add "
                    Code { source: "virtual_row_height" }
                    " to "
                    Code { source: "max_height" }
                    ": only the rows in view render, each that tall, as in the ten thousand "
                    "rows below. Sorting, filtering, selection and pinning work as before."
                }
                Text {
                    Code { source: "default_pinned_columns" }
                    " holds columns at the start or end edge while the rest scroll under "
                    "them, and the column menu pins and unpins them. Start and end follow "
                    "the page's direction. Give each pinned column but the outermost a "
                    Code { source: "width" }
                    ", so the next one knows where to sit."
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
                    Control::switch("max_height").code(|_, values| match values.str("max_height") == "true" {
                        true => vec![format!("max_height: {MAX_HEIGHT:?}")],
                        false => vec![],
                    }),
                    Control::switch("pinned").code(|_, values| match values.str("pinned") == "true" {
                        true => vec![PINNED.to_string()],
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
                    Control::switch("row_detail").code(|_, values| match values.str("row_detail") == "true" {
                        true => vec![DETAIL.to_string()],
                        false => vec![],
                    }),
                    Control::switch("show_quick_filter"),
                    Control::switch("toolbar").code(|_, values| match values.str("toolbar") == "true" {
                        true => vec![TOOLBAR.to_string()],
                        false => vec![],
                    }),
                    Control::switch("loading"),
                    Control::switch("header_filters"),
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
                        // A new table per switch: `page_sizes` and the pins seed once.
                        key: "{values.str(\"paginate\")}-{values.str(\"pinned\")}",
                        caption: "Team members",
                        size: values.str("size"),
                        striped: values.str("striped") == "true",
                        scroll: values.str("scroll") == "true",
                        sx: match values.str("scroll") == "true" {
                            true => sx().min_width(SCROLL_WIDTH),
                            false => sx(),
                        },
                        max_height: (values.str("max_height") == "true").then(|| MAX_HEIGHT.to_string()),
                        default_pinned_columns: match values.str("pinned") == "true" {
                            true => PinnedColumns::default().start(["Name"]).end(["Bonus"]),
                            false => PinnedColumns::default(),
                        },
                        selectable: values.str("selectable") == "true",
                        multi_sort: values.str("multi_sort") == "true",
                        column_menu: values.str("column_menu") == "true",
                        row_detail: match values.str("row_detail") == "true" {
                            true => (|p: &Person| Some(rsx! { "{p.name} joined as {p.role}." })).into(),
                            false => RowFn::default(),
                        },
                        show_quick_filter: values.str("show_quick_filter") == "true",
                        toolbar: (values.str("toolbar") == "true").then(|| rsx! {
                            Button { variant: "outlined", size: "sm", "Add member" }
                        }),
                        loading: values.str("loading") == "true",
                        header_filters: values.str("header_filters") == "true",
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
            DocSection {
                title: "Ten thousand rows",
                Text {
                    "Each row is 40px tall, so the table renders the dozen in view and a few "
                    "beyond each edge, and moves them along as you scroll. Switch "
                    Code { source: "virtual_row_height" }
                    " off to render all ten thousand and feel the difference in a sort."
                }
                Demo {
                    title: "Ten thousand rows",
                    component: "Table",
                    children_text: "",
                    fixed: vec![
                        r#"caption: "Stock""#.to_string(),
                        format!("max_height: {STOCK_HEIGHT:?}"),
                        "selectable: true".to_string(),
                        "row_key: |s: &Stock| s.id.to_string()".to_string(),
                        "data: stock".to_string(),
                        STOCK_COLUMNS.to_string(),
                    ],
                    controls: vec![
                        Control::switch("virtual_row_height").default("true").code(|_, values| {
                            match values.str("virtual_row_height") == "true" {
                                true => vec!["virtual_row_height: 40.0".to_string()],
                                false => vec![],
                            }
                        }),
                    ],
                    wrap: Wrap(wrap_stock),
                    render: move |values: DemoValues| rsx! {
                        StockTable { windowed: values.str("virtual_row_height") == "true" }
                    },
                }
            }
        }
    }
}

#[derive(Clone, PartialEq)]
struct Stock {
    id: u32,
    name: String,
    count: u32,
}

const FRUITS: [&str; 5] = ["Apple", "Banana", "Cherry", "Date", "Elderberry"];

fn stock() -> Vec<Stock> {
    (1..=10_000)
        .map(|id| Stock {
            id,
            name: format!("{} {id}", FRUITS[id as usize % FRUITS.len()]),
            count: id * 7 % 1_000,
        })
        .collect()
}

const STOCK_HEIGHT: &str = "320px";

// snippet: item #[derive(Clone, PartialEq)] struct Stock { id: u32, name: String, count: u32 }
// snippet: let stock: Vec<Stock> = Vec::new();
// snippet: in Table { caption: "Stock", data: stock, .. }
const STOCK_COLUMNS: &str = r#"columns: vec![
        column("Id").value(|s: &Stock| s.id).sortable().row_header(),
        column("Name").value(|s: &Stock| s.name.clone()).sortable(),
        column("Count").value(|s: &Stock| s.count).sortable(),
    ]"#;

fn wrap_stock(_: &DemoValues, code: &str) -> String {
    format!(
        r#"#[derive(Clone, PartialEq)]
struct Stock {{
    id: u32,
    name: String,
    count: u32,
}}

const FRUITS: [&str; 5] = ["Apple", "Banana", "Cherry", "Date", "Elderberry"];

let stock: Vec<Stock> = (1..=10_000)
    .map(|id| Stock {{
        id,
        name: format!("{{}} {{id}}", FRUITS[id as usize % FRUITS.len()]),
        count: id * 7 % 1_000,
    }})
    .collect();

{code}"#
    )
}

/// Built once: a switch flip re-renders the table, not ten thousand rows of data.
#[component]
fn StockTable(windowed: bool) -> Element {
    let rows = use_hook(stock);
    rsx! {
        Table {
            caption: "Stock",
            max_height: STOCK_HEIGHT,
            virtual_row_height: windowed.then_some(40.0),
            selectable: true,
            row_key: |s: &Stock| s.id.to_string(),
            data: rows,
            columns: vec![
                column("Id").value(|s: &Stock| s.id).sortable().row_header(),
                column("Name").value(|s: &Stock| s.name.clone()).sortable(),
                column("Count").value(|s: &Stock| s.count).sortable(),
            ],
        }
    }
}
