//! `TimeDelta` as a picker's value: posted as `PT1H30M`, picked on a spinbutton column per part.

use std::fmt::Write as _;
use std::sync::LazyLock;

use chrono::TimeDelta;
use dioxus::prelude::*;

use super::{
    ChronoPickerPart,
    range::Iso,
    spin_column::{SpinAt, SpinColumn, SpinOption},
    time_picker::TIME_PICKER_SX,
};
use crate::{
    components::{
        common::{ClassList, HtmlTag, Input, Part, Parts, States},
        form::use_fieldset_disabled,
        layout::use_box,
    },
    hooks::{use_element, use_localization, use_theme},
    platform::ElementApi,
    sx::Sx,
    theme::Size,
};

/// The top of the hours column without a `max`: 99 h 59 min 59 s.
const DEFAULT_MAX: TimeDelta = TimeDelta::seconds(99 * 3600 + 59 * 60 + 59);
/// The hours column stops here; `bounds` caps `max` to it.
const MAX_HOURS: i64 = 999;

/// `00` to `999`, so a pick allocates no label.
static NUMBERS: LazyLock<Vec<String>> = LazyLock::new(|| {
    (0..=MAX_HOURS)
        .map(|number| format!("{number:02}"))
        .collect()
});

/// The columns, by their `data-column`, in order.
const COLUMNS: [&str; 3] = ["Hours", "Minutes", "Seconds"];

/// `min` and `max` with their defaults: zero and 99 h 59 min 59 s; neither past 999 h 59 min 59 s.
pub(super) fn bounds(min: Option<TimeDelta>, max: Option<TimeDelta>) -> (TimeDelta, TimeDelta) {
    let top = of(MAX_HOURS, 59, 59);
    let min = min.unwrap_or_default().clamp(TimeDelta::zero(), top);
    (min, max.unwrap_or(DEFAULT_MAX).clamp(min, top))
}

/// Whole hours, minutes and seconds of a span that is not negative.
fn parts(span: TimeDelta) -> (i64, i64, i64) {
    let seconds = span.num_seconds();
    (seconds / 3600, seconds / 60 % 60, seconds % 60)
}

pub(super) fn of(hours: i64, minutes: i64, seconds: i64) -> TimeDelta {
    TimeDelta::seconds(hours * 3600 + minutes * 60 + seconds)
}

/// ISO 8601: `PT1H30M`, `PT0S` for none.
fn iso_duration(span: TimeDelta) -> String {
    let sign = if span < TimeDelta::zero() { "-" } else { "" };
    let (hours, minutes, seconds) = parts(span.abs());
    if (hours, minutes, seconds) == (0, 0, 0) {
        return "PT0S".to_string();
    }
    let mut iso = format!("{sign}PT");
    for (part, unit) in [(hours, 'H'), (minutes, 'M'), (seconds, 'S')] {
        if part > 0 {
            let _ = write!(iso, "{part}{unit}");
        }
    }
    iso
}

impl Iso for TimeDelta {
    fn iso(self) -> String {
        iso_duration(self)
    }
}

/// What `ChronoPicker::<TimeDelta>` draws, with every option resolved.
#[derive(Props, Clone, PartialEq)]
pub(super) struct DurationClockProps {
    value: Option<TimeDelta>,
    onchange: Option<EventHandler<Option<TimeDelta>>>,
    with_seconds: bool,
    step: Option<u8>,
    min: Option<TimeDelta>,
    max: Option<TimeDelta>,
    size: Input<Size>,
    focusable: bool,
    /// Soft: `aria-disabled` columns, presses and keys swallowed, nothing posted.
    #[props(default)]
    disabled: bool,
    name: Option<String>,
    class: Input<ClassList>,
    sx: Input<Sx>,
    parts: Input<Parts<ChronoPickerPart>>,
    states: Input<States>,
    attributes: Vec<Attribute>,
}

/// What a render reads, for the picks.
#[derive(Clone, Copy)]
struct DurationView {
    min: TimeDelta,
    max: TimeDelta,
    /// The value inside the bounds, or `min` with none yet.
    base: TimeDelta,
    step: i64,
    onchange: Option<EventHandler<Option<TimeDelta>>>,
}

impl DurationView {
    fn within(self, from: TimeDelta, to: TimeDelta) -> bool {
        !(to < self.min || from > self.max)
    }

    /// One column's option picked: the other parts stay, nothing carries.
    fn pick(self, column: &str, index: usize) {
        let (hours, minutes, seconds) = parts(self.base);
        let index = index as i64;
        let next = match column {
            "Hours" => of(index, minutes, seconds),
            "Minutes" => of(hours, index * self.step, seconds),
            _ => of(hours, minutes, index),
        };
        if let Some(onchange) = &self.onchange {
            onchange.call(Some(next.clamp(self.min, self.max)));
        }
    }
}

#[component]
pub(super) fn DurationClock(props: DurationClockProps) -> Element {
    let theme = use_theme();
    let names = &use_localization().date;
    let size = props.size.copied_or(theme.time_picker.size);
    // No native `disabled`, nor `inert`: either would drop a focused column's focus (todo 3881).
    let in_disabled_group = use_fieldset_disabled();
    let disabled = props.disabled || in_disabled_group;
    let (value, focusable) = (props.value, props.focusable && !disabled);
    let (min, max) = bounds(props.min, props.max);
    let base = value.map_or(min, |value| value.clamp(min, max));
    let view = DurationView {
        min,
        max,
        base,
        step: i64::from(props.step.unwrap_or(theme.time_picker.step).clamp(1, 30)),
        onchange: props.onchange,
    };
    let root = use_element();

    // One identity across renders, so a pick in one column skips the others.
    let pick = use_callback(move |(column, index): (&'static str, usize)| view.pick(column, index));
    // Enter, or typing that fills a column, moves on to the next.
    let ondone = use_callback(move |column: &'static str| {
        let next = COLUMNS
            .into_iter()
            .skip_while(|name| *name != column)
            .skip(1)
            .find_map(|name| root.query_selector(&format!("[data-column='{name}']")).ok());
        if let Some(next) = next {
            let _ = next.focus();
        }
    });

    let (hours, minutes, seconds) = parts(base);
    let step = view.step;
    let place = |index: i64, exact: bool| match (value, exact) {
        (None, _) => SpinAt::Empty(index as usize),
        (Some(_), true) => SpinAt::At(index as usize),
        (Some(_), false) => SpinAt::Past(index as usize),
    };
    let column = |name: &'static str,
                  label: &str,
                  unit: &str,
                  spoken: fn(u32) -> String,
                  options: Vec<SpinOption>,
                  at: SpinAt,
                  shown: i64,
                  wrap: bool,
                  page: usize| {
        rsx! {
            SpinColumn {
                column: name,
                label: label.to_string(),
                options,
                at,
                text: if value.is_some() { NUMBERS[shown as usize].as_str() } else { "--" },
                valuetext: spoken(shown as u32),
                wrap,
                page,
                focusable,
                disabled,
                onpick: pick,
                ondone,
            }
            span { "data-slot": ChronoPickerPart::Unit.slot(), "aria-hidden": "true", "{unit}" }
        }
    };
    let hour_options = (0..=parts(max).0.min(MAX_HOURS))
        .map(|hour| SpinOption {
            text: NUMBERS[hour as usize].as_str(),
            disabled: !view.within(of(hour, 0, 0), of(hour, 59, 59)),
        })
        .collect();
    let minute_options = (0..60)
        .step_by(step as usize)
        .map(|minute| SpinOption {
            text: NUMBERS[minute as usize].as_str(),
            disabled: !view.within(of(hours, minute, 0), of(hours, minute, 59)),
        })
        .collect();
    let seconds_column = props.with_seconds.then(|| {
        let options = (0..60)
            .map(|second| SpinOption {
                text: NUMBERS[second as usize].as_str(),
                disabled: !view.within(of(hours, minutes, second), of(hours, minutes, second)),
            })
            .collect();
        column(
            COLUMNS[2],
            names.seconds_label,
            names.seconds_short,
            names.seconds_value,
            options,
            place(seconds, true),
            seconds,
            true,
            15,
        )
    });
    let hours_column = column(
        COLUMNS[0],
        names.hours_label,
        names.hours_short,
        names.hours_value,
        hour_options,
        place(hours, true),
        hours,
        false,
        10,
    );
    let minutes_column = column(
        COLUMNS[1],
        names.minutes_label,
        names.minutes_short,
        names.minutes_value,
        minute_options,
        place(minutes / step, minutes % step == 0),
        minutes,
        true,
        (15 / step).max(1) as usize,
    );
    // A duration has no cycle to round its hours; minutes and seconds wrap
    // without carrying into the hour.
    let body = rsx! {
        div { "data-slot": ChronoPickerPart::Columns.slot(),
            {hours_column}
            {minutes_column}
            {seconds_column}
        }
    };

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .into();
    let root_box = use_box()
        .framework_sx(&TIME_PICKER_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&states)
        .prepare();
    let hidden = props.name.map(|name| {
        rsx! {
            input {
                r#type: "hidden",
                name,
                value: value.map(|_| iso_duration(base)).unwrap_or_default(),
                disabled,
            }
        }
    });
    root_box.element(&root).render(
        HtmlTag::Div,
        props.attributes,
        rsx! {
            {body}
            {hidden}
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn posts_iso_8601() {
        assert_eq!(iso_duration(of(1, 30, 0)), "PT1H30M");
        assert_eq!(iso_duration(of(0, 0, 15)), "PT15S");
        assert_eq!(iso_duration(of(100, 0, 1)), "PT100H1S");
        assert_eq!(iso_duration(TimeDelta::zero()), "PT0S");
    }

    /// Todo 856: each spinbutton says its value with the unit.
    #[test]
    fn the_columns_say_their_unit() {
        let html = dioxus_ssr::render_element(rsx! {
            crate::LiberoProvider {
                super::super::ChronoPicker::<TimeDelta> { value: of(1, 30, 0), with_seconds: true }
            }
        });
        for valuetext in ["1 hour", "30 minutes", "0 seconds"] {
            assert!(
                html.contains(&format!(r#"aria-valuetext="{valuetext}""#)),
                "{valuetext}: {html}"
            );
        }
    }

    #[test]
    fn bounds_default_to_zero_and_99_hours() {
        assert_eq!(bounds(None, None), (TimeDelta::zero(), DEFAULT_MAX));
        assert_eq!(bounds(Some(of(0, -5, 0)), None).0, TimeDelta::zero());
        assert_eq!(
            bounds(Some(of(2, 0, 0)), Some(of(1, 0, 0))),
            (of(2, 0, 0), of(2, 0, 0))
        );
        let top = of(999, 59, 59);
        assert_eq!(bounds(None, Some(of(2000, 0, 0))).1, top);
        assert_eq!(bounds(Some(of(1200, 0, 0)), None), (top, top));
    }

    /// Todo 2302: a value past 999 h renders at the cap instead of panicking.
    #[test]
    fn a_value_past_the_hours_column_renders() {
        let html = dioxus_ssr::render_element(rsx! {
            crate::LiberoProvider {
                super::super::ChronoPicker::<TimeDelta> {
                    value: of(1200, 0, 0),
                    max: of(2000, 0, 0),
                }
            }
        });
        assert!(html.contains(r#"aria-valuetext="999 hours""#), "{html}");
    }

    /// Todo 3675: the hidden input posts the clamped value the columns show.
    #[test]
    fn the_hidden_input_posts_the_clamped_value() {
        let html = dioxus_ssr::render_element(rsx! {
            crate::LiberoProvider {
                super::super::ChronoPicker::<TimeDelta> {
                    value: of(1200, 0, 0),
                    max: of(2000, 0, 0),
                    name: "span",
                }
            }
        });
        assert!(html.contains(r#"value="PT999H59M59S""#), "{html}");
    }
}
