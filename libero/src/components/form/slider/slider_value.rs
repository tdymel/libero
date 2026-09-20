//! What a `Slider` can be dragged over: a continuous `f64` range, or an
//! ordered enum that derives `SliderValue`, mapped onto the `f64` core.

/// How far one step moves: a count of options, or a distance, so `step: 1.5`
/// on an enum slider is a type error.
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

/// A value a `Slider` renders and emits. A discrete type lists its `options`
/// (`#[derive(SliderValue)]`); a continuous one overrides `position`/`at`.
pub trait SliderValue: Clone + PartialEq + 'static {
    type Step: SliderStep;

    /// Every value, in order. `None` is a continuous range.
    fn options() -> Option<&'static [Self]>
    where
        Self: Sized;

    /// Bubble text, mark caption and `aria-valuetext`. A slider's `format`
    /// prop overrides it, which is where a translation goes.
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

/// Where the value went and how far along the interaction is. `Start`/`End`
/// bracket a drag; a key press emits `Change` then `End`, so commit on `End`.
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
