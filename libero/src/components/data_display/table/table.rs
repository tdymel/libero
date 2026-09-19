use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            ClassList, HtmlTag, Input, LogicalTextAlign, States, attr, inset_focus_ring_sx,
            names_itself, use_name_warning,
        },
        layout::use_box,
    },
    hooks::use_id,
    platform::{lays_out_captions, widens_sized_tables},
    sx::{StaticSx, Sx, sx},
    theme::{TABLE_FONT_SIZE, TableDefaults},
};

use super::{
    cell_value::{SortDirection, SortKey},
    column::Column,
    core::{CaptionSpec, HeaderSpec, RowSpec, active_sort, render_body, sorted_order},
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
    // The button carries the header's padding instead, so the whole
    // padded area is clickable and not just the label. Marked from Rust:
    // `th:has(button)` never matches natively.
    .selector("& th[data-sortable]", sx().padding("0"))
    .selector(
        "& th button",
        sx().display("flex")
            .align_items("center")
            .gap("4px")
            .width("100%")
            .padding(TableDefaults::padding())
            .background("none")
            .border("0")
            .font("inherit")
            .color("inherit")
            .cursor("pointer"),
    )
    // The library's ring, not the UA's. Inset, as in `Accordion`: the
    // button fills its cell edge to edge.
    .selector("& th button:focus-visible", inset_focus_ring_sx("-2px"))
    .selector(
        "& th button svg",
        sx().width("16px")
            .height("16px")
            .flex_shrink("0")
            .opacity("0")
            .transition("opacity 150ms, transform 150ms"),
    )
    // `aria-sort` sits on the sorted header only, so it doubles as the
    // styling state: a hint on hover elsewhere, solid once sorted.
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
    // `text-align` doesn't position flex items, so a sortable header
    // needs its own justification to match its cells.
    .selector(
        "& th[data-align=\"center\"] button",
        sx().justify_content("center"),
    )
    .selector(
        "& th[data-align=\"end\"] button",
        sx().justify_content("end"),
    )
});

fn caption_sx() -> Sx {
    sx().text_align_start()
        .font_weight("600")
        .padding(TableDefaults::padding())
}

/// The caption drawn before the table where Blitz skips `<caption>`, with the
/// table's font size the web caption inherits.
static TABLE_CAPTION_SX: StaticSx =
    StaticSx::new(|| caption_sx().font_size(TABLE_FONT_SIZE.value()));

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
    /// Wraps the table in a named, focusable `role="region"` that scrolls
    /// sideways, so a table wider than its parent stays keyboard-scrollable.
    /// Named like the table; `class`, `sx` and `attributes` stay on the table.
    #[props(default)]
    scroll: bool,
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default, into)]
    class: Input<ClassList>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
}

/// A sortable data table.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{Table, column};
/// # fn app() -> Element {
/// # #[derive(Clone, PartialEq)] struct User { name: String, age: u32 }
/// # let users = use_signal(Vec::<User>::new);
/// # rsx! {
/// Table {
///     data: users(),
///     columns: vec![
///         column("Name").value(|u: &User| u.name.clone()).sortable(),
///         column("Age").value(|u: &User| u.age).sortable(),
///     ],
/// }
/// # } }
/// ```
///
/// Name it with `caption`, or with `aria_label` / `aria_labelledby` when the
/// title sits elsewhere. A wide one takes `scroll: true`:
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{Table, column};
/// # fn app() -> Element {
/// # #[derive(Clone, PartialEq)] struct User { name: String }
/// # let users = use_signal(Vec::<User>::new);
/// # rsx! {
/// Table {
///     caption: "Users",
///     scroll: true,
///     empty: rsx! { "No users yet." },
///     data: users(),
///     columns: vec![column("Name").value(|u: &User| u.name.clone())],
/// }
/// # } }
/// ```
// Generic shim: only the projection below compiles per `T`; the body it hands
// off to is non-generic.
#[component]
pub fn Table<T: Clone + PartialEq + 'static>(props: TableProps<T>) -> Element {
    let sort = use_signal(|| None::<(String, SortDirection)>);
    let caption_id = use_id();
    use_name_warning(
        props.caption.is_some() || names_itself(&props.attributes),
        "Table: no `caption`, `aria_label` or `aria-labelledby`, so it is announced without a \
         name.",
    );
    let scroll = use_box().framework_sx(&TABLE_SCROLL_SX).prepare();
    let caption_box = use_box().framework_sx(&TABLE_CAPTION_SX).prepare();

    let headers: Vec<HeaderSpec> = props
        .columns
        .iter()
        .map(|column| HeaderSpec {
            header: column.header.clone(),
            align: column.align,
            sortable: column.sortable,
            row_header: column.row_header,
        })
        .collect();

    let active = active_sort(&headers, sort.read().as_ref());

    let order = match active {
        Some((index, direction)) => {
            let sort_key = &props.columns[index].sort_key;
            let keys: Vec<SortKey> = props.data.iter().map(|row| sort_key(row)).collect();
            sorted_order(&keys, direction)
        }
        None => (0..props.data.len()).collect(),
    };

    let rows: Vec<RowSpec> = order
        .into_iter()
        .map(|index| RowSpec {
            index,
            cells: props
                .columns
                .iter()
                .map(|column| {
                    let row = &props.data[index];
                    match &column.render {
                        Some(render) => (String::new(), Some(render(row))),
                        None => ((column.text)(row), None),
                    }
                })
                .collect(),
        })
        .collect();

    let mut caption = props.caption.map(|text| CaptionSpec {
        text,
        id: caption_id(),
    });
    // The region carries the table's own name: its caption, else a copy of
    // the caller's label.
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

    let table = use_box()
        .framework_sx(&TABLE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare()
        .render(
            HtmlTag::Table,
            attributes,
            render_body(caption, headers, rows, props.empty, active, sort),
        );
    let table = match before {
        Some(spec) => {
            let caption = caption_box.render(
                HtmlTag::Div,
                vec![attr("id", spec.id)],
                rsx! { "{spec.text}" },
            );
            // One box, as the web's table with its caption, so a flex row
            // does not set them side by side.
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
