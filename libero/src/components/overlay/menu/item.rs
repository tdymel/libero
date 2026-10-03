use std::rc::Rc;

use dioxus::{core::Runtime, prelude::*};
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

/// What a row's handlers read when they run, rewritten on each level render, so a
/// row that skipped a render calls no stale callback.
pub(super) struct RowEvents {
    pub(super) level: Level,
    pub(super) typeahead: Typeahead,
    /// Labels for typeahead, `None` for an item it must skip.
    pub(super) labels: Vec<Option<String>>,
    pub(super) hover: HoverDelay,
    /// Per item, its `onselect` and `close_on_select`.
    pub(super) choices: Vec<(Option<Callback<()>>, Option<bool>)>,
}

/// Equal when the row would draw the same: an arrow key redraws two rows, not all (todo 2092).
#[derive(Props, Clone)]
pub(super) struct MenuRowProps {
    events: CopyValue<Rc<RowEvents>>,
    /// The level's scope, which owns `events` and every handle in it.
    owner: ScopeId,
    item: MenuItem,
    index: usize,
    anchor: Option<ElementHandle>,
    /// The roving `tabindex`: the focused item, or the first-focus target.
    tabbable: bool,
    expanded: bool,
    level_id: String,
    /// Some item on the level is checkable: every row keeps a check column.
    checks: bool,
}

impl PartialEq for MenuRowProps {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
            && self.tabbable == other.tabbable
            && self.expanded == other.expanded
            && self.checks == other.checks
            && self.anchor == other.anchor
            && self.events == other.events
            && self.owner == other.owner
            && self.level_id == other.level_id
            && self.item.draws_like(&other.item)
    }
}

/// One `menuitem`, `menuitemradio` or `menuitemcheckbox` row.
#[component]
pub(super) fn MenuRow(props: MenuRowProps) -> Element {
    let MenuRowProps {
        events,
        owner,
        item,
        index,
        anchor,
        tabbable,
        expanded: is_expanded,
        level_id,
        checks,
    } = props;
    let has_submenu = item.submenu_items().is_some();
    let check = item.check;
    let disabled = item.disabled;
    let item_id = format!("{level_id}-item-{index}");
    let child_id = format!("{level_id}-{index}");
    let description_id = format!("{item_id}-description");
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
    // The level's state is its scope's, and a portaled row is no descendant of it.
    let as_level = move |run: &mut dyn FnMut(&RowEvents)| {
        Runtime::current().in_scope(owner, || {
            let events = events.peek().clone();
            run(&events);
        });
    };
    let onkeydown = move |event: KeyboardEvent| {
        as_level(&mut |events| {
            let space = matches!(logical_key(&event), Key::Character(ref text) if text == " ");
            if is_link && space && !has_shortcut_modifier(&event) && !events.typeahead.is_typing() {
                event.prevent_default();
                events.level.click(index);
                return;
            }
            events.level.item_keydown(
                event.clone(),
                index,
                opens.is_some(),
                &events.typeahead,
                &events.labels,
                &events.hover,
            )
        })
    };
    let onmouseenter = move |_: MouseEvent| {
        as_level(&mut |events| events.hover.enter(events.level, index, opens));
    };
    let onclick = move |_: MouseEvent| {
        if disabled {
            return;
        }
        as_level(&mut |events| {
            let (onselect, close) = events.choices.get(index).copied().unwrap_or_default();
            events.hover.cancel();
            events.level.choose(index, onselect, has_submenu, close);
        })
    };

    let content = rsx! {
        // On every row of a level with a checkable item, so labels line up.
        if check.is_some() || checks {
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
            Runtime::current().in_scope(owner, || anchor.mount()(event));
        }
    };
    let role = check.map_or("menuitem", Check::role);
    let tabindex = if tabbable { "0" } else { "-1" };

    if is_link {
        return rsx! {
            a {
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
