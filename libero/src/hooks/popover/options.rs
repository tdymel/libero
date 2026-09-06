// `Side` and `Align` are themed (`TooltipDefaults::side`), so they live in
// `theme/` and are re-exported here.
pub use crate::theme::{Align, Side};

/// What the floating box's width follows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PopoverWidth {
    /// Its own content.
    #[default]
    Auto,
    /// Exactly the anchor's width. A dropdown wants this: portaling removes
    /// the positioned parent a `width: 100%` used to resolve against.
    Match,
    /// At least the anchor's width, growing with its content.
    Min,
}

/// The side and align a box actually landed on - the preferred pair unless
/// flipping moved it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Placement {
    pub side: Side,
    pub align: Align,
}

/// How a popover is placed. `Default` is a dropdown: below its anchor, left
/// edges aligned, flipping and shifting to stay on screen, with the theme's
/// gap and collision padding.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PopoverOptions {
    /// The preferred side. Flipping may override it.
    pub side: Side,
    pub align: Align,
    /// Pixels between the anchor's edge and the box.
    pub gap: f64,
    /// How close to a viewport edge the box may come before flipping or
    /// shifting. The box's width is capped at the viewport less this at both
    /// edges.
    pub padding: f64,
    pub flip: bool,
    pub shift: bool,
    pub width: PopoverWidth,
    /// Not a placement input: the box is measured again whenever it changes.
    /// For an anchor that resizes while the box is open - a multi-select
    /// whose trigger grows a chip on every pick. Nothing else re-measures.
    pub remeasure: u64,
}

impl PopoverOptions {
    /// The theme's `gap` and `padding`, which is why this is not `Default`:
    /// reading a theme needs the running provider.
    pub fn new(gap: f64, padding: f64) -> Self {
        Self {
            side: Side::default(),
            align: Align::default(),
            gap,
            padding,
            flip: true,
            shift: true,
            width: PopoverWidth::default(),
            remeasure: 0,
        }
    }

    pub fn side(mut self, side: Side) -> Self {
        self.side = side;
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn gap(mut self, gap: f64) -> Self {
        self.gap = gap;
        self
    }

    pub fn padding(mut self, padding: f64) -> Self {
        self.padding = padding;
        self
    }

    pub fn flip(mut self, flip: bool) -> Self {
        self.flip = flip;
        self
    }

    pub fn shift(mut self, shift: bool) -> Self {
        self.shift = shift;
        self
    }

    pub fn width(mut self, width: PopoverWidth) -> Self {
        self.width = width;
        self
    }

    pub fn remeasure(mut self, key: u64) -> Self {
        self.remeasure = key;
        self
    }
}
