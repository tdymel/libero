use std::{cell::RefCell, rc::Rc, time::Duration};

use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            CheckIcon, ChevronRightIcon, HtmlTag, Input, States, base_props, disabled_look_sx,
            has_shortcut_modifier, inset_focus_ring_sx,
        },
        layout::{Divider, paper_sx, use_box},
        typography::Kbd,
    },
    hooks::{
        Align, DismissHandle, DismissOptions, ElementHandle, PopoverOptions, Side, TYPEAHEAD_RESET,
        Typeahead, current_localization, typeahead_match, use_dismiss, use_element, use_popover_on,
        use_theme, use_typeahead,
    },
    platform::{ElementApi, TimerSubscription, logical_key, timer},
    sx::{StaticSx, sx},
    theme::{
        MENU_ITEM_FONT, MENU_ITEM_MIN_HEIGHT, MENU_ITEM_PAD_X, MENU_ITEM_RADIUS, MENU_LABEL_FONT,
        MENU_MAX_HEIGHT, MENU_PADDING, MenuDefaults, Size, SizeCss, Z_INDEX_POPOVER,
    },
};

use super::{
    entry::{Check, MenuEntry, MenuItem, flatten, shortcut_hint},
    state::{MenuFocus, MenuState, menu_id, trigger_id},
};

// Every row carries its index, whichever of the three item roles it has, so
// the rules key on that rather than on a role.
const ITEM: &str = "& [data-menu-index]";

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
                .color("muted.7")
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
            "& [data-menu-index]:hover:not([aria-disabled=\"true\"])",
            sx().background("muted.1"),
        )
        .selector("& [data-menu-index]:focus", sx().background("muted.1"))
        // `appearance: none` and `border: 0` take the UA's ring with them.
        // Inset, because the box clips at its padding edge while it scrolls.
        .selector(
            "& [data-menu-index]:focus-visible",
            inset_focus_ring_sx("-2px"),
        )
        // The fade dims the shortcut's key cap along with the label.
        .selector(
            "& [data-menu-index][aria-disabled=\"true\"]",
            disabled_look_sx("not-allowed"),
        )
        // Wraps rather than truncates once the box meets the viewport's edge:
        // an ellipsis hides the label from sighted users (1.4.10). Inside a
        // long word too.
        .selector(
            "& [data-menu-label]",
            sx().flex("1")
                .min_width("0")
                .padding("4px 0")
                .white_space("normal")
                .with("overflow-wrap", "anywhere"),
        )
        .selector(
            "& [data-menu-section]",
            sx().display("inline-flex").align_items("center"),
        )
        .selector(
            "& :is([data-menu-chevron], [data-menu-check])",
            sx().display("inline-flex")
                .width("1em")
                .height("1em")
                .with("margin-inline-end", "-4px"),
        )
        .selector(
            "& :is([data-menu-chevron], [data-menu-check]) svg",
            sx().width("100%").height("100%"),
        )
        // The submenu opens on the left under RTL, so the chevron points there.
        .rtl(sx().selector("& [data-menu-chevron] svg", sx().transform("scaleX(-1)")))
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
        #[props(default)]
        close_on_select: Option<bool>,
        /// Whether the arrow keys wrap from the last item to the first.
        #[props(default)]
        loop_focus: Option<bool>,
        /// Item height and font size.
        #[props(default, into)]
        size: Input<Size>,
        /// The menu's corner radius. The items nest inside it with a radius
        /// tightened by the menu's padding.
        #[props(default, into)]
        radius: Input<Size>,
        /// The trigger opens nothing, and an open menu closes.
        #[props(default)]
        disabled: Option<bool>,
        /// Hears ArrowLeft and ArrowRight when the menu has no submenu of its
        /// own to answer them with - a menubar moving to the next menu. With
        /// `None` those keys do what they always did.
        #[props(default)]
        onedge: Option<Callback<MenuEdge>>,
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
    let disabled = props.disabled.unwrap_or(false);
    let id = state.id();

    let wrapper = use_element();
    let floating = use_element();
    let owner = use_hook(dioxus::core::current_scope_id);
    let wrapper_box = use_box().prepare();

    let opened = state.is_open();
    let open = opened && !disabled;
    // A disabled menu is a closed one, not an open one that happens to be
    // hidden: otherwise the trigger's `aria-expanded` claims a menu nobody can
    // see, and re-enabling it would pop the menu back up unasked. Covers both
    // disabling an open menu and a programmatic `open()` while disabled.
    use_effect(use_reactive!(|(disabled, opened)| {
        if disabled && opened {
            state.close();
        }
    }));

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
                    // Alt+ArrowDown and the like are the browser's or the page's.
                    if disabled || has_shortcut_modifier(&event) {
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
                        Key::Escape if state.is_open() => {
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
            // Never equal while it holds items, so a closed level gets none and
            // skips its parent's renders. Its submenu levels unmount meanwhile.
            items: if open { props.items } else { Vec::new() },
            id: menu_id(&id),
            labelledby: trigger_id(&id),
            anchor: wrapper,
            floating,
            root: owner,
            wrapper,
            open,
            request: state.request().0,
            initial: state.request().1,
            onclose,
            close_all: None,
            onedge: props.onedge,
            parents: Parents(Vec::new()),
            onpointerenter: None,
            side: props.side,
            align: props.align,
            size: props.size.copied_or(theme.menu.size),
            radius: props.radius.copied_or(theme.menu.radius),
            loop_focus: props.loop_focus.unwrap_or(theme.menu.loop_focus),
            close_on_select: props
                .close_on_select
                .unwrap_or(theme.menu.close_on_select),
            depth: 0,
            attributes: props.attributes,
            class: props.class,
            sx: props.sx,
            states: props.states,
        }
    }
}

/// Which neighbour of the menu ArrowLeft or ArrowRight asked for, handed to
/// [`MenuProps::onedge`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuEdge {
    Previous,
    Next,
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
    onedge: Option<Callback<MenuEdge>>,
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

    /// Keys on the box itself, which a click on a group label, a separator or
    /// the padding focuses. An item's own keys bubble here too and pass.
    fn box_keydown(self, event: KeyboardEvent) {
        if !self.floating.is_focused() || self.len == 0 || has_shortcut_modifier(&event) {
            return;
        }
        match event.key() {
            Key::ArrowDown | Key::Home => {
                event.prevent_default();
                self.focus(0);
            }
            Key::ArrowUp | Key::End => {
                event.prevent_default();
                self.focus(self.len - 1);
            }
            Key::Tab => self.tab_out(),
            _ => {}
        }
    }

    fn choose(self, index: usize, onselect: Option<Callback<()>>, submenu: bool) {
        if submenu {
            self.enter_submenu(index);
            return;
        }
        if let Some(onselect) = onselect {
            onselect.call(());
        }
        if self.close_on_select {
            self.close_all.call(true);
        }
    }

    /// The keys on item `index`. `opens` says it has a submenu that is enabled.
    fn item_keydown(
        self,
        event: KeyboardEvent,
        index: usize,
        opens: bool,
        typeahead: &Typeahead,
        labels: &[Option<String>],
        hover: &HoverDelay,
    ) {
        // Ctrl/Alt/Meta with a navigation key is the browser's chord.
        let navigation = matches!(
            event.key(),
            Key::ArrowDown | Key::ArrowUp | Key::ArrowLeft | Key::ArrowRight | Key::Home | Key::End
        );
        if navigation && has_shortcut_modifier(&event) {
            return;
        }
        let key = logical_key(&event);
        match key {
            Key::ArrowDown => {
                event.prevent_default();
                self.step(index, true);
            }
            Key::ArrowUp => {
                event.prevent_default();
                self.step(index, false);
            }
            Key::Home => {
                event.prevent_default();
                self.focus(0);
            }
            Key::End => {
                event.prevent_default();
                self.focus(self.len - 1);
            }
            Key::ArrowRight if opens => {
                event.prevent_default();
                hover.cancel();
                self.enter_submenu(index);
            }
            // Closes this submenu only, and focus goes back to the item that
            // opened it. On the root it is the menubar's.
            Key::ArrowLeft if self.depth > 0 => {
                event.prevent_default();
                self.dismiss.dismiss();
            }
            // Nothing here answers these, so they go up - at any depth for
            // ArrowRight, the root only for ArrowLeft.
            Key::ArrowLeft | Key::ArrowRight if self.onedge.is_some() => {
                event.prevent_default();
                if let Some(onedge) = self.onedge {
                    onedge.call(match key {
                        Key::ArrowLeft => MenuEdge::Previous,
                        _ => MenuEdge::Next,
                    });
                }
            }
            Key::Tab => self.tab_out(),
            Key::Character(ref text) if !has_shortcut_modifier(&event) => {
                let Some(ch) = text.chars().next() else {
                    return;
                };
                // A space mid-query is part of "save as". Otherwise it is left
                // alone, and the button turns it into a click.
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
                    self.focus(row);
                }
            }
            _ => {}
        }
    }
}

/// Remembers where focus returns to, and focuses the requested item.
/// `first` is where [`MenuFocus::First`] lands: the checked item of a
/// menu that has one, because opening a menu of choices should put the
/// reader on the one in effect, not make them hunt for it.
fn use_level_focus(
    level: Level,
    open: bool,
    placed: bool,
    request: u64,
    initial: MenuFocus,
    first: usize,
) {
    let (floating, dismiss, len) = (level.floating, level.dismiss, level.len);
    // Remembered on the opening edge, before anything has moved focus: focus
    // only enters the box once it is placed, which is a measurement later.
    // For the root that is the trigger; for a submenu, its item in the parent.
    use_effect(use_reactive!(|(open,)| {
        let (mut active, mut open_child) = (level.active, level.open_child);
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
    // A level mounting open was asked before it existed: that request counts.
    let mut seen = use_signal(|| if open { 0 } else { request });
    use_effect(use_reactive!(|(open, placed, request, len, first)| {
        // Read first, branch second: this subscribes the effect to a remount.
        let mounted = floating.mount_token().is_some();
        if !open || !placed || !mounted || len == 0 || request <= *seen.peek() {
            return;
        }
        seen.set(request);
        let index = match initial {
            MenuFocus::First => first,
            MenuFocus::Last => len - 1,
        };
        // Out of this dispatch: the click that opened the menu ends by
        // focusing the trigger.
        spawn(async move { level.focus(index) });
    }));
}

/// The pointer resting on an item. The timer's callback runs outside every
/// scope - and on the web with no runtime at all - so it only records what to
/// do, in a root-owned signal, and an effect does it
/// ([[codebase/platform-timer]]).
#[derive(Clone)]
struct HoverDelay {
    fire: Signal<Option<HoverAction>>,
    timer: Rc<RefCell<Option<Box<dyn TimerSubscription>>>>,
    delay: Duration,
}

impl HoverDelay {
    fn schedule(&self, action: HoverAction) {
        let fire = self.fire;
        // Replacing the subscription drops the old one, which cancels it.
        *self.timer.borrow_mut() = timer().map(|timer| {
            timer.after(
                self.delay,
                Box::new(move || {
                    let mut fire = fire;
                    fire.set(Some(action));
                }),
            )
        });
    }

    fn cancel(&self) {
        self.timer.borrow_mut().take();
    }

    /// The pointer entered item `index`, which opens submenu `opens`.
    fn enter(&self, level: Level, index: usize, opens: Option<usize>) {
        self.cancel();
        let expanded = *level.open_child.peek();
        match expanded {
            // Another item's submenu is open: wait, so a pointer crossing this
            // item on its way into that submenu does not close it.
            Some(open) if open != index => self.schedule(HoverAction {
                focus: index,
                open: opens,
            }),
            open => {
                level.focus(index);
                if opens.is_some() && open != opens {
                    self.schedule(HoverAction {
                        focus: index,
                        open: opens,
                    });
                }
            }
        }
    }
}

fn use_hover_delay(level: Level, delay: Duration) -> HoverDelay {
    let fire = use_hook(|| Signal::new_in_scope(None::<HoverAction>, ScopeId::ROOT));
    let timer: Rc<RefCell<Option<Box<dyn TimerSubscription>>>> =
        use_hook(|| Rc::new(RefCell::new(None)));
    use_drop({
        let timer = timer.clone();
        move || {
            timer.borrow_mut().take();
            fire.manually_drop();
        }
    });
    use_effect(move || {
        let Some(action) = fire() else {
            return;
        };
        let (mut fire, mut open_child) = (fire, level.open_child);
        fire.set(None);
        level.focus(action.focus);
        open_child.set(action.open);
    });
    HoverDelay { fire, timer, delay }
}

// Every prop is a value or a stable handle; a submenu's `items` never compares
// equal (see `MenuItem`'s `PartialEq`), so a level redraws with its parent.
#[derive(Props, Clone, PartialEq)]
struct MenuLevelProps {
    items: Vec<MenuEntry>,
    id: String,
    labelledby: String,
    anchor: ElementHandle,
    /// This level's box. Every level above it reads the box to tell whether
    /// focus is still inside, so it is owned by `root`, the root `Menu`'s scope,
    /// never by the level itself: a handle read by a scope that is not a
    /// descendant of its owner is a dioxus warning (todo 283).
    floating: ElementHandle,
    root: ScopeId,
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
    /// The root's `onedge`, handed to every level: ArrowRight on a plain item
    /// in a submenu moves on too, APG's menubar.
    onedge: Option<Callback<MenuEdge>>,
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
    class: Input<crate::components::common::ClassList>,
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
    // Focus left a submenu. Onto another level of this menu, only the submenu
    // closes; anywhere else, the whole menu does. The root's box never saw
    // focus go - it was in the portaled submenu - so without this an outside
    // click left the root open (todo 322). The root's `holds_focus` covers
    // every level, since each one registers with every level above it.
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
            ..Default::default()
        },
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
    let cancel_callback = use_callback({
        let hover = hover.clone();
        move |()| hover.cancel()
    });

    // Two element handles per item that opens a submenu - the submenu's
    // anchor and its box. Kept across renders so neither changes under the
    // submenu; created on first need. The anchor is owned by this scope, the
    // box by the root `Menu`, which every level that reads it descends from.
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

    // Only an open level draws its rows: a closed one redraws with every
    // parent render, and the rows only ever land in the portaled box.
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
            hover: hover.clone(),
            // The roving `tabindex`: exactly one item is tabbable, the focused
            // one, or where focus lands before anything has been focused.
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
        .states(&states)
        .style(open.then(|| popover.style()).flatten())
        .prepare();

    popover.show(rows.map(|rows| {
        let mut attributes = props.attributes.clone();
        attributes.extend(dismiss.floating_events());
        let onpointerenter = props.onpointerenter;
        let hover = hover.clone();
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
            }
        }
    }
}

/// A level's rows: items, groups with their labels, separators. `index` and
/// `group` count on across the nesting.
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

/// What every item on one level is drawn from. The three handlers each need
/// the typeahead buffer, the labels and the hover timer, so they travel
/// together.
struct ItemDraw {
    level: Level,
    typeahead: Typeahead,
    labels: Rc<Vec<Option<String>>>,
    hover: HoverDelay,
    /// The roving `tabindex`: exactly one item is tabbable, the focused one,
    /// or the first before anything has been focused.
    tabbable: usize,
    expanded: Option<usize>,
    level_id: String,
    /// Some item on the level is checkable: every row keeps a check column.
    checks: bool,
}

/// One `menuitem` row - or `menuitemradio` / `menuitemcheckbox`, when the
/// item is [`radio`](MenuItem::radio) / [`checkbox`](MenuItem::checkbox):
/// its three handlers, its aria, and the check, leading,
/// trailing, shortcut and chevron slots.
fn menu_item(
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
    let (level, tabbable) = (*level, *tabbable);
    let has_submenu = item.submenu_items().is_some();
    let check = item.check;
    let disabled = item.disabled;
    let onselect = item.onselect_callback();
    let item_id = format!("{level_id}-item-{index}");
    let child_id = format!("{level_id}-{index}");
    let is_expanded = has_submenu && *expanded == Some(index);
    let opens = (has_submenu && !disabled).then_some(index);

    let onkeydown = {
        let (typeahead, labels, hover) = (typeahead.clone(), labels.clone(), hover.clone());
        move |event: KeyboardEvent| {
            level.item_keydown(event, index, opens.is_some(), &typeahead, &labels, &hover)
        }
    };
    let onmouseenter = {
        let hover = hover.clone();
        move |_: MouseEvent| hover.enter(level, index, opens)
    };
    let onclick = {
        let hover = hover.clone();
        move |_: MouseEvent| {
            if disabled {
                return;
            }
            hover.cancel();
            level.choose(index, onselect, has_submenu);
        }
    };

    rsx! {
        button {
            key: "{index}",
            r#type: "button",
            "role": check.map_or("menuitem", Check::role),
            id: "{item_id}",
            "aria-checked": check.map(|check| check.is_checked().to_string()),
            "aria-keyshortcuts": item.shortcut.clone(),
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
            // Drawn on every row of a level with a checkable item, checked or
            // not, so all its labels line up.
            if check.is_some() || *checks {
                span { "data-menu-check": "",
                    if check.is_some_and(Check::is_checked) {
                        CheckIcon {}
                    }
                }
            }
            if let Some(leading) = item.leading.clone() {
                span { "data-menu-section": "leading", {leading} }
            }
            span { "data-menu-label": "", "{item.label}" }
            if let Some(trailing) = item.trailing.clone() {
                span { "data-menu-section": "trailing", {trailing} }
            }
            // Out of the name: `aria-keyshortcuts` announces it instead.
            if let Some(keys) = item.shortcut.as_deref() {
                span { "data-menu-section": "shortcut", "aria-hidden": "true",
                    Kbd { {shortcut_hint(keys, &current_localization().menu)} }
                }
            }
            if has_submenu {
                span { "data-menu-chevron": "", ChevronRightIcon {} }
            }
        }
    }
}
