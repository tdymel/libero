use std::{collections::HashSet, rc::Rc};

use dioxus::prelude::*;

use crate::{
    components::{
        accessibility::use_announcer,
        common::{
            ClassList, HtmlTag, Input, LogicalTextAlign, Parts, States, attr, inset_focus_ring_sx,
            names_itself, use_name_warning,
        },
        form::use_checkbox_look,
        layout::{ScrollArea, ScrollAreaBase, scroll_area_base, use_box},
        overlay::MenuPart,
    },
    hooks::{listener, use_id, use_localization, use_theme},
    platform::{lays_out_captions, widens_sized_tables},
    sx::{StaticSx, Sx, sx},
    theme::{CHECKBOX_BOX_SIZE, NamedColorCss, ScrollAxis, Size, TABLE_PAD_X, TableDefaults},
    utils::warn,
};

use super::{
    column::{Column, ColumnDefaults},
    column_menu::{ColumnMenu, MenuColumn},
    core::{
        BodySpec, CaptionSpec, RowFn, RowSpec, SortedRows, TableSort, active_sort, header_specs,
        render_body,
    },
    paging::{TablePager, clamp_page, page_rows, use_page_reset, use_page_size_reseed},
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
    .selector("& th[data-sortable], & th[data-menu]", sx().padding("0"))
    .selector(
        "& [data-sort-button]",
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
    .selector(
        "& [data-sort-button]:focus-visible",
        inset_focus_ring_sx("-2px"),
    )
    .selector(
        "& [data-sort-button] svg",
        sx().width("16px")
            .height("16px")
            .flex_shrink("0")
            .opacity("0")
            .transition("opacity 150ms, transform 150ms"),
    )
    // `aria-sort`, on the sorted header only, doubles as the styling state.
    .selector(
        "& th[data-sortable]:not([aria-sort]) [data-sort-button]:hover svg, \
             & th[data-sortable]:not([aria-sort]) [data-sort-button]:focus svg",
        sx().opacity("0.5"),
    )
    .selector(
        "& th[aria-sort=\"ascending\"] [data-sort-button] svg, \
             & th[aria-sort=\"descending\"] [data-sort-button] svg",
        sx().opacity("1"),
    )
    .selector(
        "& th[aria-sort=\"ascending\"] [data-sort-button] svg",
        sx().transform("rotate(180deg)"),
    )
    // `text-align` doesn't position flex items.
    .selector(
        "& th[data-align=\"center\"] [data-sort-button]",
        sx().justify_content("center"),
    )
    .selector(
        "& th[data-align=\"end\"] [data-sort-button]",
        sx().justify_content("end"),
    )
    .selector(
        "& [data-sort-button] [data-sort-order]",
        sx().font_size("0.75em").font_weight("600").opacity("0.75"),
    )
    // The label takes the room, the menu button its glyph's.
    .selector(
        "& [data-header]",
        sx().display("flex").align_items("center"),
    )
    .selector(
        "& [data-header] > [data-sort-button], & [data-header-text]",
        sx().flex("1").min_width("0"),
    )
    .selector(
        "& [data-header-text]",
        sx().padding(TableDefaults::padding()),
    )
    .selector(
        "& [data-column-menu]",
        sx().display("flex")
            .align_items("center")
            .justify_content("center")
            .width("24px")
            .height("24px")
            .margin_inline("4px")
            .padding("0")
            .border("0")
            .border_radius("4px")
            .background("none")
            .color("inherit")
            .cursor("pointer"),
    )
    .selector(
        "& [data-column-menu] svg",
        sx().width("16px").height("16px"),
    )
    .selector(
        "& [data-column-menu]:focus-visible",
        inset_focus_ring_sx("0"),
    )
    // With a mouse, shown on its header's hover or focus (todo 1260); a touch
    // screen has no hover. Faded, not hidden, so Tab still reaches it.
    .media(
        "(hover: hover) and (pointer: fine)",
        sx().selector(
            "& th[data-menu]:not(:hover):not(:focus-within) \
             [data-column-menu]:not([aria-expanded=\"true\"])",
            sx().opacity("0"),
        ),
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
    // On each `th`: a sticky `thead` or `tr` has no box natively. Opaque and
    // above the rows, which scroll under it.
    .when(
        "sticky-header",
        sx().selector(
            "& thead th",
            sx().position("sticky")
                .top("0")
                .z_index("1")
                .background(NamedColorCss::SURFACE.value()),
        ),
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

/// As tall as the table, up to `max_height`; not the parent's height.
static TABLE_SCROLL_SX: StaticSx =
    StaticSx::new(|| scroll_area_base(sx().height("auto").max_width("100%")));

#[derive(Props, Clone, PartialEq)]
pub struct TableProps<T: Clone + PartialEq + 'static> {
    /// One row each, in source order until a column is sorted.
    data: Vec<T>,
    /// Built with [`column`](super::column).
    columns: Vec<Column<T>>,
    /// What every column starts from; a column's own setting wins.
    #[props(default)]
    column_defaults: ColumnDefaults,
    /// A visible title above the header row, and the table's accessible name.
    #[props(default, into)]
    caption: Option<String>,
    /// Shown in one full-width row when `data` is empty.
    #[props(default)]
    empty: Option<Element>,
    /// Scrolls a table wider than its parent sideways, in a `ScrollArea`.
    #[props(default)]
    scroll: bool,
    /// Caps the height, a CSS length: the rows scroll in a `ScrollArea` (both
    /// axes) under a header that stays put.
    #[props(default, into)]
    max_height: Option<String>,
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
    /// The shown page, 1-based; set, the page is controlled. Clamped into range.
    #[props(default)]
    page: Option<u32>,
    /// Seeds the page once. Ignored when `page` is set.
    #[props(default = 1)]
    default_page: u32,
    /// The page a page button, a sort or a page-size pick asks for.
    #[props(default)]
    onpagechange: Option<EventHandler<u32>>,
    /// Rows per page; set, the page size is controlled. Turns on pagination.
    #[props(default)]
    page_size: Option<usize>,
    /// Seeds the page size once. Turns on pagination.
    #[props(default)]
    default_page_size: Option<usize>,
    /// The size a pick in the page-size picker asks for.
    #[props(default)]
    onpagesizechange: Option<EventHandler<usize>>,
    /// The page-size picker's choices; empty hides it. Turns on pagination,
    /// the first one seeding the size.
    #[props(default)]
    page_sizes: Vec<usize>,
    /// `data` comes sorted: a header click only asks via `onsortchange`.
    #[props(default)]
    manual_sort: bool,
    /// `data` is the current page: the table only draws the page controls.
    #[props(default)]
    manual_pagination: bool,
    /// Rows over all pages with `manual_pagination`. Unset, `data`'s length.
    #[props(default)]
    row_count: Option<usize>,
    /// The hidden columns' headers; set, visibility is controlled. A hidden
    /// sorted column keeps sorting.
    #[props(default)]
    hidden_columns: Option<Vec<String>>,
    /// Seeds the hidden columns once. Ignored when `hidden_columns` is set.
    #[props(default)]
    default_hidden_columns: Vec<String>,
    /// The hidden columns a column menu pick asks for.
    #[props(default)]
    onhiddencolumnschange: Option<EventHandler<Vec<String>>>,
    /// A menu button in each header: sort, hide the column, show or hide others.
    #[props(default)]
    column_menu: bool,
    /// The column menus' `parts`: they open in a portal, out of `sx`'s reach.
    #[props(default, into)]
    column_menu_parts: Input<Parts<MenuPart>>,
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
        page: props.page,
        default_page: props.default_page,
        onpagechange: props.onpagechange,
        page_size: props.page_size,
        default_page_size: props
            .default_page_size
            .or(props.page_sizes.first().copied())
            .unwrap_or(10),
        onpagesizechange: props.onpagesizechange,
        hidden_columns: props.hidden_columns,
        default_hidden_columns: props.default_hidden_columns,
        onhiddencolumnschange: props.onhiddencolumnschange,
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
    let paginated = props.page_size.is_some()
        || props.default_page_size.is_some()
        || !props.page_sizes.is_empty();
    use_page_reset(state);
    use_page_size_reseed(state, &props.page_sizes, props.default_page_size);
    let caption_id = use_id();
    use_name_warning(
        props.caption.is_some() || names_itself(&props.attributes),
        "Table: no `caption`, `aria_label` or `aria-labelledby`, so it is announced without a \
         name.",
    );
    let size = props.size.copied_or(use_theme().table.size);
    let labels = use_localization().table;
    let bounded = props.max_height.is_some();
    let scrolls = props.scroll || bounded;
    let size_states: Input<States> = States::new().with(size.state_name(), true).into();
    let caption_box = use_box()
        .framework_sx(&TABLE_CAPTION_SX)
        .states(&size_states)
        .prepare();
    let look = use_checkbox_look(size, props.selectable);
    let mut sorted = use_hook(|| CopyValue::new(SortedRows::<T>::default()));

    let headers = header_specs(
        &props.columns,
        &props.column_defaults,
        &state.hidden_columns.read(),
    );
    let active = active_sort(&headers, &state.sort.read(), props.multi_sort);
    let menus = match props.column_menu {
        true => {
            let columns: Rc<[MenuColumn]> = headers
                .iter()
                .map(|spec| MenuColumn {
                    header: spec.header.clone(),
                    sortable: spec.sortable,
                    hideable: spec.hideable,
                    hidden: spec.hidden,
                })
                .collect();
            let active = Rc::new(active.clone());
            (0..headers.len())
                .map(|index| {
                    rsx! {
                        ColumnMenu {
                            index,
                            columns: columns.clone(),
                            active: active.clone(),
                            sort: state.sort,
                            multi_sort: props.multi_sort,
                            hidden: state.hidden_columns,
                            labels,
                            size,
                            parts: props.column_menu_parts.clone(),
                        }
                    }
                })
                .collect()
        }
        false => Vec::new(),
    };

    let data = Rc::new(props.data);
    let key_of = |index: usize| {
        props
            .row_key
            .call(&data[index])
            .unwrap_or_else(|| index.to_string())
    };
    let selection = look.map(|look| Selection {
        slice: state.selection,
        keys: (0..data.len()).map(key_of).collect(),
        announcer,
        labels,
        size,
        look: Rc::new(look),
    });
    // Stable, so a row's box memoizes when its row did not change.
    let current = selection.clone();
    let toggle = use_callback(move |(key, on): (String, bool)| {
        if let Some(selection) = &current {
            selection.toggle(&key, on);
        }
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
    // Sort, then page; a `manual_*` stage is the caller's.
    let mut order = match props.manual_sort {
        true => (0..data.len()).collect(),
        false => sorted.write().order(&data, &props.columns, &active),
    };
    let pager = paginated.then(|| {
        let total = match props.manual_pagination {
            true => props.row_count.unwrap_or(data.len()),
            false => data.len(),
        };
        let page_size = state.page_size.read().max(1);
        let page = clamp_page(total, state.page.read(), page_size);
        if !props.manual_pagination {
            let shown = page_rows(total, page, page_size);
            order.truncate(shown.end);
            order.drain(..shown.start);
        }
        rsx! {
            TablePager {
                state,
                total,
                page,
                page_size,
                page_sizes: props.page_sizes,
                caption: props.caption.clone(),
                size,
            }
        }
    });
    let rows: Vec<RowSpec> = order
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
                selection.row_cell(key.clone(), &name, is_selected == Some(true), toggle)
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
                    .zip(&headers)
                    .filter(|(_, spec)| !spec.hidden)
                    .map(|(column, _)| match &column.render {
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
        None if scrolls => props
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
        .with("sticky-header", bounded)
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
                menus,
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
            match scrolls || pager.is_some() {
                true => rsx! {
                    {caption}
                    {table}
                },
                false => rsx! {
                    div { {caption} {table} }
                },
            }
        }
        None => table,
    };
    // A tab stop, a named region, only while it overflows with no header
    // button inside to scroll it from (decision 7).
    let table = match scrolls {
        true => {
            let mut attributes = region_name;
            attributes.push(attr("data-table-scroll", "true"));
            let max_height = props.max_height.clone();
            rsx! {
                ScrollArea {
                    scrollbars: if bounded { ScrollAxis::Both } else { ScrollAxis::Horizontal },
                    framework_sx: ScrollAreaBase(&TABLE_SCROLL_SX),
                    sx: max_height.map(|height| sx().max_height(height)).unwrap_or_default(),
                    attributes,
                    {table}
                }
            }
        }
        false => table,
    };
    match pager {
        // Outside the scroll region, so the controls stay put while it scrolls.
        Some(pager) => rsx! {
            div { {table} {pager} }
        },
        None => table,
    }
}
