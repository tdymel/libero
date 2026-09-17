use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, Size, SizeCss, Sizes, Variant};

pub const BADGE_FONT_SIZE: SizeCss = SizeCss::new("--lsx-badge-font-size-");
pub const BADGE_HEIGHT: SizeCss = SizeCss::new("--lsx-badge-height-");
pub const BADGE_PADDING_X: SizeCss = SizeCss::new("--lsx-badge-padding-x-");
pub const BADGE_RADII: SizeCss = SizeCss::new("--lsx-badge-radius-");

/// The active step's values, republished unsuffixed by
/// [`BadgeDefaults::size_sx`] - the `ColorSwatch` pattern. `circle` needs the
/// current height for its `min-width` and has no way to know which size token
/// is on.
pub const BADGE_FONT: CssVar = CssVar::new("--lsx-badge-font");
pub const BADGE_BOX: CssVar = CssVar::new("--lsx-badge-box");
pub const BADGE_PAD_X: CssVar = CssVar::new("--lsx-badge-pad-x");

pub const BADGE_RADIUS: CssVar = CssVar::new("--lsx-badge-radius");
pub const BADGE_TEXT_TRANSFORM: CssVar = CssVar::new("--lsx-badge-text-transform");
pub const BADGE_LETTER_SPACING: CssVar = CssVar::new("--lsx-badge-letter-spacing");
pub const BADGE_FONT_WEIGHT: CssVar = CssVar::new("--lsx-badge-font-weight");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BadgeSizeLevel {
    pub font_size: &'static str,
    pub height: &'static str,
    pub padding_x: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BadgeDefaults {
    /// The chrome a badge takes when a call site names none.
    pub variant: Variant,
    pub size: Size,
    /// The step of [`Self::radii`] a badge takes when a call site names none.
    pub radius: Size,
    /// Uppercase is most of what separates a badge from a chip at a glance.
    /// A project turns it off here, once.
    pub text_transform: &'static str,
    /// Opens up the uppercase, which sets tighter than lowercase.
    pub letter_spacing: &'static str,
    pub font_weight: &'static str,
    pub sizes: Sizes<BadgeSizeLevel>,
    /// The badge's own radius scale, not the global one: a badge is at most
    /// 2.375rem tall, so every global step past `sm` is already a pill on most of
    /// its sizes. `xxl` is the pill itself.
    pub radii: Sizes<&'static str>,
}

impl BadgeDefaults {
    pub const DEFAULT: Self = Self {
        variant: Variant::Filled,
        size: Size::Md,
        // A pill at every height, which no fixed length below it gives.
        radius: Size::Xxl,
        text_transform: "uppercase",
        letter_spacing: "0.25px",
        font_weight: "700",
        // `xs`..`xl` are Mantine's own numbers; `xxl` continues the ramp,
        // since our scale has a sixth step and theirs does not. All rem, so
        // the box grows with a reader's own text size instead of clipping
        // the label at 200% (todo 741).
        sizes: Sizes::new(
            BadgeSizeLevel {
                font_size: "0.5625rem",
                height: "1rem",
                padding_x: "0.375rem",
            },
            BadgeSizeLevel {
                font_size: "0.625rem",
                height: "1.125rem",
                padding_x: "0.5rem",
            },
            BadgeSizeLevel {
                font_size: "0.6875rem",
                height: "1.25rem",
                padding_x: "0.625rem",
            },
            BadgeSizeLevel {
                font_size: "0.8125rem",
                height: "1.625rem",
                padding_x: "0.75rem",
            },
            BadgeSizeLevel {
                font_size: "1rem",
                height: "2rem",
                padding_x: "1rem",
            },
            BadgeSizeLevel {
                font_size: "1.125rem",
                height: "2.375rem",
                padding_x: "1.25rem",
            },
        ),
        radii: Sizes::new("2px", "4px", "6px", "8px", "12px", "9999px"),
    };

    pub fn size_sx(size: Size) -> Sx {
        sx().var(BADGE_FONT, BADGE_FONT_SIZE.value(size))
            .var(BADGE_BOX, BADGE_HEIGHT.value(size))
            .var(BADGE_PAD_X, BADGE_PADDING_X.value(size))
    }

    pub fn theme_vars() -> Sx {
        sx().font_size(BADGE_FONT.value())
            .height(BADGE_BOX.value())
            // The optical centre, minus the 1px border on each edge. It also
            // means the label does not sit on an inherited `line-height` the
            // component never states.
            .line_height(format!("calc({} - 2px)", BADGE_BOX.value()))
            .padding_left(BADGE_PAD_X.value())
            .padding_right(BADGE_PAD_X.value())
            .border_radius(BADGE_RADIUS.overridable())
            .text_transform(BADGE_TEXT_TRANSFORM.value())
            .letter_spacing(BADGE_LETTER_SPACING.value())
            .font_weight(BADGE_FONT_WEIGHT.value())
            .per_size(Self::size_sx)
    }
}

impl ToCssDeclarations for BadgeDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = Vec::new();
        for size in Size::ALL {
            let level = self.sizes.get(size);
            declarations.push(BADGE_FONT_SIZE.declare(size, level.font_size));
            declarations.push(BADGE_HEIGHT.declare(size, level.height));
            declarations.push(BADGE_PADDING_X.declare(size, level.padding_x));
        }
        for size in Size::ALL {
            declarations.push(BADGE_RADII.declare(size, self.radii.get(size)));
        }
        declarations.push(BADGE_RADIUS.declare(BADGE_RADII.value(self.radius)));
        declarations.push(BADGE_TEXT_TRANSFORM.declare(self.text_transform));
        declarations.push(BADGE_LETTER_SPACING.declare(self.letter_spacing));
        declarations.push(BADGE_FONT_WEIGHT.declare(self.font_weight));
        declarations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A px box under `overflow: hidden` clips a rem label at 200% text size
    /// (todo 741).
    #[test]
    fn every_box_length_follows_the_text_size() {
        for size in Size::ALL {
            let level = BadgeDefaults::DEFAULT.sizes.get(size);
            for length in [level.font_size, level.height, level.padding_x] {
                assert!(length.ends_with("rem"), "{size:?}: {length}");
            }
        }
    }
}
