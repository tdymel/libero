use std::rc::Rc;

use dioxus::prelude::*;

use super::{
    column_menu::{MenuColumn, toggle_column},
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
pub(super) fn TableToolbar(content: Element, search: Option<Element>) -> Element {
    use_box().framework_sx(&TOOLBAR_SX).prepare().render(
        HtmlTag::Div,
        vec![attr("data-toolbar", "true")],
        rsx! {
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
}

#[derive(Clone, PartialEq)]
pub(super) struct ToolView {
    pub columns: Rc<[MenuColumn]>,
    pub size: Size,
}

impl TableTools {
    pub fn new(hidden: StateSlice<Vec<String>>, density: StateSlice<Option<Size>>) -> Self {
        Self {
            view: Signal::new(ToolView {
                columns: Rc::new([]),
                size: Size::Md,
            }),
            hidden,
            density,
            export: CopyValue::new(None),
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
    let items: Vec<MenuEntry> = [Size::Sm, Size::Md, Size::Lg]
        .into_iter()
        .map(|pick| {
            MenuItem::new((labels.density_name)(pick))
                .radio(pick == size)
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
