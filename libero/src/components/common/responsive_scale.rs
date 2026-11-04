use crate::{
    components::common::{ScaleOrCss, responsive_var_sx},
    sx::{Input, Sx, ThemeAwareValue},
    tokens::{Responsive, Size, SizeCss},
};

/// A size-or-CSS prop (`radius`) per breakpoint: steps ride state words and range copies,
/// every breakpoint rides `scale`'s override var in the user layer (custom CSS included).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ResponsiveScale {
    /// The base value, for a reader that takes one.
    pub base: ScaleOrCss,
    /// The step per range: the state word and copy of each.
    pub steps: Responsive<Size>,
    /// Every breakpoint's override var, for under the caller's `sx`; `None` when plain.
    pub sx: Option<Sx>,
    scale: SizeCss,
}

impl ResponsiveScale {
    pub fn new(
        value: Option<&Responsive<ThemeAwareValue>>,
        fallback: Size,
        scale: SizeCss,
    ) -> Self {
        let base = ScaleOrCss::new(value.map(Responsive::base_ref), fallback);
        let Some(values) = value.filter(|value| !value.is_plain()) else {
            return Self {
                steps: base.size.into(),
                base,
                sx: None,
                scale,
            };
        };
        // A step's own value in the override var equals its copy's: no reset needed.
        let sx = responsive_var_sx(values, scale.override_var(), Some(scale));
        Self {
            steps: values.map(|value| ScaleOrCss::new(Some(&value), fallback).size),
            base,
            sx,
            scale,
        }
    }

    /// The inline override CSS of a plain value; a responsive one declares it in `sx` instead.
    pub fn custom_css(&self) -> Option<String> {
        match self.steps.is_plain() {
            true => self.base.custom_css(self.scale),
            false => None,
        }
    }

    /// `own` with the breakpoints' override vars under it; `None` when plain (use `own`).
    pub fn with_own(&self, own: &Input<Sx>) -> Option<Input<Sx>> {
        self.sx
            .as_ref()
            .map(|sx| super::with_own(Some(sx.clone()), own))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{sx::sx, tokens::responsive};

    #[test]
    fn a_plain_value_keeps_its_inline_override_and_adds_no_sx() {
        let value = Responsive::new(ThemeAwareValue::from("0"));
        let scale = ResponsiveScale::new(Some(&value), Size::Md, SizeCss::RADIUS);

        assert!(scale.sx.is_none());
        assert_eq!(scale.steps, Responsive::new(Size::Md));
        assert_eq!(scale.custom_css().as_deref(), Some("0"));
    }

    #[test]
    fn a_responsive_value_steps_per_range_and_declares_custom_css_per_breakpoint() {
        let value = responsive(ThemeAwareValue::from(Size::Sm)).md("0".into());
        let scale = ResponsiveScale::new(Some(&value), Size::Md, SizeCss::RADIUS);

        assert_eq!(scale.steps, responsive(Size::Sm).md(Size::Md));
        assert_eq!(scale.custom_css(), None);
        assert_eq!(
            scale.sx,
            Some(
                sx().var(
                    SizeCss::RADIUS.override_var(),
                    SizeCss::RADIUS.value(Size::Sm)
                )
                .breakpoint(Size::Md, sx().var(SizeCss::RADIUS.override_var(), "0"))
            )
        );
    }
}
