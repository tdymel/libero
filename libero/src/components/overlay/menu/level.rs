use std::{cell::RefCell, rc::Rc, time::Duration};

use dioxus::prelude::*;

use super::{
    entry::{Check, MenuEntry, MenuItem, flatten},
    hover::use_hover_delay,
    item::{ItemDraw, menu_item},
    keyboard::{Level, use_level_focus},
    menu::{MenuEdge, MenuPart},
    state::MenuFocus,
    styles::MENU_SX,
};
use crate::{
    components::{
        common::{HtmlTag, Input, Part, Parts, States},
        layout::{Divider, use_box},
    },
    hooks::{
        Align, DismissHandle, DismissOptions, ElementHandle, PopoverOptions, PressMarker, Side,
        TYPEAHEAD_RESET, use_dismiss, use_popover_on, use_theme, use_typeahead,
    },
    theme::Size,
};

/// Every ancestor menu's dismissal: a third level is no descendant of the first.
/// Always equal, as the handles' signals never change identity.
#[derive(Clone)]
pub(super) struct Parents(pub(super) Vec<DismissHandle>);

impl PartialEq for Parents {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

// Every prop is a value or a stable handle; a submenu's `items` never compares
// equal (see `MenuItem`'s `PartialEq`), so a level redraws with its parent.
#[derive(Props, Clone, PartialEq)]
pub(super) struct MenuLevelProps {
    items: Vec<MenuEntry>,
    id: String,
    labelledby: String,
    anchor: ElementHandle,
    /// This level's box, owned by `root`: every level above reads it, and a
    /// non-descendant reader is a dioxus warning (todo 283).
    floating: ElementHandle,
    root: ScopeId,
    /// The root `Menu`'s wrapper, which holds the trigger.
    wrapper: ElementHandle,
    open: bool,
    /// Focus an item once placed, whenever this grows.
    request: u64,
    initial: MenuFocus,
    onclose: Callback<()>,
    /// `None` on the root, which builds its own. `true` returns focus to the trigger.
    close_all: Option<Callback<bool>>,
    /// The root's, on every level: ArrowRight in a submenu moves on too (APG menubar).
    onedge: Option<Callback<MenuEdge>>,
    parents: Parents,
    /// The parent cancels a pending hover, so crossing a sibling keeps this open.
    onpointerenter: Option<Callback<()>>,
    side: Side,
    align: Align,
    size: Size,
    radius: Size,
    loop_focus: bool,
    close_on_select: bool,
    depth: usize,
    /// The wrapper's, on the root only: a press there is the trigger's toggle.
    #[props(default)]
    marker: Option<PressMarker>,
    #[props(default)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Input<crate::components::common::ClassList>,
    /// The root's, on every level, as `parts`.
    #[props(default)]
    sx: Input<crate::sx::Sx>,
    #[props(default)]
    parts: Input<Parts<MenuPart>>,
    #[props(default)]
    states: Input<States>,
}

/// One floating menu level. Submenus render here, beside the portal, so no level
/// reads across it ([[codebase/use-popover]]); only the boxes are portaled.
#[component]
pub(super) fn MenuLevel(props: MenuLevelProps) -> Element {
    let theme = use_theme();
    let open = props.open;
    let depth = props.depth;
    let flat = flatten(&props.items);
    let len = flat.len();

    let popover = use_popover_on(
        props.anchor,
        props.floating,
        open,
        PopoverOptions::new(theme.popover.gap, theme.popover.padding)
            .side(props.side)
            .align(props.align),
    );
    let floating = *popover.floating();
    let placed = popover.placed();
    // Focus left a submenu: onto another level, only it closes; elsewhere, the
    // whole menu, as the root never saw focus go (todo 322).
    let root = props.parents.0.first().copied();
    let (onclose, close_all) = (props.onclose, props.close_all);
    let focus_moved = use_callback(move |()| match (root, close_all) {
        (Some(root), Some(close_all)) if !root.holds_focus() => close_all.call(false),
        _ => onclose.call(()),
    });
    let dismiss = use_dismiss(
        props.anchor,
        floating,
        open,
        placed,
        Some(props.onclose),
        DismissOptions {
            onfocusmoved: Some(focus_moved),
            // The root hears every press; a submenu's is inside it.
            press: root.is_none(),
            marker: props.marker,
            ..Default::default()
        },
    );
    // Portaled beside its parents, so focus here would read as leaving them.
    // The guards unregister on drop.
    let parents = props.parents.0.clone();
    use_hook(move || {
        Rc::new(
            parents
                .iter()
                .map(|parent| parent.register_inside_box(&dismiss))
                .collect::<Vec<_>>(),
        )
    });

    let active = use_signal(|| None::<usize>);
    let mut open_child = use_signal(|| None::<usize>);
    let child_request = use_signal(|| (0u64, 0usize));
    let typeahead = use_typeahead(TYPEAHEAD_RESET);

    // A hook, not `Callback::new`: that allocated a new box on every render.
    let own_close_all = use_callback(move |restore: bool| match restore {
        true => dismiss.dismiss(),
        false => onclose.call(()),
    });
    let close_all = props.close_all.unwrap_or(own_close_all);

    let level = Level {
        floating,
        wrapper: props.wrapper,
        dismiss,
        close_all,
        onedge: props.onedge,
        active,
        open_child,
        child_request,
        len,
        depth,
        loop_focus: props.loop_focus,
        close_on_select: props.close_on_select,
    };

    // Only a chosen radio: a toggled setting is no "choice in effect".
    let first = flat
        .iter()
        .position(|item| item.check == Some(Check::Radio(true)))
        .unwrap_or(0);
    use_level_focus(level, open, placed, props.request, props.initial, first);
    let hover = use_hover_delay(
        level,
        Duration::from_millis(theme.menu.submenu_delay.into()),
    );
    let cancel_callback = use_callback(move |()| hover.cancel());

    // Per submenu item, its anchor (this scope's) and box (the root's), kept
    // across renders so neither changes under the submenu.
    let root = props.root;
    let anchors: Rc<RefCell<Vec<(ElementHandle, ElementHandle)>>> =
        use_hook(|| Rc::new(RefCell::new(Vec::new())));
    let anchor_of = |index: usize| {
        let mut anchors = anchors.borrow_mut();
        while anchors.len() <= index {
            anchors.push((ElementHandle::new(), ElementHandle::new_in_scope(root)));
        }
        anchors[index]
    };

    // Every submenu level stays mounted while this one is open, its own open or not.
    let submenus: Vec<(usize, Vec<MenuEntry>, (ElementHandle, ElementHandle))> = flat
        .iter()
        .enumerate()
        .filter_map(|(index, item)| Some((index, item.submenu_items()?.clone(), anchor_of(index))))
        .collect();

    let expanded = open_child();
    let level_id = props.id.clone();

    // Only an open level draws rows: a closed one redraws with every parent render.
    let rows = open.then(|| {
        let draw = ItemDraw {
            level,
            typeahead,
            // Labels for typeahead, `None` for an item it must skip.
            labels: Rc::new(
                flat.iter()
                    .map(|item| (!item.disabled).then(|| item.label.clone()))
                    .collect(),
            ),
            hover,
            tabbable: active().unwrap_or(first),
            expanded,
            level_id: level_id.clone(),
            checks: flat.iter().any(|item| item.check.is_some()),
        };
        let (mut index, mut group) = (0usize, 0usize);
        draw_rows(
            &props.items,
            &level_id,
            &mut index,
            &mut group,
            &mut |item, index| {
                let anchor = item.submenu_items().is_some().then(|| anchor_of(index).0);
                menu_item(&draw, item, index, anchor)
            },
        )
    });

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(props.size.state_name(), true)
        .with(props.radius.radius_state_name(), true)
        .with("bordered", true)
        .into();
    let menu = use_box()
        .framework_sx(&MENU_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&states)
        .style(open.then(|| popover.style()).flatten())
        .prepare();

    popover.show(rows.map(|rows| {
        let mut attributes = props.attributes.clone();
        attributes.extend(dismiss.floating_events());
        let onpointerenter = props.onpointerenter;
        menu.element(&floating)
            .attr("id", level_id.clone())
            .attr("role", "menu")
            .attr("tabindex", "-1")
            .attr("aria-labelledby", props.labelledby.clone())
            .event("onmouseenter", move |_: MouseEvent| {
                if let Some(onpointerenter) = onpointerenter {
                    onpointerenter.call(());
                }
            })
            // Leaving the box abandons whatever the pointer was waiting on.
            .event("onmouseleave", move |_: MouseEvent| hover.cancel())
            .event("onkeydown", move |event: KeyboardEvent| {
                level.box_keydown(event)
            })
            .render(HtmlTag::Div, attributes, rows)
    }));

    let parents = {
        let mut parents = props.parents.0.clone();
        parents.push(dismiss);
        Parents(parents)
    };
    let onclose_child = use_callback(move |()| open_child.set(None));
    // Logical, so `use_popover` puts it on the left under `dir="rtl"`.
    let submenu_side = Side::End;

    rsx! {
        for (index, items, (anchor, floating)) in submenus {
            MenuLevel {
                key: "{index}",
                items: if open && expanded == Some(index) { items } else { Vec::new() },
                id: format!("{}-{index}", props.id),
                labelledby: format!("{}-item-{index}", props.id),
                anchor,
                floating,
                root,
                wrapper: props.wrapper,
                open: open && expanded == Some(index),
                request: match child_request() {
                    (request, target) if target == index => request,
                    _ => 0,
                },
                initial: MenuFocus::First,
                onclose: onclose_child,
                close_all: Some(close_all),
                onedge: props.onedge,
                parents: parents.clone(),
                onpointerenter: Some(cancel_callback),
                side: submenu_side,
                align: Align::Start,
                size: props.size,
                radius: props.radius,
                loop_focus: props.loop_focus,
                close_on_select: props.close_on_select,
                depth: depth + 1,
                sx: props.sx.clone(),
                parts: props.parts.clone(),
            }
        }
    }
}

/// A level's rows. `index` and `group` count on across the nesting.
fn draw_rows(
    entries: &[MenuEntry],
    level_id: &str,
    index: &mut usize,
    group: &mut usize,
    draw_item: &mut dyn FnMut(&MenuItem, usize) -> Element,
) -> Vec<Element> {
    let mut rows = Vec::with_capacity(entries.len());
    for entry in entries {
        match entry {
            MenuEntry::Item(item) => {
                rows.push(draw_item(item, *index));
                *index += 1;
            }
            MenuEntry::Group { label, items } => {
                let label_id = format!("{level_id}-group-{group}");
                let key = *group;
                *group += 1;
                let inner = draw_rows(items, level_id, index, group, draw_item);
                rows.push(rsx! {
                    div {
                        key: "group-{key}",
                        "role": "group",
                        "aria-labelledby": "{label_id}",
                        div { id: "{label_id}", "data-slot": MenuPart::GroupLabel.slot(), "{label}" }
                        {inner.into_iter()}
                    }
                });
            }
            MenuEntry::Separator => {
                let key = rows.len();
                rows.push(rsx! {
                    Divider { key: "separator-{key}", spacing: "4px" }
                });
            }
        }
    }
    rows
}
