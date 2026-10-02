use std::rc::Rc;

use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::{
    entry::{Check, MenuItem, shortcut_hint},
    hover::HoverDelay,
    keyboard::Level,
    menu::MenuPart,
};
use crate::{
    components::{
        common::{Glyph, Part, has_shortcut_modifier, is_javascript_url},
        navigation::NewTabHint,
        typography::Kbd,
    },
    context::IconSlot,
    hooks::{ElementHandle, Typeahead, current_localization},
    platform::logical_key,
    utils::warn,
};

/// What every item on one level is drawn from.
pub(super) struct ItemDraw {
    pub(super) level: Level,
    pub(super) typeahead: Typeahead,
    pub(super) labels: Rc<Vec<Option<String>>>,
    pub(super) hover: HoverDelay,
    /// The roving `tabindex`: the focused item, or the first-focus target.
    pub(super) tabbable: usize,
    pub(super) expanded: Option<usize>,
    pub(super) level_id: String,
    /// Some item on the level is checkable: every row keeps a check column.
    pub(super) checks: bool,
}

/// One `menuitem`, `menuitemradio` or `menuitemcheckbox` row.
pub(super) fn menu_item(
    draw: &ItemDraw,
    item: &MenuItem,
    index: usize,
    anchor: Option<ElementHandle>,
) -> Element {
    let ItemDraw {
        level,
        typeahead,
        labels,
        hover,
        tabbable,
        expanded,
        level_id,
        checks,
    } = draw;
    let (level, tabbable, hover) = (*level, *tabbable, *hover);
    let has_submenu = item.submenu_items().is_some();
    let check = item.check;
    let disabled = item.disabled;
    let onselect = item.onselect_callback();
    let close_on_select = item.close_on_select;
    let item_id = format!("{level_id}-item-{index}");
    let child_id = format!("{level_id}-{index}");
    let description_id = format!("{item_id}-description");
    let is_expanded = has_submenu && *expanded == Some(index);
    let opens = (has_submenu && !disabled).then_some(index);

    let href = item.href_url().map(str::to_owned);
    if let Some(href) = &href
        && is_javascript_url(href)
    {
        warn(&format!(
            "MenuItem \"{}\": a `javascript:` href runs script on click. If it comes from user \
             data, check the scheme before passing it.",
            item.label
        ));
    }
    let is_link = href.is_some();
    let hint = is_link && item.new_tab_hint;
    let onkeydown = {
        let (typeahead, labels) = (typeahead.clone(), labels.clone());
        move |event: KeyboardEvent| {
            let space = matches!(logical_key(&event), Key::Character(ref text) if text == " ");
            if is_link && space && !has_shortcut_modifier(&event) && !typeahead.is_typing() {
                event.prevent_default();
                level.click(index);
                return;
            }
            level.item_keydown(event, index, opens.is_some(), &typeahead, &labels, &hover)
        }
    };
    let onmouseenter = move |_: MouseEvent| hover.enter(level, index, opens);
    let onclick = {
        move |_: MouseEvent| {
            if disabled {
                return;
            }
            hover.cancel();
            level.choose(index, onselect, has_submenu, close_on_select);
        }
    };

    let content = rsx! {
        // On every row of a level with a checkable item, so labels line up.
        if check.is_some() || *checks {
            span { "data-slot": MenuPart::Check.slot(),
                if check.is_some_and(Check::is_checked) {
                    Glyph { slot: IconSlot::Check, icon: lucide::check::outlined }
                }
            }
        }
        if let Some(leading) = item.leading.clone() {
            span { "data-slot": MenuPart::Leading.slot(), {leading} }
        }
        span { "data-slot": MenuPart::Label.slot(),
            "{item.label}"
            if hint {
                NewTabHint { in_text: true }
            }
        }
        if let Some(trailing) = item.trailing.clone() {
            span { "data-slot": MenuPart::Trailing.slot(), {trailing} }
        }
        // Out of the name: `aria-keyshortcuts` announces it instead.
        if let Some(keys) = item.shortcut.as_deref() {
            span { "data-slot": MenuPart::Shortcut.slot(), "aria-hidden": "true",
                Kbd { {shortcut_hint(keys, &current_localization().menu)} }
            }
        }
        if has_submenu {
            span { "data-slot": MenuPart::Chevron.slot(), Glyph { slot: IconSlot::ChevronRight, icon: lucide::chevron_right::outlined } }
        }
        // Hidden, so out of the name; `aria-describedby` still reads it.
        if let Some(description) = item.description.as_deref() {
            span { id: "{description_id}", hidden: true, "{description}" }
        }
    };
    let described_by = item.description.as_ref().map(|_| description_id.clone());
    let onmounted = move |event| {
        if let Some(anchor) = anchor {
            anchor.mount()(event);
        }
    };
    let role = check.map_or("menuitem", Check::role);
    let tabindex = if index == tabbable { "0" } else { "-1" };

    if is_link {
        return rsx! {
            a {
                key: "{index}",
                "role": role,
                id: "{item_id}",
                href: href.filter(|_| !disabled),
                target: "_blank",
                rel: "noopener noreferrer",
                "aria-checked": check.map(|check| check.is_checked().to_string()),
                "aria-keyshortcuts": item.shortcut.clone(),
                tabindex,
                "data-slot": MenuPart::Item.slot(),
                "data-menu-index": "{index}",
                "aria-disabled": disabled.then_some("true"),
                "aria-describedby": described_by,
                onmounted,
                onclick,
                onkeydown,
                onmouseenter,
                {content}
            }
        };
    }

    rsx! {
        button {
            key: "{index}",
            r#type: "button",
            "role": role,
            id: "{item_id}",
            "aria-checked": check.map(|check| check.is_checked().to_string()),
            "aria-keyshortcuts": item.shortcut.clone(),
            tabindex,
            "data-slot": MenuPart::Item.slot(),
            "data-menu-index": "{index}",
            "aria-disabled": disabled.then_some("true"),
            "aria-haspopup": has_submenu.then_some("menu"),
            "aria-expanded": has_submenu.then(|| is_expanded.to_string()),
            "aria-controls": is_expanded.then_some(child_id),
            "aria-describedby": described_by,
            onmounted,
            onclick,
            onkeydown,
            onmouseenter,
            {content}
        }
    }
}
