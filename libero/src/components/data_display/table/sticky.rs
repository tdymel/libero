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
    use_resize_fallback(element, move |event| {
        if let (true, Ok(size)) = (bounded, event.get_border_box_size()) {
            height.set(size.height);
        }
    });
    use_effect(use_reactive!(|bounded| {
        if !bounded || !element.is_mounted() {
            return;
        }
        let measured = element.dimensions();
        spawn(async move {
            if let Ok(size) = measured.await
                && *height.peek() == 0.0
            {
                height.set(size.height);
            }
        });
    }));
    (element, height)
}
