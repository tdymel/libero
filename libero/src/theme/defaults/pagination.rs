use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{Color, ColorShade, ColorValue, CssVar, Size, SizeCss, Sizes};

pub const PAGINATION_CONTROL_SIZE: SizeCss = SizeCss::new("--lsx-pagination-control-size-");
pub const PAGINATION_FONT_SIZE: SizeCss = SizeCss::new("--lsx-pagination-font-size-");

pub const PAGINATION_GAP: CssVar = CssVar::new("--lsx-pagination-gap");
pub const PAGINATION_BORDER: CssVar = CssVar::new("--lsx-pagination-border");
pub const PAGINATION_ACTIVE_BACKGROUND: CssVar = CssVar::new("--lsx-pagination-active-background");
pub const PAGINATION_ACTIVE_COLOR: CssVar = CssVar::new("--lsx-pagination-active-color");

/// Matches `base_color`'s default, so a themed and a prop colour share a step.
const ACTIVE_SHADE: ColorShade = ColorShade::S6;

/// Theme defaults for `Pagination`, set on [`Theme`](crate::theme::Theme).
/// Its strings live in [`Localization::pagination`](crate::localization::Localization::pagination).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaginationDefaults {
    pub size: Size,
    pub radius: Size,
    /// Fill of the current page; a bare [`Color`], so the contrast twin is derivable.
    pub color: Color,
    pub siblings: u8,
    pub boundaries: u8,
    pub gap: Size,
    /// Control box, in px.
    pub control_sizes: Sizes<u16>,
    pub font_sizes: Sizes<u16>,
    /// Not a `&'static str`: a raw `"muted.4"` would be declared verbatim.
    pub border: ColorValue,
}

impl PaginationDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        radius: Size::Sm,
        color: Color::Primary,
        siblings: 1,
        boundaries: 1,
        gap: Size::Xs,
        control_sizes: Sizes::new(22, 26, 32, 38, 44, 52),
        font_sizes: Sizes::new(11, 12, 14, 16, 18, 20),
        border: ColorValue::Shade(Color::Muted, ColorShade::S4),
    };

    fn size_sx(size: Size) -> Sx {
        sx().min_width(PAGINATION_CONTROL_SIZE.value(size))
            .height(PAGINATION_CONTROL_SIZE.value(size))
            .font_size(PAGINATION_FONT_SIZE.value(size))
    }

    pub fn theme_vars() -> Sx {
        sx().per_size(Self::size_sx)
    }
}

impl ToCssDeclarations for PaginationDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let active = ColorValue::Fill(self.color, ACTIVE_SHADE);
        let on_active = ColorValue::Contrast(self.color, ACTIVE_SHADE);

        let mut declarations = self
            .control_sizes
            .to_css_declarations(PAGINATION_CONTROL_SIZE, "px");
        declarations.extend(
            self.font_sizes
                .to_css_declarations(PAGINATION_FONT_SIZE, "px"),
        );
        declarations.push(PAGINATION_GAP.declare(SizeCss::SPACING.value(self.gap)));
        declarations.push(PAGINATION_BORDER.declare(self.border.value()));
        declarations.push(PAGINATION_ACTIVE_BACKGROUND.declare(active.value()));
        declarations.push(PAGINATION_ACTIVE_COLOR.declare(on_active.value()));
        declarations
    }
}
