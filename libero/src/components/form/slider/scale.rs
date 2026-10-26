//! What the two skins share: the scale that maps a caller's `SliderValue`
//! onto the `f64` engine, and the spacing the field's stacked layout needs.

use dioxus::prelude::*;

use super::slider_value::{SliderChangeEvent, SliderMark, SliderStep, SliderValue};
use super::value::{SliderCoreValue, sane_bounds};
use crate::{
    components::{
        common::Input,
        form::{Bound, FieldStatus},
    },
    sx::{Sx, sx},
    theme::{Size, SizeCss},
};

/// The two ways a `SliderValue` lays out on the track, both of which collapse
/// to the core's `(min, max, step)` - the discrete one in index space.
pub(super) enum Scale {
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
    pub(super) fn of<V: SliderValue>(
        min: Option<&V>,
        max: Option<&V>,
        step: Option<V::Step>,
    ) -> Self {
        let Some(options) = V::options() else {
            // Sanitised here rather than in the core, so the skins' marks and
            // their `V::at(min)` fallback see the same repaired range.
            let (min, max) =
                sane_bounds(min.map_or(0.0, V::position), max.map_or(100.0, V::position));
            return Self::Continuous {
                min,
                max,
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
    pub(super) fn bounds(&self) -> (f64, f64, f64) {
        match *self {
            Self::Continuous { min, max, step } => (min, max, step),
            Self::Discrete {
                first,
                last,
                stride,
            } => (first as f64, last as f64, stride as f64),
        }
    }

    /// One mark per option in range, captioned by the bubble's namer. Empty on
    /// a continuous scale.
    pub(super) fn derived_marks<V: SliderValue>(
        &self,
        name: impl Fn(&V) -> String,
    ) -> Vec<SliderMark> {
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

/// The marks the track shows, and the namer of a thumb's bubble and `aria-valuetext`:
/// `None` for a continuous value without `format`, which reads as a bare number.
pub(super) fn use_naming<V: SliderValue>(
    scale: &Scale,
    marks: &[SliderMark<V>],
    format: Option<Callback<V, String>>,
) -> (Vec<SliderMark>, Option<Callback<f64, String>>) {
    // A discrete value names itself unless the caller says how - the hook an i18n'd slider hangs on.
    let name = move |value: &V| match &format {
        Some(label) => label.call(value.clone()),
        None => value.label(),
    };
    let marks = match marks.is_empty() {
        true => scale.derived_marks::<V>(name),
        false => marks
            .iter()
            .map(|mark| SliderMark {
                value: mark.value.position(),
                label: mark.label.clone(),
            })
            .collect(),
    };
    let label = use_callback(move |position: f64| name(&V::at(position)));
    let labelled = format.is_some() || V::options().is_some();
    (marks, labelled.then_some(label))
}

/// The core's events as the skin's value: to `oninput`, else into the bound form's value.
pub(super) fn use_emit<T: Clone + 'static>(
    oninput: Option<EventHandler<SliderChangeEvent<T>>>,
    bound: &Bound<T>,
    to: impl Fn(SliderCoreValue) -> T + 'static,
) -> Callback<SliderChangeEvent<SliderCoreValue>> {
    let setter = bound.setter();
    use_callback(move |event: SliderChangeEvent<SliderCoreValue>| {
        let event = event.map(&to);
        match (&oninput, &setter) {
            (Some(oninput), _) => oninput.call(event),
            // A bound slider keeps only the values: `Start`/`End` carry nothing a form's value holds.
            (None, Some(setter)) => {
                if let SliderChangeEvent::Change(value) = event {
                    setter.set(value);
                }
            }
            (None, None) => {}
        }
    })
}

/// Keeps the label off the overhanging thumb and the helper off the mark
/// captions: the field's gap only reaches the control's box.
pub(super) fn control_spacing(
    above: bool,
    helper: bool,
    status: Option<&FieldStatus>,
) -> Input<Sx> {
    let below = helper || status.and_then(FieldStatus::message).is_some();
    if !above && !below {
        return Input::default();
    }
    let mut spacing = sx();
    if above {
        spacing = spacing.margin_top(SizeCss::SPACING.value(Size::Sm));
    }
    if below {
        spacing = spacing.margin_bottom(SizeCss::SPACING.value(Size::Md));
    }
    spacing.into()
}
