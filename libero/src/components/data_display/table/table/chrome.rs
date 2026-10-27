//! What stands around the `<table>`: its caption where Blitz lays out none, the scroll
//! area, the quick filter, the loading bar, the filter panel, the toolbar and the pager.

use std::rc::Rc;

use dioxus::prelude::*;

use super::super::{
    column::Column,
    core::CaptionSpec,
    filter::QuickFilter,
    filter_panel::{FilterPanel, FilterPanelButton, PanelColumn, PanelControl},
    measure::LeadProbe,
    overlay::LoadingBar,
    row_reorder::RowReorder,
    toolbar::TableToolbar,
    use_table::TableState,
};
use super::{TABLE_SCROLL_SX, TableProps};
use crate::{
    components::{
        accessibility::Announcer,
        common::{HtmlTag, attr, names_itself, reveal_inline},
        layout::{BoxStyle, ScrollArea, ScrollAreaBase, ScrollAreaHandle, static_skeleton_sx},
    },
    hooks::{ElementHandle, listener},
    localization::labels::table::TableLabels,
    platform::{
        SCROLL_PADDING_BOTTOM_VAR, SCROLL_PADDING_LEFT_VAR, SCROLL_PADDING_RIGHT_VAR,
        SCROLL_PADDING_TOP_VAR, lays_out_captions,
    },
    sx::sx,
    theme::{ScrollAxis, Size, TABLE_PAD_X, TABLE_PADDING_X},
};

/// Where the table's name goes, and the names its region and quick filter take.
pub(super) struct Names {
    /// The `<caption>`, where the platform lays one out in place.
    pub caption: Option<CaptionSpec>,
    /// The caption as a div before the table.
    pub before: Option<CaptionSpec>,
    pub region_name: Vec<Attribute>,
    pub described_by: Option<String>,
    pub table_label: Option<String>,
}

/// Names the table by `caption`, else by the caller's label in `attributes`.
pub(super) fn names(
    caption: Option<String>,
    caption_id: String,
    attributes: &mut Vec<Attribute>,
    scrolls: bool,
) -> Names {
    let described_by = caption.as_ref().map(|_| caption_id.clone());
    let mut caption = caption.map(|text| CaptionSpec {
        text,
        id: caption_id,
    });
    // The region takes the table's name: its caption, else the caller's label.
    let region_name: Vec<Attribute> = match &caption {
        Some(spec) => vec![attr("aria-labelledby", spec.id.clone())],
        None if scrolls => attributes
            .iter()
            .filter(|a| matches!(a.name, "aria-label" | "aria-labelledby"))
            .cloned()
            .collect(),
        None => Vec::new(),
    };
    // Without a caption, the table's own name tells its quick filter apart (todo 1376).
    let text_attr = |name: &str| {
        attributes
            .iter()
            .find(|a| a.name == name)
            .and_then(|a| match &a.value {
                dioxus::core::AttributeValue::Text(text) => Some(text.clone()),
                _ => None,
            })
    };
    let (described_by, table_label) = match described_by {
        Some(id) => (Some(id), None),
        None => match text_attr("aria-labelledby") {
            Some(ids) => (Some(ids), None),
            None => (None, text_attr("aria-label")),
        },
    };

    // Blitz skips a `<caption>`, and a scrolled table's would scroll away, sideways or
    // over its sticky header (todo 4668): a div before the table then, naming it.
    let before = if lays_out_captions() && !scrolls {
        None
    } else {
        caption.take()
    };
    if let Some(spec) = &before
        && !names_itself(attributes)
    {
        attributes.push(attr("aria-labelledby", spec.id.clone()));
    }
    Names {
        caption,
        before,
        region_name,
        described_by,
        table_label,
    }
}

/// A keyboard-focused control outside the pinned and checkbox cells.
const UNPINNED_FOCUS: &str = ":focus-visible:not([data-pin] *, [data-select] *)";

/// The table's surroundings, which `chrome` places.
pub(super) struct Chrome<T: 'static> {
    pub names: Names,
    pub caption_box: BoxStyle,
    pub root_box: BoxStyle,
    pub announcer: Announcer,
    pub reorder: Option<RowReorder>,
    /// Whether anything in the table says something in the live region.
    pub announces: bool,
    pub size: Size,
    pub row_height: Option<f64>,
    pub pinned_pads: [Option<String>; 2],
    pub head: Signal<f64>,
    pub foot_gap: f64,
    pub bottom_reached: Callback<()>,
    pub region: ElementHandle,
    /// Only a pinned table follows its area's width.
    pub tracks_area: bool,
    pub area_width: Signal<Option<f64>>,
    pub lead_probe: Option<String>,
    pub lead_px: Signal<f64>,
    pub state: TableState,
    pub results: usize,
    pub labels: TableLabels,
    pub loading_bar: bool,
    pub panel: PanelControl,
    pub active_filters: usize,
    pub columns: Rc<Vec<Column<T>>>,
    pub onfilter: Callback<()>,
    pub pager: Option<Element>,
}

/// Wraps the rendered `table` in its surroundings.
pub(super) fn chrome<T: Clone + PartialEq + 'static>(
    props: TableProps<T>,
    table: Element,
    chrome: Chrome<T>,
) -> Element {
    let Chrome {
        names,
        caption_box,
        root_box,
        announcer,
        reorder,
        announces,
        size,
        row_height,
        pinned_pads,
        head,
        foot_gap,
        bottom_reached,
        region,
        tracks_area,
        mut area_width,
        lead_probe,
        lead_px,
        state,
        results,
        labels,
        loading_bar,
        panel,
        active_filters,
        columns,
        onfilter,
        pager,
    } = chrome;
    let Names {
        before,
        region_name,
        described_by,
        table_label,
        ..
    } = names;
    let bounded = props.max_height.is_some();
    let scrolls = props.scroll || bounded;
    let onbottomreached = props.onbottomreached;
    let table = match announces {
        true => rsx! {
            {table}
            {announcer.render()}
            {reorder.as_ref().map(RowReorder::instructions)}
        },
        false => table,
    };
    let before = before.map(|spec| {
        caption_box.render(
            HtmlTag::Div,
            vec![attr("id", spec.id)],
            rsx! { "{spec.text}" },
        )
    });
    // Above the scroll area: the header starts at its top (todo 1454), the caption stays put sideways.
    let (before, above) = match scrolls {
        true => (None, before),
        false => (before, None),
    };
    let table = match before {
        // One box, so a flex row does not set them side by side.
        Some(caption) => match scrolls || pager.is_some() {
            true => rsx! {
                {caption}
                {table}
            },
            false => rsx! {
                div { {caption} {table} }
            },
        },
        None => table,
    };
    // A tab stop, a named region, only while it overflows with no header
    // button inside to scroll it from (decision 7).
    let table = match scrolls {
        true => {
            let mut attributes = region_name;
            attributes.push(attr("data-table-scroll", "true"));
            // Chrome's Tab scroll leaves a control inside the scrollport but under a pin (todo 1791).
            if pinned_pads.iter().any(Option::is_some) {
                attributes.push(listener("onfocusin", move |_: Event<FocusData>| {
                    reveal_inline(region, UNPINNED_FOCUS);
                }));
            }
            let max_height = props.max_height.clone();
            rsx! {
                ScrollArea {
                    scrollbars: if bounded { ScrollAxis::Both } else { ScrollAxis::Horizontal },
                    framework_sx: ScrollAreaBase(&TABLE_SCROLL_SX),
                    sx: {
                        // Pinned columns cover the inline edges; a focus scroll stops inside them (todo 1791).
                        // The lead widths read the cell padding, resolved on the table below: again here.
                        let mut pads = sx().var(TABLE_PAD_X, TABLE_PADDING_X.value(size));
                        if let Some(pitch) = row_height {
                            pads = pads.and(static_skeleton_sx(pitch));
                        }
                        // Blitz reads the left and right vars: its `<length>` registration resolves the calc.
                        for ((property, var), pad) in [
                            ("scroll-padding-inline-start", SCROLL_PADDING_LEFT_VAR),
                            ("scroll-padding-inline-end", SCROLL_PADDING_RIGHT_VAR),
                        ]
                        .into_iter()
                        .zip(pinned_pads)
                        {
                            if let Some(pad) = pad {
                                pads = pads.with(property, pad.clone()).with(var, pad);
                            }
                        }
                        match max_height {
                            Some(height) => {
                                // The sticky header covers the top; a focus scroll stops below it.
                                let pad = format!("{}px", head());
                                let pads = pads
                                    .max_height(height)
                                    .with("scroll-padding-top", pad.clone())
                                    .with(SCROLL_PADDING_TOP_VAR, pad);
                                // The sticky footer covers the bottom, as the header the top.
                                match foot_gap > 0.0 {
                                    true => {
                                        let pad = format!("{foot_gap}px");
                                        pads.with("scroll-padding-bottom", pad.clone())
                                            .with(SCROLL_PADDING_BOTTOM_VAR, pad)
                                    }
                                    false => pads,
                                }
                            }
                            None => pads,
                        }
                    },
                    onbottomreached: onbottomreached.map(|_| bottom_reached),
                    // The scrollbar runs beside the rows only, not the sticky header (todo 1454).
                    bar_inset_top: bounded.then(|| *head.read()),
                    // Nor the sticky footer at the bottom.
                    bar_inset_bottom: bounded.then_some(foot_gap),
                    // Its root, mounted by the area itself, is where a column drop counts.
                    handle: ScrollAreaHandle { element: region },
                    // Only a pinned table follows its area's width.
                    onresize: tracks_area.then(|| {
                        EventHandler::new(move |event: Event<ResizeData>| {
                            if let Ok(size) = event.get_border_box_size()
                                && *area_width.peek() != Some(size.width)
                            {
                                area_width.set(Some(size.width));
                            }
                        })
                    }),
                    attributes,
                    {table}
                    if let Some(width) = lead_probe {
                        LeadProbe { width, lead_px }
                    }
                }
            }
        }
        false => table,
    };
    let search = props.show_quick_filter.then(|| {
        rsx! {
            QuickFilter {
                slice: state.quick_filter,
                results,
                announcer,
                labels,
                size,
                caption: described_by,
                table_label,
            }
        }
    });
    // A placeholder while off, so toggling `loading` keeps the table's nodes.
    let bar = loading_bar.then(|| {
        rsx! {
            LoadingBar { labels }
        }
    });
    let table = match above {
        Some(caption) => rsx! {
            div {
                {caption}
                {bar}
                {table}
            }
        },
        None => rsx! {
            {bar}
            {table}
        },
    };
    // With a `toolbar`, its `TableFilterButton` stands in for the button (todo 1448).
    let own_button = props.toolbar.is_none();
    let start = props.filter_panel.then(|| {
        let columns: Rc<[PanelColumn]> = columns
            .iter()
            .filter(|column| column.filterable)
            .map(|column| PanelColumn {
                header: column.header.clone(),
                kind: column.filter_kind,
                options: column.options.clone(),
            })
            .collect();
        rsx! {
            if own_button {
                FilterPanelButton { control: panel, labels, size, active: active_filters }
            }
            FilterPanel {
                control: panel,
                columns,
                slice: state.column_filters,
                logic: state.filter_logic,
                labels,
                size,
                onfilter,
            }
        }
    });
    let search = match (start, props.toolbar) {
        (None, None) => search,
        (start, content) => Some(rsx! {
            TableToolbar { start, content, search }
        }),
    };
    // Outside the scroll region, so the controls stay put while it scrolls. One box,
    // or a flex parent squeezes the zero-height loading bar to no width (todo 1461).
    root_box.render(
        HtmlTag::Div,
        Vec::new(),
        rsx! {
            {search}
            {table}
            {pager}
        },
    )
}
