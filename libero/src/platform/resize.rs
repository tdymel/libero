use std::rc::Rc;

use dioxus::html::{HasResizeData, ResizeResult, geometry::PixelsSize};
use dioxus::prelude::{Event, MountedData, ResizeData};

use super::{ContentSubscription, Dimensions, backend};

/// Calls `callback` as a `ResizeObserver` would call `onresize`: once at the
/// first measure, then whenever `mounted`'s border box changes size. **Only
/// where `onresize` never fires** (Blitz); elsewhere `None`, and the element's
/// own `onresize` serves.
pub(crate) fn on_resize(
    mounted: &Rc<MountedData>,
    callback: Box<dyn Fn(Event<ResizeData>)>,
) -> Option<Box<dyn ContentSubscription>> {
    backend::on_resize(mounted, callback)
}

/// Whether `data` came from [`on_resize`]'s measure rather than the
/// renderer's own `resize` event.
pub(crate) fn is_measured_resize(data: &ResizeData) -> bool {
    data.downcast::<MeasuredResize>().is_some()
}

/// A resize libero measured itself, as an `onresize` handler reads it.
#[cfg_attr(
    not(all(not(target_arch = "wasm32"), feature = "native")),
    allow(dead_code)
)]
pub(super) fn measured_resize(border: Dimensions, content: Dimensions) -> Event<ResizeData> {
    let data = ResizeData::new(MeasuredResize { border, content });
    Event::new(Rc::new(data), false)
}

#[cfg_attr(
    not(all(not(target_arch = "wasm32"), feature = "native")),
    allow(dead_code)
)]
struct MeasuredResize {
    border: Dimensions,
    content: Dimensions,
}

impl HasResizeData for MeasuredResize {
    fn get_border_box_size(&self) -> ResizeResult<PixelsSize> {
        Ok(PixelsSize::new(self.border.width, self.border.height))
    }

    fn get_content_box_size(&self) -> ResizeResult<PixelsSize> {
        Ok(PixelsSize::new(self.content.width, self.content.height))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
