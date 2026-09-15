use dioxus::prelude::*;

use super::core::SliderCore;
use super::scale::{Scale, control_spacing};
use super::slider_value::{SliderChangeEvent, SliderMark, SliderValue};
use super::value::SliderCoreValue;
use crate::{
    components::{FieldStatus, Input, common::field_props, form::use_bound, use_field},
    hooks::use_theme,
    sx::ThemeAwareValue,
};

field_props! {
    pub struct SliderProps<V: SliderValue> {
        /// Strictly controlled - pair it with `oninput`. Inside a `Form`, a
        /// path `name` can supply it instead.
        #[props(default)]
        value: Option<V>,
        /// Defaults to the first option, or `0.0` on a continuous scale.
        #[props(default, into)]
        min: Option<V>,
        /// Defaults to the last option, or `100.0` on a continuous scale.
        #[props(default, into)]
        max: Option<V>,
        /// Distance one step covers: a count of options discretely, a value
        /// continuously. Measured from `min`, and it sets how many decimals
        /// an emitted value keeps.
        #[props(default, into)]
        step: Option<V::Step>,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// Formats the bubble shown on hover, drag and keyboard focus, and
        /// sets the thumb's `aria-valuetext`. Defaults to the bare value
        /// continuously, and to `SliderValue::label` discretely. Also names
        /// the derived marks, and runs during render - so it is how a
        /// discrete slider is translated, reading a locale from context.
        #[props(default)]
        format: Option<Callback<V, String>>,
        /// Ticks on the track; a labeled one gets a caption below it.
        /// Replaces the marks a discrete scale derives.
        #[props(default)]
        marks: Vec<SliderMark<V>>,
        /// Names the thumb, which is the `role="slider"` element, when the
        /// field has no `label` - an `aria_label` in `attributes` would land
        /// on the wrapper instead.
        #[props(default)]
        aria_label: Option<String>,
        /// Emits a hidden input of that name, so the value posts with a form.
        /// A path - `Settings::FIELDS.volume()` - also binds the value to the
        /// surrounding `Form`'s value when there is no `oninput`.
        #[props(default, into)]
        name: crate::components::FieldName<V>,
        /// Fires per value: a drag is the DOM's `input` event, not its
        /// `change`. `Start`/`End` bracket a drag, `Change` carries every new
        /// value.
        #[props(default)]
        oninput: Option<EventHandler<SliderChangeEvent<V>>>,
        /// Rules over the value, shown once the slider loses focus or its form
        /// is submitted.
        #[props(default, into)]
        validate: crate::components::Validators<V>,
    }
}

/// A draggable value along a track, with the field slots stacked around it.
/// Controlled: it renders `value` and asks for a new one through `oninput`.
///
/// `V` is inferred from `value`. A [`SliderValue`] that lists its options -
/// an ordered enum - makes the slider discrete: the range, the step grid, one
/// mark per option and every caption come from the list, and `min`/`max` are
/// written in that type rather than as indices.
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
        .disabled(disabled)
        .size(props.size.copied_or(theme.slider.size))
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
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
            aria_label: props.aria_label,
            labelledby: field.label_id(),
            describedby: field.describedby(),
            invalid: field.invalid(),
            name: bound.name().map(str::to_string),
            // The same `use_callback` every render, so the core's props can
            // compare equal.
            oninput: (props.oninput.is_some() || bound.is_bound()).then_some(emit),
        }
    };

    field.render(control)
}
