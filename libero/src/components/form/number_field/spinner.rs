//! `TouchInput::Spin`: under a finger a tap opens a spin column instead of the keyboard,
//! in the phone sheet below `sm` and a popover from `sm` up (todo 4133).

use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::{Decimals, Separators, aria_number, written};
use crate::{
    components::{
        buttons::{ActionIcon, Button},
        common::{COARSE_POINTER, Glyph, HtmlTag, Input, NumberValue, States, focus_ring_sx},
        form::{
            keyboardless::use_picker_focus,
            phone_sheet::{
                SheetSwipe, narrow_query, phone_sheet_sx, sheet_clearance_style,
                use_sheet_clearance, use_sheet_room, use_sheet_swipe,
            },
            slot_icon_size,
            spin::{Turn, key_steps, use_spin_gestures},
        },
        layout::{paper_sx, use_box},
    },
    context::IconSlot,
    hooks::{
        POPOVER_AVAILABLE_HEIGHT, PopoverOptions, current_localization, placement_sx, use_element,
        use_field_list_layer, use_focus_within, use_formats, use_media_query, use_popover_on,
        use_theme,
    },
    platform::{is_tab, next_task, soft_keyboard_app},
    sx::{FORCED_COLORS, StaticSx, ThemeAwareValue, sx},
    theme::{FIELD_FONT_SIZE, Size, SizeCss, Z_INDEX_POPOVER},
};

/// The steps Page Up and Page Down take.
const PAGE: usize = 10;

static SPINNER_SX: StaticSx = StaticSx::new(|| {
    paper_sx()
        .and(placement_sx(true))
        .z_index(Z_INDEX_POPOVER.value())
        .padding(SizeCss::SPACING.value(Size::Sm))
        .border_radius(SizeCss::RADIUS.value(Size::Sm))
        .box_shadow(SizeCss::SHADOW.value(Size::Lg))
        // Never past the room on its side: it scrolls instead (WCAG 1.4.10).
        .max_height(POPOVER_AVAILABLE_HEIGHT.value_or("none"))
        .overflow_y("auto")
        .display("flex")
        .flex_direction("column")
        .align_items("stretch")
        .gap(SizeCss::SPACING.value(Size::Sm))
        .selector(
            "& > [data-slot='spinner']",
            sx().display("flex")
                .align_items("center")
                .justify_content("center")
                .gap("8px")
                .padding("4px")
                .with("font-variant-numeric", "tabular-nums")
                .white_space("nowrap"),
        )
        .selector(
            "& [data-slot='spin']",
            sx().display("flex")
                .flex_direction("column")
                .align_items("stretch")
                .min_width("6ch")
                .padding("2px 6px")
                .border_radius(SizeCss::RADIUS.value(Size::Sm))
                .line_height("1.25")
                .text_align("center")
                .cursor("ns-resize")
                // A touch drags the column rather than scrolling the page.
                .touch_action("none")
                .user_select("none")
                .focus_visible(focus_ring_sx()),
        )
        // The value's band; its border stays in forced colours, where the fill goes.
        .selector(
            "& [data-slot='value']",
            sx().padding("4px 12px")
                .font_size("1.75em")
                .font_weight("500")
                .border("1px solid transparent")
                .border_radius(SizeCss::RADIUS.value(Size::Sm))
                .background("muted.1")
                .media(FORCED_COLORS, sx().border_color("CanvasText")),
        )
        .selector(
            "& [data-slot='spin']:focus-visible > [data-slot='value']",
            sx().color("primary.6"),
        )
        // The press is the single-pointer alternative to the drag (WCAG 2.5.7, 2.5.8).
        .selector(
            "& [data-slot='neighbour']",
            sx().display("flex")
                .align_items("center")
                .justify_content("center")
                .min_height("max(1.25em, 24px)")
                .color("text-dimmed")
                .hover(sx().cursor("pointer"))
                .media(COARSE_POINTER, sx().min_height("44px")),
        )
        .selector("& [data-slot='unit']", sx().color("text-dimmed"))
        .selector(
            "& > [data-slot='actions']",
            sx().display("flex")
                .align_items("center")
                .justify_content("space-between")
                .gap(SizeCss::SPACING.value(Size::Sm)),
        )
        .and(phone_sheet_sx())
});

/// A spinner field's state, shared by the field's input and its spinner.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Spin {
    /// Under a finger, a coarse pointer or a phone app: a tap spins rather than types.
    pub(super) coarse: bool,
    pub(super) open: bool,
    /// The keyboard button switched to typing, until focus leaves the field.
    pub(super) typing: bool,
}

impl Spin {
    /// Whether the input keeps the soft keyboard down and a tap opens the spinner.
    pub(super) fn keyboardless(self) -> bool {
        self.coarse && !self.typing
    }
}

/// How the field writes its number, so the column writes it the same.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Look {
    pub(super) decimals: Option<Decimals>,
    pub(super) grouped: bool,
    pub(super) separator: Separators,
}

/// `steps` steps from `from`, each kept inside the range. An empty field lands on `min`,
/// else zero, with its first step either way, as a clock's empty column does.
pub(super) fn spun<T: NumberValue>(
    from: Option<T>,
    steps: isize,
    step: T,
    min: Option<T>,
    max: Option<T>,
) -> Option<T> {
    let (mut next, left) = match from {
        Some(value) => (value, steps.unsigned_abs()),
        None if steps == 0 => return None,
        None => (
            min.or_else(T::zero)?.clamp_between(min, max),
            steps.unsigned_abs() - 1,
        ),
    };
    for _ in 0..left {
        next = match steps > 0 {
            true => next.step_up(step),
            false => next.step_down(step),
        }
        .clamp_between(min, max);
    }
    Some(next)
}

/// The field's control and the spinner its tap opens. Its own scope, so a step redraws
/// the column and not the field.
#[component]
pub(super) fn NumberSpinner<T: NumberValue>(
    control: Element,
    spin: Signal<Spin>,
    live: Signal<Option<T>>,
    bound: Option<Option<T>>,
    field_id: String,
    labelled: bool,
    field_root: Option<crate::hooks::ElementHandle>,
    look: Look,
    step: T,
    min: Option<T>,
    max: Option<T>,
    size: Size,
    unit: Option<Signal<Option<Element>>>,
    unit_id: Option<String>,
    spin_to: Callback<T>,
) -> Element {
    let theme = use_theme();
    let formats = use_formats();
    let labels = current_localization().number_field;
    let coarse = use_media_query(COARSE_POINTER);
    use_effect(move || {
        // A phone app's WebView answers no media query, and is under a finger anyway.
        let coarse = soft_keyboard_app() || coarse();
        if spin.peek().coarse != coarse {
            spin.write().coarse = coarse;
        }
    });
    let close = move |typing: bool| {
        let mut spin = spin;
        let now = *spin.peek();
        if now.open || now.typing != typing {
            spin.set(Spin {
                open: false,
                typing,
                ..now
            });
        }
    };
    let showing = spin().open;
    let current = bound.unwrap_or_else(|| live.cloned());

    let anchor = use_element();
    let popover = use_popover_on(
        anchor,
        use_element(),
        showing,
        PopoverOptions::new(theme.popover.gap, theme.popover.padding),
    );
    let floating = *popover.floating();
    let narrow = use_media_query(&narrow_query());
    let room = use_sheet_room(floating, narrow() && popover.placed());
    let clearance = use_sheet_clearance(anchor, field_root, popover, showing, theme.popover.gap);
    let SheetSwipe {
        strip,
        listeners,
        style: drag,
    } = use_sheet_swipe(floating, use_callback(move |()| close(false)));
    let dropdown_states: Input<States> = States::new().active("bordered").into();
    let dropdown = use_box()
        .framework_sx(&SPINNER_SX)
        .states(&dropdown_states)
        .style(Some(format!(
            "{}{room}{drag}font-size:{};",
            popover.vars(),
            FIELD_FONT_SIZE.value(size)
        )))
        .prepare();
    let picker_focus = use_picker_focus(anchor, floating, &field_id);
    let dialog_id = format!("{field_id}-spinner");
    let column_id = format!("{field_id}-spin");

    // Opening moves focus onto the column, once it is placed and drawn.
    let mut entered = use_hook(|| CopyValue::new(false));
    let entry_id = dialog_id.clone();
    use_effect(move || {
        let open = spin().open;
        if !open {
            entered.set(false);
        } else if popover.placed() && !*entered.peek() {
            entered.set(true);
            picker_focus.enter(&entry_id, "[role='spinbutton']");
        }
    });
    let focus_input = move || picker_focus.focus_input();
    let done = move || {
        if picker_focus.in_dropdown() {
            focus_input();
        }
        close(false);
    };
    // Escape and Android's Back close it, as a tap outside does.
    use_field_list_layer(showing, use_callback(move |()| done()));
    let settle = move || {
        spawn(async move {
            if !picker_focus.settled_inside().await {
                close(false);
            }
        });
    };
    let focus = use_focus_within(
        move || vec![anchor.mounted(), floating.mounted()],
        move |change| {
            picker_focus.track(change.element, change.within);
            match change.in_group {
                Some(false) => close(false),
                None => settle(),
                Some(true) => {}
            }
        },
    );
    // Lifts `inputmode="none"` first, then focuses: a phone browser raises the keyboard for
    // a focus inside the tap's activation; a WebView only on the input's next tap (3902).
    let keyboard = move |_: MouseEvent| {
        close(true);
        spawn(async move {
            next_task().await;
            focus_input();
        });
    };

    let text = |value: Option<T>| {
        value.map(|value| written(value, look.decimals, look.grouped, look.separator, formats))
    };
    let beside = |delta: isize| {
        let next = match current {
            // Empty, only the value the first step lands on shows, below.
            None if delta < 0 => None,
            from => spun(from, delta, step, min, max),
        };
        text(next.filter(|next| Some(*next) != current))
    };
    let (before, after) = (beside(-1), beside(1));
    let shown = text(current);
    let value_text = shown.clone();
    let shown = shown.unwrap_or_else(|| "–".to_string());

    let onturn = use_callback(move |turn: Turn<Option<T>>| {
        let next = match turn {
            Turn::By(delta) => spun(current, delta, step, min, max),
            Turn::From(from, steps) => spun(from, steps, step, min, max),
        };
        if let Some(next) = next.filter(|next| Some(*next) != current) {
            spin_to.call(next);
        }
    });
    let column = use_element();
    let gestures = use_spin_gestures(column, current, false, PAGE, onturn);
    let onkeydown = move |event: KeyboardEvent| {
        let key = event.key();
        let next = match key {
            _ if key_steps(&key, PAGE).is_some() => {
                key_steps(&key, PAGE).and_then(|steps| spun(current, steps, step, min, max))
            }
            Key::Named(NamedKey::Home) if min.is_some() => min,
            Key::Named(NamedKey::End) if max.is_some() => max,
            Key::Named(NamedKey::Enter) => {
                event.prevent_default();
                done();
                return;
            }
            _ => return,
        };
        event.prevent_default();
        if let Some(next) = next.filter(|next| Some(*next) != current) {
            spin_to.call(next);
        }
    };
    let neighbour = move |delta: isize, text: Option<String>| {
        let pressed = text.is_some().then_some(delta);
        rsx! {
            span {
                "data-slot": "neighbour",
                "aria-hidden": "true",
                onpointerdown: move |_| gestures.press(pressed),
                {text}
            }
        }
    };
    // Read here: the portal is no descendant of the slot's owner, so `LiveSlot` cannot.
    let unit = unit.and_then(|content| content.cloned()).map(|content| {
        rsx! {
            span { "data-slot": "unit", "aria-hidden": "true", {content} }
        }
    });
    let tab_id = dialog_id.clone();
    popover.show(showing.then(|| {
        dropdown
            .element(popover.floating())
            .attr("id", dialog_id)
            .attr("data-slot", "spinner-panel")
            .attr("role", "dialog")
            .attr(
                "aria-labelledby",
                labelled.then(|| format!("{field_id}-label")),
            )
            // A press keeps focus on the column.
            .event("onmousedown", move |event: MouseEvent| {
                event.prevent_default()
            })
            .event("onfocusin", focus.focusin(1))
            .event("onfocusout", focus.focusout(1))
            .event("onkeydown", move |event: KeyboardEvent| match event.key() {
                Key::Named(NamedKey::Escape) => {
                    event.prevent_default();
                    done();
                }
                _ if is_tab(&event) => picker_focus.tab(&tab_id, &event),
                _ => {}
            })
            .render(
                HtmlTag::Div,
                listeners,
                rsx! {
                    {strip}
                    div { "data-slot": "spinner",
                        div {
                            id: column_id,
                            "data-slot": "spin",
                            role: "spinbutton",
                            tabindex: "0",
                            "aria-labelledby": labelled.then(|| format!("{field_id}-label")),
                            "aria-describedby": unit_id,
                            "aria-valuenow": aria_number(current),
                            "aria-valuetext": value_text,
                            "aria-valuemin": aria_number(min),
                            "aria-valuemax": aria_number(max),
                            onmounted: column.mount(),
                            onkeydown,
                            onwheel: gestures.onwheel,
                            onpointerdown: gestures.onpointerdown,
                            onpointermove: gestures.onpointermove,
                            onpointerup: gestures.onpointerup,
                            onpointercancel: gestures.onpointercancel,
                            {neighbour(-1, before)}
                            span { "data-slot": "value", "{shown}" }
                            {neighbour(1, after)}
                        }
                        {unit}
                    }
                    div { "data-slot": "actions",
                        ActionIcon {
                            aria_label: labels.keyboard,
                            size: ThemeAwareValue::Size(slot_icon_size(size)),
                            onclick: keyboard,
                            Glyph { slot: IconSlot::Keyboard, icon: lucide::keyboard::outlined }
                        }
                        Button { onclick: move |_| done(), {labels.done} }
                    }
                },
            )
    }));

    rsx! {
        div {
            onmounted: anchor.mount(),
            onfocusin: focus.focusin(0),
            onfocusout: focus.focusout(0),
            style: sheet_clearance_style(clearance),
            {control}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::spun;

    /// An empty field lands on `min`, else zero, with its first step either way.
    #[test]
    fn an_empty_field_starts_at_min_else_zero() {
        assert_eq!(spun::<i32>(None, 1, 5, None, None), Some(0));
        assert_eq!(spun::<i32>(None, -1, 5, None, None), Some(0));
        assert_eq!(spun::<i32>(None, 3, 5, None, None), Some(10));
        assert_eq!(spun(None, 1, 5, Some(20), None), Some(20));
        assert_eq!(spun(None, -2, 5, Some(20), None), Some(20));
        assert_eq!(spun::<i32>(None, 0, 5, None, None), None);
        // A zero outside the range starts at the range.
        assert_eq!(spun(None, 1, 1, None, Some(-3)), Some(-3));
    }

    #[test]
    fn steps_stop_at_the_bounds() {
        assert_eq!(spun(Some(90), 5, 5, Some(0), Some(100)), Some(100));
        assert_eq!(spun(Some(10), -5, 5, Some(0), Some(100)), Some(0));
        assert_eq!(spun(Some(50), 0, 5, Some(0), Some(100)), Some(50));
    }

    /// No bounds: an integer stops at its type's limits instead of wrapping or panicking.
    #[test]
    fn integers_saturate_at_their_limits() {
        assert_eq!(spun(Some(250u8), 3, 5, None, None), Some(255));
        assert_eq!(spun(Some(3u32), -2, 5, None, None), Some(0));
        assert_eq!(spun(Some(i64::MIN + 1), -4, 1, None, None), Some(i64::MIN));
    }

    /// A value off the grid steps by `step` from itself, as the steppers do.
    #[test]
    fn an_off_grid_value_steps_from_itself() {
        assert_eq!(spun(Some(7), 1, 5, None, None), Some(12));
        assert_eq!(spun(Some(7), -1, 5, None, None), Some(2));
    }

    #[test]
    fn float_steps_round_to_the_step() {
        assert_eq!(spun(Some(0.1), 2, 0.1, None, None), Some(0.3));
        assert_eq!(spun(Some(1.0), -3, 0.1, None, None), Some(0.7));
        assert_eq!(spun(None, 4, 0.25, None, None), Some(0.75));
    }
}
