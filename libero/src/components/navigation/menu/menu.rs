use std::{cell::RefCell, rc::Rc, time::Duration};

use dioxus::prelude::*;

use crate::{
    components::{
        Divider, HtmlTag, Input, States,
        common::{ChevronRightIcon, base_props, focus_ring_sx},
        layout::use_box,
        surface::paper_sx,
    },
    hooks::{
        Align, DismissHandle, DismissOptions, ElementHandle, PopoverOptions, Side, TYPEAHEAD_RESET,
        typeahead_match, use_dismiss, use_element, use_popover, use_theme, use_typeahead,
    },
    platform::{ElementApi, TimerSubscription, timer},
    sx::{StaticSx, sx},
    theme::{
        MENU_ITEM_FONT, MENU_ITEM_MIN_HEIGHT, MENU_ITEM_PAD_X, MENU_ITEM_RADIUS, MENU_LABEL_FONT,
        MENU_MAX_HEIGHT, MENU_PADDING, MenuDefaults, Size, SizeCss, Z_INDEX_POPOVER,
    },
};

use super::{
    entry::{MenuEntry, MenuItem, flatten},
    state::{MenuFocus, MenuState, menu_id, trigger_id},
};

const ITEM: &str = "& [role=\"menuitem\"]";

// A surface, so its background, border and corner are `paper_sx()`'s - the
// `bordered` and `radius-{step}` tokens the box renders with are the ones its
// folds answer. Rendered through `use_box` rather than `Paper`, because it
// needs the popover's element handle and its own events. The items carry no
// class of their own: they are styled from here, `Tabs`' shape, so a menu of
// forty items costs one class rather than forty `use_box` calls.
static MENU_SX: StaticSx = StaticSx::new(|| {
    paper_sx()
        .and(MenuDefaults::theme_vars())
        // Everything positional comes from `use_popover` as an inline style.
        .z_index(Z_INDEX_POPOVER.value())
        .display("flex")
        .flex_direction("column")
        .padding(MENU_PADDING)
        .max_height(MENU_MAX_HEIGHT.value())
        .overflow_y("auto")
        // A menu floats over the page, where the surface default rests.
        .box_shadow(SizeCss::SHADOW.value(Size::Lg))
        .selector(
            "& [role=\"group\"]",
            sx().display("flex").flex_direction("column"),
        )
        .selector(
            "& [data-menu-group-label]",
            sx().padding(format!("6px {}", MENU_ITEM_PAD_X.value()))
                .font_size(MENU_LABEL_FONT.value())
                .font_weight("600")
                .color("grey.7")
                .user_select("none"),
        )
        .selector(
            ITEM,
            sx().display("flex")
                .align_items("center")
                .gap("10px")
                .width("100%")
                .flex_shrink("0")
                .min_height(MENU_ITEM_MIN_HEIGHT.value())
                .padding(format!("0 {}", MENU_ITEM_PAD_X.value()))
                .appearance("none")
                .border("0")
                .border_radius(MENU_ITEM_RADIUS.value())
                .background("transparent")
                .font("inherit")
                .font_size(MENU_ITEM_FONT.value())
                .color("inherit")
                .text_align("start")
                .white_space("nowrap")
                .cursor("pointer")
                .user_select("none"),
        )
        // Hover and focus draw the same tint: focus follows the pointer, so
        // the two only ever part for the moment a submenu delay holds focus
        // back.
        .selector(
            "& [role=\"menuitem\"]:hover:not([aria-disabled=\"true\"])",
            sx().background("grey.1"),
        )
        .selector("& [role=\"menuitem\"]:focus", sx().background("grey.1"))
        // `appearance: none` and `border: 0` take the UA's ring with them.
        // Inset, because the box clips at its padding edge while it scrolls.
        .selector(
            "& [role=\"menuitem\"]:focus-visible",
            focus_ring_sx().outline_offset("-2px"),
        )
        .selector(
            "& [role=\"menuitem\"][aria-disabled=\"true\"]",
            sx().color("grey.5").cursor("not-allowed"),
        )
        .selector(
            "& [data-menu-label]",
            sx().flex("1").overflow("hidden").text_overflow("ellipsis"),
        )
        .selector(
            "& [data-menu-section]",
            sx().display("inline-flex").align_items("center"),
        )
        .selector(
            "& [data-menu-chevron]",
            sx().display("inline-flex")
                .width("1em")
                .height("1em")
                .margin_right("-4px"),
        )
        .selector(
            "& [data-menu-chevron] svg",
            sx().width("100%").height("100%"),
        )
});

base_props! {
    pub struct MenuProps {
        /// From [`use_menu`](super::use_menu): the open state and the id the
        /// aria wiring is built from.
        state: MenuState,
        /// The menu, in order.
        items: Vec<MenuEntry>,
        /// Which side of the trigger the menu opens on. It flips when that
        /// side has no room.
        #[props(default)]
        side: Side,
        #[props(default)]
        align: Align,
        /// Whether choosing an item closes the menu.
        #[props(default = true)]
        close_on_select: bool,
        /// Whether the arrow keys wrap from the last item to the first.
        #[props(default = true)]
        loop_focus: bool,
        /// Item height and font size.
        #[props(default, into)]
        size: Input<Size>,
        /// The menu's corner radius. The items nest inside it with a radius
        /// tightened by the menu's padding.
        #[props(default, into)]
        radius: Input<Size>,
        /// The trigger opens nothing.
        #[props(default)]
        disabled: bool,
        /// The trigger - usually a `Button` carrying `state.a11y_attributes()`.
        /// Its clicks and keys are caught on the wrapper they bubble to.
        children: Element,
    }
}

/// A list of commands that drops from a trigger - APG's menu button.
///
/// The items are data, not children: the menu owns their order, so the arrow
/// keys, Home and End and typeahead are indexing into a `Vec` rather than
/// asking the DOM. Focus moves to the item (a roving `tabindex`), so a screen
/// reader follows it; Escape closes and hands focus back to the trigger, and
/// Tab closes and moves on.
///
/// `sx`, `class`, `states` and `attributes` land on the menu box - the
/// wrapper around the trigger is scaffolding, not a user-facing element.
#[component]
pub fn Menu(props: MenuProps) -> Element {
    let theme = use_theme();
    let state = props.state;
    let disabled = props.disabled;
    let id = state.id();

    let wrapper = use_element();
    let wrapper_box = use_box().prepare();

    let open = state.opened() && !disabled;

    let onclose = use_callback(move |()| state.close());

    rsx! {
        {
            wrapper_box
                .element(&wrapper)
                // The trigger is the caller's, so its clicks and keys are
                // caught where they bubble to. Enter and Space on a real button
                // arrive here as a click.
                .event("onclick", move |_: MouseEvent| {
                    if !disabled {
                        state.toggle();
                    }
                })
                .event("onkeydown", move |event: KeyboardEvent| {
                    if disabled {
                        return;
                    }
                    match event.key() {
                        Key::ArrowDown => {
                            event.prevent_default();
                            state.open_at(MenuFocus::First);
                        }
                        Key::ArrowUp => {
                            event.prevent_default();
                            state.open_at(MenuFocus::Last);
                        }
                        // Focus is on the trigger while the menu is open only
                        // when something put it back there. Off the web no
                        // document listener hears this press, so the trigger
                        // has to; on the web the stack's listener closes it
                        // too, and closing twice is closing.
                        Key::Escape if state.opened() => {
                            event.prevent_default();
                            event.stop_propagation();
                            state.close();
                        }
                        _ => {}
                    }
                })
                .render(HtmlTag::Div, Vec::new(), props.children)
        }
        MenuLevel {
            items: props.items,
            id: menu_id(&id),
            labelledby: trigger_id(&id),
            anchor: wrapper,
            wrapper,
            open,
            request: state.request().0,
            initial: state.request().1,
            onclose,
            close_all: None,
            parents: Parents(Vec::new()),
            onpointerenter: None,
            side: props.side,
            align: props.align,
            size: props.size.copied_or(theme.menu.size),
            radius: props.radius.copied_or(theme.menu.radius),
            loop_focus: props.loop_focus,
            close_on_select: props.close_on_select,
            depth: 0,
            attributes: props.attributes,
            class: props.class,
            sx: props.sx,
            states: props.states,
        }
    }
}

/// The dismissal handles of every menu above a submenu. Focus moving into the
/// submenu must count as inside each of them, not only its parent: a third
/// level is no descendant of the first.
///
/// Always equal - the handles' registration signals never change identity, so
/// there is nothing here for a comparison to find.
#[derive(Clone)]
struct Parents(Vec<DismissHandle>);

impl PartialEq for Parents {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

/// What the pointer resting on an item does once the delay runs out.
#[derive(Clone, Copy, PartialEq)]
struct HoverAction {
    focus: usize,
    /// The submenu to have open afterwards, if any - `None` closes the one
    /// that is.
    open: Option<usize>,
}

/// The `Copy` half of one menu's state, shared by every item's handlers.
#[derive(Clone, Copy)]
struct Level {
    floating: ElementHandle,
    wrapper: ElementHandle,
    dismiss: DismissHandle,
    close_all: Callback<bool>,
    active: Signal<Option<usize>>,
    open_child: Signal<Option<usize>>,
    /// Which submenu to focus into, and a counter that makes asking twice
    /// count.
    child_request: Signal<(u64, usize)>,
    len: usize,
    depth: usize,
    loop_focus: bool,
    close_on_select: bool,
}

impl Level {
    /// Moves focus to item `index` - the roving `tabindex` follows `active`.
    fn focus(self, index: usize) {
        let mut active = self.active;
        active.set(Some(index));
        let _ = self
            .floating
            .query_selector(&format!("[data-menu-index=\"{index}\"]"))
            .and_then(|item| item.focus());
    }

    fn step(self, from: usize, forward: bool) {
        let last = self.len - 1;
        let next = match (forward, from) {
            (true, from) if from < last => from + 1,
            (true, _) if self.loop_focus => 0,
            (false, 0) if self.loop_focus => last,
            (false, from) => from.saturating_sub(1),
            (true, _) => last,
        };
        self.focus(next);
    }

    /// Opens the submenu under `index` and asks it to focus its first item.
    fn enter_submenu(self, index: usize) {
        let mut open_child = self.open_child;
        open_child.set(Some(index));
        let mut request = self.child_request;
        let next = request.peek().0.wrapping_add(1);
        request.set((next, index));
    }

    /// Tab leaves the whole menu. Focus goes to the trigger first, still
    /// inside this keydown, so the browser's own Tab then moves on from
    /// there - to whatever follows the trigger, not whatever follows the
    /// portal outlet at the end of the document.
    fn tab_out(self) {
        let _ = self
            .wrapper
            .query_selector("[aria-haspopup=\"menu\"]")
            .and_then(|trigger| trigger.focus());
        self.close_all.call(false);
    }

    fn choose(self, index: usize, on_select: Option<Callback<()>>, submenu: bool) {
        if submenu {
            self.enter_submenu(index);
            return;
        }
        if let Some(on_select) = on_select {
            on_select.call(());
        }
        if self.close_on_select {
            self.close_all.call(true);
        }
    }
}

// Every prop is a value or a stable handle; a submenu's `items` never compares
// equal (see `MenuItem`'s `PartialEq`), so a level redraws with its parent.
#[derive(Props, Clone, PartialEq)]
struct MenuLevelProps {
    items: Vec<MenuEntry>,
    id: String,
    labelledby: String,
    anchor: ElementHandle,
    /// The root `Menu`'s wrapper, which holds the trigger.
    wrapper: ElementHandle,
    open: bool,
    /// Focus an item once placed, whenever this grows.
    request: u64,
    initial: MenuFocus,
    onclose: Callback<()>,
    /// `None` on the root, which builds it from its own dismissal. `true`
    /// hands focus back to the trigger.
    close_all: Option<Callback<bool>>,
    parents: Parents,
    /// The pointer entered this box - the parent cancels a pending hover, so
    /// crossing a sibling on the way in does not close this submenu.
    onpointerenter: Option<Callback<()>>,
    side: Side,
    align: Align,
    size: Size,
    radius: Size,
    loop_focus: bool,
    close_on_select: bool,
    depth: usize,
    #[props(default)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Input<crate::components::ClassList>,
    #[props(default)]
    sx: Input<crate::sx::Sx>,
    #[props(default)]
    states: Input<States>,
}

/// One floating menu - the root, or a submenu. Submenus are rendered *here*,
/// beside the portal rather than inside it, so every level is a descendant of
/// `Menu` and reads nothing across the portal boundary
/// ([[codebase/use-popover]]). Only the boxes are portaled.
#[component]
fn MenuLevel(props: MenuLevelProps) -> Element {
    let theme = use_theme();
    let open = props.open;
    let depth = props.depth;
    let flat = flatten(&props.items);
    let len = flat.len();

    // Every one of these is a hook, so all of them run before anything
    // branches on `open`.
    let popover = use_popover(
        props.anchor,
        open,
        PopoverOptions::new(theme.popover.gap, theme.popover.padding)
            .side(props.side)
            .align(props.align),
    );
    let floating = *popover.floating();
    let placed = popover.placed();
    let dismiss = use_dismiss(
        props.anchor,
        floating,
        open,
        placed,
        Some(props.onclose),
        DismissOptions::default(),
    );
    // A submenu is portaled beside its parent, not inside it, so focus moving
    // into it reads as focus leaving every menu above it. Registered for as
    // long as this level exists; the guards unregister on drop.
    let parents = props.parents.0.clone();
    use_hook(move || {
        Rc::new(
            parents
                .iter()
                .map(|parent| parent.register_inside(floating))
                .collect::<Vec<_>>(),
        )
    });

    let active = use_signal(|| None::<usize>);
    let mut open_child = use_signal(|| None::<usize>);
    let child_request = use_signal(|| (0u64, 0usize));
    let typeahead = use_typeahead(TYPEAHEAD_RESET);

    let close_all = props.close_all.unwrap_or_else(|| {
        let onclose = props.onclose;
        Callback::new(move |restore: bool| match restore {
            true => dismiss.dismiss(),
            false => onclose.call(()),
        })
    });

    let level = Level {
        floating,
        wrapper: props.wrapper,
        dismiss,
        close_all,
        active,
        open_child,
        child_request,
        len,
        depth,
        loop_focus: props.loop_focus,
        close_on_select: props.close_on_select,
    };

    // Remembered on the opening edge, before anything has moved focus: focus
    // only enters the box once it is placed, which is a measurement later.
    // For the root that is the trigger; for a submenu, its item in the parent.
    use_effect(use_reactive!(|(open,)| {
        let (mut active, mut open_child) = (active, open_child);
        match open {
            true => dismiss.focus_return().remember_active(),
            false => {
                active.set(None);
                open_child.set(None);
            }
        }
    }));

    // Focuses the requested item once the box is placed - `focus()` on the
    // pre-placement `visibility: hidden` box answers `Ok` and moves nothing.
    // A request only counts while it is newer than the last one handled, so a
    // submenu the pointer opened does not steal focus from its parent item.
    let mut seen = use_signal(|| props.request);
    let request = props.request;
    let initial = props.initial;
    use_effect(use_reactive!(|(open, placed, request, len)| {
        // Read first, branch second: this subscribes the effect to a remount.
        let mounted = floating.mount_token().is_some();
        if !open || !placed || !mounted || len == 0 || request <= *seen.peek() {
            return;
        }
        seen.set(request);
        let index = match initial {
            MenuFocus::First => 0,
            MenuFocus::Last => len - 1,
        };
        // Out of this dispatch: the click that opened the menu ends by
        // focusing the trigger.
        spawn(async move { level.focus(index) });
    }));

    // The pointer resting on an item. The timer's callback runs outside every
    // scope - and on the web with no runtime at all - so it only records what
    // to do, in a root-owned signal, and the effect below does it
    // ([[codebase/platform-timer]]).
    let hover_fire = use_hook(|| Signal::new_in_scope(None::<HoverAction>, ScopeId::ROOT));
    let hover_timer: Rc<RefCell<Option<Box<dyn TimerSubscription>>>> =
        use_hook(|| Rc::new(RefCell::new(None)));
    use_drop({
        let hover_timer = hover_timer.clone();
        move || {
            hover_timer.borrow_mut().take();
            hover_fire.manually_drop();
        }
    });
    use_effect(move || {
        let Some(action) = hover_fire() else {
            return;
        };
        let mut hover_fire = hover_fire;
        hover_fire.set(None);
        level.focus(action.focus);
        open_child.set(action.open);
    });

    let delay = Duration::from_millis(theme.menu.submenu_delay.into());
    let schedule = {
        let hover_timer = hover_timer.clone();
        move |action: HoverAction| {
            // Replacing the subscription drops the old one, which cancels it.
            *hover_timer.borrow_mut() = timer().map(|timer| {
                timer.after(
                    delay,
                    Box::new(move || {
                        let mut hover_fire = hover_fire;
                        hover_fire.set(Some(action));
                    }),
                )
            });
        }
    };
    let cancel = {
        let hover_timer = hover_timer.clone();
        move || {
            hover_timer.borrow_mut().take();
        }
    };
    let cancel_callback = use_callback({
        let cancel = cancel.clone();
        move |()| cancel()
    });

    // Labels for typeahead, `None` for an item it must skip.
    let labels: Rc<Vec<Option<String>>> = Rc::new(
        flat.iter()
            .map(|item| (!item.disabled).then(|| item.label.clone()))
            .collect(),
    );

    // One element handle per item that opens a submenu - the submenu's
    // anchor. Kept across renders so a submenu's anchor never changes under
    // it; created on first need, owned by this scope.
    let anchors: Rc<RefCell<Vec<ElementHandle>>> = use_hook(|| Rc::new(RefCell::new(Vec::new())));
    let anchor_of = |index: usize| {
        let mut anchors = anchors.borrow_mut();
        while anchors.len() <= index {
            anchors.push(ElementHandle::new());
        }
        anchors[index]
    };

    // The roving `tabindex`: exactly one item is tabbable, the focused one, or
    // the first before anything has been focused.
    let tabbable = active().unwrap_or(0);
    let expanded = open_child();
    let level_id = props.id.clone();

    let mut index = 0usize;
    let mut group = 0usize;
    let mut submenus: Vec<(usize, Vec<MenuEntry>, ElementHandle)> = Vec::new();

    let mut draw_item = |item: &MenuItem, index: usize| -> Element {
        let submenu = item.submenu_items().cloned();
        let has_submenu = submenu.is_some();
        let disabled = item.disabled;
        let on_select = item.on_select_callback();
        let item_id = format!("{level_id}-item-{index}");
        let child_id = format!("{level_id}-{index}");
        let anchor = has_submenu.then(|| anchor_of(index));
        if let (Some(items), Some(anchor)) = (submenu, anchor) {
            submenus.push((index, items, anchor));
        }
        let is_expanded = has_submenu && expanded == Some(index);

        let onkeydown = {
            let typeahead = typeahead.clone();
            let labels = labels.clone();
            let cancel = cancel.clone();
            move |event: KeyboardEvent| {
                match event.key() {
                    Key::ArrowDown => {
                        event.prevent_default();
                        level.step(index, true);
                    }
                    Key::ArrowUp => {
                        event.prevent_default();
                        level.step(index, false);
                    }
                    Key::Home => {
                        event.prevent_default();
                        level.focus(0);
                    }
                    Key::End => {
                        event.prevent_default();
                        level.focus(level.len - 1);
                    }
                    Key::ArrowRight if has_submenu && !disabled => {
                        event.prevent_default();
                        cancel();
                        level.enter_submenu(index);
                    }
                    // Closes this submenu only, and focus goes back to the
                    // item that opened it. On the root it is a menubar's key
                    // (D3), so it does nothing here.
                    Key::ArrowLeft if level.depth > 0 => {
                        event.prevent_default();
                        level.dismiss.dismiss();
                    }
                    Key::Tab => level.tab_out(),
                    Key::Character(ref text) if !has_shortcut_modifier(&event) => {
                        let Some(ch) = text.chars().next() else {
                            return;
                        };
                        // A space mid-query is part of "save as". Otherwise it
                        // is left alone, and the button turns it into a click.
                        if ch == ' ' && !typeahead.is_typing() {
                            return;
                        }
                        let query = typeahead.push(ch);
                        let found = typeahead_match(labels.len(), Some(index), &query, |row| {
                            labels[row].as_deref()
                        });
                        if ch == ' ' || found.is_some() {
                            event.prevent_default();
                        }
                        if let Some(row) = found {
                            level.focus(row);
                        }
                    }
                    _ => {}
                }
            }
        };

        let onmouseenter = {
            let schedule = schedule.clone();
            let cancel = cancel.clone();
            move |_: MouseEvent| {
                cancel();
                let opens = (has_submenu && !disabled).then_some(index);
                let expanded = *open_child.peek();
                match expanded {
                    // Another item's submenu is open: wait, so a pointer
                    // crossing this item on its way into that submenu does not
                    // close it.
                    Some(open) if open != index => schedule(HoverAction {
                        focus: index,
                        open: opens,
                    }),
                    open => {
                        level.focus(index);
                        if opens.is_some() && open != opens {
                            schedule(HoverAction {
                                focus: index,
                                open: opens,
                            });
                        }
                    }
                }
            }
        };

        let onclick = {
            let cancel = cancel.clone();
            move |_: MouseEvent| {
                if disabled {
                    return;
                }
                cancel();
                level.choose(index, on_select, has_submenu);
            }
        };

        rsx! {
            button {
                key: "{index}",
                r#type: "button",
                "role": "menuitem",
                id: "{item_id}",
                tabindex: if index == tabbable { "0" } else { "-1" },
                "data-menu-index": "{index}",
                "aria-disabled": disabled.then_some("true"),
                "aria-haspopup": has_submenu.then_some("menu"),
                "aria-expanded": has_submenu.then(|| is_expanded.to_string()),
                "aria-controls": is_expanded.then_some(child_id),
                onmounted: move |event| {
                    if let Some(anchor) = anchor {
                        anchor.mount()(event);
                    }
                },
                onclick,
                onkeydown,
                onmouseenter,
                if let Some(leading) = item.leading.clone() {
                    span { "data-menu-section": "leading", {leading} }
                }
                span { "data-menu-label": "", "{item.label}" }
                if let Some(trailing) = item.trailing.clone() {
                    span { "data-menu-section": "trailing", {trailing} }
                }
                if has_submenu {
                    span { "data-menu-chevron": "", ChevronRightIcon {} }
                }
            }
        }
    };

    fn draw(
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
                    let inner = draw(items, level_id, index, group, draw_item);
                    rows.push(rsx! {
                        div {
                            key: "group-{key}",
                            "role": "group",
                            "aria-labelledby": "{label_id}",
                            div { id: "{label_id}", "data-menu-group-label": "", "{label}" }
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

    let rows = draw(
        &props.items,
        &level_id,
        &mut index,
        &mut group,
        &mut draw_item,
    );

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
        .states(&states)
        .style(popover.style())
        .prepare();

    popover.show(open.then(|| {
        let mut attributes = props.attributes.clone();
        attributes.extend(dismiss.floating_events());
        let onpointerenter = props.onpointerenter;
        let cancel = cancel.clone();
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
            .event("onmouseleave", move |_: MouseEvent| cancel())
            .render(HtmlTag::Div, attributes, rsx! { {rows.into_iter()} })
    }));

    let parents = {
        let mut parents = props.parents.0.clone();
        parents.push(dismiss);
        Parents(parents)
    };
    let onclose_child = use_callback(move |()| open_child.set(None));

    rsx! {
        for (index, items, anchor) in submenus {
            MenuLevel {
                key: "{index}",
                items,
                id: format!("{}-{index}", props.id),
                labelledby: format!("{}-item-{index}", props.id),
                anchor,
                wrapper: props.wrapper,
                open: open && expanded == Some(index),
                request: match child_request() {
                    (request, target) if target == index => request,
                    _ => 0,
                },
                initial: MenuFocus::First,
                onclose: onclose_child,
                close_all: Some(close_all),
                parents: parents.clone(),
                onpointerenter: Some(cancel_callback),
                side: Side::Right,
                align: Align::Start,
                size: props.size,
                radius: props.radius,
                loop_focus: props.loop_focus,
                close_on_select: props.close_on_select,
                depth: depth + 1,
            }
        }
    }
}

// Shift is part of ordinary typing; the rest mark a browser or OS shortcut
// that must not be mistaken for typeahead. `Tree`'s rule.
fn has_shortcut_modifier(event: &KeyboardEvent) -> bool {
    let modifiers = event.modifiers();
    modifiers.ctrl() || modifiers.alt() || modifiers.meta()
}
