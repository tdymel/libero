use std::fmt::Display;

use super::{Size, SizeCss};
use crate::css::CssDeclaration;

/// One value per [`Size`], e.g. a theme's spacing scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sizes<T> {
    pub xs: T,
    pub sm: T,
    pub md: T,
    pub lg: T,
    pub xl: T,
    pub xxl: T,
}

impl<T> Sizes<T> {
    pub const fn new(xs: T, sm: T, md: T, lg: T, xl: T, xxl: T) -> Self {
        Self {
            xs,
            sm,
            md,
            lg,
            xl,
            xxl,
        }
    }
}

impl<T: Copy> Sizes<T> {
    pub const fn get(&self, size: Size) -> T {
        match size {
            Size::Xs => self.xs,
            Size::Sm => self.sm,
            Size::Md => self.md,
            Size::Lg => self.lg,
            Size::Xl => self.xl,
            Size::Xxl => self.xxl,
        }
    }
}

impl<T: Display + Copy> Sizes<T> {
    /// One CSS var declaration per size, e.g. `--lsx-spacing-xs:4px;`.
    pub(crate) fn to_css_declarations(self, css: SizeCss, unit: &str) -> Vec<CssDeclaration> {
        Size::ALL
            .into_iter()
            .map(|size| css.declare(size, format!("{}{unit}", self.get(size))))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_happy_path() {
        const SIZES: Sizes<u8> = Sizes::new(1, 2, 3, 4, 5, 6);

        assert_eq!(SIZES.get(Size::Xs), 1);
        assert_eq!(SIZES.get(Size::Md), 3);
        assert_eq!(SIZES.get(Size::Xl), 5);
        assert_eq!(SIZES.get(Size::Xxl), 6);
    }
}
