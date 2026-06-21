use super::{Size, Sizes};

pub const SPACING_CSS_VAR: &str = "--libero-spacing";

#[derive(Clone, PartialEq, Eq)]
pub struct Theme {
    spacing: Sizes<i64>,
}

impl Theme {
    pub const fn new(spacing: Sizes<i64>) -> Self {
        Self { spacing }
    }

    pub const fn spacing(&self) -> &Sizes<i64> {
        &self.spacing
    }

    pub const fn spacing_px(&self, size: Size) -> i64 {
        self.spacing.get(size)
    }

    pub fn css_variables(&self) -> String {
        format!(
            ":root {{ {}: {}px; {}-xs: {}px; {}-sm: {}px; {}-md: {}px; {}-lg: {}px; {}-xl: {}px; }}",
            SPACING_CSS_VAR,
            self.spacing_px(Size::Xs),
            SPACING_CSS_VAR,
            self.spacing_px(Size::Xs),
            SPACING_CSS_VAR,
            self.spacing_px(Size::Sm),
            SPACING_CSS_VAR,
            self.spacing_px(Size::Md),
            SPACING_CSS_VAR,
            self.spacing_px(Size::Lg),
            SPACING_CSS_VAR,
            self.spacing_px(Size::Xl)
        )
    }

    pub fn spacing_css_value(&self, factor: i64) -> String {
        format!("calc({} * var({}))", factor, SPACING_CSS_VAR)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::new(Sizes::new(4, 8, 12, 16, 24))
    }
}
