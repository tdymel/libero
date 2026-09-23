use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        buttons::ActionIcon,
        common::{Glyph, Input},
        form::{SLOT_BUTTON_SX, slot_icon_size},
    },
    context::IconSlot,
    hooks::{ElementHandle, current_localization},
    platform::ElementApi,
    sx::ThemeAwareValue,
    theme::Size,
};

/// A clearable field's x, or `None` with nothing to clear. It hands the focus
/// to `target` before it unmounts ([[principles/focus-after-removal]]).
pub(crate) fn clear_button(
    show: bool,
    size: Size,
    target: ElementHandle,
    mut onclear: impl FnMut(MouseEvent) + 'static,
) -> Option<Element> {
    let icon_size: Input<ThemeAwareValue> = ThemeAwareValue::Size(slot_icon_size(size)).into();
    show.then(|| {
        rsx! {
            ActionIcon {
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

/// Closing a searchable list clears its query and refocuses the trigger.
/// Opening is `ComboboxCore`'s: the list is hidden until measured.
pub(crate) fn use_refocus_on_close(
    opened: bool,
    searchable: bool,
    trigger: ElementHandle,
    mut query: Signal<String>,
) {
    let mut was_open = use_signal(|| false);
    use_effect(use_reactive!(|(opened, searchable)| {
        if !searchable {
            return;
        }
        // `peek`, so writing it below cannot re-trigger this effect forever.
        let previously = *was_open.peek();
        if previously && !opened {
            query.set(String::new());
            let _ = trigger.focus();
        }
        if previously != opened {
            was_open.set(opened);
        }
    }));
}
