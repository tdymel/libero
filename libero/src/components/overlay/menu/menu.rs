use dioxus::prelude::*;

use super::{
    entry::MenuEntry,
    level::{MenuLevel, Parents},
    state::{MenuFocus, MenuState, menu_id},
};
use crate::{
    components::{
        common::{
            HtmlTag, Input, ScaleOrCss, base_props, css_string, has_shortcut_modifier, parts_enum,
        },
        layout::use_box,
    },
    hooks::{Align, Side, id_selector, use_element, use_press_marker, use_theme},
    platform::mounted_by_selector,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::Size,
};

// No box of its own, so a trigger sized in percent fills the Menu's parent (minmax 3893).
static MENU_WRAPPER_SX: StaticSx = StaticSx::new(|| sx().display("contents"));

parts_enum! {
    /// [`Menu`]'s inner parts, for its `parts` prop. Every level's box takes them,
    /// submenus too. An item sits in the box or in a group, so `Item` is a descendant.
    pub enum MenuPart {
        /// A `menuitem`, `menuitemradio` or `menuitemcheckbox` row.
        Item = "item" => "& [data-slot='item']",
        /// A group's visible name.
        GroupLabel = "group-label" => "& [data-slot='group-label']",
        /// The check column, on every row of a level with a checkable item.
        Check = "check" => "& [data-slot='item'] > [data-slot='check']",
        Leading = "leading" => "& [data-slot='item'] > [data-slot='leading']",
        Label = "label" => "& [data-slot='item'] > [data-slot='label']",
        Trailing = "trailing" => "& [data-slot='item'] > [data-slot='trailing']",
        /// The key hint, hidden from screen readers.
        Shortcut = "shortcut" => "& [data-slot='item'] > [data-slot='shortcut']",
        /// A submenu item's arrow.
        Chevron = "chevron" => "& [data-slot='item'] > [data-slot='chevron']",
    }
}

base_props! {
    parts(MenuPart);
    pub struct MenuProps {
        /// From [`use_menu`](super::use_menu).
        state: MenuState,
        /// The menu, in order.
        items: Vec<MenuEntry>,
        /// The preferred side. The menu flips when that side has no room.
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
        /// The menu's corner radius: a size word or any CSS, as `radius: "0"`.
        #[props(default, into)]
        radius: Input<ThemeAwareValue>,
        /// The trigger opens nothing, and an open menu closes.
        #[props(default)]
        disabled: Option<bool>,
        /// ArrowLeft/ArrowRight no submenu answers, e.g. a menubar's next menu.
        #[props(default)]
        onedge: Option<Callback<MenuEdge>>,
        /// The trigger, carrying `state.a11y_attributes()`. `sx` and co. style the menu box.
        children: Element,
    }
}

/// A list of commands that drops from a trigger: APG's menu button.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::{Button, Menu, MenuItem, use_menu};
/// # fn app() -> Element {
/// let menu = use_menu();
/// rsx! {
///     Menu {
///         state: menu,
///         items: vec![
///             MenuItem::new("Rename").onselect(|_| {}).into(),
///             MenuItem::new("Delete").onselect(|_| {}).into(),
///         ],
///         Button { attributes: menu.a11y_attributes(), "Actions" }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/overlay/menu>
#[component]
pub fn Menu(props: MenuProps) -> Element {
    let theme = use_theme();
    let state = props.state;
    let disabled = props.disabled.unwrap_or(false);
    let id = state.id();

    let wrapper = use_element();
    let trigger = use_element();
    let floating = use_element();
    let owner = use_hook(dioxus::core::current_scope_id);
    let wrapper_box = use_box().framework_sx(&MENU_WRAPPER_SX).prepare();
    let opened = state.is_open();
    // Found, not `onmounted`: on a WebView each is a blocking round trip, one per row menu (todo 2749).
    let wrapper_selector = format!("[data-menu-wrapper={}]", css_string(&id));
    let labelledby = state.labelledby();
    let trigger_selector = id_selector(&state.trigger_element_id());
    let first_child = format!("{wrapper_selector} > *");
    use_effect(use_reactive!(|wrapper_selector| {
        wrapper.point_at(mounted_by_selector(&wrapper_selector));
    }));
    // The contents-box wrapper measures nothing, so the menu anchors on the trigger.
    // Found per open, as the caller may have swapped it; no id falls back to the first child.
    use_effect(use_reactive!(|(trigger_selector, first_child, opened)| {
        let _ = opened;
        trigger.point_at(
            mounted_by_selector(&trigger_selector).or_else(|| mounted_by_selector(&first_child)),
        );
    }));

    // No empty `role="menu"`: it reads as "menu, 0 items" (todo 447). The state
    // stays open, so items arriving later show the menu.
    let empty = props.items.is_empty();
    let open = opened && !disabled && !empty;
    use_effect(use_reactive!(|empty| state.set_empty(empty)));
    // Disabled closes the menu, so `aria-expanded` reads false; re-enabling
    // does not reopen it.
    use_effect(use_reactive!(|(disabled, opened)| {
        if disabled && opened {
            state.close();
        }
    }));

    let onclose = use_callback(move |()| state.close());
    let marker = use_press_marker();
    let wrapper_attributes: Vec<Attribute> = open
        .then(|| marker.attribute())
        .flatten()
        .into_iter()
        .collect();

    rsx! {
        {
            wrapper_box
                .attr("data-menu-wrapper", id.clone())
                // The caller's trigger bubbles here; Enter and Space arrive as a click.
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
                        Key::Named(NamedKey::ArrowDown) => {
                            event.prevent_default();
                            state.open_at(MenuFocus::First);
                        }
                        Key::Named(NamedKey::ArrowUp) => {
                            event.prevent_default();
                            state.open_at(MenuFocus::Last);
                        }
                        // Off the web no document listener hears this press;
                        // on the web closing twice is harmless.
                        Key::Named(NamedKey::Escape) if state.is_open() => {
                            event.prevent_default();
                            event.stop_propagation();
                            state.close();
                        }
                        _ => {}
                    }
                })
                .render(HtmlTag::Div, wrapper_attributes, props.children)
        }
        MenuLevel {
            // Never equal while it holds items, so a closed level gets none and
            // skips its parent's renders. Its submenu levels unmount meanwhile.
            items: if open { props.items } else { Vec::new() },
            id: menu_id(&id),
            labelledby,
            anchor: trigger,
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
            radius: ScaleOrCss::new(props.radius.as_ref(), theme.menu.radius),
            loop_focus: props.loop_focus.unwrap_or(theme.menu.loop_focus),
            close_on_select: props
                .close_on_select
                .unwrap_or(theme.menu.close_on_select),
            depth: 0,
            marker: Some(marker),
            attributes: props.attributes,
            class: props.class,
            sx: props.sx,
            parts: props.parts,
            states: props.states,
        }
    }
}

/// The neighbour ArrowLeft or ArrowRight asked for, see `MenuProps::onedge`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuEdge {
    Previous,
    Next,
}
