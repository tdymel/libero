use crate::{
    sx::{Input, Sx, ThemeAwareValue, sx},
    tokens::{Responsive, Size, SizeCss},
};

/// The part of a size-or-CSS prop the state classes cannot carry: a custom base and every
/// breakpoint, as `apply` declares them. `None` for a bare size, which rides its state class.
pub(crate) fn responsive_sx(
    value: &Responsive<ThemeAwareValue>,
    apply: impl Fn(Sx, ThemeAwareValue) -> Sx,
) -> Option<Sx> {
    let base = match value.base_ref() {
        ThemeAwareValue::Size(_) if value.breakpoints().next().is_none() => return None,
        ThemeAwareValue::Size(_) => sx(),
        custom => apply(sx(), custom.clone()),
    };
    Some(value.breakpoints().fold(base, |base, (size, value)| {
        base.breakpoint(size, apply(sx(), value))
    }))
}

/// `extra` under the caller's `sx`, so the caller's own declaration still wins.
pub(crate) fn with_own(extra: Option<Sx>, own: &Input<Sx>) -> Input<Sx> {
    match extra {
        Some(extra) => extra.and(own.as_ref().cloned().unwrap_or_default()).into(),
        None => own.clone(),
    }
}

/// A size-or-CSS prop (`radius: "md"`, `radius: "0"`) after its fallback. Custom CSS keeps
/// the fallback's state class and replaces its value through the scale's override var.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ScaleOrCss {
    /// The state class or scale step: the prop's size, else the fallback.
    pub size: Size,
    custom: Option<ThemeAwareValue>,
}

impl ScaleOrCss {
    /// `value`, else `fallback` (the group's or the theme's).
    pub fn new(value: Option<&ThemeAwareValue>, fallback: Size) -> Self {
        match value {
            Some(ThemeAwareValue::Size(size)) => Self {
                size: *size,
                custom: None,
            },
            custom => Self {
                size: fallback,
                custom: custom.cloned(),
            },
        }
    }

    /// CSS text: the size off `scale`, or the custom value.
    pub fn resolve(&self, scale: SizeCss) -> String {
        self.custom_css(scale)
            .unwrap_or_else(|| scale.value(self.size))
    }

    /// The value for `scale.override_var()` on the instance; `None` for a size.
    pub fn custom_css(&self, scale: SizeCss) -> Option<String> {
        self.custom
            .as_ref()
            .and_then(|css| css.resolve(Some(scale)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::responsive;

    #[test]
    fn custom_css_keeps_the_fallback_step_and_overrides_it() {
        let custom = ScaleOrCss::new(Some(&"0".into()), Size::Md);
        let size = ScaleOrCss::new(Some(&Size::Lg.into()), Size::Md);

        assert_eq!(custom.size, Size::Md);
        assert_eq!(custom.custom_css(SizeCss::RADIUS).as_deref(), Some("0"));
        assert_eq!(
            (size.size, size.custom_css(SizeCss::RADIUS)),
            (Size::Lg, None)
        );
        assert_eq!(
            size.resolve(SizeCss::RADIUS),
            SizeCss::RADIUS.value(Size::Lg)
        );
    }

    #[test]
    fn a_bare_size_adds_nothing() {
        assert!(responsive_sx(&Responsive::new(Size::Md.into()), Sx::gap).is_none());
    }

    #[test]
    fn custom_css_and_breakpoints_are_declared() {
        let value = responsive(ThemeAwareValue::from("0")).md(Size::Lg.into());
        let declared = responsive_sx(&value, Sx::gap).expect("a custom base");

        assert_eq!(
            declared,
            sx().gap("0").breakpoint(Size::Md, sx().gap(Size::Lg))
        );
    }
}
