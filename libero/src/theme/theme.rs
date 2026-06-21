pub const SPACING_CSS_VAR: &str = "--libero-spacing";

#[derive(Clone, PartialEq, Eq)]
pub struct Theme {
    spacing_unit_px: i64,
}

impl Theme {
    pub const fn new(spacing_unit_px: i64) -> Self {
        Self { spacing_unit_px }
    }

    pub const fn spacing_unit_px(&self) -> i64 {
        self.spacing_unit_px
    }

    pub fn css_variables(&self) -> String {
        format!(
            ":root {{ {}: {}px; }}",
            SPACING_CSS_VAR, self.spacing_unit_px
        )
    }

    pub fn spacing_css_value(&self, factor: i64) -> String {
        format!("calc({} * var({}))", factor, SPACING_CSS_VAR)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::new(8)
    }
}
