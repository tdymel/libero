use dioxus::prelude::*;

use crate::{
    components::{ClassList, HtmlTag, Input, States, layout::use_box},
    sx::{StaticSx, Sx, sx},
    theme::TableDefaults,
};

use super::{
    cell_value::{SortDirection, SortKey},
    column::Column,
    core::{HeaderSpec, RowSpec, active_sort, render_body, sorted_order},
};

static TABLE_SX: StaticSx = StaticSx::new(|| {
    TableDefaults::theme_vars()
        .width("100%")
        // Row borders only render at all in the collapsing model.
        .border_collapse("collapse")
        .selector(
            "& th, & td",
            sx().text_align("start").vertical_align("middle"),
        )
        .selector("& thead th", sx().font_weight("600"))
        .selector(
            "& th[data-align=\"center\"], & td[data-align=\"center\"]",
            sx().text_align("center"),
        )
        .selector(
            "& th[data-align=\"end\"], & td[data-align=\"end\"]",
            sx().text_align("end"),
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
        .selector(
            "& th button svg",
            sx().width("16px")
                .height("16px")
                .flex_shrink("0")
                .opacity("0")
                .transition("opacity 150ms, transform 150ms"),
        )
        // `aria-sort` is on sortable headers only, so it doubles as the
        // styling state: a hint on hover, solid once the column is sorted.
        .selector(
            "& th[aria-sort=\"none\"]:hover svg, & th[aria-sort=\"none\"]:focus-within svg",
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

#[derive(Props, Clone, PartialEq)]
pub struct TableProps<T: Clone + PartialEq + 'static> {
    /// One row each, in source order until a column is sorted.
    data: Vec<T>,
    /// Built with [`column`](super::column).
    columns: Vec<Column<T>>,
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
/// Give it an accessible name with `aria_label` when the surrounding text
/// doesn't already provide one.
// Generic shim: only the projection below compiles per `T`; the body it hands
// off to is non-generic.
#[component]
pub fn Table<T: Clone + PartialEq + 'static>(props: TableProps<T>) -> Element {
    let sort = use_signal(|| None::<(String, SortDirection)>);

    let headers: Vec<HeaderSpec> = props
        .columns
        .iter()
        .map(|column| HeaderSpec {
            header: column.header.clone(),
            align: column.align,
            sortable: column.sortable,
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
                .map(|column| (column.render)(&props.data[index]))
                .collect(),
        })
        .collect();

    use_box()
        .framework_sx(&TABLE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare()
        .render(
            HtmlTag::Table,
            props.attributes,
            render_body(headers, rows, active, sort),
        )
}
