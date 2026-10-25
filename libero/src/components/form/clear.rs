use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        buttons::ActionIcon,
        common::{Glyph, Input},
        form::{PreparedField, SLOT_BUTTON_SX, slot_icon_size},
    },
    context::IconSlot,
    hooks::{ElementHandle, current_localization, id_selector},
    platform::{ElementApi, focus_selector},
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
    onclear: impl FnMut(MouseEvent) + 'static,
) -> Option<Element> {
    clear_button_named(show, size, target, field, None, onclear)
}

/// [`clear_button`] with its own accessible `name` in place of "Clear".
pub(crate) fn clear_button_named(
    show: bool,
    size: Size,
    target: ElementHandle,
    field: Option<&PreparedField>,
    name: Option<String>,
    mut onclear: impl FnMut(MouseEvent) + 'static,
) -> Option<Element> {
    let name = name.unwrap_or_else(|| current_localization().common.clear.to_string());
    let icon_size: Input<ThemeAwareValue> = ThemeAwareValue::Size(slot_icon_size(size)).into();
    let id = field.map(|field| format!("{}-clear", field.id()));
    let labelledby = field
        .and_then(PreparedField::label_id)
        .zip(id.clone())
        .map(|(label, id)| format!("{id} {label}"));
    let control = field.map(|field| id_selector(field.id()));
    show.then(|| {
        rsx! {
            ActionIcon {
                id,
                aria_labelledby: labelledby,
                aria_label: name,
                size: icon_size,
                sx: &SLOT_BUTTON_SX,
                onclick: move |event: MouseEvent| {
                    // A target the handle cannot reach (a WebView) is found by the field's id.
                    if target.focus().is_err()
                        && let Some(control) = &control
                    {
                        let _ = focus_selector(control);
                    }
                    onclear(event);
                },
                Glyph { slot: IconSlot::Close, icon: lucide::x::outlined }
            }
        }
    })
}
