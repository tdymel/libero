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
    column::Column,
    core::{CaptionSpec, RowSpec, TableSort, active_sort, header_specs, render_body, row_order},
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
    });
    let caption_id = use_id();
    use_name_warning(
        props.caption.is_some() || names_itself(&props.attributes),
        "Table: no `caption`, `aria_label` or `aria-labelledby`, so it is announced without a \
         name.",
    );
    let scroll = use_box().framework_sx(&TABLE_SCROLL_SX).prepare();
    let caption_box = use_box().framework_sx(&TABLE_CAPTION_SX).prepare();

    let headers = header_specs(&props.columns);
    let active = active_sort(&headers, &state.sort.read());

    let rows: Vec<RowSpec> = row_order(&props.data, &props.columns, active)
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

    let table = use_box()
        .framework_sx(&TABLE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare()
        .render(
            HtmlTag::Table,
            attributes,
            render_body(caption, headers, rows, props.empty, active, state.sort),
        );
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
