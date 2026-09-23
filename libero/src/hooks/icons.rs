use dioxus::prelude::*;
use pictogram_core::Svg as SvgData;

use crate::context::{IconContext, IconSlot};

/// The glyph for `slot`: the nearest [`IconProvider`](crate::IconProvider)'s, else `default`.
/// Works without a provider. Reactive.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::{IconSlot, components::Pictogram, hooks::use_icon};
/// # fn app() -> Element {
/// let close = use_icon(IconSlot::Close, pictogram_icons_lucide::x::outlined);
/// rsx! { Pictogram { icon: close } }
/// # }
/// ```
pub fn use_icon(slot: IconSlot, default: SvgData) -> SvgData {
    try_use_context::<IconContext>()
        .and_then(|icons| icons.0.read().get(slot))
        .unwrap_or(default)
}
