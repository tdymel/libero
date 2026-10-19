use std::{cell::Cell, rc::Rc};

use dioxus::prelude::*;

use crate::{
    hooks::{ElementHandle, use_element, use_resize_fallback},
    platform::ElementApi,
};

/// The height of a capped table's `thead` or `tfoot`, as its resize reports it. The
/// first report can come a second late, and a Tab before it scrolls a row under the
/// sticky edge (todo 1523): it is measured at mount too.
pub(super) fn use_edge_height(bounded: bool) -> (ElementHandle, Signal<f64>) {
    let mut height = use_signal(|| 0.0);
    let element = use_element();
    let reports: Rc<Cell<u32>> = use_hook(|| Rc::new(Cell::new(0)));
    // Written only on a change, so a measure each bounded turn settles after one render.
    let mut record = move |next: f64| {
        if *height.peek() != next {
            height.set(next);
        }
    };
    use_resize_fallback(element, {
        let reports = reports.clone();
        move |event| {
            if let (true, Ok(size)) = (bounded, event.get_border_box_size()) {
                reports.set(reports.get().wrapping_add(1));
                record(size.height);
            }
        }
    });
    // Measured on every bounded turn: a resize while unbounded was not recorded.
    use_effect(use_reactive!(|bounded| {
        if !bounded || !element.is_mounted() {
            return;
        }
        let measured = element.dimensions();
        let (reports, seen) = (reports.clone(), reports.get());
        spawn(async move {
            if let Ok(size) = measured.await
                && reports.get() == seen
            {
                record(size.height);
            }
        });
    }));
    (element, height)
}
