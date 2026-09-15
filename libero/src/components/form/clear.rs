use dioxus::prelude::*;

use crate::{
    components::{ActionIcon, Input, common::CloseIcon, form::slot_icon_size},
    hooks::{ElementHandle, current_localization},
    platform::ElementApi,
    sx::ThemeAwareValue,
    theme::Size,
};

/// The x a clearable field draws in its frame's trailing slot, or `None` while
/// there is nothing to clear.
///
/// Pressing it empties the field, and the render that follows drops the button,
/// so the focus it held would fall to the body. It hands the focus to `target`,
/// the field's own control - the trigger or the input - which is what the user
/// reaches for next ([[principles/focus-after-removal]]). The handler can do
/// that itself: unlike a removed row's successor, the target is already on
/// screen, so there is nothing to wait for a render to draw.
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
                onclick: move |event: MouseEvent| {
                    let _ = target.focus();
                    onclear(event);
                },
                CloseIcon {}
            }
        }
    })
}

/// Closing a searchable list clears its query and hands focus back to the
/// trigger, which would otherwise be lost to the body - the box the user was
/// typing in has just unmounted.
///
/// Opening is deliberately *not* handled here. The list is `visibility: hidden`
/// until `use_popover` has measured it, and focusing a hidden element does
/// nothing while still reporting success, so focusing the box on mount never
/// took. `ComboboxCore` does it instead, once the box is on screen - that is
/// what `autofocus` is.
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
