use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::{floating_window::Adjust, options::FloatingWindowPart};
use crate::{
    components::{
        buttons::ActionIcon,
        common::{Glyph, Input, Part, Parts},
        overlay::{Menu, MenuEntry, MenuItem, MenuPart, use_menu},
        typography::Title,
    },
    context::IconSlot,
    hooks::use_id,
    localization::FloatingWindowLabels,
};

/// Its own scope, so a drag frame or a host re-render skips the heading and
/// the close button: every prop compares equal until the title changes.
#[component]
pub(super) fn WindowTitleBar(
    title: Option<String>,
    title_id: Signal<String>,
    pinned: bool,
    resizable: bool,
    labels: FloatingWindowLabels,
    menu_parts: Input<Parts<MenuPart>>,
    close_label: &'static str,
    onclose: Callback<()>,
    onadjust: Callback<Adjust>,
    onpointerdown: Callback<Event<PointerData>>,
    onkeydown: Callback<Event<KeyboardData>>,
) -> Element {
    let hint_id = use_id();
    let menu = use_menu();
    let item = |label: &'static str, adjust: Adjust| -> MenuEntry {
        MenuItem::new(label)
            .onselect(move |_| onadjust.call(adjust))
            .into()
    };
    let items: Vec<MenuEntry> = [
        (!pinned).then(|| item(labels.move_item, Adjust::Move)),
        resizable.then(|| item(labels.resize_item, Adjust::Resize)),
        (!pinned || resizable).then(|| item(labels.reset_item, Adjust::Reset)),
    ]
    .into_iter()
    .flatten()
    .collect();
    let mut trigger = menu.a11y_attributes();
    trigger.push(Attribute::new(
        "data-slot",
        FloatingWindowPart::Menu.slot(),
        None,
        false,
    ));
    let (move_label, move_hint) = (labels.move_handle, labels.move_hint);
    rsx! {
        div { "data-slot": FloatingWindowPart::TitleBar.slot(),
            div {
                "data-slot": FloatingWindowPart::Handle.slot(),
                // Focusable because it is the keyboard move handle, and named
                // for that job; the heading inside it is the window's name.
                role: if !pinned { "group" },
                tabindex: if !pinned { "0" },
                "aria-label": if !pinned { move_label },
                "aria-describedby": if !pinned { hint_id() },
                onpointerdown: move |event| {
                    if !pinned {
                        onpointerdown.call(event);
                    }
                },
                onkeydown: move |event| {
                    if !pinned {
                        onkeydown.call(event);
                    }
                },
                if let Some(title) = title {
                    Title {
                        "data-slot": FloatingWindowPart::Title.slot(),
                        id: title_id(),
                        component: "h2",
                        size: "sm",
                        "{title}"
                    }
                }
            }
            if !pinned {
                span { id: hint_id(), hidden: true, "{move_hint}" }
            }
            if !items.is_empty() {
                Menu { state: menu, items, parts: menu_parts,
                    ActionIcon {
                        variant: "standard",
                        color: "muted",
                        size: "sm",
                        aria_label: labels.menu,
                        attributes: trigger,
                        Glyph { slot: IconSlot::ChevronDown, icon: lucide::chevron_down::outlined }
                    }
                }
            }
            ActionIcon {
                "data-slot": FloatingWindowPart::Close.slot(),
                variant: "standard",
                color: "muted",
                size: "sm",
                aria_label: close_label,
                onclick: move |_| onclose.call(()),
                Glyph { slot: IconSlot::Close, icon: lucide::x::outlined }
            }
        }
    }
}
