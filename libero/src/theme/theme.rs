use super::{Color, ColorScale, Size, Sizes};

pub const SPACING_CSS_VAR: &str = "--libero-spacing";
pub const FONT_SIZE_CSS_VAR: &str = "--libero-font-size";
pub const BORDER_RADIUS_CSS_VAR: &str = "--libero-border-radius";
pub const PRIMARY_COLOR_CSS_VAR: &str = "--libero-color-primary";
pub const SECONDARY_COLOR_CSS_VAR: &str = "--libero-color-secondary";

#[derive(Clone, PartialEq)]
pub struct Theme {
    spacing: Sizes<i64>,
    font_size: Sizes<f32>,
    border_radius: Sizes<i64>,
    primary: ColorScale,
    secondary: ColorScale,
}

impl Theme {
    pub const fn new(
        spacing: Sizes<i64>,
        font_size: Sizes<f32>,
        border_radius: Sizes<i64>,
        primary: ColorScale,
        secondary: ColorScale,
    ) -> Self {
        Self {
            spacing,
            font_size,
            border_radius,
            primary,
            secondary,
        }
    }

    pub const fn spacing(&self) -> &Sizes<i64> {
        &self.spacing
    }

    pub const fn spacing_px(&self, size: Size) -> i64 {
        self.spacing.get(size)
    }

    pub const fn font_size(&self) -> &Sizes<f32> {
        &self.font_size
    }

    pub const fn font_size_rem(&self, size: Size) -> f32 {
        self.font_size.get(size)
    }

    pub const fn border_radius(&self) -> &Sizes<i64> {
        &self.border_radius
    }

    pub const fn border_radius_px(&self, size: Size) -> i64 {
        self.border_radius.get(size)
    }

    pub const fn primary(&self) -> &ColorScale {
        &self.primary
    }

    pub const fn secondary(&self) -> &ColorScale {
        &self.secondary
    }

    pub const fn color(&self, color: Color) -> &ColorScale {
        match color {
            Color::Primary => &self.primary,
            Color::Secondary => &self.secondary,
        }
    }

    pub fn css_variables(&self) -> String {
        let mut css = format!(
            ":root {{ {}: {}px; {}-xs: {}px; {}-sm: {}px; {}-md: {}px; {}-lg: {}px; {}-xl: {}px; {}-xs: {}rem; {}-sm: {}rem; {}-md: {}rem; {}-lg: {}rem; {}-xl: {}rem; {}-xs: {}px; {}-sm: {}px; {}-md: {}px; {}-lg: {}px; {}-xl: {}px;",
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
            self.spacing_px(Size::Xl),
            FONT_SIZE_CSS_VAR,
            self.font_size_rem(Size::Xs),
            FONT_SIZE_CSS_VAR,
            self.font_size_rem(Size::Sm),
            FONT_SIZE_CSS_VAR,
            self.font_size_rem(Size::Md),
            FONT_SIZE_CSS_VAR,
            self.font_size_rem(Size::Lg),
            FONT_SIZE_CSS_VAR,
            self.font_size_rem(Size::Xl),
            BORDER_RADIUS_CSS_VAR,
            self.border_radius_px(Size::Xs),
            BORDER_RADIUS_CSS_VAR,
            self.border_radius_px(Size::Sm),
            BORDER_RADIUS_CSS_VAR,
            self.border_radius_px(Size::Md),
            BORDER_RADIUS_CSS_VAR,
            self.border_radius_px(Size::Lg),
            BORDER_RADIUS_CSS_VAR,
            self.border_radius_px(Size::Xl)
        );

        for index in 0..10 {
            css.push_str(&format!(
                " {}-{}: {}; {}-{}: {};",
                PRIMARY_COLOR_CSS_VAR,
                index,
                self.primary.get(index),
                SECONDARY_COLOR_CSS_VAR,
                index,
                self.secondary.get(index)
            ));
        }

        css.push_str(" }");
        css
    }

    pub fn spacing_css_value(&self, factor: i64) -> String {
        format!("calc({} * var({}))", factor, SPACING_CSS_VAR)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::new(
            Sizes::new(4, 8, 12, 16, 24),
            Sizes::new(0.75, 0.875, 1.0, 1.125, 1.25),
            Sizes::new(2, 4, 8, 12, 16),
            ColorScale::from_anchor("#228BE6", 6),
            ColorScale::from_anchor("#7950F2", 6),
        )
    }
}
