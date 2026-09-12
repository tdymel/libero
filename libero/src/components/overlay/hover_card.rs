use std::{cell::Cell, rc::Rc};

use dioxus::prelude::*;

use super::hover_intent::{TRIGGER_WRAPPER_SX, use_hover_intent};
use crate::{
    components::{
        FOCUSABLE_SELECTOR, HtmlTag, Input, States, common::base_props, layout::use_box,
        surface::paper_sx,
    },
    hooks::{
        Align, DismissOptions, ElementHandle, PopoverOptions, Side, use_dismiss, use_element,
        use_popover, use_silent_focus, use_theme,
    },
    platform::{ElementApi, focus_visible, next_task},
    sx::StaticSx,
    theme::{Size, SizeCss, Z_INDEX_POPOVER},
};

// A surface, so background, corner and elevation are `paper_sx()`'s and the
// `radius-{step}`/`shadow-{step}` tokens the card renders with are answered by
// its folds. Rendered through `use_box` rather than `Paper`, because it needs
// the popover's element handle and its own events - `Menu`'s shape.
static HOVER_CARD_SX: StaticSx = StaticSx::new(|| {
    paper_sx()
        // Everything positional comes from `use_popover` as an inline style.
        .z_index(Z_INDEX_POPOVER.value())
        .padding(SizeCss::SPACING.value(Size::Md))
});

/// The Tab bridges sit on these, not on the wrapper or the card: those carry
/// `use_dismiss`'s `onkeydown`, and dioxus calls only one listener per event
/// name on the element an event targets.
const CONTENTS: &str = "display: contents";

base_props! {
    pub struct HoverCardProps {
        /// What the card shows. It may hold links and buttons: the card is a
        /// non-modal `dialog`, not a tooltip.
        content: Element,
        /// Which side of the trigger the card opens on. It flips when that
        /// side has no room.
        #[props(default)]
        side: Side,
        #[props(default)]
        align: Align,
        /// Milliseconds the pointer must rest on the trigger before the card
        /// opens.
        #[props(default)]
        open_delay: Option<u32>,
        /// Milliseconds the card waits after the pointer leaves - also the time
        /// the pointer has to cross into the card.
        #[props(default)]
        close_delay: Option<u32>,
        /// Forces the card open or closed; `None` leaves it to hover and
        /// focus. A card forced open cannot be dismissed.
        #[props(default)]
        open: Option<bool>,
        #[props(default, into)]
        radius: Input<Size>,
        #[props(default, into)]
        shadow: Input<Size>,
        /// Renders `children` bare - no wrapper, no card.
        #[props(default)]
        disabled: Option<bool>,
        /// The trigger. `class`/`sx`/`states`/`attributes` land on the *card*.
        children: Element,
    }
}

/// A card that opens while its trigger is hovered or focused, and stays open
/// while the pointer or focus is inside it - WCAG 1.4.13's hoverable,
/// persistent and, through Escape, dismissible.
///
/// The card is a `role="dialog"`, so give it a name: an `aria_label`, or an
/// `aria-labelledby` pointing into `content`.
///
/// Keyboard: focusing the trigger opens the card, Tab moves from the trigger
/// into it and from its last focusable element on to whatever follows the
/// trigger, Shift+Tab walks back, and Escape closes it.
#[component]
pub fn HoverCard(props: HoverCardProps) -> Element {
    let theme = use_theme();
    let defaults = theme.hover_card;
    let open_delay = props.open_delay.unwrap_or(defaults.open_delay);
    let close_delay = props.close_delay.unwrap_or(defaults.close_delay);

    // Two independent reasons to be open. A pointer that leaves does not
    // close a card the keyboard is still in, and focus leaving does not close
    // one the pointer is still over.
    let hovered = use_hover_intent();
    let mut focused = use_signal(|| false);
    let open = !props.disabled.unwrap_or(false) && props.open.unwrap_or(hovered.get() || focused());

    let anchor = use_element();
    let popover = use_popover(
        anchor,
        open,
        PopoverOptions::new(theme.popover.gap, theme.popover.padding)
            .side(props.side)
            .align(props.align),
    );
    let floating = *popover.floating();
    let placed = popover.placed();

    // Escape hands focus back to the trigger when it was inside the card, and
    // that `focusin` must not reopen what the user just closed. Cleared by
    // the next one, or by focus leaving the trigger - which is how it goes
    // when focus was on the trigger already and no `focusin` follows.
    let returning = use_hook(|| Rc::new(Cell::new(false)));
    let onclose = use_callback({
        let returning = returning.clone();
        move |()| {
            returning.set(*focused.peek());
            hovered.set(false);
            focused.set(false);
        }
    });
    // `outside` is ours: `use_dismiss` has one `onclose` for every reason, and
    // focus leaving must clear only `focused`. A card forced open takes no
    // Escape, so it never sits on the Escape stack above a `Modal` it cannot
    // leave.
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

    // A click focuses the trigger too, and a card that opened for that focus
    // would stay up after the pointer has gone - `Tooltip`'s `:focus-visible`
    // rule. The web asks it (`focus_visible`); elsewhere a focus that follows
    // a press is the pointer's, not the keyboard's.
    let pressed = use_hook(|| Rc::new(Cell::new(false)));

    // Focus leaving is decided a task later, when it has landed: any `focusin`
    // on the trigger or the card in between bumps this, and a bumped count
    // means focus only moved between them. Nothing is asked of the platform,
    // so it holds on every backend that orders the task after the `focusin`.
    let mut moves = use_signal(|| 0u64);
    let focus_left = move |_: FocusEvent| {
        let seen = *moves.peek();
        spawn(async move {
            next_task().await;
            if *moves.peek() == seen {
                focused.set(false);
            }
        });
    };
    // Blitz's Tab and libero's own `focus()` fire neither event: the silent
    // move stands in for both, with the trigger and the card as one.
    use_silent_focus({
        let (pressed, returning) = (pressed.clone(), returning.clone());
        move |moved| {
            let holds = |element: &ElementHandle, now: bool| {
                element.mounted().is_some_and(|mounted| match now {
                    true => moved.is_in(&mounted),
                    false => moved.was_in(&mounted),
                })
            };
            let within = |now| holds(&anchor, now) || holds(&floating, now);
            let mut focused = focused;
            match (within(false), within(true)) {
                (false, true) => {
                    if !returning.replace(false) && !pressed.replace(false) {
                        dismiss.focus_return().remember_active();
                        focused.set(true);
                    }
                }
                (true, false) => {
                    returning.set(false);
                    focused.set(false);
                }
                _ => {}
            }
        }
    });

    // A dialog with no name is announced as "dialog" and nothing else.
    crate::components::common::use_name_warning(
        crate::components::common::names_itself(&props.attributes),
        "HoverCard: the card is a dialog and needs a name - pass `aria_label`, or \
         `aria-labelledby` pointing into `content`.",
    );

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
                        onfocusin: move |_| {
                            moves += 1;
                            focused.set(true);
                        },
                        onfocusout: focus_left,
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
                    onfocusin: {
                        let returning = returning.clone();
                        move |event: FocusEvent| {
                            moves += 1;
                            // The web answers itself (todo 477: no stale press).
                            let pointer = pressed.replace(false);
                            let keyboard = focus_visible(&event).unwrap_or(!pointer);
                            if !returning.replace(false) && keyboard {
                                // The trigger holds focus now, so this is what
                                // Escape hands it back to.
                                dismiss.focus_return().remember_active();
                                focused.set(true);
                            }
                        }
                    },
                    onfocusout: move |event| {
                        returning.set(false);
                        focus_left(event);
                    },
                    onkeydown: move |event| trigger_tab(&event, open && placed, floating),
                    {props.children}
                }
            },
        )
}

/// Tab on the trigger enters the card. It is portaled to the end of the
/// document, so the browser's own Tab would skip it.
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
        // An enclosing `Modal`'s `FocusTrap` answers every Tab itself, and
        // would take focus straight back out of the portaled card.
        event.stop_propagation();
        let _ = first.focus();
    }
}

/// Tab past either end of the card goes back through the trigger. Forwards,
/// focus lands on the trigger inside this keydown and the browser's own Tab
/// moves on from there - to whatever follows the trigger, not whatever follows
/// the portal outlet. `Menu`'s `tab_out`.
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
