use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        buttons::ActionIcon,
        common::{Glyph, Input},
        form::{PreparedField, SLOT_BUTTON_SX, slot_icon_size},
    },
    context::IconSlot,
    hooks::{ElementHandle, current_localization},
    platform::ElementApi,
    sx::ThemeAwareValue,
    theme::Size,
};

/// A clearable field's x, or `None` with nothing to clear. It hands the focus
/// to `target` before it unmounts ([[principles/focus-after-removal]]).
/// `field`'s label, if drawn, joins its name: "Clear Fruit" (todo 1498).
pub(crate) fn clear_button(
    show: bool,
    size: Size,
    target: ElementHandle,
    field: Option<&PreparedField>,
    mut onclear: impl FnMut(MouseEvent) + 'static,
) -> Option<Element> {
    let icon_size: Input<ThemeAwareValue> = ThemeAwareValue::Size(slot_icon_size(size)).into();
    let id = field.map(|field| format!("{}-clear", field.id()));
    let labelledby = field
        .and_then(PreparedField::label_id)
        .zip(id.clone())
        .map(|(label, id)| format!("{id} {label}"));
    show.then(|| {
        rsx! {
            ActionIcon {
                id,
                aria_labelledby: labelledby,
                aria_label: current_localization().common.clear,
                size: icon_size,
                sx: &SLOT_BUTTON_SX,
                onclick: move |event: MouseEvent| {
                    let _ = target.focus();
                    onclear(event);
                },
                Glyph { slot: IconSlot::Close, icon: lucide::x::outlined }
            }
        }
    })
}
