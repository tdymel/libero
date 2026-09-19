use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            HtmlTag, Input, States, base_props, disabled_look_sx, has_shortcut_modifier,
            inset_focus_ring_sx,
        },
        layout::use_box,
    },
    hooks::{
        Align, ElementHandle, Side, TYPEAHEAD_RESET, typeahead_match, use_element, use_id,
        use_theme, use_typeahead,
    },
    platform::{ElementApi, logical_key},
    sx::{StaticSx, sx},
    theme::{
        MENUBAR_GAP, MENUBAR_TRIGGER_FONT, MENUBAR_TRIGGER_PAD_X, MENUBAR_TRIGGER_PAD_Y,
        MENUBAR_TRIGGER_RADIUS, MenubarDefaults, Size,
    },
};

use crate::components::overlay::{Menu, MenuEdge, MenuEntry, MenuFocus, MenuState};

const TRIGGER: &str = "& [data-menubar-index]";

// The triggers carry no class of their own: styled from the bar, so a row of
// them costs one class.
static MENUBAR_SX: StaticSx = StaticSx::new(|| {
    MenubarDefaults::theme_vars()
        .display("flex")
        .flex_direction("row")
        .align_items("center")
        // A crowded bar wraps, and a trigger longer than the bar wraps its
        // label, instead of widening the page (1.4.10).
        .flex_wrap("wrap")
        .gap(MENUBAR_GAP.value())
        // Each `Menu` wrapper is the flex item. `anywhere` already floors it at a glyph, and
        // no min-content measure spares Blitz a label broken per glyph (886).
        .selector("& > *", sx().min_width("0"))
        .selector(
            TRIGGER,
            sx().display("inline-flex")
                .align_items("center")
                .padding(format!(
                    "{} {}",
                    MENUBAR_TRIGGER_PAD_Y.value(),
                    MENUBAR_TRIGGER_PAD_X.value()
                ))
                .appearance("none")
                .border("0")
                .border_radius(MENUBAR_TRIGGER_RADIUS.value())
                .background("transparent")
                .font("inherit")
                .font_size(MENUBAR_TRIGGER_FONT.value())
                .color("inherit")
                .flex_shrink("0")
                .max_width("100%")
                .with("overflow-wrap", "anywhere")
                .cursor("pointer")
                .user_select("none"),
        )
        // Hover, focus and an open menu share one tint - the open menu's
        // trigger keeps it after the pointer has gone. `Menu`'s items draw the
        // same.
        .selector(
            "& [data-menubar-index]:hover:not([aria-disabled=\"true\"])",
            sx().background("muted.1"),
        )
        .selector("& [data-menubar-index]:focus", sx().background("muted.1"))
        .selector(
            "& [data-menubar-index][aria-expanded=\"true\"]",
            sx().background("muted.1"),
        )
        // `appearance: none` and `border: 0` take the UA's ring with them.
        .selector(
            "& [data-menubar-index]:focus-visible",
            inset_focus_ring_sx("-2px"),
        )
        .selector(
            "& [data-menubar-index][aria-disabled=\"true\"]",
            disabled_look_sx("not-allowed"),
        )
});

/// One top-level menu of a [`Menubar`].
#[derive(Clone, PartialEq)]
pub struct MenubarMenu {
    /// The trigger's text, and what typeahead on the bar matches.
    pub label: String,
    /// Exactly [`Menu`]'s item model.
    pub items: Vec<MenuEntry>,
    /// The trigger stays in view and in the arrow order, and opens nothing.
    pub disabled: bool,
}

impl MenubarMenu {
    pub fn new(label: impl Into<String>, items: Vec<MenuEntry>) -> Self {
        Self {
            label: label.into(),
            items,
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

base_props! {
    pub struct MenubarProps {
        /// The top-level menus, in order.
        menus: Vec<MenubarMenu>,
        /// `role="menubar"` needs a name.
        #[props(into)]
        aria_label: String,
        /// Whether the arrow keys wrap at the ends - along the bar, and down
        /// each menu.
        #[props(default)]
        loop_focus: Option<bool>,
        /// Which side of its trigger every menu opens on.
        #[props(default)]
        side: Side,
        #[props(default)]
        align: Align,
        /// The triggers' font and padding, and each menu's item size.
        #[props(default, into)]
        size: Input<Size>,
        /// The triggers' and the menus' corner radius.
        #[props(default, into)]
        radius: Input<Size>,
    }
}

/// Everything the triggers' handlers share. The menu states outlive the list:
/// one per menu the bar has ever had, so a menu that comes back gets its own
/// state again, and a switch closes every one of them.
#[derive(Clone)]
struct Row {
    bar: ElementHandle,
    states: Rc<[MenuState]>,
    disabled: Rc<[bool]>,
    loop_focus: bool,
    /// The trigger focused last - the tab stop while no menu is open.
    current: Signal<Option<usize>>,
    /// The disabled trigger an open menu was left for: the bar stays in open
    /// mode, so the next enabled trigger opens its menu (APG, todo 568).
    armed: Rc<Cell<Option<usize>>>,
}

impl Row {
    fn enabled(&self, index: usize) -> bool {
        self.disabled.get(index).is_some_and(|disabled| !disabled)
    }

    /// The next trigger from `from`, disabled ones included, or `None` at an
    /// end that does not wrap.
    fn step(&self, from: usize, forward: bool) -> Option<usize> {
        step(self.disabled.len(), from, forward, self.loop_focus)
    }

    fn open_index(&self) -> Option<usize> {
        (0..self.disabled.len()).find(|&index| self.states[index].is_open())
    }

    fn focus(&self, index: usize) {
        let mut current = self.current;
        current.set(Some(index));
        let _ = self
            .bar
            .query_selector(&format!("[data-menubar-index=\"{index}\"]"))
            .and_then(|trigger| trigger.focus());
    }

    fn close_others(&self, keep: usize) {
        for (index, state) in self.states.iter().enumerate() {
            if index != keep {
                state.close();
            }
        }
    }

    /// Moves the open menu to `index`. Its trigger takes focus first, still
    /// inside this event, so the menu remembers the trigger as the place to
    /// hand focus back to - not an item of the menu that is closing.
    fn switch(&self, index: usize, focus: MenuFocus) {
        self.focus(index);
        self.close_others(index);
        self.states[index].open_at(focus);
    }

    /// Focus, or the open menu, moves to `index`. A disabled trigger takes
    /// focus and opens nothing, but keeps the bar in open mode for the next.
    fn go(&self, index: usize) {
        let open = self.open_index();
        let open_mode = open.is_some() || self.armed.get().is_some();
        let enabled = self.enabled(index);
        // Set before the focus moves, so the old trigger's blur sees the new value.
        self.armed.set((open_mode && !enabled).then_some(index));
        match (open_mode, open) {
            (true, open) if open != Some(index) && enabled => self.switch(index, MenuFocus::First),
            (true, _) => {
                self.focus(index);
                self.close_others(index);
            }
            (false, _) => self.focus(index),
        }
    }
}

/// A row of menus - APG's menubar, the desktop application's File / Edit /
/// View.
///
/// The bar owns everything: which menu is open (at most one), and which
/// trigger is the bar's single tab stop. Left and Right move along the bar, in
/// an open menu too, and while one menu is open the pointer resting on another
/// trigger switches to it. Each menu is a [`Menu`], so everything inside one is
/// `Menu`'s.
///
/// `sx`, `class`, `states` and `attributes` land on the bar.
#[component]
pub fn Menubar(props: MenubarProps) -> Element {
    let theme = use_theme();
    let id = use_id();
    let bar = use_element();
    let current = use_signal(|| None::<usize>);
    let typeahead = use_typeahead(TYPEAHEAD_RESET);
    // A tap sends a compatibility `mouseenter` before its click: switching
    // there made the click close the menu it had just opened.
    let touch: Rc<Cell<bool>> = use_hook(|| Rc::new(Cell::new(false)));
    let armed: Rc<Cell<Option<usize>>> = use_hook(|| Rc::new(Cell::new(None)));

    // `use_menu()` in a loop over `menus` would hand hook slots from one menu
    // to another whenever the list changes length. The states are created
    // here instead, on first need, and kept - `MenuLevel`'s anchor pool.
    let pool: Rc<RefCell<Vec<MenuState>>> = use_hook(|| Rc::new(RefCell::new(Vec::new())));
    let len = props.menus.len();
    let states: Rc<[MenuState]> = {
        let mut pool = pool.borrow_mut();
        while pool.len() < len {
            let index = pool.len();
            pool.push(MenuState::new(format!("{}-{index}", id())));
        }
        // A menu dropped while open would otherwise mount open when it comes back.
        for state in &pool[len..] {
            state.close();
        }
        pool.iter().copied().collect()
    };

    let loop_focus = props.loop_focus.unwrap_or(theme.menubar.loop_focus);
    let row = Row {
        bar,
        states,
        disabled: props.menus.iter().map(|menu| menu.disabled).collect(),
        loop_focus,
        current,
        armed,
    };

    // The single tab stop: the open menu's trigger, else the one focused
    // last, else the first.
    let tabbable = row
        .open_index()
        .or(current())
        .filter(|&index| index < len)
        .unwrap_or(0);

    let labels: Rc<Vec<Option<String>>> = Rc::new(
        props
            .menus
            .iter()
            .map(|menu| (!menu.disabled).then(|| menu.label.clone()))
            .collect(),
    );

    let size = props.size.copied_or(theme.menubar.size);
    let radius = props.radius.copied_or(theme.menubar.radius);

    let columns = props.menus.iter().enumerate().map(|(index, menu)| {
        let state = row.states[index];
        let disabled = menu.disabled;

        let onedge = {
            let row = row.clone();
            Callback::new(move |edge: MenuEdge| {
                if let Some(next) = row.step(index, edge == MenuEdge::Next) {
                    row.go(next);
                }
            })
        };

        let onkeydown = {
            let row = row.clone();
            let typeahead = typeahead.clone();
            let labels = labels.clone();
            move |event: KeyboardEvent| {
                // Ctrl/Alt/Meta chords are the browser's, typeahead included.
                if has_shortcut_modifier(&event) {
                    return;
                }
                let target = match logical_key(&event) {
                    Key::ArrowRight => row.step(index, true),
                    Key::ArrowLeft => row.step(index, false),
                    Key::Home => (len > 0).then_some(0),
                    Key::End => len.checked_sub(1),
                    Key::Escape => {
                        row.armed.set(None);
                        return;
                    }
                    Key::Character(ref text) if !has_shortcut_modifier(&event) => {
                        let Some(ch) = text.chars().next() else {
                            return;
                        };
                        // A space is the trigger's own click unless a query is
                        // being typed.
                        if ch == ' ' && !typeahead.is_typing() {
                            return;
                        }
                        let query = typeahead.push(ch);
                        typeahead_match(labels.len(), Some(index), &query, |row| {
                            labels[row].as_deref()
                        })
                    }
                    // ArrowDown, ArrowUp and Escape are `Menu`'s.
                    _ => return,
                };
                event.prevent_default();
                if let Some(target) = target {
                    row.go(target);
                }
            }
        };

        let onclick = {
            let row = row.clone();
            // `Menu` toggles this one on the click's way up; the others close
            // here.
            move |_: MouseEvent| {
                if !disabled {
                    row.close_others(index);
                }
            }
        };

        let onpointerenter = {
            let touch = touch.clone();
            move |event: PointerEvent| touch.set(event.data().pointer_type() == "touch")
        };

        let onmouseenter = {
            let row = row.clone();
            let touch = touch.clone();
            move |_: MouseEvent| {
                if disabled || touch.get() {
                    return;
                }
                if row.open_index().is_some_and(|open| open != index) {
                    row.switch(index, MenuFocus::First);
                }
            }
        };

        let onfocus = {
            let mut current = current;
            move |_: FocusEvent| {
                if *current.peek() != Some(index) {
                    current.set(Some(index));
                }
            }
        };

        // Focus leaving the armed trigger any other way ends open mode.
        let onblur = {
            let armed = row.armed.clone();
            move |_: FocusEvent| {
                if armed.get() == Some(index) {
                    armed.set(None);
                }
            }
        };

        let attributes = state.a11y_attributes();
        let label = menu.label.clone();

        rsx! {
            Menu {
                key: "{index}",
                state,
                items: menu.items.clone(),
                side: props.side,
                align: props.align,
                size,
                radius,
                loop_focus,
                disabled,
                onedge,
                button {
                    r#type: "button",
                    "role": "menuitem",
                    tabindex: if index == tabbable { "0" } else { "-1" },
                    "data-menubar-index": "{index}",
                    "aria-disabled": disabled.then_some("true"),
                    onkeydown,
                    onclick,
                    onpointerenter,
                    onmouseenter,
                    onfocus,
                    onblur,
                    ..attributes,
                    "{label}"
                }
            }
        }
    });
    let columns: Vec<Element> = columns.collect();

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with(radius.radius_state_name(), true)
        .into();

    let root = use_box()
        .framework_sx(&MENUBAR_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .prepare();

    root.element(&bar)
        .attr("role", "menubar")
        .attr("aria-orientation", "horizontal")
        .attr("aria-label", props.aria_label.clone())
        .render(HtmlTag::Div, props.attributes, columns)
}

/// The trigger after (or before) `from` in a row of `len`, or `None` at an end
/// that does not wrap and for a lone trigger. Disabled triggers count: they
/// take focus like any other.
fn step(len: usize, from: usize, forward: bool, loop_focus: bool) -> Option<usize> {
    let next = match (forward, from) {
        (true, from) if from + 1 < len => from + 1,
        (true, _) if loop_focus => 0,
        (false, 0) if loop_focus => len.checked_sub(1)?,
        (false, from) if from > 0 => from - 1,
        _ => return None,
    };
    (next != from).then_some(next)
}

#[cfg(test)]
mod tests {
    use super::step;

    #[test]
    fn the_arrows_walk_every_trigger_and_wrap_only_when_asked() {
        assert_eq!(step(3, 0, true, false), Some(1));
        assert_eq!(step(3, 2, true, false), None);
        assert_eq!(step(3, 2, true, true), Some(0));
        assert_eq!(step(3, 0, false, false), None);
        assert_eq!(step(3, 0, false, true), Some(2));
        assert_eq!(step(3, 1, false, true), Some(0));
    }

    #[test]
    fn a_lone_trigger_goes_nowhere() {
        assert_eq!(step(1, 0, true, true), None);
        assert_eq!(step(1, 0, false, true), None);
        assert_eq!(step(1, 0, true, false), None);
    }
}
