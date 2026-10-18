use dioxus::prelude::*;

use crate::{hooks::ElementHandle, platform::ElementApi};

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
