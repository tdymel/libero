use dioxus::prelude::*;

use super::core::SliderCore;
use super::slider_value::{SliderChangeEvent, SliderMark, SliderStep, SliderValue};
use crate::{
    components::{ClassList, Input, States},
    sx::{Sx, ThemeAwareValue},
    theme::Size,
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

    /// One mark per option in range, captioned with the option's own label.
    /// Empty for a continuous scale, which has nothing to enumerate.
    fn derived_marks<V: SliderValue>(&self) -> Vec<SliderMark> {
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
            .map(|index| SliderMark::labeled(index as f64, options[index].label()))
            .collect()
    }
}

// Hand-written rather than `base_props!`, which is not generic - as
// `TreeProps` is.
#[derive(Props, Clone, PartialEq)]
pub struct SliderProps<V: SliderValue> {
    /// Strictly controlled - pair it with `on_change`.
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
    size: Input<Size>,
    /// Track corner radius, independent of `size`.
    #[props(default, into)]
    radius: Input<Size>,
    #[props(default, into)]
    color: Input<ThemeAwareValue>,
    #[props(default)]
    disabled: Option<bool>,
    /// Formats the bubble shown on hover, drag and keyboard focus, and
    /// sets the thumb's `aria-valuetext`. Defaults to the bare value
    /// continuously, and to `SliderValue::label` discretely.
    #[props(default)]
    label: Option<Callback<V, String>>,
    /// Ticks on the track; a labeled one gets a caption below it.
    /// Replaces the marks a discrete scale derives.
    #[props(default)]
    marks: Vec<SliderMark<V>>,
    /// Names the thumb, which is the `role="slider"` element - an
    /// `aria_label` in `attributes` would land on the root instead.
    #[props(default)]
    aria_label: Option<String>,
    /// Emits a hidden input of that name, so the value posts with a form.
    #[props(default)]
    name: Option<String>,
    /// `Start`/`End` bracket a drag, `Change` carries every new value.
    #[props(default)]
    on_change: Option<EventHandler<SliderChangeEvent<V>>>,
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default, into)]
    class: Input<ClassList>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
}

/// A draggable value along a track. Controlled: it renders `value` and asks
/// for a new one through `on_change`.
///
/// `V` is inferred from `value`. A [`SliderValue`] that lists its options -
/// an ordered enum - makes the slider discrete: the range, the step grid, one
/// mark per option and every caption come from the list, and `min`/`max` are
/// written in that type rather than as indices.
#[component]
pub fn Slider<V: SliderValue>(props: SliderProps<V>) -> Element {
    let scale = Scale::of::<V>(props.min.as_ref(), props.max.as_ref(), props.step);
    let (min, max, step) = scale.bounds();

    let marks = match props.marks.is_empty() {
        true => scale.derived_marks::<V>(),
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
    let labelled = props.label.is_some() || V::options().is_some();
    let label = props.label;
    let label = use_callback(move |position: f64| {
        let value = V::at(position);
        match &label {
            Some(label) => label.call(value),
            None => value.label(),
        }
    });

    let on_change = props.on_change;
    let emit = use_callback(move |event: SliderChangeEvent| {
        if let Some(on_change) = &on_change {
            on_change.call(event.map(V::at));
        }
    });

    rsx! {
        SliderCore {
            value: props.value.position(),
            min,
            max,
            step,
            marks,
            attributes: props.attributes,
            class: props.class,
            sx: props.sx,
            states: props.states,
            size: props.size,
            radius: props.radius,
            color: props.color,
            disabled: props.disabled,
            label: labelled.then_some(label),
            aria_label: props.aria_label,
            name: props.name,
            on_change: props.on_change.is_some().then(|| EventHandler::new(move |event| emit.call(event))),
        }
    }
}
