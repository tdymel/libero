//! What the two skins share: the scale that maps a caller's `SliderValue`
//! onto the `f64` engine, and the spacing the field's stacked layout needs.

use super::slider_value::{SliderMark, SliderStep, SliderValue};
use super::value::sane_bounds;
use crate::{
    components::common::Input,
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

    /// One mark per option in range, captioned by `name` - the same namer the
    /// bubble uses, so a translated slider is translated everywhere.
    /// Empty for a continuous scale, which has nothing to enumerate.
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

/// The field's own gap is measured to the control's *box*, and a slider fills
/// that box edge to edge: the thumb overhangs the track at the top, and the
/// `marks-labeled` reserve ends at the caption's baseline box at the bottom.
/// So a label would land on the thumb and a helper on the mark captions
/// unless the control pushes them off itself.
pub(super) fn control_spacing(above: bool, below: bool) -> Input<Sx> {
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
