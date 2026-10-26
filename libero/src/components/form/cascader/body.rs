use std::rc::Rc;

use dioxus::prelude::*;

use crate::{components::layout::use_box, hooks::use_element, platform::ElementApi};

use super::{
    core::CascaderLayout,
    nodes::FlatPath,
    rows::{CASCADER_COLUMNS_SX, CascaderRows},
};

pub(super) struct Body {
    pub(super) layout: CascaderLayout,
    /// The cursor's depth, the number of open columns.
    pub(super) depth: usize,
    pub(super) visible: Rc<Vec<FlatPath>>,
    pub(super) path_row: Option<usize>,
    pub(super) listbox_id: String,
    pub(super) label_id: Option<String>,
    /// Names the roots' and the paths' listbox when there is no `label_id`.
    pub(super) aria_label: Option<String>,
    pub(super) column_width: String,
    pub(super) max_height: &'static str,
}

/// The strip's inline end, past it on purpose: the browser clamps. RTL `scrollLeft` counts down from 0 (todo 4238).
pub(super) fn strip_end_x(rtl: bool) -> f64 {
    let end = f64::from(u32::MAX);
    match rtl {
        true => -end,
        false => end,
    }
}

/// A strip of columns, or the flat list of paths.
pub(super) fn use_cascader_body(rows: CascaderRows, parts: Body) -> Element {
    let Body {
        layout,
        depth,
        visible,
        path_row,
        listbox_id,
        label_id,
        aria_label,
        column_width,
        max_height,
    } = parts;

    // A narrow strip scrolls sideways; each new column scrolls into view, as the cursor lives there.
    let strip = use_element();
    use_effect(use_reactive!(|depth| {
        let _ = depth;
        // Subscribes to every reopen, which mounts a new strip at the same handle.
        if strip.mount_token().is_some() {
            let _ = strip.scroll_to(strip_end_x(strip.is_rtl()), 0.0);
        }
    }));
    let columns = use_box()
        .framework_sx(&CASCADER_COLUMNS_SX)
        .prepare()
        .element(&strip);
    match layout {
        CascaderLayout::Columns => rows.columns(
            columns,
            &listbox_id,
            label_id.clone(),
            aria_label.clone(),
            &column_width,
            max_height,
        ),
        CascaderLayout::Paths => rows.paths(
            &visible,
            path_row,
            &listbox_id,
            label_id,
            aria_label,
            max_height,
        ),
    }
}
