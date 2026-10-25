//! One column of a digital clock: an APG spinbutton over a fixed list of values.

use dioxus::prelude::*;

use crate::{
    components::{
        common::{Part, has_shortcut_modifier},
        form::{
            ChronoPickerPart,
            spin::{Turn, key_steps, use_spin_gestures},
        },
    },
    hooks::use_element,
    utils::fold_digits,
};

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
    let text = fold_digits(text).to_lowercase();
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
    let text = fold_digits(text).to_lowercase();
    options.iter().enumerate().any(|(index, option)| {
        let own = option.text.to_lowercase();
        index != picked && !option.disabled && own.len() > text.len() && own.starts_with(&text)
    })
}

/// A number for `aria-valuenow`: the text's, else the option's place.
fn number(text: &str, index: usize) -> i64 {
    text.parse().unwrap_or(index as i64)
}

/// `aria-valuemin` and `aria-valuemax`: the open options, with the current value kept inside.
fn value_range(options: &[SpinOption], now: Option<i64>) -> (Option<i64>, Option<i64>) {
    let numbers: Vec<i64> = options
        .iter()
        .enumerate()
        .filter(|(_, option)| !option.disabled)
        .map(|(index, option)| number(option.text, index))
        .chain(now)
        .collect();
    (numbers.iter().min().copied(), numbers.iter().max().copied())
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
    /// `aria-disabled`: keys, typing, wheel, drags and presses do nothing.
    pub disabled: bool,
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
        disabled,
        onpick,
        ondone,
    } = props;
    let handle = use_element();
    // The keys typed so far, until they pick a value no key can change.
    let mut typing = use_hook(|| CopyValue::new(String::new()));

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
        if disabled || has_shortcut_modifier(&event) {
            return;
        }
        let options = &options_for_keys;
        let open = || {
            options
                .iter()
                .enumerate()
                .filter(|(_, option)| !option.disabled)
        };
        let key = event.key();
        let steps = key_steps(&key, page);
        let next = match key {
            _ if steps.is_some() => steps.and_then(|delta| stepped(options, at, delta, wrap)),
            Key::Named(NamedKey::Home) => open().next().map(|(index, _)| index),
            Key::Named(NamedKey::End) => open().next_back().map(|(index, _)| index),
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
            Key::Named(NamedKey::Enter) => {
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

    let options_for_turn = options.clone();
    let onturn = use_callback(move |turn: Turn<SpinAt>| match turn {
        Turn::By(delta) => pick(stepped(&options_for_turn, at, delta, wrap)),
        Turn::From(from, steps) => {
            let next = match steps {
                0 => match from {
                    SpinAt::At(index) => Some(index),
                    _ => None,
                },
                steps => stepped(&options_for_turn, from, steps, wrap),
            };
            if let Some(next) = next {
                onpick.call((column, next));
            }
        }
    });
    let gestures = use_spin_gestures(handle, at, disabled, page, onturn);

    // An empty column shows no neighbours: nothing is picked to step from.
    let beside = |delta: isize| match at {
        SpinAt::Empty(_) => "",
        _ => stepped(&options, at, delta, wrap)
            .filter(|index| Some(*index) != current)
            .map_or("", |index| options[index].text),
    };
    let (before, after) = (beside(-1), beside(1));
    let now = match at {
        SpinAt::At(index) => Some(number(text, index)),
        SpinAt::Past(index) => Some(number(text, index)),
        SpinAt::Empty(_) => None,
    };
    let (value_min, value_max) = value_range(&options, now);
    let neighbour = move |delta: isize, text: &'static str| {
        rsx! {
            span {
                "data-slot": ChronoPickerPart::Neighbour.slot(),
                "aria-hidden": "true",
                onpointerdown: move |_| gestures.press((!text.is_empty() && !disabled).then_some(delta)),
                "{text}"
            }
        }
    };
    rsx! {
        div {
            "data-slot": ChronoPickerPart::Spin.slot(),
            "data-column": column,
            role: "spinbutton",
            tabindex: if focusable { "0" } else { "-1" },
            "aria-disabled": disabled.then_some("true"),
            "aria-label": label,
            "aria-valuenow": now,
            "aria-valuetext": now.is_some().then(|| valuetext.unwrap_or_else(|| text.to_string())),
            "aria-valuemin": value_min,
            "aria-valuemax": value_max,
            onmounted: handle.mount(),
            onkeydown,
            onwheel: gestures.onwheel,
            onblur: move |_| typing.set(String::new()),
            onpointerdown: gestures.onpointerdown,
            onpointermove: gestures.onpointermove,
            onpointerup: gestures.onpointerup,
            onpointercancel: gestures.onpointercancel,
            {neighbour(-1, before)}
            span { "data-slot": ChronoPickerPart::Value.slot(), "{text}" }
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
    fn the_value_range_leaves_out_disabled_options() {
        let mut minutes = options(&MINUTES);
        for option in &mut minutes[..6] {
            option.disabled = true;
        }
        assert_eq!(value_range(&minutes, Some(30)), (Some(30), Some(55)));
        // A value ruled out stays inside its own range.
        assert_eq!(value_range(&minutes, Some(5)), (Some(5), Some(55)));
        assert_eq!(value_range(&minutes, None), (Some(30), Some(55)));
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

    /// Todo 2396: an Arabic-Indic or full-width keyboard types the same hour.
    #[test]
    fn typing_digits_of_another_script_picks_the_same_option() {
        let minutes = options(&MINUTES);
        assert_eq!(typed(&minutes, "\u{0663}\u{0665}"), typed(&minutes, "35"));
        assert_eq!(typed(&minutes, "\u{FF13}"), typed(&minutes, "3"));
        assert!(typing_goes_on(&minutes, "\u{FF13}", 6));
    }
}
