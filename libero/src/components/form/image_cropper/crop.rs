//! The crop box's geometry, in fractions of the image so it needs no natural
//! pixel size: WebView cannot read one.

/// A crop, as fractions 0-1 of the image's width and height from its top-left.
///
/// ```rust
/// # use libero::components::CropRect;
/// let crop = CropRect { x: 0.25, y: 0.5, width: 0.5, height: 0.5 };
/// let pixels = crop.to_pixels(800, 600);
/// assert_eq!((pixels.x, pixels.y, pixels.width, pixels.height), (200, 300, 400, 300));
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CropRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// A [`CropRect`] in the source image's pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PixelRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// The mask over the image outside the crop. The crop itself stays a rectangle.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CropShape {
    #[default]
    Rect,
    /// An ellipse inscribed in the box: a circle with a square `aspect`.
    Circle,
}

/// How a `FileField` with `crop` crops a picked image.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CropOptions {
    /// Locks the crop's width over height: `1.0` is square.
    pub aspect: Option<f64>,
    pub shape: CropShape,
    /// Scales the cropped image down until its longer side is at most this, in px.
    pub max_size: Option<u32>,
}

impl Default for CropRect {
    fn default() -> Self {
        Self::FULL
    }
}

/// Which part of the box a drag or a key moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Grip {
    Move,
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

impl Grip {
    /// The handles, corners first: the corners are the keyboard's tab stops.
    pub(super) const HANDLES: [Grip; 8] = [
        Grip::NorthWest,
        Grip::NorthEast,
        Grip::SouthEast,
        Grip::SouthWest,
        Grip::North,
        Grip::East,
        Grip::South,
        Grip::West,
    ];

    /// How the grip moves the box's left/right and top/bottom edge: -1 the
    /// near edge, 1 the far one, 0 neither.
    fn sides(self) -> (i8, i8) {
        match self {
            Grip::Move => (0, 0),
            Grip::North => (0, -1),
            Grip::South => (0, 1),
            Grip::East => (1, 0),
            Grip::West => (-1, 0),
            Grip::NorthEast => (1, -1),
            Grip::NorthWest => (-1, -1),
            Grip::SouthEast => (1, 1),
            Grip::SouthWest => (-1, 1),
        }
    }

    pub(super) fn slot(self) -> &'static str {
        match self {
            Grip::Move => "box",
            Grip::North => "n",
            Grip::South => "s",
            Grip::East => "e",
            Grip::West => "w",
            Grip::NorthEast => "ne",
            Grip::NorthWest => "nw",
            Grip::SouthEast => "se",
            Grip::SouthWest => "sw",
        }
    }
}

impl CropRect {
    /// The whole image.
    pub const FULL: Self = Self {
        x: 0.0,
        y: 0.0,
        width: 1.0,
        height: 1.0,
    };

    /// The crop in an image of `width` x `height` pixels, kept inside it.
    pub fn to_pixels(self, width: u32, height: u32) -> PixelRect {
        let scale =
            |fraction: f64, size: u32| (fraction.clamp(0.0, 1.0) * size as f64).round() as u32;
        let x = scale(self.x, width).min(width);
        let y = scale(self.y, height).min(height);
        PixelRect {
            x,
            y,
            width: scale(self.width, width).min(width - x),
            height: scale(self.height, height).min(height - y),
        }
    }

    fn right(self) -> f64 {
        self.x + self.width
    }

    fn bottom(self) -> f64 {
        self.y + self.height
    }

    /// The largest centred box at `ratio`, the box's width over its height in
    /// fractions; the whole image without one.
    pub(super) fn largest(ratio: Option<f64>) -> Self {
        let Some(ratio) = ratio.filter(|ratio| ratio.is_finite() && *ratio > 0.0) else {
            return Self::FULL;
        };
        let (width, height) = match ratio >= 1.0 {
            true => (1.0, 1.0 / ratio),
            false => (ratio, 1.0),
        };
        Self {
            x: (1.0 - width) / 2.0,
            y: (1.0 - height) / 2.0,
            width,
            height,
        }
    }

    /// Where an unset box starts: 80% of [`Self::largest`], centred, in whole
    /// percent, so a drag can move it at once (Cropper.js's `autoCropArea`).
    pub(super) fn starting(ratio: Option<f64>) -> Self {
        let percent = |fraction: f64| (fraction * 100.0).round() / 100.0;
        let largest = Self::largest(ratio);
        let (width, height) = (percent(largest.width * 0.8), percent(largest.height * 0.8));
        Self {
            x: percent((1.0 - width) / 2.0),
            y: percent((1.0 - height) / 2.0),
            width,
            height,
        }
    }

    /// The box shifted by `dx`, `dy`, stopped at the image's edges.
    pub(super) fn moved(self, dx: f64, dy: f64) -> Self {
        Self {
            x: (self.x + dx).clamp(0.0, (1.0 - self.width).max(0.0)),
            y: (self.y + dy).clamp(0.0, (1.0 - self.height).max(0.0)),
            ..self
        }
    }

    /// The box `factor` times as large around its centre, keeping its shape, at
    /// least `min` on each side and shifted back inside the image.
    pub(super) fn scaled(self, factor: f64, min: f64) -> Self {
        if !factor.is_finite() || self.width <= 0.0 || self.height <= 0.0 {
            return self;
        }
        let most = (1.0 / self.width).min(1.0 / self.height);
        let least = (min / self.width).max(min / self.height).min(most);
        let factor = factor.clamp(least, most);
        let (width, height) = (self.width * factor, self.height * factor);
        Self {
            x: self.x + (self.width - width) / 2.0,
            y: self.y + (self.height - height) / 2.0,
            width,
            height,
        }
        .moved(0.0, 0.0)
    }

    /// The box after `grip` moved by `dx`, `dy`, inside the image and at least
    /// `min` on each side. `ratio` locks width over height, in fractions.
    pub(super) fn resized(
        self,
        grip: Grip,
        dx: f64,
        dy: f64,
        ratio: Option<f64>,
        min: f64,
    ) -> Self {
        if grip == Grip::Move {
            return self.moved(dx, dy);
        }
        match ratio.filter(|ratio| ratio.is_finite() && *ratio > 0.0) {
            Some(ratio) => self.resized_locked(grip, dx, dy, ratio, min),
            None => self.resized_free(grip, dx, dy, min),
        }
    }

    fn resized_free(self, grip: Grip, dx: f64, dy: f64, min: f64) -> Self {
        let (sx, sy) = grip.sides();
        let (mut left, mut top, mut right, mut bottom) =
            (self.x, self.y, self.right(), self.bottom());
        match sx {
            -1 => left = (left + dx).clamp(0.0, (right - min).max(0.0)),
            1 => right = (right + dx).clamp((left + min).min(1.0), 1.0),
            _ => {}
        }
        match sy {
            -1 => top = (top + dy).clamp(0.0, (bottom - min).max(0.0)),
            1 => bottom = (bottom + dy).clamp((top + min).min(1.0), 1.0),
            _ => {}
        }
        Self {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
        }
    }

    /// Grows from the opposite corner or edge; an edge grip keeps the box
    /// centred across it. A corner follows whichever axis moved more.
    fn resized_locked(self, grip: Grip, dx: f64, dy: f64, ratio: f64, min: f64) -> Self {
        let (sx, sy) = grip.sides();
        let by_width = self.width + dx * sx as f64;
        let by_height = (self.height + dy * sy as f64) * ratio;
        let wanted = match (sx, sy) {
            (0, _) => by_height,
            (_, 0) => by_width,
            _ if (by_width - self.width).abs() >= (by_height - self.width).abs() => by_width,
            _ => by_height,
        };

        // The anchor: the far edge for a moving side, the centre otherwise.
        let anchor_x = match sx {
            -1 => self.right(),
            1 => self.x,
            _ => self.x + self.width / 2.0,
        };
        let anchor_y = match sy {
            -1 => self.bottom(),
            1 => self.y,
            _ => self.y + self.height / 2.0,
        };
        let room = |anchor: f64, side: i8| match side {
            -1 => anchor,
            1 => 1.0 - anchor,
            _ => 2.0 * anchor.min(1.0 - anchor),
        };
        let max_width = room(anchor_x, sx).min(room(anchor_y, sy) * ratio);
        let min_width = min.max(min * ratio);
        if min_width > max_width {
            return self;
        }
        let width = wanted.clamp(min_width, max_width);
        let height = width / ratio;
        let place = |anchor: f64, side: i8, size: f64| match side {
            -1 => anchor - size,
            1 => anchor,
            _ => anchor - size / 2.0,
        };
        Self {
            x: place(anchor_x, sx, width),
            y: place(anchor_y, sy, height),
            width,
            height,
        }
    }
}
