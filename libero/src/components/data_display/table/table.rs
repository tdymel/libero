use std::{collections::HashSet, rc::Rc};

use dioxus::prelude::*;

use crate::{
    components::{
        accessibility::use_announcer,
        common::{
            ClassList, HtmlTag, Input, LogicalTextAlign, States, attr, inset_focus_ring_sx,
            names_itself, use_name_warning,
        },
        layout::use_box,
    },
    hooks::{listener, use_id, use_localization, use_theme},
    platform::{lays_out_captions, widens_sized_tables},
    sx::{StaticSx, Sx, sx},
    theme::{CHECKBOX_BOX_SIZE, Size, TABLE_PAD_X, TableDefaults},
    utils::warn,
};

use super::{
    column::Column,
    core::{
        BodySpec, CaptionSpec, RowFn, RowSpec, TableSort, active_sort, header_specs, render_body,
        row_order,
    },
    selection::Selection,
    use_table::{TableConfig, use_table},
};

static TABLE_SX: StaticSx = StaticSx::new(|| {
    let vars = TableDefaults::theme_vars();
    match widens_sized_tables() {
        true => vars.width("100%"),
        false => vars.min_width("100%"),
    }
    // Separate, with the row lines on the cells: Blitz's collapsing model paints
    // one grid from the first cell's top border, a black 3px one here (todo 772).
    .border_collapse("separate")
    .border_spacing("0")
    .selector(
        "& th, & td",
        sx().text_align_start().vertical_align("middle"),
    )
    .selector("& thead th", sx().font_weight("600"))
    // A row header is semantics, not a look: it reads like its row.
    .selector("& tbody th", sx().font_weight("inherit"))
    .selector("& caption", caption_sx())
    .selector(
        "& th[data-align=\"center\"], & td[data-align=\"center\"]",
        sx().text_align("center"),
    )
    .selector(
        "& th[data-align=\"end\"], & td[data-align=\"end\"]",
        sx().text_align_end(),
    )
    // The button takes the padding, so all of it clicks. Marked from Rust:
    // `th:has(button)` never matches natively.
    .selector("& th[data-sortable]", sx().padding("0"))
    .selector(
        "& th button",
        // Blitz's UA sheet centres a button's content: start it, as its cells.
        sx().display("flex")
            .align_items("center")
            .justify_content("flex-start")
            .gap("4px")
            .width("100%")
            .padding(TableDefaults::padding())
            .background("none")
            .border("0")
            .font("inherit")
            .color("inherit")
            .cursor("pointer"),
    )
    // Inset: the button fills its cell edge to edge.
    .selector("& th button:focus-visible", inset_focus_ring_sx("-2px"))
    .selector(
        "& th button svg",
        sx().width("16px")
            .height("16px")
            .flex_shrink("0")
            .opacity("0")
            .transition("opacity 150ms, transform 150ms"),
    )
    // `aria-sort`, on the sorted header only, doubles as the styling state.
    .selector(
        "& th[data-sortable]:not([aria-sort]):hover svg, \
             & th[data-sortable]:not([aria-sort]):focus-within svg",
        sx().opacity("0.5"),
    )
    .selector(
        "& th[aria-sort=\"ascending\"] svg, & th[aria-sort=\"descending\"] svg",
        sx().opacity("1"),
    )
    .selector(
        "& th[aria-sort=\"ascending\"] svg",
        sx().transform("rotate(180deg)"),
    )
    // `text-align` doesn't position flex items.
    .selector(
        "& th[data-align=\"center\"] button",
        sx().justify_content("center"),
    )
    .selector(
        "& th[data-align=\"end\"] button",
        sx().justify_content("end"),
    )
    .selector(
        "& th button [data-sort-order]",
        sx().font_size("0.75em").font_weight("600").opacity("0.75"),
    )
    // As wide as its box and padding, spelled out: Blitz sizes a `width: 1px`
    // cell below its content.
    .per_size(|size| {
        sx().selector(
            "& [data-select]",
            sx().box_sizing("border-box").width(format!(
                "calc({} + 2 * {})",
                CHECKBOX_BOX_SIZE.value(size),
                TABLE_PAD_X.value()
            )),
        )
    })
    .when(
        "row-click",
        sx().selector("& tbody tr:not([data-empty])", sx().cursor("pointer")),
    )
});

fn caption_sx() -> Sx {
    sx().text_align_start()
        .font_weight("600")
        .padding(TableDefaults::padding())
}

/// The caption drawn before the table where Blitz skips `<caption>`, with the
/// font size and padding the web caption inherits from the table.
static TABLE_CAPTION_SX: StaticSx = StaticSx::new(|| caption_sx().per_size(TableDefaults::size_sx));

static TABLE_SCROLL_SX: StaticSx = StaticSx::new(|| sx().overflow_x("auto").max_width("100%"));

#[derive(Props, Clone, PartialEq)]
pub struct TableProps<T: Clone + PartialEq + 'static> {
    /// One row each, in source order until a column is sorted.
    data: Vec<T>,
    /// Built with [`column`](super::column).
    columns: Vec<Column<T>>,
    /// A visible title above the header row, and the table's accessible name.
    #[props(default, into)]
    caption: Option<String>,
    /// Shown in one full-width row when `data` is empty.
    #[props(default)]
    empty: Option<Element>,
    /// Wraps the table in a named, focusable region that scrolls sideways.
    #[props(default)]
    scroll: bool,
    /// The sorted column, empty for source order; set, the sort is controlled.
    /// Only the first entry naming a sortable header applies.
    #[props(default)]
    sort: Option<Vec<TableSort>>,
    /// Seeds the sort once. Ignored when `sort` is set.
    #[props(default)]
    default_sort: Vec<TableSort>,
    /// The sort a header click asks for: ascending, descending, then empty.
    #[props(default)]
    onsortchange: Option<EventHandler<Vec<TableSort>>>,
    /// Sorts by several columns: Shift, Ctrl or Cmd click, or any tap on a touch
    /// screen, adds a column after the sorted ones; a plain click sorts by it alone.
    #[props(default)]
    multi_sort: bool,
    /// Adds a checkbox column, with a select-all box in its header.
    #[props(default)]
    selectable: bool,
    /// The selected rows' `row_key`s; set, the selection is controlled.
    #[props(default)]
    selection: Option<Vec<String>>,
    /// Seeds the selection once. Ignored when `selection` is set.
    #[props(default)]
    default_selection: Vec<String>,
    /// The selection a checkbox asks for. Select-all keeps keys of rows not in `data`.
    #[props(default)]
    onselectionchange: Option<EventHandler<Vec<String>>>,
    /// A row's identity, unique per row: its DOM node follows it through a sort
    /// or a data change. Unset, the row's index in `data`.
    #[props(default, into)]
    row_key: RowFn<T, String>,
    /// Called with the clicked row. Pointer only: give keyboard users a button
    /// or link in a cell for the same action.
    #[props(default)]
    onrowclick: Option<EventHandler<T>>,
    /// A row's states, as its `data-state`, for `sx().selector("& tbody tr", sx().when(..))`.
    #[props(default, into)]
    row_states: RowFn<T, States>,
    /// Extra attributes on a row's `tr`.
    #[props(default, into)]
    row_attrs: RowFn<T, Vec<Attribute>>,
    /// Cell padding and font size.
    #[props(default, into)]
    size: Input<Size>,
    /// Shades every other body row.
    #[props(default)]
    striped: bool,
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default, into)]
    class: Input<ClassList>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
}

/// A sortable data table over rows of `T`.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Table, column};
/// # #[derive(Clone, PartialEq)] struct User { name: String, age: u32 }
/// # fn app() -> Element {
/// let users = use_signal(Vec::<User>::new);
/// rsx! {
///     Table {
///         caption: "Users",
///         empty: rsx! { "No users yet." },
///         data: users(),
///         columns: vec![
///             column("Name").value(|u: &User| u.name.clone()).sortable().row_header(),
///             column("Age").value(|u: &User| u.age).sortable(),
///         ],
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/table>
// Generic shim: only the projection below compiles per `T`.
#[component]
pub fn Table<T: Clone + PartialEq + 'static>(props: TableProps<T>) -> Element {
    let state = use_table(TableConfig {
        sort: props.sort,
        default_sort: props.default_sort,
        onsortchange: props.onsortchange,
        selection: props.selection,
        default_selection: props.default_selection,
        onselectionchange: props.onselectionchange,
    });
    let announcer = use_announcer();
    let touch = use_hook(|| CopyValue::new(false));
    use_hook(|| {
        if props.selectable && !props.row_key.is_set() {
            warn(
                "Table: `selectable` without `row_key` keys the selection by row index, so it \
                 moves to other rows when `data` changes.",
            );
        }
    });
    let caption_id = use_id();
    use_name_warning(
        props.caption.is_some() || names_itself(&props.attributes),
        "Table: no `caption`, `aria_label` or `aria-labelledby`, so it is announced without a \
         name.",
    );
    let size = props.size.copied_or(use_theme().table.size);
    let labels = use_localization().table;
    let scroll = use_box().framework_sx(&TABLE_SCROLL_SX).prepare();
    let size_states: Input<States> = States::new().with(size.state_name(), true).into();
    let caption_box = use_box()
        .framework_sx(&TABLE_CAPTION_SX)
        .states(&size_states)
        .prepare();

    let headers = header_specs(&props.columns);
    let active = active_sort(&headers, &state.sort.read(), props.multi_sort);

    let data = Rc::new(props.data);
    let key_of = |index: usize| {
        props
            .row_key
            .call(&data[index])
            .unwrap_or_else(|| index.to_string())
    };
    let selection = props.selectable.then(|| Selection {
        slice: state.selection,
        keys: (0..data.len()).map(key_of).collect(),
        announcer,
        labels,
        size,
    });
    let selected_keys = match props.selectable {
        true => state.selection.read(),
        false => Vec::new(),
    };
    let selected: HashSet<&str> = selected_keys.iter().map(String::as_str).collect();
    // A row's checkbox is named by its row header, else its first cell.
    let name_column = props
        .columns
        .iter()
        .position(|column| column.row_header)
        .unwrap_or(0);
    let rows: Vec<RowSpec> = row_order(&data, &props.columns, &active)
        .into_iter()
        .map(|index| {
            let row = &data[index];
            let key = match &selection {
                Some(selection) => selection.keys[index].clone(),
                None => key_of(index),
            };
            let mut attributes = props.row_attrs.call(row).unwrap_or_default();
            if let Some(onrowclick) = props.onrowclick {
                let data = data.clone();
                attributes.push(listener("onclick", move |_: Event<MouseData>| {
                    onrowclick.call(data[index].clone());
                }));
            }
            let is_selected = selection.as_ref().map(|_| selected.contains(key.as_str()));
            let select = selection.as_ref().map(|selection| {
                let name = props
                    .columns
                    .get(name_column)
                    .map(|column| (column.text)(row))
                    .filter(|name| !name.is_empty())
                    .unwrap_or_else(|| key.clone());
                selection.row_cell(key.clone(), &name, is_selected == Some(true))
            });
            RowSpec {
                key,
                selected: is_selected,
                select,
                states: props
                    .row_states
                    .call(row)
                    .and_then(|states| states.data_state()),
                attributes,
                cells: props
                    .columns
                    .iter()
                    .map(|column| match &column.render {
                        Some(render) => (String::new(), Some(render(row))),
                        None => ((column.text)(row), None),
                    })
                    .collect(),
            }
        })
        .collect();

    let mut caption = props.caption.map(|text| CaptionSpec {
        text,
        id: caption_id(),
    });
    // The region takes the table's name: its caption, else the caller's label.
    let region_name: Vec<Attribute> = match &caption {
        Some(spec) => vec![attr("aria-labelledby", spec.id.clone())],
        None if props.scroll => props
            .attributes
            .iter()
            .filter(|a| matches!(a.name, "aria-label" | "aria-labelledby"))
            .cloned()
            .collect(),
        None => Vec::new(),
    };

    // Blitz skips a `<caption>`: there it is a div before the table, naming it.
    let mut attributes = props.attributes;
    let before = if lays_out_captions() {
        None
    } else {
        caption.take()
    };
    if let Some(spec) = &before
        && !names_itself(&attributes)
    {
        attributes.push(attr("aria-labelledby", spec.id.clone()));
    }

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with("striped", props.striped)
        .with("row-click", props.onrowclick.is_some())
        .into();
    let empty = props.empty.unwrap_or_else(|| rsx! { "{labels.no_rows}" });
    let table = use_box()
        .framework_sx(&TABLE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .prepare()
        .render(
            HtmlTag::Table,
            attributes,
            render_body(BodySpec {
                caption,
                headers,
                rows,
                empty,
                active,
                sort: state.sort,
                touch: props.multi_sort.then_some(touch),
                sort_order: labels.sort_order,
                select_all: selection
                    .as_ref()
                    .map(|selection| selection.header_cell(&selected)),
            }),
        );
    // The selection's live region: valid in no part of a table, so beside it.
    let table = match props.selectable {
        true => rsx! {
            {table}
            {announcer.render()}
        },
        false => table,
    };
    let table = match before {
        Some(spec) => {
            let caption = caption_box.render(
                HtmlTag::Div,
                vec![attr("id", spec.id)],
                rsx! { "{spec.text}" },
            );
            // One box, so a flex row does not set them side by side.
            match props.scroll {
                true => rsx! {
                    {caption}
                    {table}
                },
                false => {
                    return rsx! {
                        div { {caption} {table} }
                    };
                }
            }
        }
        None => table,
    };
    if !props.scroll {
        return table;
    }
    scroll
        .attr("role", "region")
        .attr("tabindex", "0")
        .render(HtmlTag::Div, region_name, table)
}
