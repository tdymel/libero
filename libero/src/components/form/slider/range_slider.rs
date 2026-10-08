use dioxus::prelude::*;

use super::core::{SliderCore, SliderPart};
use super::scale::{Scale, control_spacing};
use super::slider_value::{SliderChangeEvent, SliderMark, SliderStep, SliderValue};
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
    pub struct RangeSliderProps<V: SliderValue> {
        /// The two ends in track order. Controlled: pair it with `oninput`, or
        /// bind a path `name` in a `Form`.
        #[props(default)]
        value: Option<(V, V)>,
        /// Defaults to the first option, or `0.0` on a continuous scale.
        #[props(default, into)]
        min: Option<V>,
        /// Defaults to the last option, or `100.0` on a continuous scale. Off the
        /// `step` grid, the track ends at the last step below it.
        #[props(default, into)]
        max: Option<V>,
        /// One step from `min`: options discretely, a value continuously. Sets
        /// the decimals an emitted value keeps. Defaults to `1.0` on a continuous
        /// scale, so `0.0..1.0` has two stops; `0.0` keeps a thousandth of the range.
        #[props(default, into)]
        step: Option<V::Step>,
        /// The smallest gap the two thumbs keep. They may meet by default.
        #[props(default, into)]
        min_range: Option<V::Step>,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// Formats the bubbles, `aria-valuetext` and derived marks; runs during
        /// render, so it can translate.
        #[props(default)]
        format: Option<Callback<V, String>>,
        /// Ticks on the track, a labeled one captioned. Replaces derived marks;
        /// one outside the track (past an off-grid `max`'s last step too) is dropped.
        #[props(default)]
        marks: Vec<SliderMark<V>>,
        /// Names the pair without a `label`, as a group and before each thumb's
        /// name; one in `attributes` lands on the wrapper.
        #[props(default)]
        aria_label: Option<String>,
        /// Names the lower thumb; defaults to the localization's `slider.minimum`.
        #[props(default)]
        aria_label_from: Option<String>,
        /// Names the upper thumb; defaults to the localization's `slider.maximum`.
        #[props(default)]
        aria_label_to: Option<String>,
        /// Posts two inputs of that name in track order. A path also binds the
        /// pair to the surrounding `Form`.
        #[props(default, into)]
        name: crate::components::form::FieldName<(V, V)>,
        /// Fires per pair; `Start`/`End` bracket a drag.
        #[props(default)]
        oninput: Option<EventHandler<SliderChangeEvent<(V, V)>>>,
        /// Rules over the pair, shown on blur or submit.
        #[props(default, into)]
        validate: crate::components::form::Validators<(V, V)>,
        /// A bubble over the value a hovering mouse or pen points at, before a press
        /// moves the nearer thumb there. Escape hides it until the pointer leaves. It follows
        /// the pointer and goes with it, so it is not hoverable (WCAG 1.4.13).
        #[props(default)]
        preview_on_hover: bool,
    }
}

/// Two thumbs on one track that never cross, on [`Slider`](super::Slider)'s engine.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{RangeSlider, SliderChangeEvent};
/// # fn app() -> Element {
/// let mut price = use_signal(|| (20.0, 80.0));
/// rsx! {
///     RangeSlider {
///         label: "Price",
///         value: price(),
///         min_range: 5.0,
///         oninput: move |event: SliderChangeEvent<(f64, f64)>| price.set(event.value()),
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/range-slider>
#[component]
pub fn RangeSlider<V: SliderValue>(props: RangeSliderProps<V>) -> Element {
    let theme = use_theme();
    let labels = use_localization().slider;
    let scale = Scale::of::<V>(props.min.as_ref(), props.max.as_ref(), props.step);
    let (min, max, step) = scale.bounds();
    let bound = use_bound(&props.name, props.oninput.is_some());
    let disabled = bound.disabled(props.disabled);
    let value = bound
        .value()
        .or_else(|| props.value.clone())
        .unwrap_or_else(|| (V::at(min), V::at(max)));

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
        let event = event.map(|value| (V::at(value.thumb(0)), V::at(value.thumb(1))));
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

    // The thumbs carry `role="slider"`, and `for` names only a labelable
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
        .required_in_name(labels.required)
        .disabled(disabled)
        .size(props.size.copied_or(theme.slider.size))
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&props.states)
        .attributes(&props.attributes)
        .prepare();
    crate::components::common::use_name_warning(
        field.label_id().is_some()
            || props.aria_label.is_some()
            || props.aria_label_from.is_some()
            || props.aria_label_to.is_some(),
        "RangeSlider: no `label` or `aria_label`, so the thumbs are announced as just \"Minimum\" and \"Maximum\".",
    );

    let above = !props.label.is_none() || !props.description.is_none();
    let below = !props.helper.is_none()
        || props
            .status
            .as_ref()
            .and_then(FieldStatus::message)
            .is_some();
    let control_sx = control_spacing(above, below);

    // Unlabelled, each thumb's own name carries the group's and says required,
    // as the label's id would.
    let unlabelled = field.label_id().is_none();
    let group_label = props.aria_label.filter(|_| unlabelled);
    let thumb_name = |name: String| {
        let name = match &group_label {
            Some(group) => format!("{group} {name}"),
            None => name,
        };
        match required && unlabelled {
            true => format!("{name} {}", labels.required),
            false => name,
        }
    };

    let (from, to) = &value;
    let control = rsx! {
        SliderCore {
            value: SliderCoreValue::Range { from: from.position(), to: to.position() },
            min,
            max,
            step,
            min_range: props.min_range.map_or(0.0, SliderStep::as_f64),
            marks,
            // The thumbs' `aria-valuetext` names a discrete value, so the captions would say it twice.
            captions_hidden: V::options().is_some(),
            sx: control_sx,
            attributes: props.attributes,
            size: props.size,
            color: props.color,
            disabled: Some(bound.disabled(props.disabled)),
            readonly: props.readonly.unwrap_or(false),
            label: labelled.then_some(label),
            aria_label: thumb_name(props.aria_label_from.unwrap_or_else(|| labels.minimum.to_string())),
            aria_label_to: thumb_name(props.aria_label_to.unwrap_or_else(|| labels.maximum.to_string())),
            labelledby: field.label_id(),
            group_label: group_label.clone(),
            describedby: field.describedby(),
            invalid: field.invalid(),
            name: bound.name().map(str::to_string),
            // The same `use_callback` every render, so the core's props can
            // compare equal.
            oninput: (props.oninput.is_some() || bound.is_bound()).then_some(emit),
            preview_on_hover: props.preview_on_hover,
        }
    };

    field.render(control)
}
