use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, Size, SizeCss, Sizes};

pub const AVATAR_SIZE: SizeCss = SizeCss::new("--lsx-avatar-size-");
pub const AVATAR_FONT_SIZE: SizeCss = SizeCss::new("--lsx-avatar-font-size-");
pub const AVATAR_RADII: SizeCss = SizeCss::new("--lsx-avatar-radius-");
pub const AVATAR_RADIUS: CssVar = CssVar::new("--lsx-avatar-radius");

pub const AVATAR_GROUP_SPACING: CssVar = CssVar::new("--lsx-avatar-group-spacing");
pub const AVATAR_GROUP_RING: CssVar = CssVar::new("--lsx-avatar-group-ring");
/// A member's paint order inside a group, set per element by `AvatarGroup`.
/// The first avatar gets the highest, so each circle overlaps the next.
pub const AVATAR_GROUP_INDEX: CssVar = CssVar::new("--lsx-avatar-group-index");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AvatarDefaults {
    pub size: Size,
    /// The step of [`Self::radii`] an avatar takes when a call site names
    /// none. Overridden per instance the way `ImageDefaults::radius` is,
    /// through `AVATAR_RADIUS`'s override twin.
    pub radius: Size,
    /// The square's side, in px.
    pub sizes: Sizes<u16>,
    /// Derived from the square (`side / 2.5`), not a second scale to keep in
    /// step with it.
    pub font_sizes: Sizes<u16>,
    /// The avatar's own radius scale. `xxl` is a circle, which no fixed
    /// length gives: the global `xl` (64px) is a rounded square on the two
    /// largest avatars and a circle on the two smallest.
    pub radii: Sizes<&'static str>,
}

impl AvatarDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        radius: Size::Xxl,
        // Mantine's scale, plus an `xxl` continuing its steps.
        sizes: Sizes::new(20, 28, 38, 56, 84, 120),
        font_sizes: Sizes::new(8, 11, 15, 22, 34, 48),
        radii: Sizes::new("2px", "4px", "8px", "16px", "32px", "9999px"),
    };

    fn size_sx(size: Size) -> Sx {
        sx().width(AVATAR_SIZE.value(size))
            // `min-width` as well: a flex parent squashes a width-only square
            // into an ellipse, and an `AvatarGroup` *is* a flex row.
            .min_width(AVATAR_SIZE.value(size))
            .height(AVATAR_SIZE.value(size))
            .font_size(AVATAR_FONT_SIZE.value(size))
    }

    /// The `Pagination` shape: the size-independent declarations once, then a
    /// per-size block, so every `Avatar` on a page shares one recycled class.
    pub fn theme_vars() -> Sx {
        sx().border_radius(AVATAR_RADIUS.overridable())
            .per_size(Self::size_sx)
    }
}

impl ToCssDeclarations for AvatarDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = self.sizes.to_css_declarations(AVATAR_SIZE, "px");
        declarations.extend(self.font_sizes.to_css_declarations(AVATAR_FONT_SIZE, "px"));
        for size in Size::ALL {
            declarations.push(AVATAR_RADII.declare(size, self.radii.get(size)));
        }
        declarations.push(AVATAR_RADIUS.declare(AVATAR_RADII.value(self.radius)));
        declarations
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AvatarGroupDefaults {
    /// How far each member is pulled over the one before it.
    pub spacing: Size,
    /// Width of the ring in the page colour that separates two overlapping
    /// members. Without it the overlap is unreadable.
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
