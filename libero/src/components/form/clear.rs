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

/// Closing a searchable list clears its query and refocuses the trigger, unless the
/// search box's blur closed it: focus went elsewhere then (todo 1497).
/// Opening is `ComboboxCore`'s: the list is hidden until measured.
/// Returns the blur mark, which the search box sets before it closes.
pub(crate) fn use_refocus_on_close(
    opened: bool,
    searchable: bool,
    trigger: ElementHandle,
    mut query: Signal<String>,
) -> Signal<bool> {
    let mut was_open = use_signal(|| false);
    let mut blurred = use_signal(|| false);
    use_effect(use_reactive!(|(opened, searchable)| {
        if !searchable {
            return;
        }
        // `peek`, so writing it below cannot re-trigger this effect forever.
        let previously = *was_open.peek();
        if previously && !opened {
            query.set(String::new());
            if !*blurred.peek() {
                let _ = trigger.focus();
            }
        }
        if previously != opened {
            was_open.set(opened);
            blurred.set(false);
        }
    }));
    blurred
}
