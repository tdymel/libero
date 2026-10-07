use super::MONO_FONT_FAMILY;
use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, Size, SizeCss, Sizes};

pub const KBD_FONT_SIZE: SizeCss = SizeCss::new("--lsx-kbd-font-size-");

pub const KBD_FONT_FAMILY: CssVar = CssVar::new("--lsx-kbd-font-family");
pub const KBD_BACKGROUND: CssVar = CssVar::new("--lsx-kbd-background");
pub const KBD_BORDER: CssVar = CssVar::new("--lsx-kbd-border");
pub const KBD_COLOR: CssVar = CssVar::new("--lsx-kbd-color");

/// Theme defaults for `Kbd`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KbdDefaults {
    pub size: Size,
    /// Pixels at a 16px root, written as `rem`, so a key grows with a raised text size.
    pub font_sizes: Sizes<u16>,
    pub font_family: &'static str,
    pub background: &'static str,
    pub border: &'static str,
    pub color: &'static str,
}

impl KbdDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Sm,
        font_sizes: Sizes::new(10, 12, 14, 16, 20, 24),
        font_family: MONO_FONT_FAMILY,
        // The code block's `muted` steps, so a key follows the palette (todo 396).
        background: "var(--lsx-muted-1)",
        border: "var(--lsx-muted-4)",
        color: "var(--lsx-muted-7)",
    };

    fn size_sx(size: Size) -> Sx {
        sx().font_size(KBD_FONT_SIZE.value(size))
    }

    pub fn theme_vars() -> Sx {
        let border = format!("1px solid {}", KBD_BORDER.value());
        let border_bottom = format!("3px solid {}", KBD_BORDER.value());

        let base = sx()
            .font_family(KBD_FONT_FAMILY.value())
            .background(KBD_BACKGROUND.value())
            .color(KBD_COLOR.value())
            .border_top(border.clone())
            .border_left(border.clone())
            .border_right(border)
            // Thicker, so it reads as a keycap rather than a flat pill.
            .border_bottom(border_bottom)
            .border_radius(SizeCss::RADIUS.value(Size::Sm));

        base.per_size(Self::size_sx)
    }
}

impl ToCssDeclarations for KbdDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations: Vec<_> = Size::ALL
            .into_iter()
            .map(|size| {
                let rem = f32::from(self.font_sizes.get(size)) / 16.0;
                KBD_FONT_SIZE.declare(size, format!("{rem}rem"))
            })
            .collect();
        declarations.push(KBD_FONT_FAMILY.declare(self.font_family));
        declarations.push(KBD_BACKGROUND.declare(self.background));
        declarations.push(KBD_BORDER.declare(self.border));
        declarations.push(KBD_COLOR.declare(self.color));
        declarations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Todo 2542: rem, the same size as the old px at a 16px root.
    #[test]
    fn the_font_sizes_are_rem() {
        let css: Vec<String> = KbdDefaults::DEFAULT
            .to_css_declarations()
            .iter()
            .map(ToString::to_string)
            .collect();

        assert!(
            css.iter()
                .any(|d| d.contains("--lsx-kbd-font-size-xs") && d.contains(":0.625rem")),
            "{css:?}"
        );
        assert!(
            css.iter()
                .any(|d| d.contains("--lsx-kbd-font-size-xxl") && d.contains(":1.5rem")),
            "{css:?}"
        );
        assert!(!css.iter().any(|d| d.contains("px")), "{css:?}");
    }
}
