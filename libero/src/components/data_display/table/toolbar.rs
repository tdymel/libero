use std::rc::Rc;

use dioxus::prelude::*;

use super::{
    column_menu::{MenuColumn, toggle_column},
    filter_panel::{FilterPanelButton, PanelControl},
    use_table::StateSlice,
};
use crate::{
    components::{
        buttons::Button,
        common::{HtmlTag, attr},
        layout::use_box,
        overlay::{Menu, MenuEntry, MenuItem, use_menu},
    },
    hooks::use_localization,
    sx::{StaticSx, sx},
    theme::Size,
    utils::warn,
};

static TOOLBAR_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_wrap("wrap")
        .align_items("center")
        .gap("8px")
        .margin("0 0 8px")
        // The quick filter keeps to the inline end, and drops its own margin.
        .selector("& > [data-toolbar-end]", sx().margin_inline_start("auto"))
        .selector("& > [data-toolbar-end] > *", sx().margin("0"))
});

/// The row above a table: the caller's `toolbar`, then the quick filter at
/// the end. No `role=toolbar`, which would promise arrow-key focus.
#[component]
pub(super) fn TableToolbar(
    /// Before `content`: the `filter_panel` button.
    start: Option<Element>,
    content: Option<Element>,
    search: Option<Element>,
) -> Element {
    use_box().framework_sx(&TOOLBAR_SX).prepare().render(
        HtmlTag::Div,
        vec![attr("data-toolbar", "true")],
        rsx! {
            {start}
            {content}
            if let Some(search) = search {
                div { "data-toolbar-end": true, {search} }
            }
        },
    )
}

/// What a `Table` hands the pieces in its `toolbar`.
#[derive(Clone, Copy)]
pub(super) struct TableTools {
    pub view: Signal<ToolView>,
    pub hidden: StateSlice<Vec<String>>,
    /// Overrides the table's `size` once picked.
    pub density: StateSlice<Option<Size>>,
    /// The CSV of the filtered, sorted rows over every page, shown columns only.
    pub export: CopyValue<Option<Rc<dyn Fn() -> String>>>,
    pub panel: PanelControl,
}

#[derive(Clone, PartialEq)]
pub(super) struct ToolView {
    pub columns: Rc<[MenuColumn]>,
    pub size: Size,
    /// The active filters' count, `None` without a `filter_panel`.
    pub filters: Option<usize>,
}

impl TableTools {
    pub fn new(
        hidden: StateSlice<Vec<String>>,
        density: StateSlice<Option<Size>>,
        panel: PanelControl,
    ) -> Self {
        Self {
            view: Signal::new(ToolView {
                columns: Rc::new([]),
                size: Size::Md,
                filters: None,
            }),
            hidden,
            density,
            export: CopyValue::new(None),
            panel,
        }
    }

    /// Guarded, so an unchanged view wakes no piece.
    pub fn show(mut self, view: ToolView) {
        if *self.view.peek() != view {
            self.view.set(view);
        }
    }
}

fn use_tools(piece: &str) -> Option<TableTools> {
    let tools = try_use_context::<TableTools>();
    use_hook(|| {
        if tools.is_none() {
            warn(&format!(
                "{piece}: outside a `Table`'s `toolbar`, so it renders nothing."
            ));
        }
    });
    tools
}

/// A menu of the hideable columns, each a checkbox that shows or hides it.
/// The last shown column stays. Goes in a `Table`'s `toolbar`.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Table, TableColumnsButton, column};
/// # fn app() -> Element {
/// rsx! {
///     Table {
///         caption: "Users",
///         toolbar: rsx! { TableColumnsButton {} },
///         data: vec!["Ada".to_string()],
///         columns: vec![column("Name").value(|name: &String| name.clone())],
///     }
/// }
/// # }
/// ```
#[component]
pub fn TableColumnsButton() -> Element {
    let tools = use_tools("TableColumnsButton");
    let menu = use_menu();
    let labels = use_localization().table;
    let Some(tools) = tools else {
        return rsx! {};
    };
    let view = tools.view.read();
    let hidden = tools.hidden;
    let shown = view.columns.iter().filter(|column| !column.hidden).count();
    let items: Vec<MenuEntry> = view
        .columns
        .iter()
        .filter(|column| column.hideable)
        .map(|column| {
            let (header, off) = (column.header.clone(), column.hidden);
            MenuItem::new(header.clone())
                .checkbox(!off)
                .disabled(!off && shown <= 1)
                .keep_open()
                .onselect(move |_| hidden.set(toggle_column(&hidden.read(), &header, off)))
                .into()
        })
        .collect();
    rsx! {
        Menu { state: menu, items, size: view.size,
            Button {
                variant: "standard",
                size: view.size,
                attributes: menu.a11y_attributes(),
                "data-table-tool": "columns",
                "{labels.columns}"
            }
        }
    }
}

const DENSITIES: [Size; 3] = [Size::Sm, Size::Md, Size::Lg];

/// The density a size outside the three reads as, so one item is always marked (todo 1459).
fn nearest_density(size: Size) -> Size {
    match size {
        Size::Xs => Size::Sm,
        Size::Xl | Size::Xxl => Size::Lg,
        size => size,
    }
}

/// A menu of three row densities, the table's `size` small, medium or large;
/// a pick drives `density`. Goes in a `Table`'s `toolbar`.
#[component]
pub fn TableDensityButton() -> Element {
    let tools = use_tools("TableDensityButton");
    let menu = use_menu();
    let labels = use_localization().table;
    let Some(tools) = tools else {
        return rsx! {};
    };
    let size = tools.view.read().size;
    let density = tools.density;
    let marked = nearest_density(size);
    let items: Vec<MenuEntry> = DENSITIES
        .into_iter()
        .map(|pick| {
            MenuItem::new((labels.density_name)(pick))
                .radio(pick == marked)
                .onselect(move |_| density.set(Some(pick)))
                .into()
        })
        .collect();
    rsx! {
        Menu { state: menu, items, size,
            Button {
                variant: "standard",
                size,
                attributes: menu.a11y_attributes(),
                "data-table-tool": "density",
                "{labels.density}"
            }
        }
    }
}

/// Calls `onexport` with the table as CSV ([`table_csv`](super::table_csv)):
/// the filtered, sorted rows over every page, in the shown columns. The
/// caller saves it. Goes in a `Table`'s `toolbar`.
#[component]
pub fn TableExportButton(onexport: EventHandler<String>) -> Element {
    let tools = use_tools("TableExportButton");
    let labels = use_localization().table;
    let Some(tools) = tools else {
        return rsx! {};
    };
    let size = tools.view.read().size;
    rsx! {
        Button {
            variant: "standard",
            size,
            "data-table-tool": "export",
            onclick: move |_| {
                if let Some(csv) = tools.export.peek().clone() {
                    onexport.call(csv());
                }
            },
            "{labels.export}"
        }
    }
}

/// The Filters button with the active filters' count: opens the dialog of
/// every column filter. Goes in the `toolbar` of a `Table` with `filter_panel`.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Table, TableFilterButton, column};
/// # fn app() -> Element {
/// rsx! {
///     Table {
///         caption: "Users",
///         filter_panel: true,
///         toolbar: rsx! { TableFilterButton {} },
///         data: vec!["Ada".to_string()],
///         columns: vec![column("Name").value(|name: &String| name.clone())],
///     }
/// }
/// # }
/// ```
#[component]
pub fn TableFilterButton() -> Element {
    let tools = use_tools("TableFilterButton");
    let labels = use_localization().table;
    let mut warned = use_hook(|| CopyValue::new(false));
    let Some(tools) = tools else {
        return rsx! {};
    };
    let view = tools.view.read();
    let Some(active) = view.filters else {
        if !warned() {
            warned.set(true);
            warn("TableFilterButton: the `Table` has no `filter_panel`, so it renders nothing.");
        }
        return rsx! {};
    };
    rsx! {
        FilterPanelButton { control: tools.panel, labels, size: view.size, active }
    }
}

#[cfg(test)]
mod tests {
    use super::{DENSITIES, nearest_density};
    use crate::theme::Size;

    #[test]
    fn every_size_marks_one_density() {
        for size in [Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl, Size::Xxl] {
            assert!(DENSITIES.contains(&nearest_density(size)), "{size:?}");
        }
        assert_eq!(nearest_density(Size::Xs), Size::Sm);
        assert_eq!(nearest_density(Size::Xl), Size::Lg);
    }
}
