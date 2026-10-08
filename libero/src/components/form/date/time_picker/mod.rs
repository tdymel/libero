mod render;
mod styles;
mod view;

use dioxus::prelude::*;

use chrono::{NaiveTime, Timelike};

use super::{
    ChronoPickerPart,
    date_value::{PickerArgs, PickerOptions, Sealed},
    format::uses_twelve_hours,
    parse_time::MIDNIGHT,
    props::date_props,
    spin_column::{SpinAt, SpinColumn, SpinOption},
};
use crate::{
    components::{
        common::{ClassList, HtmlTag, Input, Parts, States, input_from_str},
        layout::use_box,
    },
    hooks::{use_element, use_formats, use_localization, use_theme},
    platform::ElementApi,
    sx::Sx,
    theme::{Size, TimePickerVariant},
};

pub(super) use self::styles::TIME_PICKER_SX;
use self::view::{ClockView, Column, Hand, use_face_drag};

input_from_str!(TimePickerVariant);

date_props! {
    picker TimePickerProps(NaiveTime, NaiveTime): clock, limits
}

/// A time to pick, on columns of hours and minutes or on a clock face.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::chrono::NaiveTime;
/// # use libero::components::TimePicker;
/// # fn app() -> Element {
/// let mut time = use_signal(|| None::<NaiveTime>);
/// rsx! { TimePicker { value: time(), onchange: move |v| time.set(v) } }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/chrono-picker>
#[component]
pub fn TimePicker(props: TimePickerProps) -> Element {
    let theme = use_theme();
    let time_format = use_formats().time;
    NaiveTime::picker(PickerArgs {
        value: props.value,
        onchange: props.onchange,
        options: PickerOptions {
            min: props.min,
            max: props.max,
            variant: props.variant.copied_or(theme.time_picker.variant),
            with_seconds: props.with_seconds.unwrap_or(false),
            step: props.step,
            twelve_hour: props
                .twelve_hour
                .unwrap_or_else(|| uses_twelve_hours(time_format)),
            ..PickerOptions::default()
        },
        today: None,
        size: props.size,
        focusable: props.focusable.unwrap_or(true),
        name: props.name,
        class: props.class,
        sx: props.sx,
        parts: props.parts,
        states: props.states,
        attributes: props.attributes,
    })
}

/// What `TimePicker` draws, every option resolved. Plain props, so `ChronoPicker` can pass attributes on.
#[derive(Props, Clone, PartialEq)]
pub(super) struct ClockProps {
    value: Option<NaiveTime>,
    onchange: Option<EventHandler<Option<NaiveTime>>>,
    variant: TimePickerVariant,
    with_seconds: bool,
    step: Option<u8>,
    twelve_hour: bool,
    min: Option<NaiveTime>,
    max: Option<NaiveTime>,
    size: Input<Size>,
    focusable: bool,
    name: Option<String>,
    class: Input<ClassList>,
    sx: Input<Sx>,
    #[props(default)]
    parts: Input<Parts<ChronoPickerPart>>,
    states: Input<States>,
    attributes: Vec<Attribute>,
    /// Called once the last hand is picked, or Enter on the last column.
    #[props(default)]
    oncomplete: Option<Callback<()>>,
}

#[component]
pub(super) fn Clock(props: ClockProps) -> Element {
    let theme = use_theme();
    let size = props.size.copied_or(theme.time_picker.size);
    let value = props.value;
    let base = value.or(props.min).unwrap_or(MIDNIGHT);

    let hand = use_signal(|| Hand::Hour);
    let clock = ClockView {
        names: &use_localization().date,
        value,
        min: props.min,
        max: props.max,
        onchange: props.onchange,
        base,
        pm: base.hour() >= 12,
        twelve: props.twelve_hour,
        with_seconds: props.with_seconds,
        step: props.step.unwrap_or(theme.time_picker.step).clamp(1, 30),
        focusable: props.focusable,
        hand,
        oncomplete: props.oncomplete,
    };
    let root = use_element();

    // One identity across renders, so the columns' props compare equal and a
    // pick in one column skips the others.
    let pick = use_callback(move |(name, index): (&'static str, usize)| {
        if let Some(column) = Column::named(name) {
            clock.pick(column, index);
        }
    });
    // Enter, or typing that fills a column, moves on to the next, as a native
    // time input; after the last the time is complete.
    let oncomplete = props.oncomplete;
    let ondone = use_callback(move |name: &'static str| {
        let later = Column::ALL
            .into_iter()
            .skip_while(|column| column.name() != name);
        let next = later.skip(1).find_map(|column| {
            root.query_selector(&format!("[data-column='{}']", column.name()))
                .ok()
        });
        match (next, oncomplete) {
            (Some(next), _) => {
                let _ = next.focus();
            }
            (None, Some(oncomplete)) => oncomplete.call(()),
            (None, None) => {}
        }
    });
    let face = use_element();
    let drag = use_face_drag(clock, face);

    let body = match props.variant {
        TimePickerVariant::Digital => clock.digital_view(pick, ondone),
        TimePickerVariant::Analog => clock.analog_view(face, drag),
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
                value: value.map(|value| value.to_string()).unwrap_or_default(),
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

    /// Todo 2309: `role="slider"` needs an `aria-valuenow`, also with no time picked.
    #[test]
    fn an_empty_analog_face_keeps_its_value_and_says_no_time() {
        let html = dioxus_ssr::render_element(rsx! {
            crate::LiberoProvider {
                TimePicker { variant: "analog", twelve_hour: false }
            }
        });
        let face = html.split(r#"data-slot="face""#).nth(1).expect("a face");
        let face = face.split('>').next().unwrap_or_default();
        assert!(face.contains("aria-valuenow=0 "), "{face}");
        assert!(
            face.contains(r#"aria-valuetext="No time selected""#),
            "{face}"
        );
    }
}
