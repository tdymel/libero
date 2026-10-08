use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{Color, CssVar, Size, SizeCss, Sizes, Variant};

pub const AVATAR_SIZE: SizeCss = SizeCss::new("--lsx-avatar-size-");
pub const AVATAR_FONT_SIZE: SizeCss = SizeCss::new("--lsx-avatar-font-size-");
pub const AVATAR_RADII: SizeCss = SizeCss::new("--lsx-avatar-radius-");
pub const AVATAR_RADIUS: CssVar = CssVar::new("--lsx-avatar-radius");

pub const AVATAR_GROUP_SPACING: CssVar = CssVar::new("--lsx-avatar-group-spacing");
pub const AVATAR_GROUP_RING: CssVar = CssVar::new("--lsx-avatar-group-ring");
/// A member's paint order in a group, set by `AvatarGroup`: the first overlaps the next.
pub const AVATAR_GROUP_INDEX: CssVar = CssVar::new("--lsx-avatar-group-index");

/// Theme defaults for `Avatar`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AvatarDefaults {
    /// Placeholder chrome, also for `AvatarGroup` members. `Tonal`: initials on a
    /// light tint read at any size.
    pub variant: Variant,
    /// The tint of the placeholder, also for `AvatarGroup` members and its chip.
    pub color: Color,
    pub size: Size,
    /// The step of [`Self::radii`] an avatar takes when a call site names none.
    pub radius: Size,
    /// The square's side, in px.
    pub sizes: Sizes<u16>,
    /// Pixels at a 16px root, written as `rem`; about `side / 2.5`, floored at 10 px.
    pub font_sizes: Sizes<u16>,
    /// Its own radius scale; `xxl` is a circle, which no fixed global step gives.
    pub radii: Sizes<&'static str>,
}

impl AvatarDefaults {
    pub const DEFAULT: Self = Self {
        variant: Variant::Tonal,
        color: Color::Primary,
        size: Size::Md,
        radius: Size::Xxl,
        sizes: Sizes::new(20, 28, 38, 56, 84, 120),
        font_sizes: Sizes::new(10, 12, 15, 22, 34, 48),
        radii: Sizes::new("2px", "4px", "8px", "16px", "32px", "9999px"),
    };

    fn size_sx(size: Size) -> Sx {
        sx().width(AVATAR_SIZE.value(size))
            // A flex parent (`AvatarGroup`) squashes a width-only square into an ellipse.
            .min_width(AVATAR_SIZE.value(size))
            .height(AVATAR_SIZE.value(size))
            .font_size(AVATAR_FONT_SIZE.value(size))
    }

    pub fn theme_vars() -> Sx {
        sx().border_radius(AVATAR_RADIUS.overridable())
            .per_size(Self::size_sx)
    }
}

impl ToCssDeclarations for AvatarDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = self.sizes.to_css_declarations(AVATAR_SIZE, "px");
        declarations.extend(Size::ALL.into_iter().map(|size| {
            let rem = f32::from(self.font_sizes.get(size)) / 16.0;
            AVATAR_FONT_SIZE.declare(size, format!("{rem}rem"))
        }));
        for size in Size::ALL {
            declarations.push(AVATAR_RADII.declare(size, self.radii.get(size)));
        }
        declarations.push(AVATAR_RADIUS.declare(AVATAR_RADII.value(self.radius)));
        declarations
    }
}

/// Theme defaults for `AvatarGroup`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AvatarGroupDefaults {
    /// How far each member is pulled over the one before it.
    pub spacing: Size,
    /// The page-coloured ring that separates two overlapping members.
    pub ring: &'static str,
}

impl AvatarGroupDefaults {
    pub const DEFAULT: Self = Self {
        spacing: Size::Sm,
        ring: "2px",
    };
}

impl ToCssDeclarations for AvatarGroupDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            AVATAR_GROUP_SPACING.declare(SizeCss::SPACING.value(self.spacing)),
            AVATAR_GROUP_RING.declare(self.ring),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Todos 2363, 2601: rem, with the smallest initials at 10 px.
    #[test]
    fn the_font_sizes_are_rem_with_a_floor() {
        let css: Vec<String> = AvatarDefaults::DEFAULT
            .to_css_declarations()
            .iter()
            .map(ToString::to_string)
            .collect();

        assert!(
            css.iter()
                .any(|d| d.contains("--lsx-avatar-font-size-xs") && d.contains(":0.625rem")),
            "{css:?}"
        );
        assert!(
            css.iter()
                .any(|d| d.contains("--lsx-avatar-font-size-xxl") && d.contains(":3rem")),
            "{css:?}"
        );
        assert!(
            !css.iter()
                .any(|d| d.contains("font-size") && d.contains("px")),
            "{css:?}"
        );
    }
}
