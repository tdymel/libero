use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::{floating_window::Adjust, options::FloatingWindowPart};
use crate::{
    components::{
        buttons::{ActionIcon, Button},
        common::{Glyph, Part},
    },
    context::IconSlot,
    hooks::{escape_closes, use_element},
    localization::FloatingWindowLabels,
    platform::ElementApi,
};

/// The step buttons the menu's Move or Resize shows, until Done or Escape.
#[component]
pub(super) fn WindowSteps(
    adjust: Adjust,
    labels: FloatingWindowLabels,
    onstep: Callback<(f64, f64)>,
    ondone: Callback<()>,
) -> Element {
    let group = use_element();
    // Again when the menu switches Move to Resize: focus is on its trigger.
    use_effect(use_reactive!(|adjust| {
        let _ = adjust;
        if group.is_mounted()
            && let Ok(first) = group.query_selector("button")
        {
            let _ = first.focus();
        }
    }));
    let (name, [up, down, left, right]) = match adjust {
        Adjust::Resize => (
            labels.resize_handle,
            [labels.shorter, labels.taller, labels.narrower, labels.wider],
        ),
        _ => (
            labels.move_handle,
            [
                labels.move_up,
                labels.move_down,
                labels.move_left,
                labels.move_right,
            ],
        ),
    };
    let step = |label: &'static str, delta: (f64, f64), icon: Element| {
        rsx! {
            ActionIcon {
                variant: "standard",
                size: "sm",
                aria_label: label,
                onclick: move |_| onstep.call(delta),
                {icon}
            }
        }
    };
    rsx! {
        div {
            "data-slot": FloatingWindowPart::Steps.slot(),
            role: "group",
            "aria-label": name,
            onmounted: group.mount(),
            // Escape leaves the buttons, not the window.
            onkeydown: move |event: Event<KeyboardData>| {
                if escape_closes(&event) {
                    event.stop_propagation();
                    event.prevent_default();
                    ondone.call(());
                }
            },
            {step(up, (0.0, -1.0), rsx! { Glyph { slot: IconSlot::ChevronUp, icon: lucide::chevron_up::outlined } })}
            {step(down, (0.0, 1.0), rsx! { Glyph { slot: IconSlot::ChevronDown, icon: lucide::chevron_down::outlined } })}
            {step(left, (-1.0, 0.0), rsx! { Glyph { slot: IconSlot::ChevronLeft, icon: lucide::chevron_left::outlined } })}
            {step(right, (1.0, 0.0), rsx! { Glyph { slot: IconSlot::ChevronRight, icon: lucide::chevron_right::outlined } })}
            Button { variant: "outlined", size: "xs", onclick: move |_| ondone.call(()), "{labels.done}" }
        }
    }
}
