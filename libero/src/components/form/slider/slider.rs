use dioxus::prelude::*;

use super::core::{SliderCore, SliderPart};
use super::scale::{Scale, control_spacing};
use super::slider_value::{SliderChangeEvent, SliderMark, SliderSegment, SliderValue};
use super::value::SliderCoreValue;
use crate::{
    components::{
        common::Input,
        form::{FieldStatus, field_props, use_bound, use_field},
    },
    hooks::{use_localization, use_theme},
    sx::ThemeAwareValue,
};

field_props! {
    parts(SliderPart);
    without(radius);
    pub struct SliderProps<V: SliderValue> {
        /// Controlled: pair it with `oninput`, or bind a path `name` in a `Form`.
        #[props(default)]
        value: Option<V>,
        /// Defaults to the first option, or `0.0` on a continuous scale.
        #[props(default, into)]
        min: Option<V>,
        /// Defaults to the last option, or `100.0` on a continuous scale. Off the
        /// `step` grid, the track ends at the last step below it.
        #[props(default, into)]
        max: Option<V>,
        /// One step from `min`: options discretely, a value continuously. Sets
        /// the decimals an emitted value keeps.
        #[props(default, into)]
        step: Option<V::Step>,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// Formats the bubble, `aria-valuetext` and derived marks; runs during
        /// render, so it can translate.
        #[props(default)]
        format: Option<Callback<V, String>>,
        /// Ticks on the track, a labeled one captioned. Replaces derived marks;
        /// one outside the track (past an off-grid `max`'s last step too) is dropped.
        #[props(default)]
        marks: Vec<SliderMark<V>>,
        /// Splits a line track into stretches with gaps, each from its `start` to
        /// the next; a labeled one joins the bubble and `aria-valuetext`.
        #[props(default)]
        segments: Vec<SliderSegment<V>>,
        /// Names the thumb without a `label`; one in `attributes` lands on the wrapper.
        #[props(default)]
        aria_label: Option<String>,
        /// What the value posts as. A path also binds it to the surrounding `Form`.
        #[props(default, into)]
        name: crate::components::form::FieldName<V>,
        /// Fires per value; `Start`/`End` bracket a drag.
        #[props(default)]
        oninput: Option<EventHandler<SliderChangeEvent<V>>>,
        /// Rules over the value, shown on blur or submit.
        #[props(default, into)]
        validate: crate::components::form::Validators<V>,
        #[props(default)]
        track: SliderTrack,
    }
}

/// How a [`Slider`]'s track is drawn.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum SliderTrack {
    /// A thin rail, filled up to the value.
    #[default]
    Line,
    /// A row of rounded bars as tall as these fractions (0 to 1) of the track,
    /// like a voice message's waveform; the bars up to the value are filled.
    Bars(Vec<f64>),
}

/// A draggable value along a track. A [`SliderValue`] enum that lists its
/// options makes it discrete.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Slider, SliderChangeEvent};
/// # fn app() -> Element {
/// let mut volume = use_signal(|| 40.0);
/// rsx! {
///     Slider {
///         label: "Volume",
///         value: volume(),
///         oninput: move |event: SliderChangeEvent| volume.set(event.value()),
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/slider>
#[component]
pub fn Slider<V: SliderValue>(props: SliderProps<V>) -> Element {
    let theme = use_theme();
    let scale = Scale::of::<V>(props.min.as_ref(), props.max.as_ref(), props.step);
    let (min, max, step) = scale.bounds();
    let bound = use_bound(&props.name, props.oninput.is_some());
    let disabled = bound.disabled(props.disabled);
    let value = bound
        .value()
        .or_else(|| props.value.clone())
        .unwrap_or_else(|| V::at(min));

    // A discrete value names itself unless the caller says how - which is the
    // hook an i18n'd slider hangs on, so every caption has to go through it.
    let named = props.format;
    let name = move |value: &V| match &named {
        Some(label) => label.call(value.clone()),
        None => value.label(),
    };

    let marks = match props.marks.is_empty() {
        true => scale.derived_marks::<V>(&name),
        false => props
            .marks
            .iter()
            .map(|mark| SliderMark {
                value: mark.value.position(),
                label: mark.label.clone(),
            })
            .collect(),
    };

    // A discrete value always names itself; a continuous one only when the
    // caller says how, which is what keeps `aria-valuetext` off a bare number.
    let labelled = props.format.is_some() || V::options().is_some();
    let label = use_callback(move |position: f64| name(&V::at(position)));

    let oninput = props.oninput;
    let setter = bound.setter();
    let emit = use_callback(move |event: SliderChangeEvent<SliderCoreValue>| {
        let event = event.map(|value| V::at(value.thumb(0)));
        match (&oninput, &setter) {
            (Some(oninput), _) => oninput.call(event),
            // A bound slider keeps only the values - `Start`/`End` carry
            // nothing a form's value holds.
            (None, Some(setter)) => {
                if let SliderChangeEvent::Change(value) = event {
                    setter.set(value);
                }
            }
            (None, None) => {}
        }
    });

    let required = props.required.unwrap_or(false);
    let required_word = use_localization().slider.required;

    // The thumb carries `role="slider"`, and `for` names only a labelable
    // element - so the field hands its ids over rather than applying them.
    let field = use_field()
        .labelled_by()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .rules(props.validate.check(&value))
        .bound(&bound)
        .required(required)
        .required_in_name(required_word)
        .disabled(disabled)
        .size(props.size.copied_or(theme.slider.size))
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&props.states)
        .aria_label(props.aria_label.as_deref())
        .attributes(&props.attributes)
        .prepare();
    crate::components::common::use_name_warning(
        field.label_id().is_some() || props.aria_label.is_some(),
        "Slider: no `label` or `aria_label`, so the thumb is announced as just \"slider\".",
    );

    let above = !props.label.is_none() || !props.description.is_none();
    let below = !props.helper.is_none()
        || props
            .status
            .as_ref()
            .and_then(FieldStatus::message)
            .is_some();
    let control_sx = control_spacing(above, below);

    let control = rsx! {
        SliderCore {
            value: SliderCoreValue::Single(value.position()),
            min,
            max,
            step,
            marks,
            // The thumb's `aria-valuetext` names a discrete value, so the captions would say it twice.
            captions_hidden: V::options().is_some(),
            sx: control_sx,
            attributes: props.attributes,
            size: props.size,
            color: props.color,
            disabled: Some(bound.disabled(props.disabled)),
            readonly: props.readonly.unwrap_or(false),
            label: labelled.then_some(label),
            // Unlabelled, the thumb's own name says required instead of the label.
            aria_label: props.aria_label.map(|name| match required && field.label_id().is_none() {
                true => format!("{name} {required_word}"),
                false => name,
            }),
            labelledby: field.label_id(),
            describedby: field.describedby(),
            invalid: field.invalid(),
            name: bound.name().map(str::to_string),
            // The same `use_callback` every render, so the core's props can
            // compare equal.
            oninput: (props.oninput.is_some() || bound.is_bound()).then_some(emit),
            segments: props
                .segments
                .iter()
                .map(|segment| SliderSegment {
                    start: segment.start.position(),
                    label: segment.label.clone(),
                })
                .collect::<Vec<_>>(),
            bars: match props.track {
                SliderTrack::Line => None,
                SliderTrack::Bars(heights) => Some(heights),
            },
        }
    };

    field.render(control)
}
