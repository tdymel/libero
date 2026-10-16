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
    /// In px at a 16px root; declared as rem.
    pub control_sizes: Sizes<u16>,
    /// In px at a 16px root; declared as rem.
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
        font_sizes: Sizes::new(12, 14, 16, 18, 20, 24),
        // 3:1 on the page in both schemes (WCAG 1.4.11, todo 3597).
        border: ColorValue::Shade(Color::Muted, ColorShade::S6),
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

        // Declared as rem, so the box and its label follow the reader's text size.
        let rem = |px: u16| format!("{}rem", f32::from(px) / 16.0);
        let mut declarations = Vec::new();
        for size in Size::ALL {
            declarations
                .push(PAGINATION_CONTROL_SIZE.declare(size, rem(self.control_sizes.get(size))));
            declarations.push(PAGINATION_FONT_SIZE.declare(size, rem(self.font_sizes.get(size))));
        }
        declarations.push(PAGINATION_GAP.declare(SizeCss::SPACING.value(self.gap)));
        declarations.push(PAGINATION_BORDER.declare(self.border.value()));
        declarations.push(PAGINATION_ACTIVE_BACKGROUND.declare(active.value()));
        declarations.push(PAGINATION_ACTIVE_COLOR.declare(on_active.value()));
        declarations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Todo 3599: md is 16px like the other dense md text, and the ladder rises.
    #[test]
    fn md_text_is_16px_and_the_ladder_rises() {
        let sizes = PaginationDefaults::DEFAULT.font_sizes;
        assert_eq!(sizes.get(Size::Md), 16);
        for pair in Size::ALL.windows(2) {
            assert!(sizes.get(pair[0]) < sizes.get(pair[1]), "{pair:?}");
        }
    }

    /// Todo 3593: a px box under a rem label clips it at 200% text size.
    #[test]
    fn the_box_and_its_label_are_declared_in_rem() {
        let declarations = PaginationDefaults::DEFAULT.to_css_declarations();
        for var in [PAGINATION_CONTROL_SIZE, PAGINATION_FONT_SIZE] {
            for size in Size::ALL {
                let name = var.name(size);
                let declaration = declarations.iter().find(|d| d.property() == name);
                let value = declaration.map(|d| d.value());
                assert!(
                    value.is_some_and(|v| v.ends_with("rem")),
                    "{name}: {value:?}"
                );
            }
        }
    }
}
