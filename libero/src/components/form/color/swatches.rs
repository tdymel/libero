//! The preset colors a `ColorPicker` offers under its panel.

use super::ColorCode;
use crate::utils::warn;

/// A list of preset colors, built from `ColorCode`s or from CSS strings.
///
/// Strings are parsed at runtime - the list usually comes from config or a
/// design token file. One that parses as no color is skipped with a dev
/// warning, never a panic.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{ColorCode, ColorPicker};
/// # fn app() -> Element {
/// # let color = use_signal(ColorCode::default);
/// # rsx! {
/// ColorPicker { value: color(), swatches: ["#2e2e2e", "#868e96", "rgba(250, 82, 82, 0.5)"] }
/// # } }
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Swatches(Vec<ColorCode>);

impl Swatches {
    pub fn new(colors: Vec<ColorCode>) -> Self {
        Self(colors)
    }

    pub fn into_vec(self) -> Vec<ColorCode> {
        self.0
    }

    fn parsed<'a>(texts: impl IntoIterator<Item = &'a str>) -> Self {
        texts
            .into_iter()
            .filter_map(|text| match text.parse() {
                Ok(color) => Some(color),
                Err(_) => {
                    warn(&format!(
                        "ColorPicker: swatch `{text}` is not a color and is skipped."
                    ));
                    None
                }
            })
            .collect()
    }
}

impl std::ops::Deref for Swatches {
    type Target = [ColorCode];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec<ColorCode>> for Swatches {
    fn from(colors: Vec<ColorCode>) -> Self {
        Self(colors)
    }
}

impl From<Vec<&str>> for Swatches {
    fn from(texts: Vec<&str>) -> Self {
        Self::parsed(texts)
    }
}

impl From<Vec<String>> for Swatches {
    fn from(texts: Vec<String>) -> Self {
        Self::parsed(texts.iter().map(String::as_str))
    }
}

impl From<&[&str]> for Swatches {
    fn from(texts: &[&str]) -> Self {
        Self::parsed(texts.iter().copied())
    }
}

impl<const N: usize> From<[&str; N]> for Swatches {
    fn from(texts: [&str; N]) -> Self {
        Self::parsed(texts)
    }
}

impl<const N: usize> From<[ColorCode; N]> for Swatches {
    fn from(colors: [ColorCode; N]) -> Self {
        Self(colors.into())
    }
}

impl FromIterator<ColorCode> for Swatches {
    fn from_iter<T: IntoIterator<Item = ColorCode>>(colors: T) -> Self {
        Self(colors.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings_parse_and_nonsense_is_skipped() {
        let swatches = Swatches::from(["#ff0000", "not a color", "rgba(0, 0, 255, 0.5)"]);
        assert_eq!(swatches.len(), 2);
        assert_eq!(swatches[0].to_hex(), "#ff0000");
        assert_eq!(swatches[1].to_rgba(), "rgba(0, 0, 255, 0.5)");
    }
}
