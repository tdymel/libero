//! What a `Slider` can be dragged over: a continuous range, or a finite
//! ordered set. The component itself is `f64`-only - this is the layer that
//! maps a caller's own type onto it.
//!
//! `f64` is the only impl libero ships: a slider's values are the caller's
//! domain, not ours. An ordered enum of theirs derives the rest.

/// How far one arrow press or one `step` moves. A count over the options in
/// a discrete set, a distance in a continuous one - so `step: 1.5` on an
/// enum-valued slider is a type error, not a runtime surprise.
pub trait SliderStep: Copy + PartialEq + 'static {
    fn as_f64(self) -> f64;
}

impl SliderStep for f64 {
    fn as_f64(self) -> f64 {
        self
    }
}

impl SliderStep for usize {
    fn as_f64(self) -> f64 {
        self as f64
    }
}

/// A value a `Slider` renders and emits.
///
/// A discrete type lists its `options` and gets its range, step grid, marks
/// and captions derived from them; the default `position`/`at` walk that
/// list. A continuous one returns `None` and must override both.
///
/// `#[derive(SliderValue)]` writes the discrete impl for an ordered enum.
pub trait SliderValue: Clone + PartialEq + 'static {
    type Step: SliderStep;

    /// Every value, in order. `None` is a continuous range.
    fn options() -> Option<&'static [Self]>
    where
        Self: Sized;

    /// Bubble text, mark caption and `aria-valuetext`.
    fn label(&self) -> String;

    /// Where this value sits on the track's scale - its index, discretely.
    fn position(&self) -> f64
    where
        Self: Sized,
    {
        Self::options()
            .and_then(|options| options.iter().position(|option| option == self))
            .map_or(0.0, |index| index as f64)
    }

    /// The inverse of `position`, clamped to the option list.
    fn at(position: f64) -> Self
    where
        Self: Sized,
    {
        let options = Self::options().expect("a continuous `SliderValue` must override `at`");
        let last = options.len().saturating_sub(1);
        options[(position.round().max(0.0) as usize).min(last)].clone()
    }
}

impl SliderValue for f64 {
    type Step = f64;

    fn options() -> Option<&'static [Self]> {
        None
    }

    fn label(&self) -> String {
        self.to_string()
    }

    fn position(&self) -> f64 {
        *self
    }

    fn at(position: f64) -> Self {
        position
    }
}

/// A tick on the track, with an optional caption under it. A discrete
/// `Slider` derives one per option; an explicit `marks` replaces that.
#[derive(Clone, Debug, PartialEq)]
pub struct SliderMark<V = f64> {
    pub value: V,
    pub label: Option<String>,
}

impl<V> SliderMark<V> {
    pub fn new(value: V) -> Self {
        Self { value, label: None }
    }

    pub fn labeled(value: V, label: impl Into<String>) -> Self {
        Self {
            value,
            label: Some(label.into()),
        }
    }
}

impl From<f64> for SliderMark {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

/// Where the value went, and how far along the interaction is. `Start` and
/// `End` bracket one pointer drag; a key press emits `Change` then `End`,
/// because it settles on its value at once. So `End` always means a value the
/// user is done choosing, whichever way they chose it, and committing on it
/// is enough.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SliderChangeEvent<V = f64> {
    Start(V),
    Change(V),
    End(V),
}

impl<V> SliderChangeEvent<V> {
    pub fn value(self) -> V {
        match self {
            Self::Start(value) | Self::Change(value) | Self::End(value) => value,
        }
    }

    /// Same phase, mapped value - how the `f64` core's events become the
    /// caller's own type.
    pub(in crate::components::form) fn map<W>(
        self,
        map: impl FnOnce(V) -> W,
    ) -> SliderChangeEvent<W> {
        match self {
            Self::Start(value) => SliderChangeEvent::Start(map(value)),
            Self::Change(value) => SliderChangeEvent::Change(map(value)),
            Self::End(value) => SliderChangeEvent::End(map(value)),
        }
    }
}
