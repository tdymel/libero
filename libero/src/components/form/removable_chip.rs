use dioxus::prelude::*;

use crate::{
    components::{ActionIcon, Chip, Input, form::glyphs::CloseIcon},
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{CHIP_HEIGHT, Size},
};

/// A chip's own layout, so every field that draws one gets the long-label fix.
static REMOVABLE_CHIP_SX: StaticSx = StaticSx::new(|| {
    // Capped at its slot by `Chip`'s own `max-width`, and allowed below its
    // content: a flex item's floor is otherwise its min-content size.
    sx().min_width("0")
        // A value can be arbitrarily long. The label is the only part that may
        // shrink: without `min-width: 0` an unbroken 240-character label is its
        // own floor, and the x was pushed clean out of its chip. Seen in a
        // browser; the box model alone looked correct.
        .selector(
            "& > [data-slot='label']",
            sx().min_width("0")
                .overflow("hidden")
                .text_overflow("ellipsis")
                .white_space("nowrap"),
        )
        // It never shrinks - an x too narrow to hit is worse than a clipped
        // label.
        .selector("& > [data-slot='remove']", sx().flex("0 0 auto"))
});

/// The x. A native `<button>` inherits neither `color` nor `font-size` - it
/// takes the UA's `buttontext` and 13.3px.
///
/// The hover tint is `currentColor` at 20%, so it reads on a filled chip and a
/// tonal one alike without either knowing the other's colour.
static REMOVE_BUTTON_SX: StaticSx = StaticSx::new(|| {
    sx().color("inherit")
        .font_size("inherit")
        .border_radius("50%")
        .selector(
            "&:hover",
            sx().background("color-mix(in srgb, currentColor 20%, transparent)"),
        )
});

/// The default chip of a field that holds a list - `MultiSelect`'s selection,
/// `TagsField`'s tags, a `multiple` `FileField`'s files: the label, and an x
/// that drops it.
///
/// `size` is the field's; chips ride inside the control, so they sit one step
/// down the same scale.
///
/// A plain function, not a component: `remove` is a `Callback`, which always
/// compares equal, so a memoized chip would keep dropping the index it was
/// first drawn with ([[codebase/dioxus-memoization-traps]]).
pub(crate) fn removable_chip(
    label: String,
    remove: Callback<()>,
    size: Size,
    disabled: bool,
) -> Element {
    let size = size.step_down();
    // A fraction of the chip's own height, not of its font: the two do not
    // scale at the same rate (20 -> 36px against 11 -> 15px), so an `em` x
    // shrinks against its chip as the field grows.
    let icon_size: Input<ThemeAwareValue> =
        ThemeAwareValue::String(format!("calc({} * 0.6)", CHIP_HEIGHT.value(size))).into();
    rsx! {
        Chip { size, sx: &REMOVABLE_CHIP_SX,
            // Its own element, so it is a flex item the chip can let shrink. A
            // bare text node is an anonymous one, which no selector reaches.
            span { "data-slot": "label", "{label}" }
            span {
                // Centred by the field's own sx - a bare inline span would hang
                // the button off the label's baseline.
                "data-slot": "remove",
                // Load-bearing twice, and only the first reason is obvious. The
                // field holds the focus that keeps its list open, so the press
                // must not move it onto the button. And the button is about to
                // be removed: a focused one would take the focus down with it,
                // to the body ([[principles/focus-after-removal]]). With this,
                // no mouse removal moves focus at all, which is why no
                // field repairs it. Tested in `tests/all/events.rs`.
                onmousedown: move |event: MouseEvent| event.prevent_default(),
                // A remove is not a click on the field, which would open the
                // list under the chip that just went away.
                onclick: move |event: MouseEvent| event.stop_propagation(),
                ActionIcon {
                    aria_label: "Remove {label}",
                    size: icon_size,
                    disabled,
                    sx: &REMOVE_BUTTON_SX,
                    // The field is one tab stop: its keys, not a button per
                    // chip, are how a keyboard removes one.
                    tabindex: "-1",
                    onclick: move |_| remove.call(()),
                    CloseIcon {}
                }
            }
        }
    }
}
