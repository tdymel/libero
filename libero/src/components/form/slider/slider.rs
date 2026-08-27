use dioxus::prelude::*;

use super::core::SliderCore;
use super::slider_value::{SliderChangeEvent, SliderMark, SliderStep, SliderValue};
use crate::{
    components::{FieldStatus, Input, common::field_props, form::use_field},
    sx::{Sx, ThemeAwareValue, sx},
    theme::{Size, SizeCss},
};

/// The two ways a `SliderValue` lays out on the track, both of which collapse
/// to the core's `(min, max, step)` - the discrete one in index space.
enum Scale {
    Continuous {
        min: f64,
        max: f64,
        step: f64,
    },
    /// Indices into `V::options()`, and how many of them one step covers.
    Discrete {
        first: usize,
        last: usize,
        stride: usize,
    },
}

impl Scale {
    fn of<V: SliderValue>(min: Option<&V>, max: Option<&V>, step: Option<V::Step>) -> Self {
        let Some(options) = V::options() else {
            return Self::Continuous {
                min: min.map_or(0.0, V::position),
                max: max.map_or(100.0, V::position),
                step: step.map_or(1.0, SliderStep::as_f64),
            };
        };

        let last_option = options.len().saturating_sub(1);
        let index = |value: &V| (value.position() as usize).min(last_option);
        let first = min.map_or(0, index);
        Self::Discrete {
            first,
            last: max.map_or(last_option, index).max(first),
            // A zero stride would put every option on the same spot.
            stride: step.map_or(1, |step| (step.as_f64() as usize).max(1)),
        }
    }

    /// `(min, max, step)` as the core wants them.
    fn bounds(&self) -> (f64, f64, f64) {
        match *self {
            Self::Continuous { min, max, step } => (min, max, step),
            Self::Discrete {
                first,
                last,
                stride,
            } => (first as f64, last as f64, stride as f64),
        }
    }

    /// One mark per option in range, captioned by `name` - the same namer the
    /// bubble uses, so a translated slider is translated everywhere.
    /// Empty for a continuous scale, which has nothing to enumerate.
    fn derived_marks<V: SliderValue>(&self, name: impl Fn(&V) -> String) -> Vec<SliderMark> {
        let Self::Discrete {
            first,
            last,
            stride,
        } = *self
        else {
            return Vec::new();
        };
        let options = V::options().unwrap_or_default();
        (first..=last)
            .step_by(stride)
            .map(|index| SliderMark::labeled(index as f64, name(&options[index])))
            .collect()
    }
}

field_props! {
    pub struct SliderProps<V: SliderValue> {
        /// Strictly controlled - pair it with `oninput`.
        value: V,
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
        /// continuously, and to `SliderValue::label` discretely.
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
        #[props(default)]
        name: Option<String>,
        /// Fires per value: a drag is the DOM's `input` event, not its
        /// `change`. `Start`/`End` bracket a drag, `Change` carries every new
        /// value.
        #[props(default)]
        oninput: Option<EventHandler<SliderChangeEvent<V>>>,
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
    let scale = Scale::of::<V>(props.min.as_ref(), props.max.as_ref(), props.step);
    let (min, max, step) = scale.bounds();

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
    let emit = use_callback(move |event: SliderChangeEvent| {
        if let Some(oninput) = &oninput {
            oninput.call(event.map(V::at));
        }
    });

    let disabled = props.disabled.unwrap_or(false);
    let required = props.required.unwrap_or(false);

    // The thumb carries `role="slider"`, and `for` names only a labelable
    // element - so the field hands its ids over rather than applying them.
    let field = use_field()
        .labelled_by()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .required(required)
        .disabled(disabled)
        .size(props.size.copied_or(Size::Md))
        .radius(props.radius.copied_or(Size::Xl))
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .attributes(&props.attributes)
        .prepare();

    // The field's own 4px gap is measured to the control's box, and the
    // slider fills that box edge to edge: the thumb overhangs the track at the
    // top, and the `marks-labeled` reserve ends exactly at the caption's
    // baseline box at the bottom. So a label lands on the thumb and the helper
    // lands on the mark captions unless the control pushes them off itself.
    let above = !props.label.is_none() || !props.description.is_none();
    let below = !props.helper.is_none()
        || props
            .status
            .as_ref()
            .and_then(FieldStatus::message)
            .is_some();
    let mut spacing = sx();
    if above {
        spacing = spacing.margin_top(SizeCss::SPACING.value(Size::Sm));
    }
    if below {
        spacing = spacing.margin_bottom(SizeCss::SPACING.value(Size::Md));
    }
    let control_sx: Input<Sx> = match above || below {
        true => spacing.into(),
        false => Input::default(),
    };

    let control = rsx! {
        SliderCore {
            value: props.value.position(),
            min,
            max,
            step,
            marks,
            sx: control_sx,
            attributes: props.attributes,
            size: props.size,
            radius: props.radius,
            color: props.color,
            disabled: props.disabled,
            label: labelled.then_some(label),
            aria_label: props.aria_label,
            labelledby: field.label_id(),
            describedby: field.describedby(),
            invalid: field.invalid(),
            required,
            name: props.name,
            oninput: props.oninput.is_some().then(|| EventHandler::new(move |event| emit.call(event))),
        }
    };

    field.render(control)
}
