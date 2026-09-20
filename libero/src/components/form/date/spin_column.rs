//! One column of a digital clock: an APG spinbutton over a fixed list of values.

use dioxus::prelude::*;

use crate::{
    components::common::has_shortcut_modifier,
    hooks::{DragMove, DragOptions, DragStart, use_drag, use_element},
    platform,
};

/// The wheel travel, in pixels, that turns a column one step.
const WHEEL_STEP: f64 = 40.0;
/// The drag travel, in pixels, that turns a column one step.
const DRAG_STEP: f64 = 24.0;

/// One value a column can show.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SpinOption {
    pub text: &'static str,
    /// Skipped by every step: outside `min` and `max`.
    pub disabled: bool,
}

/// Where a column's value sits among its options.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum SpinAt {
    /// On this option.
    At(usize),
    /// Off the options, just past this one: a minute off the step.
    Past(usize),
    /// No value yet: the first step lands here, or on the next open option.
    Empty(usize),
}

/// The open option one step from `from` in the direction of `sign`.
fn neighbour(options: &[SpinOption], from: usize, sign: isize, wrap: bool) -> Option<usize> {
    let count = options.len() as isize;
    let mut at = from as isize;
    for _ in 0..count {
        at += sign;
        if wrap {
            at = at.rem_euclid(count);
        } else if !(0..count).contains(&at) {
            return None;
        }
        if !options[at as usize].disabled {
            return Some(at as usize);
        }
    }
    None
}

/// The option `delta` steps from `at`, stopping at the last open one short of
/// an end. `None` when an empty or in-between value has nowhere to land.
pub(super) fn stepped(
    options: &[SpinOption],
    at: SpinAt,
    delta: isize,
    wrap: bool,
) -> Option<usize> {
    let open = |index: usize| options.get(index).is_some_and(|option| !option.disabled);
    let steps = delta.unsigned_abs();
    let (mut index, left, mut landed) = match at {
        SpinAt::At(index) => (index, steps, true),
        SpinAt::Past(index) if delta < 0 && open(index) => (index, steps - 1, true),
        SpinAt::Empty(index) if open(index) => (index, steps.saturating_sub(1), true),
        SpinAt::Past(index) | SpinAt::Empty(index) => (index, steps, false),
    };
    for _ in 0..left {
        match neighbour(options, index, delta.signum(), wrap) {
            Some(next) => {
                index = next;
                landed = true;
            }
            None => break,
        }
    }
    landed.then_some(index)
}

/// The option typed text picks: an exact match without leading zeros first (`1` is `01`,
/// not `12`), then a prefix.
pub(super) fn typed(options: &[SpinOption], text: &str) -> Option<usize> {
    let text = text.to_lowercase();
    let open = || {
        options
            .iter()
            .enumerate()
            .filter(|(_, option)| !option.disabled)
    };
    let bare = |option: &SpinOption| {
        let trimmed = option.text.trim_start_matches('0');
        if trimmed.is_empty() { "0" } else { trimmed }.to_lowercase()
    };
    open()
        .find(|(_, option)| bare(option) == text || option.text.to_lowercase() == text)
        .or_else(|| open().find(|(_, option)| option.text.to_lowercase().starts_with(&text)))
        .map(|(index, _)| index)
}

/// Whether more typing could still pick another option than `picked`.
fn typing_goes_on(options: &[SpinOption], text: &str, picked: usize) -> bool {
    let text = text.to_lowercase();
    options.iter().enumerate().any(|(index, option)| {
        let own = option.text.to_lowercase();
        index != picked && !option.disabled && own.len() > text.len() && own.starts_with(&text)
    })
}

/// The travel left over and the step a wheel event takes: a notch is one step,
/// a trackpad's small deltas add up to one.
fn wheel_steps(held: f64, travel: f64) -> (f64, isize) {
    // A turn back drops what was held the other way.
    let total = if held * travel < 0.0 {
        travel
    } else {
        held + travel
    };
    match total.abs() >= WHEEL_STEP {
        // Down the page brings the value below up: the next one.
        true => (0.0, total.signum() as isize),
        false => (total, 0),
    }
}

/// A number for `aria-valuenow`: the text's, else the option's place.
fn number(text: &str, index: usize) -> i64 {
    text.parse().unwrap_or(index as i64)
}

#[derive(Props, Clone, PartialEq)]
pub(super) struct SpinColumnProps {
    /// `data-column`: stable across locales, so a selector can find it.
    pub column: &'static str,
    /// The spinbutton's accessible name.
    pub label: String,
    pub options: Vec<SpinOption>,
    pub at: SpinAt,
    /// What the column shows: its option's text, a value between the options, or `--`.
    pub text: &'static str,
    /// `aria-valuetext` with its unit (`2 hours`); `text` without one.
    pub valuetext: Option<String>,
    /// Steps past either end come round to the other.
    pub wrap: bool,
    /// The steps Page Up and Page Down take.
    pub page: usize,
    pub focusable: bool,
    /// Called with `column` and the picked index: one callback serves every column,
    /// so a pick leaves the others' props equal.
    pub onpick: Callback<(&'static str, usize)>,
    /// Called with `column` on Enter, or once typing can pick nothing else.
    pub ondone: Callback<&'static str>,
}

/// The value between its two neighbours, a spinbutton. Keys, typing, wheel and a
/// vertical drag turn it; a press on a neighbour picks that.
#[component]
pub(super) fn SpinColumn(props: SpinColumnProps) -> Element {
    let SpinColumnProps {
        column,
        label,
        options,
        at,
        text,
        valuetext,
        wrap,
        page,
        focusable,
        onpick,
        ondone,
    } = props;
    let handle = use_element();
    // The keys typed so far, until they pick a value no key can change.
    let mut typing = use_hook(|| CopyValue::new(String::new()));
    let mut wheel = use_hook(|| CopyValue::new(0.0_f64));
    // Where a drag started, and the neighbour a press landed on.
    let mut drag_from = use_hook(|| CopyValue::new(None::<(SpinAt, isize)>));
    let mut pressed = use_hook(|| CopyValue::new(None::<isize>));

    let current = match at {
        SpinAt::At(index) => Some(index),
        _ => None,
    };
    let pick = move |next: Option<usize>| {
        if let Some(next) = next.filter(|next| Some(*next) != current) {
            onpick.call((column, next));
        }
    };
    let options_for_keys = options.clone();
    let onkeydown = move |event: KeyboardEvent| {
        if has_shortcut_modifier(&event) {
            return;
        }
        let options = &options_for_keys;
        let open = || {
            options
                .iter()
                .enumerate()
                .filter(|(_, option)| !option.disabled)
        };
        let next = match event.key() {
            Key::ArrowUp => stepped(options, at, 1, wrap),
            Key::ArrowDown => stepped(options, at, -1, wrap),
            Key::PageUp => stepped(options, at, page as isize, wrap),
            Key::PageDown => stepped(options, at, -(page as isize), wrap),
            Key::Home => open().next().map(|(index, _)| index),
            Key::End => open().next_back().map(|(index, _)| index),
            Key::Character(key)
                if key.chars().count() == 1 && key.chars().all(char::is_alphanumeric) =>
            {
                let mut text = format!("{}{key}", typing.peek());
                let mut found = typed(options, &text);
                if found.is_none() {
                    text = key.clone();
                    found = typed(options, &text);
                }
                event.prevent_default();
                let Some(found) = found else {
                    typing.set(String::new());
                    return;
                };
                pick(Some(found));
                if typing_goes_on(options, &text, found) {
                    typing.set(text);
                } else {
                    typing.set(String::new());
                    ondone.call(column);
                }
                return;
            }
            Key::Enter => {
                event.prevent_default();
                typing.set(String::new());
                ondone.call(column);
                return;
            }
            _ => return,
        };
        event.prevent_default();
        typing.set(String::new());
        pick(next);
    };

    let options_for_wheel = options.clone();
    let onwheel = move |event: Event<WheelData>| {
        event.prevent_default();
        let travel = platform::wheel_travel_y(&event.data(), WHEEL_STEP, page as f64);
        let (total, delta) = wheel_steps(*wheel.peek(), travel);
        wheel.set(total);
        if delta != 0 {
            pick(stepped(&options_for_wheel, at, delta, wrap));
        }
    };

    let options_for_drag = options.clone();
    let onstart = use_callback(move |_: DragStart| drag_from.set(Some((at, 0))));
    let onmove = use_callback(move |step: DragMove| {
        let Some((from, taken)) = drag_from.cloned() else {
            return;
        };
        // Up brings the value below into the middle, as a wheel's rim.
        let steps = (-step.delta().y / DRAG_STEP).round() as isize;
        if steps == taken {
            return;
        }
        drag_from.set(Some((from, steps)));
        let next = match steps {
            0 => match from {
                SpinAt::At(index) => Some(index),
                _ => None,
            },
            steps => stepped(&options_for_drag, from, steps, wrap),
        };
        if let Some(next) = next {
            onpick.call((column, next));
        }
    });
    let options_for_press = options.clone();
    let onend = use_callback(move |()| {
        let dragged = drag_from.cloned().is_some_and(|(_, taken)| taken != 0);
        drag_from.set(None);
        if let Some(delta) = pressed.take().filter(|_| !dragged) {
            pick(stepped(&options_for_press, at, delta, wrap));
        }
    });
    let drag = use_drag(DragOptions {
        capture: handle,
        onstart,
        onmove,
        onend,
    });

    // An empty column shows no neighbours: nothing is picked to step from.
    let beside = |delta: isize| match at {
        SpinAt::Empty(_) => "",
        _ => stepped(&options, at, delta, wrap)
            .filter(|index| Some(*index) != current)
            .map_or("", |index| options[index].text),
    };
    let (before, after) = (beside(-1), beside(1));
    let numbers: Vec<i64> = options
        .iter()
        .enumerate()
        .map(|(index, option)| number(option.text, index))
        .collect();
    let now = match at {
        SpinAt::At(index) => Some(number(text, index)),
        SpinAt::Past(index) => Some(number(text, index)),
        SpinAt::Empty(_) => None,
    };
    let neighbour = move |delta: isize, text: &'static str| {
        rsx! {
            span {
                "data-slot": "neighbour",
                "aria-hidden": "true",
                onpointerdown: move |_| pressed.set((!text.is_empty()).then_some(delta)),
                "{text}"
            }
        }
    };
    rsx! {
        div {
            "data-slot": "spin",
            "data-column": column,
            role: "spinbutton",
            tabindex: if focusable { "0" } else { "-1" },
            "aria-label": label,
            "aria-valuenow": now,
            "aria-valuetext": now.is_some().then(|| valuetext.unwrap_or_else(|| text.to_string())),
            "aria-valuemin": numbers.iter().min().copied(),
            "aria-valuemax": numbers.iter().max().copied(),
            onmounted: handle.mount(),
            onkeydown,
            onwheel,
            onblur: move |_| typing.set(String::new()),
            onpointerdown: drag.onpointerdown,
            onpointermove: drag.onpointermove,
            onpointerup: drag.onpointerup,
            onpointercancel: drag.onpointercancel,
            {neighbour(-1, before)}
            span { "data-slot": "value", "{text}" }
            {neighbour(1, after)}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(texts: &[&'static str]) -> Vec<SpinOption> {
        texts
            .iter()
            .map(|text| SpinOption {
                text,
                disabled: false,
            })
            .collect()
    }

    const MINUTES: [&str; 12] = [
        "00", "05", "10", "15", "20", "25", "30", "35", "40", "45", "50", "55",
    ];

    #[test]
    fn steps_wrap_or_stop_at_the_ends() {
        let minutes = options(&MINUTES);
        assert_eq!(stepped(&minutes, SpinAt::At(11), 1, true), Some(0));
        assert_eq!(stepped(&minutes, SpinAt::At(0), -1, true), Some(11));
        assert_eq!(stepped(&minutes, SpinAt::At(11), 1, false), Some(11));
        // A page short of the end stops on the last open option.
        assert_eq!(stepped(&minutes, SpinAt::At(9), 3, false), Some(11));
        assert_eq!(stepped(&minutes, SpinAt::At(1), 3, true), Some(4));
    }

    #[test]
    fn steps_skip_disabled_options() {
        let mut minutes = options(&MINUTES);
        for option in &mut minutes[..6] {
            option.disabled = true;
        }
        // Nowhere to go: it stays.
        assert_eq!(stepped(&minutes, SpinAt::At(6), -1, false), Some(6));
        assert_eq!(stepped(&minutes, SpinAt::At(11), 1, true), Some(6));
        assert_eq!(stepped(&minutes, SpinAt::Empty(0), 1, true), Some(6));
    }

    #[test]
    fn a_value_off_the_options_steps_onto_them() {
        let minutes = options(&MINUTES);
        // 07 sits past 05: up lands on 10, down on 05.
        assert_eq!(stepped(&minutes, SpinAt::Past(1), 1, true), Some(2));
        assert_eq!(stepped(&minutes, SpinAt::Past(1), -1, true), Some(1));
        // No value yet: the first step lands on the start either way.
        assert_eq!(stepped(&minutes, SpinAt::Empty(6), 1, true), Some(6));
        assert_eq!(stepped(&minutes, SpinAt::Empty(6), -1, true), Some(6));
    }

    #[test]
    fn a_wheel_notch_takes_one_step_and_small_deltas_add_up() {
        assert_eq!(wheel_steps(0.0, 100.0), (0.0, 1));
        assert_eq!(wheel_steps(0.0, -360.0), (0.0, -1));
        assert_eq!(wheel_steps(0.0, 15.0), (15.0, 0));
        assert_eq!(wheel_steps(30.0, 15.0), (0.0, 1));
        assert_eq!(wheel_steps(30.0, -15.0), (-15.0, 0));
    }

    #[test]
    fn typing_picks_by_the_bare_number_then_the_start() {
        let hours = options(&[
            "12", "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11",
        ]);
        assert_eq!(typed(&hours, "1"), Some(1));
        assert!(typing_goes_on(&hours, "1", 1));
        assert_eq!(typed(&hours, "12"), Some(0));
        assert!(!typing_goes_on(&hours, "12", 0));
        assert_eq!(typed(&hours, "0"), Some(1));

        let minutes = options(&MINUTES);
        assert_eq!(typed(&minutes, "3"), Some(6));
        assert_eq!(typed(&minutes, "35"), Some(7));
        assert_eq!(typed(&minutes, "0"), Some(0));
        assert_eq!(typed(&minutes, "7"), None);

        let halves = options(&["AM", "PM"]);
        assert_eq!(typed(&halves, "p"), Some(1));
        assert!(!typing_goes_on(&halves, "p", 1));
    }
}
