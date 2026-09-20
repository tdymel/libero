use std::{cell::Cell, rc::Rc};

use dioxus::prelude::*;

use super::hover_intent::{TRIGGER_WRAPPER_SX, use_hover_intent};
use crate::{
    components::{
        common::{FOCUSABLE_SELECTOR, HtmlTag, Input, States, base_props},
        layout::{paper_sx, use_box},
    },
    hooks::{
        Align, DismissOptions, ElementHandle, PopoverOptions, Side, use_dismiss, use_element,
        use_focus_within, use_popover_on, use_theme,
    },
    platform::{ElementApi, next_task},
    sx::StaticSx,
    theme::{Size, SizeCss, Z_INDEX_POPOVER},
};

// `paper_sx()` through `use_box`, not `Paper`: it needs the popover's element and events.
static HOVER_CARD_SX: StaticSx = StaticSx::new(|| {
    paper_sx()
        .z_index(Z_INDEX_POPOVER.value())
        .padding(SizeCss::SPACING.value(Size::Md))
});

/// Holds the Tab bridges: dioxus calls one listener per event name per element,
/// and the wrapper and card carry `use_dismiss`'s `onkeydown`.
const CONTENTS: &str = "display: contents";

base_props! {
    pub struct HoverCardProps {
        /// What the card shows; it may hold links and buttons.
        content: Element,
        /// The preferred side. The card flips when that side has no room.
        #[props(default)]
        side: Side,
        #[props(default)]
        align: Align,
        /// Milliseconds the pointer must rest before the card opens.
        #[props(default)]
        open_delay: Option<u32>,
        /// Milliseconds the card waits after the pointer leaves.
        #[props(default)]
        close_delay: Option<u32>,
        /// Forces the card open or closed; `None` leaves it to hover and focus.
        #[props(default)]
        open: Option<bool>,
        #[props(default, into)]
        radius: Input<Size>,
        #[props(default, into)]
        shadow: Input<Size>,
        /// Renders `children` bare: no wrapper, no card.
        #[props(default)]
        disabled: Option<bool>,
        /// The trigger, holding a focusable element. `class`/`sx`/`attributes` style the card.
        children: Element,
    }
}

/// A preview card, a named `dialog`, that opens while its trigger is hovered or focused.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::{Anchor, HoverCard};
/// # fn app() -> Element {
/// # rsx! {
/// HoverCard { aria_label: "Profile", content: rsx! { "Ada Lovelace, mathematician" },
///     Anchor { to: "/users/ada", "@ada" }
/// }
/// # } }
/// ```
///
/// Docs: <https://libero-ui.dev/overlay/hover-card>
#[component]
pub fn HoverCard(props: HoverCardProps) -> Element {
    let theme = use_theme();
    let defaults = theme.hover_card;
    let open_delay = props.open_delay.unwrap_or(defaults.open_delay);
    let close_delay = props.close_delay.unwrap_or(defaults.close_delay);

    // Hover and focus are independent reasons to stay open.
    let hovered = use_hover_intent();
    let mut focused = use_signal(|| false);
    let open = !props.disabled.unwrap_or(false) && props.open.unwrap_or(hovered.get() || focused());

    let anchor = use_element();
    let popover = use_popover_on(
        anchor,
        use_element(),
        open,
        PopoverOptions::new(theme.popover.gap, theme.popover.padding)
            .side(props.side)
            .align(props.align),
    );
    let floating = *popover.floating();
    let placed = popover.placed();

    // Escape's focus return to the trigger must not reopen the card. Cleared
    // by the next `focusin`, or focus leaving the trigger.
    let returning = use_hook(|| Rc::new(Cell::new(false)));
    let onclose = use_callback({
        let returning = returning.clone();
        move |()| {
            returning.set(*focused.peek());
            hovered.set(false);
            focused.set(false);
        }
    });
    // `outside` is ours: focus leaving must clear only `focused`. A forced-open
    // card takes no Escape, so it never blocks an enclosing `Modal`'s.
    let dismissible = props.open.is_none();
    let dismiss = use_dismiss(
        anchor,
        floating,
        open,
        placed,
        Some(onclose),
        DismissOptions {
            escape: dismissible,
            outside: false,
            ..Default::default()
        },
    );

    // Opens for keyboard focus only, as `Tooltip`: a focus after a press is the pointer's.
    let pressed = use_hook(|| Rc::new(Cell::new(false)));

    // Focus leaving is decided a task later: a `focusin` in between bumps this,
    // meaning focus only moved between trigger and card.
    let moves = use_signal(|| 0u64);
    let focus_left = move || {
        let seen = *moves.peek();
        spawn(async move {
            next_task().await;
            if *moves.peek() == seen {
                focused.set(false);
            }
        });
    };
    // Element 0 is the trigger, 1 the card.
    let focus = use_focus_within(move || vec![anchor.mounted(), floating.mounted()], {
        let (pressed, returning) = (pressed.clone(), returning.clone());
        move |change| {
            let (mut focused, mut moves) = (focused, moves);
            match (change.element, change.within) {
                (0, true) => {
                    moves += 1;
                    // The web answers itself (todo 477: no stale press).
                    let pointer = pressed.replace(false);
                    let keyboard = change.focus_visible().unwrap_or(!pointer);
                    if !returning.replace(false) && keyboard {
                        // What Escape hands focus back to.
                        dismiss.focus_return().remember_active();
                        focused.set(true);
                    }
                }
                (_, true) => {
                    moves += 1;
                    focused.set(true);
                }
                (element, false) => {
                    if element == 0 || change.in_group == Some(false) {
                        returning.set(false);
                    }
                    match change.in_group {
                        Some(true) => {}
                        Some(false) => focused.set(false),
                        None => focus_left(),
                    }
                }
            }
        }
    });

    crate::components::common::use_name_warning(
        crate::components::common::names_itself(&props.attributes),
        "HoverCard: the card is a dialog and needs a name - pass `aria_label`, or \
         `aria-labelledby` pointing into `content`.",
    );
    // Keyboard opens the card through the trigger's focus; plain text never takes it.
    #[cfg(debug_assertions)]
    use_effect(move || {
        if let Err(crate::platform::PlatformError::NotFound) =
            anchor.query_selector(FOCUSABLE_SELECTOR)
        {
            crate::utils::warn(
                "HoverCard: `children` holds nothing focusable, so no keyboard can open the \
                 card - wrap the trigger in a link or a button.",
            );
        }
    });

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(
            props.radius.copied_or(defaults.radius).radius_state_name(),
            true,
        )
        .with(
            props.shadow.copied_or(defaults.shadow).shadow_state_name(),
            true,
        )
        .into();

    // Every hook above the branch - `prepare()` is the hook.
    let wrapper = use_box().framework_sx(&TRIGGER_WRAPPER_SX).prepare();
    let card = use_box()
        .framework_sx(&HOVER_CARD_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .style(popover.style())
        .prepare();

    if props.disabled.unwrap_or(false) {
        // The slot outlives this branch: a card open when disabled would stay up.
        popover.show(None);
        return props.children;
    }

    popover.show(open.then(|| {
        let mut attributes = props.attributes.clone();
        attributes.extend(dismiss.floating_events());
        card.element(&floating)
            .attr("role", "dialog")
            .event("onmouseenter", move |_: MouseEvent| hovered.hover(true, 0))
            .event("onmouseleave", move |_: MouseEvent| {
                hovered.hover(false, close_delay)
            })
            .render(
                HtmlTag::Div,
                attributes,
                rsx! {
                    div {
                        style: CONTENTS,
                        onfocusin: focus.focusin(1),
                        onfocusout: focus.focusout(1),
                        onkeydown: move |event| card_tab(&event, anchor, floating),
                        {props.content}
                    }
                },
            )
    }));

    wrapper
        .element(&anchor)
        .event("onmouseenter", move |_: MouseEvent| {
            hovered.hover(true, open_delay)
        })
        .event("onmouseleave", {
            // A press on a trigger that held focus already fired no `focusin`
            // to clear it, and would swallow the next keyboard focus.
            let pressed = pressed.clone();
            move |_: MouseEvent| {
                pressed.set(false);
                hovered.hover(false, close_delay)
            }
        })
        .render(
            HtmlTag::Span,
            dismiss.anchor_events(),
            rsx! {
                span {
                    style: CONTENTS,
                    onpointerdown: {
                        let pressed = pressed.clone();
                        move |_| pressed.set(true)
                    },
                    onfocusin: focus.focusin(0),
                    onfocusout: focus.focusout(0),
                    onkeydown: move |event| trigger_tab(&event, open && placed, floating),
                    {props.children}
                }
            },
        )
}

/// Tab on the trigger enters the card, which is portaled out of the Tab order.
fn trigger_tab(event: &KeyboardEvent, open: bool, floating: ElementHandle) {
    if !open || event.key() != Key::Tab || event.modifiers().shift() {
        return;
    }
    let first = floating
        .query_selector_all(FOCUSABLE_SELECTOR)
        .ok()
        .and_then(|items| items.into_iter().next());
    if let Some(first) = first {
        event.prevent_default();
        // An enclosing `Modal`'s `FocusTrap` would take focus back out.
        event.stop_propagation();
        let _ = first.focus();
    }
}

/// Tab past either end goes back via the trigger, so the browser's Tab moves on
/// from there, not from the portal outlet. `Menu`'s `tab_out`.
fn card_tab(event: &KeyboardEvent, anchor: ElementHandle, floating: ElementHandle) {
    if event.key() != Key::Tab {
        return;
    }
    let Ok(items) = floating.query_selector_all(FOCUSABLE_SELECTOR) else {
        return;
    };
    let backwards = event.modifiers().shift();
    let edge = match backwards {
        true => items.first(),
        false => items.last(),
    };
    if !edge.is_some_and(|item| item.is_focused()) {
        return;
    }
    let Ok(trigger) = anchor.query_selector(FOCUSABLE_SELECTOR) else {
        return;
    };
    if backwards {
        event.prevent_default();
    }
    let _ = trigger.focus();
}
